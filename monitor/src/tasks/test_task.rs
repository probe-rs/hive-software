use std::fs::{self};
use std::os::unix::fs::PermissionsExt;
use std::time::Duration;

use anyhow::Result;
use axum::body::Bytes;
use comm_types::test::{TaskRunnerMessage, TestOptions, TestResults, TestRunError, TestRunStatus};
use controller::hardware::HiveHardware;
use thiserror::Error;
use tokio::sync::mpsc::{self, Sender as MpscSender};

use crate::config::{HIVE_GID, RUNNER_UID};
use crate::tasks::scheduler::CURRENT_TEST_TASK_OPTIONS;
use crate::tasks::util::sandbox::Sandbox;
use crate::tasks::util::subprocess;
use crate::HARDWARE;
use crate::HARDWARE_DB_DATA_CHANGED;

use super::scheduler::TaskScheduler;
use super::{ws::WsTicket, Task};

#[cfg(doc)]
use super::TaskManager;

/// Path to where the received runner binary is stored
const RUNNER_BINARY_PATH: &str = "./data/runner/runner";
/// Path to the runner seccomp BPF filter file
const RUNNER_SECCOMP_FILTER_PATH: &str = "./data/seccomp/runner_seccomp.bpf";
/// Runner Binary max execution time before it is killed
const RUNNER_BINARY_TIMEOUT_SEC: u64 = 300;

#[derive(Debug, Error)]
pub(super) enum TestTaskError {
    #[error("The testserver is shutting down and the test task was discarded")]
    Shutdown,
    #[error("Failed to receive the test results from the runner\n\nRunner output: \n{0}")]
    RunnerError(String),
    #[error(
        "Runner binary took more than {} seconds to run. Is it deadlocked?",
        RUNNER_BINARY_TIMEOUT_SEC
    )]
    RunnerTimeout,
}

/// A test task which can be sent to a [`TaskManager`]
#[derive(Debug)]
pub struct TestTask {
    /// This is a channel which is directly connected to the specific websocket handler of this task.
    /// Therefore, anything which is sent into this channel is sent over the websocket to the requesting user
    status_and_result_sender: Option<MpscSender<TaskRunnerMessage>>,
    /// The WS ticket associated with this task
    ws_ticket: Option<WsTicket>,
    pub runner_binary: Bytes,
    pub options: TestOptions,
}

impl TestTask {
    pub fn new(runner_binary: Bytes, options: TestOptions) -> Self {
        Self {
            status_and_result_sender: None,
            ws_ticket: None,
            runner_binary,
            options,
        }
    }

    /// Generates a random websocket ticket and appends it to the struct and returns its value
    pub fn generate_ws_ticket(&mut self) -> WsTicket {
        let ticket = WsTicket::new();

        self.ws_ticket = Some(ticket.clone());

        ticket
    }

    /// Insert the provided sender as status and result sender for the [`TaskManager`]
    pub fn insert_status_and_result_sender(&mut self, sender: MpscSender<TaskRunnerMessage>) {
        self.status_and_result_sender = Some(sender);
    }

    /// Prepare the test environment, run the tests and return the received result
    fn run_test(
        &self,
        hardware: &mut HiveHardware,
        scheduler: &mut TaskScheduler,
    ) -> Result<TestResults> {
        let status_sender = self.status_and_result_sender.as_ref().unwrap();

        status_sender.blocking_send(TaskRunnerMessage::Status(
            "Preparing runner binary".to_owned(),
        ))?;
        fs::write(RUNNER_BINARY_PATH, &self.runner_binary)?;

        // Set as executable
        fs::set_permissions(RUNNER_BINARY_PATH, fs::Permissions::from_mode(0o777))?;

        // Check if a reinitialization is required due to changes to the hardware data in the DB or changes to the testprogram
        status_sender.blocking_send(TaskRunnerMessage::Status(
            "Reinitializing hardware".to_owned(),
        ))?;
        scheduler.reinitialize_hardware(hardware);

        // Unlock probes
        for testchannel in hardware.testchannels.iter() {
            let testchannel = testchannel.lock().unwrap();

            testchannel.unlock_probe();
        }

        // Start runner in sandbox and execute tests
        status_sender.blocking_send(TaskRunnerMessage::Status(
            "Starting runner and execute tests".to_owned(),
        ))?;
        log::info!("Starting runner in sandbox and execute tests");

        let sandbox = Sandbox::new(RUNNER_SECCOMP_FILTER_PATH);

        let mut runner_process = sandbox.run(
            RUNNER_BINARY_PATH,
            &RUNNER_UID.to_string(),
            &HIVE_GID.to_string(),
        );

        // Set hardware changed flag, as runner may leave hardware in unknown state
        *HARDWARE_DB_DATA_CHANGED.blocking_lock() = true;

        let runner_stdout_pipe = runner_process.stdout.take().unwrap();
        let runner_stderr_pipe = runner_process.stderr.take().unwrap();

        let (runner_stdout, runner_stderr) = match subprocess::subprocess_wait_timeout(
            runner_stdout_pipe,
            runner_stderr_pipe,
            Duration::from_secs(RUNNER_BINARY_TIMEOUT_SEC),
        ) {
            Ok(data) => Ok(data),
            Err(err) => match err.kind() {
                std::io::ErrorKind::TimedOut => {
                    // Kill runner process due to timeout
                    let _ = runner_process.kill(); // TODO: Maybe error handling?
                    let _ = runner_process.wait();

                    return Err(TestTaskError::RunnerTimeout.into());
                }
                _ => Err(err),
            },
        }?;

        runner_process.wait()?; // Make sure runner process has terminated

        // Try to receive a value as the runner command blocks until the runner is finished. If no message is received by then something went wrong
        status_sender.blocking_send(TaskRunnerMessage::Status("Collecting results".to_owned()))?;
        log::info!("Collecting results");
        match scheduler.try_recv_test_result() {
            Ok(results) => Ok(results),
            Err(err) => match err {
                mpsc::error::TryRecvError::Empty => Err(TestTaskError::RunnerError(format!(
                    "stdout: {}\n\nstderr: {}",
                    runner_stdout, runner_stderr
                ))
                .into()),
                mpsc::error::TryRecvError::Disconnected => {
                    // This might be a bug or simply a shutdown operation
                    log::warn!("Testresult sender part has been dropped, stopping test manager");
                    Err(TestTaskError::Shutdown.into())
                }
            },
        }
    }
}

impl Task for TestTask {
    fn run(self, scheduler: &mut TaskScheduler) {
        let mut hardware = HARDWARE.lock().unwrap();

        // Set current test options to supplied options by test task
        let mut test_options = CURRENT_TEST_TASK_OPTIONS.blocking_lock();
        *test_options = self.options.clone();
        drop(test_options);

        let test_results = self
            .run_test(&mut hardware, scheduler)
            .unwrap_or_else(|err| TestResults {
                status: TestRunStatus::Error,
                results: None,
                error: Some(TestRunError {
                    err: err.to_string(),
                    source: err.source().map(|err| err.to_string()),
                }),
            });

        // Error is ignored as the websocket connection might have failed for some reason which means that the receiver has been dropped
        let _ = self
            .status_and_result_sender
            .as_ref()
            .unwrap()
            .blocking_send(TaskRunnerMessage::Results(test_results));

        log::info!("Finished task, reinitializing...");

        scheduler.reinitialize_hardware(&mut hardware);

        // Wait for channel to get closed before dropping the sender for a clean channel shutdown
        while !self.status_and_result_sender.as_ref().unwrap().is_closed() {}
    }
}
