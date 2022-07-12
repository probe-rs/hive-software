//! The task runner, which receives tasks from the [`TaskManager`] and executes them to completion
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::sync::Arc;

use anyhow::Result;
use comm_types::test::{TaskRunnerMessage, TestResults, TestRunError, TestRunStatus};
use controller::common::hardware::HiveHardware;
use thiserror::Error;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver as MpscReceiver};

use crate::database::MonitorDb;
use crate::{
    flash, init, testprogram, ACTIVE_TESTPROGRAM_CHANGED, HARDWARE, HARDWARE_DB_DATA_CHANGED,
    SHUTDOWN_SIGNAL,
};

use super::{TaskManager, TaskType, TestTask};

mod ipc;

/// Path to where the received runner binary is stored
const RUNNER_BINARY_PATH: &str = "./data/runner/runner";

#[derive(Debug, Error)]
pub(super) enum TaskRunnerError {
    #[error("The testserver is shutting down and the test request was discarded")]
    Shutdown,
    #[error("Failed to receive the test results from the runner\n\n Runner output: \n{0}")]
    RunnerError(String),
    #[error("Failed to build the provided project\n\n Cargo output: \n{0}")]
    BuildError(String),
}

/// The testmanager of the monitor accepts external test tasks and returns the test results to the requesting party
pub(crate) struct TaskRunner {
    test_result_receiver: Option<MpscReceiver<TestResults>>,
    db: Arc<MonitorDb>,
}

impl TaskRunner {
    /// Create a new [`TestManager`]
    pub fn new(db: Arc<MonitorDb>) -> Self {
        Self {
            test_result_receiver: None,
            db,
        }
    }

    /// Runs the testmanager
    ///
    /// This starts all necessary async tasks and runs forever until [`SHUTDOWN_SIGNAL`] is received
    pub fn run(mut self, runtime: Arc<Runtime>, task_manager: &TaskManager) {
        // Start IPC server used for communication with the runner
        let (test_result_sender, test_result_receiver) = mpsc::channel(1);
        self.test_result_receiver = Some(test_result_receiver);
        runtime.spawn(ipc::ipc_server(self.db.clone(), test_result_sender));

        // Start task receiver
        self.run_tasks(runtime, task_manager);
    }

