//! Handles the running of tests and starts/stops the runner and communicates with it
use std::process::Command;
use std::sync::Arc;

use anyhow::Result;
use axum::body::Bytes;
use comm_types::test::{TestOptions, TestResults, TestRunError, TestRunStatus};
use controller::common::hardware::HiveHardware;
use thiserror::Error;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver as MpscReceiver, Sender as MpscSender};
use tokio::sync::oneshot::{self, Receiver as OneshotReceiver, Sender as OneshotSender};

use crate::database::MonitorDb;
use crate::{flash, init, HARDWARE, HARDWARE_DB_DATA_CHANGED, SHUTDOWN_SIGNAL};

mod ipc;
mod workspace;

/// The maximum amount of tasks which are allowed in the test task queue.
const TASK_CHANNEL_BUF_SIZE: usize = 10;

#[derive(Debug, Error)]
pub(crate) enum TestManagerError {
    #[error("The testserver is shutting down and the test request was discarded")]
    Shutdown,
    #[error("Failed to receive the test results from the runner.\n\n Runner output: \n{0}")]
    RunnerError(String),
    #[error("Failed to build the provided project.\n\n Cargo output: \n{0}")]
    BuildError(String),
}

/// The testmanager of the monitor accepts external test tasks and returns the test results to the requesting party
pub(crate) struct TestManager {
    test_task_sender: MpscSender<TestTask>,
    test_task_receiver: MpscReceiver<TestTask>,
    reinit_task_sender: MpscSender<ReinitializationTask>,
    reinit_task_receiver: MpscReceiver<ReinitializationTask>,
    test_result_receiver: Option<MpscReceiver<TestResults>>,
    db: Arc<MonitorDb>,
}

/// A test task which can be sent to a [`TestManager`]
#[derive(Debug)]
pub(crate) struct TestTask {
    pub result_sender: OneshotSender<TestResults>,
    pub probe_rs_project: Bytes,
    pub options: TestOptions,
}

impl TestTask {
    pub fn new(
        probe_rs_project: Bytes,
        options: TestOptions,
    ) -> (Self, OneshotReceiver<TestResults>) {
        let (result_sender, result_receiver) = oneshot::channel();

        (
            Self {
                result_sender,
                probe_rs_project,
                options,
            },
            result_receiver,
        )
    }
}

/// A hardware reinitialization task which can be sent to a [`TestManager`]
pub(crate) struct ReinitializationTask {
    task_complete_sender: OneshotSender<()>,
}

impl ReinitializationTask {
    pub fn new() -> (Self, OneshotReceiver<()>) {
        let (task_complete_sender, task_complete_receiver) = oneshot::channel();

        (
            Self {
                task_complete_sender,
            },
            task_complete_receiver,
        )
    }
}

impl TestManager {
    /// Create a new [`TestManager`]
    pub fn new(db: Arc<MonitorDb>) -> Self {
        let (test_task_sender, test_task_receiver) = mpsc::channel(TASK_CHANNEL_BUF_SIZE);
        let (reinit_task_sender, reinit_task_receiver) = mpsc::channel(1);

        Self {
            test_task_sender,
            test_task_receiver,
            reinit_task_sender,
            reinit_task_receiver,
            test_result_receiver: None,
            db,
        }
    }

