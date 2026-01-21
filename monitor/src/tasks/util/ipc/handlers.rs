//! IPC request handlers
use std::sync::Arc;

use axum::Extension;
use comm_types::bincode::{Bincode, ServerParseError};
use comm_types::defines::DefineRegistry;
use comm_types::ipc::{HiveProbeData, HiveTargetData, IpcMessage};
use comm_types::test::{TestOptions, TestResults};
use hive_db::BincodeDb;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;

use crate::database::{MonitorDb, keys};

/// Supply probe hardware data to the runner
pub async fn probe_handler(Extension(db): Extension<Arc<MonitorDb>>) -> Bincode<IpcMessage> {
    log::debug!("Received an IPC request on probe handler");

    let data: HiveProbeData = db
        .config_tree
        .b_get(&keys::config::ASSIGNED_PROBES)
        .unwrap()
        .expect("Probe data was not found in the database. The data should be initialized before the runner is started.");

    Bincode(IpcMessage::ProbeInitData(Box::new(data)))
}

/// Supply target hardware data to the runner
pub async fn target_handler(Extension(db): Extension<Arc<MonitorDb>>) -> Bincode<IpcMessage> {
    log::debug!("Received an IPC request on target handler");

    let data: HiveTargetData = db
        .config_tree
        .b_get(&keys::config::ASSIGNED_TARGETS)
        .unwrap()
        .expect("Target data was not found in the database. The data should be initialized before the runner is started.");

    Bincode(IpcMessage::TargetInitData(Box::new(data)))
}

/// Supply current Hive Define data to the runner
pub async fn define_handler(
    Extension(define_registry): Extension<&Mutex<DefineRegistry>>,
) -> Bincode<IpcMessage> {
    log::debug!("Received an IPC request on define handler");

    let registry = define_registry.lock().await;

    Bincode(IpcMessage::HiveDefineData(Box::new(registry.clone())))
}

/// Supply the current test options to the runner
pub async fn test_options_handler(
    Extension(options_mutex): Extension<&Mutex<TestOptions>>,
) -> Bincode<IpcMessage> {
    log::debug!("Received an IPC request on options handler");

    let options = options_mutex.lock().await;

    Bincode(IpcMessage::TestOptionData(Box::new(options.clone())))
}

/// Receive test results from the runner
pub async fn test_result_handler(
    Extension(test_result_sender): Extension<Sender<TestResults>>,
    Bincode(message): Bincode<IpcMessage>,
) -> Result<Bincode<IpcMessage>, ServerParseError> {
    if let IpcMessage::TestResults(results) = message {
        // send the received test results to the testmanager
        test_result_sender.send(*results).await.unwrap();
    } else {
        return Err(ServerParseError::InvalidBincode);
    }

    Ok(Bincode(IpcMessage::Empty))
}
