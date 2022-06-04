//! Handles the running of tests and starts/stops the runner and communicates with it
use std::process::Command;
use std::sync::Arc;

use comm_types::results::TestResults;
use controller::common::hardware::HiveHardware;
use tokio::fs::File;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver as MpscReceiver, Sender as MpscSender};
use tokio::sync::oneshot::{self, Receiver as OneshotReceiver, Sender as OneshotSender};
use tokio_tar::Archive;

use crate::database::HiveDb;
use crate::{flash, init, HARDWARE, HARDWARE_DB_DATA_CHANGED, SHUTDOWN_SIGNAL};

mod ipc;

/// The maximum amount of tasks which are allowed in the test task queue.
const TASK_CHANNEL_BUF_SIZE: usize = 10;

/// The testmanager of the monitor accepts external test tasks and returns the test results to the requesting party
pub(crate) struct TestManager {
    test_task_sender: MpscSender<TestTask>,
    test_task_receiver: MpscReceiver<TestTask>,
    reinit_task_sender: MpscSender<ReinitializationTask>,
    reinit_task_receiver: MpscReceiver<ReinitializationTask>,
    test_result_receiver: Option<MpscReceiver<TestResults>>,
    db: Arc<HiveDb>,
}

/// A test task which can be sent to a [`TestManager`]
pub(crate) struct TestTask {
    result_sender: OneshotSender<TestResults>,
    tarball: Archive<File>,
    //... other options like which targets to test etc should be implemented here
}

impl TestTask {
    pub fn new(tarball: Archive<File>) -> (Self, OneshotReceiver<TestResults>) {
        let (result_sender, result_receiver) = oneshot::channel();

        (
            Self {
                result_sender,
                tarball,
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
    pub fn new(db: Arc<HiveDb>) -> Self {
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

    pub fn run(&mut self, db: Arc<HiveDb>, runtime: Arc<Runtime>) {
        let mut shutdown_receiver = SHUTDOWN_SIGNAL.subscribe();

        let (test_result_sender, test_result_receiver) = mpsc::channel(1);
        self.test_result_receiver = Some(test_result_receiver);

        // start IPC server used for communication with the runner
        runtime.spawn(ipc::ipc_server(db, test_result_sender));

        loop {
            let task;

            // Poll task receiver and shutdown receiver
            loop {
                if let Ok(received_task) = self.reinit_task_receiver.try_recv() {
                    let mut hardware = HARDWARE.lock().unwrap();

                    // Check if a reinitialization is required due to changes to the hardware data in the DB, otherwise skip
                    let mut data_changed = *HARDWARE_DB_DATA_CHANGED.blocking_lock();
                    if data_changed {
                        self.reinitialize_hardware(&mut hardware);
                        data_changed = false;
                    }
                    drop(data_changed);

                    received_task.task_complete_sender.send(()).expect("Failed to send reint task complete to task creator. Please ensure that the oneshot channel is not dropped on the task creator for the duration of the reinitialization.")
                }

                if let Ok(received_task) = self.test_task_receiver.try_recv() {
                    // Save task and break out of the polling loop to complete the task
                    task = received_task;
                    break;
                }

                match shutdown_receiver.try_recv() {
                    // Exit inifnite loop function to allow the program to shutdown
                    Ok(_) => return,
                    Err(err) => match err {
                        tokio::sync::broadcast::error::TryRecvError::Empty => (),
                        _ => panic!("Failed to receive shutdown signal"),
                    },
                }
            }

            let mut hardware = HARDWARE.lock().unwrap();

            // Check if a reinitialization is required due to changes to the hardware data in the DB
            let mut data_changed = *HARDWARE_DB_DATA_CHANGED.blocking_lock();
            if data_changed {
                self.reinitialize_hardware(&mut hardware);
                data_changed = false;
            }
            drop(data_changed);

            // Unlock probes
            for testchannel in hardware.testchannels.iter() {
                let testchannel = testchannel.lock().unwrap();

                testchannel.unlock_probe();
            }

            // TODO later this should start up the vm and run the runner there

            // Start runner and execute tests
            Command::new("runner").output().expect(
                "Failed to run the runner. Is the runner command accessible to the application?",
            );

            match self.test_result_receiver.as_mut().unwrap().try_recv() {
                Ok(results) => task.result_sender.send(results).expect("Failed to send test results to task creator. Please ensure that the oneshot channel is not dropped on the task creator for the duration of the test run."),
                Err(err) => match err {
                    mpsc::error::TryRecvError::Empty => todo!("failed to receive any message. This might indicate that the runner did not run successfully"),
                    mpsc::error::TryRecvError::Disconnected => {
                        // This might be a bug or simply a shutdown operation
                        log::warn!(
                            "Testresult sender part has been dropped, stopping test manager"
                        );
                        return;
                    }
                },
            }

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
}