    pub fn run(&mut self, db: Arc<MonitorDb>, runtime: Arc<Runtime>) {
        let mut shutdown_receiver = SHUTDOWN_SIGNAL.subscribe();

        let (test_result_sender, test_result_receiver) = mpsc::channel(1);
        self.test_result_receiver = Some(test_result_receiver);

        // start IPC server used for communication with the runner
        runtime.spawn(ipc::ipc_server(db, test_result_sender));

        enum TaskType {
            TestTask(TestTask),
            ReinitTask(ReinitializationTask),
            Shutdown,
        }

        loop {
            let task;

            // Poll task receiver and shutdown receiver
            loop {
                let task_type = runtime.block_on(async {
                    tokio::select! {
                        received = self.reinit_task_receiver.recv() => {
                            match received {
                                Some(reinit_task) => TaskType::ReinitTask(reinit_task),
                                None => panic!("Reinitialization task sender has been dropped before the taskmanager was shut down"),
                            }
                        }

                        received = self.test_task_receiver.recv() => {
                            match received {
                                Some(received_task) => TaskType::TestTask(received_task),
                                None => panic!("Test task sender has been dropped before the taskmanager was shut down"),
                            }
                        }

                        received = shutdown_receiver.recv() => {
                            match received {
                                Ok(_) => TaskType::Shutdown,
                                Err(_) => panic!("Shutdown sender has been dropped before the taskmanager was shut down"),
                            }
                        }
                    }
                });

                match task_type {
                    TaskType::TestTask(test_task) => {
                        task = test_task;
                        break;
                    }
                    TaskType::ReinitTask(reinit_task) => {
                        let mut hardware = HARDWARE.lock().unwrap();

                        // Check if a reinitialization is required due to changes to the hardware data in the DB, otherwise skip
                        let mut data_changed = HARDWARE_DB_DATA_CHANGED.blocking_lock();
                        if *data_changed {
                            self.reinitialize_hardware(&mut hardware);
                            *data_changed = false;
                        }
                        drop(data_changed);

                        reinit_task.task_complete_sender.send(()).expect("Failed to send reinit task complete to task creator. Please ensure that the oneshot channel is not dropped on the task creator for the duration of the reinitialization.")
                    }
                    TaskType::Shutdown => return,
                }
            }

            let mut hardware = HARDWARE.lock().unwrap();

            let test_results =
                self.run_test(&mut hardware, &task)
                    .unwrap_or_else(|err| TestResults {
                        status: TestRunStatus::Error,
                        results: None,
                        error: Some(TestRunError {
                            err: err.to_string(),
                            source: err.source().map(|err| err.to_string()),
                        }),
                    });

            task.result_sender.send(test_results).expect("Failed to send test results to task creator. Please ensure that the oneshot channel is not dropped on the task creator for the duration of the test run.");

            workspace::restore_workspace();

            self.reinitialize_hardware(&mut hardware);
        }
    }

    /// Returns a new task sender, which can then be used to send new [`TestTask`]s to the testmanager
    pub fn get_test_task_sender(&self) -> MpscSender<TestTask> {
        self.test_task_sender.clone()
    }

    /// Returns a new task sender, which can then be used to send new [`ReinitializationTask`]s to the testmanager
    pub fn get_reinit_task_sender(&self) -> MpscSender<ReinitializationTask> {
        self.reinit_task_sender.clone()
    }

    fn reinitialize_hardware(&self, hardware: &mut HiveHardware) {
        // Reinitialize hardware
        init::init_hardware(self.db.clone(), hardware);

        // Reinitialize probes
        for testchannel in hardware.testchannels.iter() {
            let testchannel = testchannel.lock().unwrap();
            testchannel.reinitialize_probe().unwrap_or_else(|err|{
                log::warn!(
                    "Failed to reinitialize the debug probe connected to {}: {}. Skipping the remaining tests on this Testchannel.",
                    testchannel.get_channel(),
                    err
                )
            })
        }

        // Reflash testprograms
        flash::flash_testbinaries(self.db.clone());
    }

    /// Prepare the test environment, run the tests and return the received result
    fn run_test(&mut self, hardware: &mut HiveHardware, task: &TestTask) -> Result<TestResults> {
        workspace::prepare_workspace(&task.probe_rs_project)?;

        workspace::build_runner()?;

        // Check if a reinitialization is required due to changes to the hardware data in the DB
        let mut data_changed = HARDWARE_DB_DATA_CHANGED.blocking_lock();
        if *data_changed {
            self.reinitialize_hardware(hardware);
            *data_changed = false;
        }
        drop(data_changed);

        // Unlock probes
        for testchannel in hardware.testchannels.iter() {
            let testchannel = testchannel.lock().unwrap();

            testchannel.unlock_probe();
        }

        // TODO later this should start up the vm and run the runner there

        // Start runner and execute tests
        let runner_output = Command::new(format!("{}/runner", workspace::RUNNER_BINARY_PATH))
            .output()
            .expect(
                "Failed to run the runner. Is the runner command accessible to the application?",
            );

        // Try to receive a value as the runner command blocks until the runner is finished. If no message is received by then something went wrong
        match self.test_result_receiver.as_mut().unwrap().try_recv() {
            Ok(results) => Ok(results),
            Err(err) => match err {
                mpsc::error::TryRecvError::Empty => Err(TestManagerError::RunnerError(
                    String::from_utf8(runner_output.stdout)
                        .unwrap_or_else(|_| "Could not parse runner output to utf8".to_owned()),
                )
                .into()),
                mpsc::error::TryRecvError::Disconnected => {
                    // This might be a bug or simply a shutdown operation
                    log::warn!("Testresult sender part has been dropped, stopping test manager");
                    Err(TestManagerError::Shutdown.into())
                }
            },
        }
    }
}
