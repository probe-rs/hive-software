//! IPC request handlers
use std::sync::Arc;

use axum::Extension;
use comm_types::cbor::{Cbor, ServerParseError};
use comm_types::ipc::{HiveProbeData, HiveTargetData, IpcMessage};
use comm_types::test::TestResults;
use tokio::sync::mpsc::Sender;

use crate::database::{keys, CborDb, HiveDb};

pub(crate) async fn probe_handler(Extension(db): Extension<Arc<HiveDb>>) -> Cbor<IpcMessage> {
    log::debug!("Received an IPC request on probe handler");

    let data: HiveProbeData = db
        .config_tree
        .c_get(keys::config::ASSIGNED_PROBES)
        .unwrap()
        .expect("Probe data was not found in the database. The data should be initialized before the runner is started.");

    Cbor(IpcMessage::ProbeInitData(data))
}

pub(crate) async fn target_handler(Extension(db): Extension<Arc<HiveDb>>) -> Cbor<IpcMessage> {
    log::info!("Received an IPC request on target handler");

    let data: HiveTargetData = db
        .config_tree
        .c_get(keys::config::ASSIGNED_TARGETS)
        .unwrap()
        .expect("Target data was not found in the database. The data should be initialized before the runner is started.");

    Cbor(IpcMessage::TargetInitData(data))
}

pub(crate) async fn test_result_handler(
    Cbor(message): Cbor<IpcMessage>,
    Extension(test_result_sender): Extension<Sender<TestResults>>,
) -> Result<Cbor<IpcMessage>, ServerParseError> {
    if let IpcMessage::TestResults(results) = message {
        // send the received test results to the testmanager
        test_result_sender.send(results).await.unwrap();
    } else {
        return Err(ServerParseError::InvalidCbor);
    }

    Ok(Cbor(IpcMessage::Empty))
}
