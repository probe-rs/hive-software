//! Handles the running of tests and starts/stops the runner and communicates with it
use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use anyhow::{Error, Result};
use axum::body::Bytes;
use cargo_toml::Manifest;
use comm_types::test::{TestOptions, TestResults};
use controller::common::hardware::HiveHardware;
use tar::Archive;
use thiserror::Error;
use tokio::runtime::Runtime;
use tokio::sync::mpsc::{self, Receiver as MpscReceiver, Sender as MpscSender};
use tokio::sync::oneshot::{self, Receiver as OneshotReceiver, Sender as OneshotSender};

use crate::database::HiveDb;
use crate::{flash, init, HARDWARE, HARDWARE_DB_DATA_CHANGED, SHUTDOWN_SIGNAL};

mod ipc;

/// The maximum amount of tasks which are allowed in the test task queue.
const TASK_CHANNEL_BUF_SIZE: usize = 10;
/// Path to the Hive workspace where the provided project is unpacked and built
const WORKSPACE_PATH: &str = "./workspace";
const RUNNER_BINARY_PATH: &str = "./workspace/bin";

#[derive(Debug, Error)]
pub(crate) enum TestManagerError {
    #[error("The testserver is shutting down and the test request was discarded")]
    Shutdown,
    #[error("Failed to receive the test results from the runner.\n\n Runner output: \n{0}")]
    RunnerError(String),
    #[error("Failed to build the provided project.\n\n Cargo output: \n{0}")]
    BuildError(String),
}

/// Errors which happen if the provided cargofile for testing is invalid
#[derive(Debug, Error)]
enum CargofileError {
    #[error("No cargofile found in root folder")]
    NoCargoFile,
    #[error("Crate probe-rs and its required dependencies not found in provided project")]
    WrongProject,
    #[error("Cargofile in root is not a workspace")]
    NoWorkspace,
}

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
#[derive(Debug)]
pub(crate) struct TestTask {
    pub result_sender: OneshotSender<Result<TestResults, Error>>,
    pub probe_rs_project: Bytes,
    pub options: TestOptions,
}

impl TestTask {
    pub fn new(
        probe_rs_project: Bytes,
        options: TestOptions,
    ) -> (Self, OneshotReceiver<Result<TestResults, Error>>) {
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
                    let mut data_changed = HARDWARE_DB_DATA_CHANGED.blocking_lock();
                    if *data_changed {
                        self.reinitialize_hardware(&mut hardware);
                        *data_changed = false;
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

            let test_results = self.run_test(&mut hardware, &task);

            task.result_sender.send(test_results).expect("Failed to send test results to task creator. Please ensure that the oneshot channel is not dropped on the task creator for the duration of the test run.");

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
        Self::prepare_workspace(&task.probe_rs_project)?;

        Self::build_runner()?;

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
        let runner_output = Command::new(format!("{}/runner", RUNNER_BINARY_PATH))
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

    /// Unpack the provided probe-rs tarball into the workspace and check if it is a valid probe-rs project
    ///
    /// # Panics
    /// If the [`WORKSPACE_PATH`] does not exist. This means that the environment in which the monitor runs in has not been configured properly
    fn prepare_workspace(probe_rs_project: &Bytes) -> Result<()> {
        let workspace_path = Path::new(WORKSPACE_PATH);

        if !workspace_path.exists() {
            panic!("Could not find path {}. This is likely a configuration issue. Please make sure that the Hive workspace containing the sourcefiles is located at this path", WORKSPACE_PATH)
        }

        let project_path = workspace_path.join("probe-rs-testcandidate");

        let mut tarball = Archive::new(probe_rs_project.as_ref());

        tarball.unpack(&project_path)?;

        let cargofile_path = project_path.join("probe-rs/Cargo.toml");

        if !cargofile_path.exists() {
            return Err(CargofileError::NoCargoFile.into());
        }

        let manifest = Manifest::from_path(cargofile_path)?;

        if let Some(workspace) = manifest.workspace {
            if !workspace.members.contains(&"probe-rs".to_owned()) {
                return Err(CargofileError::WrongProject.into());
            }
        } else {
            return Err(CargofileError::NoWorkspace.into());
        }

        Ok(())
    }

    /// Builds the runner binary with the provided probe-rs test dependency using Cargo
    ///
    /// # Panics
    /// If the [`RUNNER_BINARY_PATH`] does not exist. This means that the environment in which the monitor runs in has not been configured properly
    fn build_runner() -> Result<()> {
        if !Path::new(RUNNER_BINARY_PATH).exists() {
            panic!("Could not find path {}. This is likely a configuration issue. Please make sure that the ramdisk for storing the binary is correctly mounted at the requested path.", RUNNER_BINARY_PATH);
        }

        let build_output = Command::new("cargo")
            .args(["build", "-p", "runner", "--target-dir", RUNNER_BINARY_PATH])
            .output()
            .expect(
                "Failed to run cargo build. Is Cargo installed and accessible to the application?",
            );

        if !build_output.status.success() {
            return Err(TestManagerError::BuildError(
                String::from_utf8(build_output.stdout)
                    .unwrap_or_else(|_| "Could not parse cargo build output to utf8".to_owned()),
            )
            .into());
        }

        Ok(())
    }
}
