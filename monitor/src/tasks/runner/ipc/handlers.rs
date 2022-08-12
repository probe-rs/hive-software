//! IPC request handlers
use std::sync::Arc;

use axum::Extension;
use comm_types::cbor::{Cbor, ServerParseError};
use comm_types::defines::DefineRegistry;
use comm_types::ipc::{HiveProbeData, HiveTargetData, IpcMessage};
use comm_types::test::{TestOptions, TestResults};
use hive_db::CborDb;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

use crate::database::{keys, MonitorDb};

/// Supply probe hardware data to the runner
pub async fn probe_handler(Extension(db): Extension<Arc<MonitorDb>>) -> Cbor<IpcMessage> {
    log::debug!("Received an IPC request on probe handler");

    let data: HiveProbeData = db
        .config_tree
        .c_get(&keys::config::ASSIGNED_PROBES)
        .unwrap()
        .expect("Probe data was not found in the database. The data should be initialized before the runner is started.");

    Cbor(IpcMessage::ProbeInitData(Box::new(data)))
}

/// Supply target hardware data to the runner
pub async fn target_handler(Extension(db): Extension<Arc<MonitorDb>>) -> Cbor<IpcMessage> {
    log::debug!("Received an IPC request on target handler");

    let data: HiveTargetData = db
        .config_tree
        .c_get(&keys::config::ASSIGNED_TARGETS)
        .unwrap()
        .expect("Target data was not found in the database. The data should be initialized before the runner is started.");

    Cbor(IpcMessage::TargetInitData(Box::new(data)))
}

/// Supply current Hive Define data to the runner
pub async fn define_handler(
    Extension(define_registry): Extension<&Mutex<DefineRegistry>>,
) -> Cbor<IpcMessage> {
    log::debug!("Received an IPC request on define handler");

    let registry = define_registry.lock().await;

    Cbor(IpcMessage::HiveDefineData(Box::new(registry.clone())))
}

/// Supply the current test options to the runner
pub async fn test_options_handler(
    Extension(options_mutex): Extension<&Mutex<TestOptions>>,
) -> Cbor<IpcMessage> {
    log::debug!("Received an IPC request on options handler");

    let options = options_mutex.lock().await;

    Cbor(IpcMessage::TestOptionData(Box::new(options.clone())))
}

/// Receive test results from the runner
pub async fn test_result_handler(
    Cbor(message): Cbor<IpcMessage>,
    Extension(test_result_sender): Extension<Sender<TestResults>>,
) -> Result<Cbor<IpcMessage>, ServerParseError> {
    if let IpcMessage::TestResults(results) = message {
        // send the received test results to the testmanager
        test_result_sender.send(*results).await.unwrap();
    } else {
        return Err(ServerParseError::InvalidCbor);
    }

    Ok(Cbor(IpcMessage::Empty))
}
