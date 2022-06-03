//! Handles the running of tests and starts/stops the runner and communicates with it
use std::process::Command;
use std::sync::Arc;

use comm_types::results::TestResults;
use tokio::fs::File;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver as MpscReceiver, Sender as MpscSender};
use tokio::sync::oneshot::{self, Receiver as OneshotReceiver, Sender as OneshotSender};
use tokio_tar::Archive;

use crate::database::HiveDb;
use crate::{flash, init, HARDWARE, SHUTDOWN_SIGNAL};

mod ipc;

/// The maximum amount of tasks which are allowed in the test task queue.
const TASK_CHANNEL_BUF_SIZE: usize = 10;

/// The testmanager of the monitor accepts external test tasks and returns the test results to the requesting party
pub(crate) struct TestManager {
    task_sender: MpscSender<TestTask>,
    task_receiver: MpscReceiver<TestTask>,
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

impl TestManager {
    /// Create a new [`TestManager`]
    pub fn new(db: Arc<HiveDb>) -> Self {
        let (task_sender, task_receiver) = mpsc::channel(TASK_CHANNEL_BUF_SIZE);

        Self {
            task_sender,
            task_receiver,
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
                if let Ok(received_task) = self.task_receiver.try_recv() {
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

            // Reinitialize hardware
            init::init_hardware(self.db.clone(), &mut hardware);

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

    /// Returns a new task sender, which can then be used to send new [`TestTask`]s to the testmanager
    pub fn get_task_sender(&self) -> MpscSender<TestTask> {
        self.task_sender.clone()
    }
}
