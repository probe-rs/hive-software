//! IPC request handlers
use std::sync::Arc;

use axum::Extension;
use ciborium::cbor;
use comm_types::ipc::{HiveProbeData, HiveTargetData};
use comm_types::{cbor::CborValue, ipc::IpcMessage};

use crate::database::{keys, CborDb, HiveDb};

use super::error::ServerError;
use super::extractors::Cbor;

pub(crate) async fn probe_handler(Extension(db): Extension<Arc<HiveDb>>) -> CborValue {
    log::debug!("Received an IPC request on probe handler");

    let data: HiveProbeData = db
        .config_tree
        .c_get(keys::config::PROBES)
        .unwrap()
        .expect("Probe data was not found in the database. The data should be initialized before the runner is started.");

    CborValue(cbor!(IpcMessage::ProbeInitData(data)).unwrap())
}

pub(crate) async fn target_handler(Extension(db): Extension<Arc<HiveDb>>) -> CborValue {
    log::info!("Received an IPC request on target handler");

    let data: HiveTargetData = db
        .config_tree
        .c_get(keys::config::TARGETS)
        .unwrap()
        .expect("Target data was not found in the database. The data should be initialized before the runner is started.");

    CborValue(cbor!(IpcMessage::TargetInitData(data)).unwrap())
}

pub(crate) async fn runner_log_handler(Cbor(message): Cbor) -> CborValue {
    log::info!("Received {:#?} on runner log handler.", message);
    todo!();

    CborValue(cbor!(IpcMessage::Empty).unwrap())
}

pub(crate) async fn test_result_handler(Cbor(message): Cbor) -> Result<CborValue, ServerError> {
    log::info!("Received {:#?} on test result handler.", message);
    if let IpcMessage::TestResults(results) = message {
        log::info!("Received test results on result handler: {:#?}", results);
        todo!();
    } else {
        return Err(ServerError::WrongMessageType);
    }

    Ok(CborValue(cbor!(IpcMessage::Empty).unwrap()))
}
