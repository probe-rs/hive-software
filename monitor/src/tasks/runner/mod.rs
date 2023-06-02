//! The task runner receives tasks from the [`TaskManager`] and executes them to completion.
//!
//! This is where any task related initialization and handling happens.
use std::fs::{self, OpenOptions};
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::prelude::AsRawFd;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use comm_types::test::{TaskRunnerMessage, TestOptions, TestResults, TestRunError, TestRunStatus};
use command_fds::{CommandFdExt, FdMapping};
use controller::hardware::{reset_probe_usb, HiveHardware};
use lazy_static::lazy_static;
use thiserror::Error;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver as MpscReceiver};
use tokio::sync::Mutex;
use users::{get_group_by_name, get_user_by_name};
use wait_timeout::ChildExt;

use crate::database::MonitorDb;
use crate::{
    flash, init, testprogram, ACTIVE_TESTPROGRAM_CHANGED, HARDWARE, HARDWARE_DB_DATA_CHANGED,
    SHUTDOWN_SIGNAL,
};

use super::{TaskManager, TaskType, TestTask};

mod ipc;

/// Path to where the received runner binary is stored
const RUNNER_BINARY_PATH: &str = "./data/runner/runner";
/// Runner Binary max execution time before it is killed
const RUNNER_BINARY_TIMEOUT_SEC: u64 = 300;
/// Path to the runner seccomp BPF filter file
const RUNNER_SECCOMP_FILTER_PATH: &str = "./data/seccomp/runner_seccomp.bpf";
/// Name of the Hive group used to get access to hive specific functionalities
const HIVE_GROUP_NAME: &str = "hive";
/// Username of the user which executes the runner in the sandbox
const RUNNER_USER_NAME: &str = "runner";

lazy_static! {
    static ref HIVE_GID: u32 = {
        if let Some(group) = get_group_by_name(HIVE_GROUP_NAME) {
            group.gid()
        } else {
            panic!("Failed to find a group named '{}' on this system. This user group is required by the monitor. Is the system setup properly?", HIVE_GROUP_NAME);
        }
    };
    static ref RUNNER_UID: u32 = {
        if let Some(user) = get_user_by_name(RUNNER_USER_NAME) {
            user.uid()
        } else {
            panic!("Failed to find a user named '{}' on this system. This user is required by the monitor. Is the system setup properly?", RUNNER_USER_NAME);
        }
    };
    /// [`TestOptions`] of the currently running test task
    pub static ref CURRENT_TEST_TASK_OPTIONS: Mutex<TestOptions> =
        Mutex::new(TestOptions::default());
}

#[derive(Debug, Error)]
pub(super) enum TaskRunnerError {
    #[error("The testserver is shutting down and the test request was discarded")]
    Shutdown,
    #[error("Failed to receive the test results from the runner\n\nRunner output: \n{0}")]
    RunnerError(String),
    #[error(
        "Runner binary took more than {} seconds to run. Is it deadlocked?",
        RUNNER_BINARY_TIMEOUT_SEC
    )]
    RunnerTimeout,
}

/// The task runner of the monitor accepts external test tasks and returns the test results to the requesting party
pub struct TaskRunner {
    test_result_receiver: Option<MpscReceiver<TestResults>>,
    db: Arc<MonitorDb>,
}

impl TaskRunner {
    pub fn new(db: Arc<MonitorDb>) -> Self {
        Self {
            test_result_receiver: None,
            db,
        }
    }

