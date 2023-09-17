//! The task scheduler receives tasks from the [`TaskManager`] and executes them to completion.
use std::sync::Arc;

use comm_types::test::{TestOptions, TestResults};
use controller::hardware::{reset_probe_usb, HiveHardware};
use lazy_static::lazy_static;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::{self, Receiver as MpscReceiver};
use tokio::sync::Mutex;

use crate::database::MonitorDb;
use crate::{
    flash, init, testprogram, ACTIVE_TESTPROGRAM_CHANGED, HARDWARE_DB_DATA_CHANGED, SHUTDOWN_SIGNAL,
};

use super::reinit_task::ReinitializationTask;
use super::util::ipc;
use super::{Task, TaskType, TestTask};

lazy_static! {
    /// [`TestOptions`] of the currently running test task
    pub static ref CURRENT_TEST_TASK_OPTIONS: Mutex<TestOptions> =
        Mutex::new(TestOptions::default());
}

/// The task scheduler receives tasks and runs them accordingly.
///
/// It handles all task-global things such as the IPC-Server and Hardware reinitialization
pub struct TaskScheduler {
    test_result_receiver: Option<MpscReceiver<TestResults>>,
    test_task_receiver: MpscReceiver<TestTask>,
    reinit_task_receiver: MpscReceiver<ReinitializationTask>,
    db: Arc<MonitorDb>,
    runtime: Arc<Runtime>,
}

impl TaskScheduler {
    pub(super) fn new(
        test_task_receiver: MpscReceiver<TestTask>,
        reinit_task_receiver: MpscReceiver<ReinitializationTask>,
        runtime: Arc<Runtime>,
        db: Arc<MonitorDb>,
    ) -> Self {
        Self {
            test_result_receiver: None,
            test_task_receiver,
            reinit_task_receiver,
            runtime,
            db,
        }
    }

    /// Runs the scheduler
    ///
    /// This starts all necessary async tasks and runs forever until [`static@SHUTDOWN_SIGNAL`] is received
    pub fn run(mut self) {
        let mut shutdown_receiver = SHUTDOWN_SIGNAL.subscribe();

        // Start IPC server used for communication with the runner
        self.start_ipc_server();

        // Poll task receiver and shutdown receiver
        loop {
            let task_type = self.runtime.block_on(async {
                loop {
                    tokio::select! {
                        result = shutdown_receiver.recv() => {
                            result.expect("Failed to receive global shutdown signal");
                            return TaskType::Shutdown;
                        }
                        test_task = self.test_task_receiver.recv() => {
                            if let Some(task) = test_task {
                                return TaskType::TestTask(task);
                            }
                        }
                        reinit_task = self.reinit_task_receiver.recv() => {
                            if let Some(task) = reinit_task {
                                return TaskType::ReinitTask(task);
                            }
                        }
                    }
                }
            });

            match task_type {
                TaskType::TestTask(test_task) => test_task.run(&mut self),
                TaskType::ReinitTask(reinit_task) => reinit_task.run(&mut self),
                TaskType::Shutdown => return,
            }
        }
    }

    /// Starts the IPC server used for communication with the runner binary
    fn start_ipc_server(&mut self) {
        let (test_result_sender, test_result_receiver) = mpsc::channel(1);
        self.test_result_receiver = Some(test_result_receiver);
        self.runtime
            .spawn(ipc::ipc_server(self.db.clone(), test_result_sender));
    }

    /// Reinitialize the entire hardware before or after a task run, if it was changed.
    ///
    /// Also reinitializes the active testprogram if testprograms have been changed or the hardware has been changed.
    pub fn reinitialize_hardware(&self, hardware: &mut HiveHardware) {
        // Check if a reinitialization is required due to changes to the hardware data or changes to testprogram data, otherwise skip
        let mut hardware_data_changed = HARDWARE_DB_DATA_CHANGED.blocking_lock();
        let mut testprogram_data_changed = ACTIVE_TESTPROGRAM_CHANGED.blocking_lock();

        if *hardware_data_changed {
            // Reinitialize hardware
            init::init_hardware(self.db.clone(), hardware);

            // Reinitialize probes
            for testchannel in hardware.testchannels.iter() {
                let testchannel = testchannel.lock().unwrap();

                // Reset debug probe, if available
                if let Some(probe_info) = testchannel.get_probe_info() {
                    if let Err(err) = reset_probe_usb(&probe_info) {
                        log::warn!(
                            "Failed to reset usb interface of debug probe {:?}: {}",
                            probe_info,
                            err
                        );
                    }
                }

                testchannel.reinitialize_probe().unwrap_or_else(|err|{
                log::warn!(
                    "Failed to reinitialize the debug probe connected to {}: {}. Skipping it for any subsequent monitor operations until reinitialization.",
                    testchannel.get_channel(),
                    err,
                )
            })
            }
        }

        if *testprogram_data_changed || *hardware_data_changed {
            // Rebuild and link testbinaries
            testprogram::sync_binaries(self.db.clone(), hardware);

            // Reflash testprograms
            flash::flash_testbinaries(self.db.clone(), hardware);

            *hardware_data_changed = false;
            *testprogram_data_changed = false;
        }
    }

    /// Tries to receive test results from the IPC server communicating with the runner binary
    pub fn try_recv_test_result(&mut self) -> Result<TestResults, TryRecvError> {
        self.test_result_receiver.as_mut().expect("No test result receiver found when trying to receive results. This is a bug, please file an issue.").try_recv()
    }
}
