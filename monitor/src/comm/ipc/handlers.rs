//! IPC request handlers
use ciborium::cbor;
use comm_types::{cbor::CborValue, ipc::IpcMessage};

use super::extractors::Cbor;

pub(crate) async fn probe_handler() -> CborValue {
    log::info!("Received a request on probe handler");
    todo!()
}

pub(crate) async fn target_handler() -> CborValue {
    log::info!("Received a request on target handler");
    todo!()
}

pub(crate) async fn runner_log_handler(Cbor(message): Cbor) -> CborValue {
    log::info!("Received {:#?} on runner log handler.", message);
    todo!();

    CborValue(cbor!(IpcMessage::Empty).unwrap())
}

pub(crate) async fn test_result_handler(Cbor(message): Cbor) -> CborValue {
    log::info!("Received {:#?} on test result handler.", message);
    if let IpcMessage::TestResults(results) = message {
        log::info!("Received test results on result handler: {:#?}", results);
    }
    todo!();

    CborValue(cbor!(IpcMessage::Empty).unwrap())
}