    /// Runs the testmanager
    ///
    /// This starts all necessary async tasks and runs forever until [`static@SHUTDOWN_SIGNAL`] is received
    pub fn run(mut self, runtime: Arc<Runtime>, task_manager: &TaskManager) {
        // Start IPC server used for communication with the runner
        let (test_result_sender, test_result_receiver) = mpsc::channel(1);
        self.test_result_receiver = Some(test_result_receiver);
        runtime.spawn(ipc::ipc_server(self.db.clone(), test_result_sender));

        // Start task receiver
        self.run_tasks(runtime, task_manager)
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

                        self.reinitialize_hardware(&mut hardware);

                        reinit_task.task_complete_sender.send(Ok(())).expect("Failed to send reinit task complete to task creator. Please ensure that the oneshot channel is not dropped on the task creator for the duration of the reinitialization.")
                    }
                    TaskType::Shutdown => return,
                }
            }

            let mut hardware = HARDWARE.lock().unwrap();

            // Set current test options to supplied options by test task
            let mut test_options = CURRENT_TEST_TASK_OPTIONS.blocking_lock();
            *test_options = task.options.clone();
            drop(test_options);

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

    /// Reinitialize the entire hardware before or after a test task run, if it was changed.
    ///
    /// Also reinitializes the active testprogram if testprograms have been changed or the hardware has been changed.
    fn reinitialize_hardware(&self, hardware: &mut HiveHardware) {
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
        status_sender.blocking_send(TaskRunnerMessage::Status(
            "Reinitializing hardware".to_owned(),
        ))?;
        self.reinitialize_hardware(hardware);

        // Unlock probes
        for testchannel in hardware.testchannels.iter() {
            let testchannel = testchannel.lock().unwrap();

            testchannel.unlock_probe();
        }

        // Get runner seccomp FD to use bubblewrap sandbox with seccomp
        let runner_seccomp_bpf = OpenOptions::new().read(true).write(false).open(RUNNER_SECCOMP_FILTER_PATH).expect("Failed to open runner seccomp rule file. This is likely caused by a configuration issue or a corrupted installation.");

        // Start runner in sandbox and execute tests
        status_sender.blocking_send(TaskRunnerMessage::Status(
            "Starting runner and execute tests".to_owned(),
        ))?;
        log::info!("Starting runner in sandbox and execute tests");
        let mut child = Command::new("bwrap").args([
            "--die-with-parent", "--new-session",
            // Add runner seccomp filter
            "--seccomp", "25",
            // Unshare all namespaces and run under restricted user/group id
            "--unshare-all", "--uid", &RUNNER_UID.to_string(), "--gid", &HIVE_GID.to_string(),
            // Bind library folder for usage of shared objects used by runner binary
            "--ro-bind", "/lib/", "/lib/",
            "--ro-bind", "/usr/lib/debug/", "/usr/lib/debug/",
            // Bind required ressources in /etc
            "--ro-bind", "/etc/localtime", "/etc/localtime",
            "--ro-bind", "/etc/ld.so.cache", "/etc/ld.so.cache",
            "--ro-bind-try", "/etc/ld.so.preload", "/etc/ld.so.preload",
            // Bind required ressources in /proc
            "--proc", "/proc", 
            "--ro-bind", "/proc/cpuinfo", "/proc/cpuinfo",
            // Bind required devices
            "--dev-bind", "/dev/i2c-1", "/dev/i2c-1",
            "--dev-bind", "/dev/bus/usb/001/", "/dev/bus/usb/001/",
            "--dev-bind", "/dev/bus/usb/002/", "/dev/bus/usb/002/",
            "--ro-bind", "/sys/bus/usb/devices/", "/sys/bus/usb/devices/",
            "--ro-bind", "/sys/devices/platform/scb/fd500000.pcie/pci0000:00/0000:00:00.0/0000:01:00.0/usb1/", "/sys/devices/platform/scb/fd500000.pcie/pci0000:00/0000:00:00.0/0000:01:00.0/usb1/",
            "--ro-bind", "/sys/devices/platform/scb/fd500000.pcie/pci0000:00/0000:00:00.0/0000:01:00.0/usb2/", "/sys/devices/platform/scb/fd500000.pcie/pci0000:00/0000:00:00.0/0000:01:00.0/usb2/",
            "--ro-bind", "/run/udev/control", "/run/udev/control",
            "--ro-bind", "/run/udev/data/", "/run/udev/data/",
            "--ro-bind", "/sys/class/hidraw", "/sys/class/hidraw",
            // Bind log as rw so runner can save logs
            "--bind", "./data/logs/", "./data/logs/",
            // Bind testprograms as r so runner can use them to flash
            "--ro-bind", "./data/testprograms/", "./data/testprograms/",
            // Bind runner dir to get access to ipc and runner executable
            "--ro-bind", "./data/runner/", "./data/runner/",
            RUNNER_BINARY_PATH
            ]).fd_mappings(vec![
                FdMapping { parent_fd: runner_seccomp_bpf.as_raw_fd(), child_fd: 25 },
                ]).unwrap()
            .stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("Failed to run bubblewrap sandbox with runner. Is the bwrap command accessible to the application?");

        // Set hardware changed flag, as runner may leave hardware in unknown state
        *HARDWARE_DB_DATA_CHANGED.blocking_lock() = true;

        if child
            .wait_timeout(Duration::from_secs(RUNNER_BINARY_TIMEOUT_SEC))?
            .is_none()
        {
            // Kill runner process due to timeout
            let _ = child.kill();
            let _ = child.wait();

            return Err(TaskRunnerError::RunnerTimeout.into());
        }

        // Try to receive a value as the runner command blocks until the runner is finished. If no message is received by then something went wrong
        status_sender.blocking_send(TaskRunnerMessage::Status("Collecting results".to_owned()))?;
        log::info!("Collecting results");
        match self.test_result_receiver.as_mut().unwrap().try_recv() {
            Ok(results) => Ok(results),
            Err(err) => match err {
                mpsc::error::TryRecvError::Empty => {
                    let mut runner_stdout = vec![];
                    child
                        .stdout
                        .take()
                        .unwrap()
                        .read_to_end(&mut runner_stdout)?;

                    let mut runner_stderr = vec![];
                    child
                        .stderr
                        .take()
                        .unwrap()
                        .read_to_end(&mut runner_stderr)?;

                    Err(TaskRunnerError::RunnerError(format!(
                        "stdout: {}\n\nstderr: {}",
                        String::from_utf8(runner_stdout).unwrap_or_else(|_| {
                            "Could not parse runner output to utf8".to_owned()
                        }),
                        String::from_utf8(runner_stderr)
                            .unwrap_or_else(|_| "Could not parse runner output to utf8".to_owned())
                    ))
                    .into())
                }
                mpsc::error::TryRecvError::Disconnected => {
                    // This might be a bug or simply a shutdown operation
                    log::warn!("Testresult sender part has been dropped, stopping test manager");
                    Err(TaskRunnerError::Shutdown.into())
                }
            },
        }
    }
}
