//! Handles all ipc communications
//!
//! IPC is done using HTTP with CBOR payloads
use std::sync::Arc;
use std::vec;

use axum::body::Bytes;
use axum::extract::Request;
use comm_types::defines::DefineRegistry;
use comm_types::ipc::{HiveProbeData, HiveTargetData, IpcMessage};
use comm_types::test::{TestOptions, TestResult, TestResults, TestRunStatus};
use http_body_util::Full;
use hyper_util::client::legacy::Client;
use hyperlocal::{UnixClientExt, UnixConnector};
use tokio::sync::Notify;
use tokio::sync::mpsc::Receiver;
use tokio::sync::oneshot::Sender;
use tokio::task::JoinHandle;

use crate::SHUTDOWN_SIGNAL;

mod requests;
mod retry;

type IpcClient = Client<UnixConnector, Full<Bytes>>;
type IpcRequest = Request<Full<Bytes>>;

/// Messages which are passed between the [`std::thread`] and the tokio runtime
#[derive(Debug)]
pub enum Message {
    TestResult(TestResult),
}

/// This function is the async entrypoint of tokio. All ipc from and to the monitor application are done here
pub async fn ipc(
    test_result_receiver: Receiver<Message>,
    init_data_sender: Sender<(HiveProbeData, HiveTargetData, DefineRegistry, TestOptions)>,
    notify_results_ready: Arc<Notify>,
) {
    let client: IpcClient = Client::unix();

    let ipc_handler = tokio::spawn(async move {
        let client_copy = client.clone();

        let initialization = request_initialization_data_task(client_copy, init_data_sender);

        let send_results =
            send_test_results_task(client, notify_results_ready, test_result_receiver);

        initialization
            .await
            .expect("Failed to get initialization data from monitor");

        send_results
            .await
            .expect("Failed to send test results to monitor");
    });

    ipc_handler.await.unwrap();
}

fn request_initialization_data_task(
    client: IpcClient,
    init_data_sender: Sender<(HiveProbeData, HiveTargetData, DefineRegistry, TestOptions)>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let probes = retry::try_request(client.clone(), requests::get_probes())
            .await
            .unwrap();

        let targets = retry::try_request(client.clone(), requests::get_targets())
            .await
            .unwrap();

        let defines = retry::try_request(client.clone(), requests::get_defines())
            .await
            .unwrap();

        let options = retry::try_request(client.clone(), requests::get_options())
            .await
            .unwrap();

        let probe_data;
        if let IpcMessage::ProbeInitData(data) = probes {
            probe_data = data;
        } else {
            panic!(
                "Received wrong IpcMessage enum variant from the monitor. This is a bug, please open an issue."
            )
        }

        let target_data;
        if let IpcMessage::TargetInitData(data) = targets {
            target_data = data;
        } else {
            panic!(
                "Received wrong IpcMessage enum variant from the monitor. This is a bug, please open an issue."
            )
        }

        let define_data;
        if let IpcMessage::HiveDefineData(data) = defines {
            define_data = data;
        } else {
            panic!(
                "Received wrong IpcMessage enum variant from the monitor. This is a bug, please open an issue."
            )
        }

        let options_data;
        if let IpcMessage::TestOptionData(data) = options {
            options_data = data;
        } else {
            panic!(
                "Received wrong IpcMessage enum variant from the monitor. This is a bug, please open an issue."
            )
        }

        // Notify main thread with init data, so it can start with testing
        init_data_sender.send((*probe_data, *target_data, *define_data, *options_data)).expect("Failed to send init data to main thread. Is the receiver still in scope and the thread still running?");
    })
}

fn send_test_results_task(
    client: IpcClient,
    notify_results_ready: Arc<Notify>,
    mut test_result_receiver: Receiver<Message>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut shutdown_signal = SHUTDOWN_SIGNAL.subscribe();

        tokio::select! {
            _ = notify_results_ready.notified() => {}
            result = shutdown_signal.recv() => {
                result.expect("Failed to receive global shutdown signal");
                return;
            }
        }

        let mut results = vec![];

        // collect Test Results from channel cache
        while let Some(msg) = test_result_receiver.recv().await {
            match msg {
                Message::TestResult(result) => {
                    results.push(result);
                }
            }
        }

        retry::try_request(
            client,
            requests::post_test_results(TestResults {
                status: TestRunStatus::Ok,
                results: Some(results),
                error: None,
            }),
        )
        .await
        .unwrap();
    })
}