    /// Receives and runs tasks
    fn run_tasks(&mut self, runtime: Arc<Runtime>, task_manager: &TaskManager) {
        let mut shutdown_receiver = SHUTDOWN_SIGNAL.subscribe();

        loop {
            let task;

            // Poll task receiver and shutdown receiver
            loop {
                let task_type = runtime.block_on(async {
                    let mut test_task_receiver = task_manager.get_test_task_receiver().await;
                    let mut reinit_task_receiver = task_manager.get_reinit_task_receiver().await;

                    loop {
                        tokio::select! {
                            result = shutdown_receiver.recv() => {
                                result.expect("Failed to receive global shutdown signal");
                                return TaskType::Shutdown;
                            }
                            test_task = test_task_receiver.recv() => {
                                if let Some(task) = test_task {
                                    return TaskType::TestTask(task);
                                }
                            }
                            reinit_task = reinit_task_receiver.recv() => {
                                if let Some(task) = reinit_task {
                                    return TaskType::ReinitTask(task);
                                }
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

                        // Check if a reinitialization is required due to changes to the hardware data in the DB or changes to testprogram data, otherwise skip
                        let mut hardware_data_changed = HARDWARE_DB_DATA_CHANGED.blocking_lock();
                        let mut testprogram_data_changed =
                            ACTIVE_TESTPROGRAM_CHANGED.blocking_lock();

                        if !*hardware_data_changed && *testprogram_data_changed {
                            testprogram::sync_binaries(self.db.clone(), &hardware);
                            *testprogram_data_changed = false;
                        } else if *hardware_data_changed {
                            self.reinitialize_hardware(&mut hardware);
                            *hardware_data_changed = false;
                            *testprogram_data_changed = false;
                        }

                        drop(hardware_data_changed);
                        drop(testprogram_data_changed);

                        reinit_task.task_complete_sender.send(Ok(())).expect("Failed to send reinit task complete to task creator. Please ensure that the oneshot channel is not dropped on the task creator for the duration of the reinitialization.")
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

            // Error is ignored as the websocket connection might have failed for some reason which means that the receiver has been dropped
            let _ = task
                .status_and_result_sender
                .as_ref()
                .unwrap()
                .blocking_send(TaskRunnerMessage::Results(test_results));

            log::info!("Finished task, reinitializing...");

            self.reinitialize_hardware(&mut hardware);

            // Wait for channel to get closed before dropping the sender for a clean channel shutdown
            while !task.status_and_result_sender.as_ref().unwrap().is_closed() {}
        }
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

        // Rebuild and link testbinaries
        testprogram::sync_binaries(self.db.clone(), hardware);

        // Reflash testprograms
        flash::flash_testbinaries(self.db.clone(), hardware);
    }

    /// Prepare the test environment, run the tests and return the received result
    fn run_test(&mut self, hardware: &mut HiveHardware, task: &TestTask) -> Result<TestResults> {
        let status_sender = task.status_and_result_sender.as_ref().unwrap();

        status_sender.blocking_send(TaskRunnerMessage::Status(
            "Preparing runner binary".to_owned(),
        ))?;
        fs::write(RUNNER_BINARY_PATH, &task.runner_binary)?;

        // Set as executable
        fs::set_permissions(RUNNER_BINARY_PATH, fs::Permissions::from_mode(0o777))?;

        // Check if a reinitialization is required due to changes to the hardware data in the DB or changes to the testprogram
        let mut hardware_data_changed = HARDWARE_DB_DATA_CHANGED.blocking_lock();
        let mut testprogram_data_changed = ACTIVE_TESTPROGRAM_CHANGED.blocking_lock();

        if !*hardware_data_changed && *testprogram_data_changed {
            status_sender.blocking_send(TaskRunnerMessage::Status(
                "Building testbinaries".to_owned(),
            ))?;
            log::info!("Building testbinaries");
            testprogram::sync_binaries(self.db.clone(), hardware);
            *testprogram_data_changed = false;
        } else if *hardware_data_changed {
            status_sender.blocking_send(TaskRunnerMessage::Status(
                "Reinitializing hardware".to_owned(),
            ))?;
            log::info!("Reinitializing hardware");
            self.reinitialize_hardware(hardware);
            *hardware_data_changed = false;
            *testprogram_data_changed = false;
        }

        drop(hardware_data_changed);
        drop(testprogram_data_changed);

        // Unlock probes
        for testchannel in hardware.testchannels.iter() {
            let testchannel = testchannel.lock().unwrap();

            testchannel.unlock_probe();
        }

        // TODO later this should start up the vm and run the runner there

        // Start runner and execute tests
        status_sender.blocking_send(TaskRunnerMessage::Status(
            "Starting runner and executing tests".to_owned(),
        ))?;
        log::info!("Starting runner and executing tests");
        let runner_output = Command::new(RUNNER_BINARY_PATH).output().expect(
            "Failed to run the runner. This is an implementation error or a configuration issue.",
        );

        // Try to receive a value as the runner command blocks until the runner is finished. If no message is received by then something went wrong
        status_sender.blocking_send(TaskRunnerMessage::Status("Collecting results".to_owned()))?;
        log::info!("Collecting results");
        match self.test_result_receiver.as_mut().unwrap().try_recv() {
            Ok(results) => Ok(results),
            Err(err) => match err {
                mpsc::error::TryRecvError::Empty => Err(TaskRunnerError::RunnerError(
                    String::from_utf8(runner_output.stdout)
                        .unwrap_or_else(|_| "Could not parse runner output to utf8".to_owned()),
                )
                .into()),
                mpsc::error::TryRecvError::Disconnected => {
                    // This might be a bug or simply a shutdown operation
                    log::warn!("Testresult sender part has been dropped, stopping test manager");
                    Err(TaskRunnerError::Shutdown.into())
                }
            },
        }
    }
}
