//! Handles all ipc communications
use std::mem;
use std::path::Path;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::{io, pin::Pin};

use axum::http::Uri;
use comm_types::defines::DefineRegistry;
use comm_types::ipc::{HiveProbeData, HiveTargetData, IpcMessage};
use comm_types::test::{TestResult, TestResults, TestRunStatus};
use hyper::client::connect::{Connected, Connection};
use hyper::{Body, Client};
use lazy_static::lazy_static;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::oneshot::Sender;
use tokio::sync::{Mutex, Notify};
use tokio::{net::UnixStream, sync::mpsc::Receiver};

use crate::SHUTDOWN_SIGNAL;

mod requests;
mod retry;

/// The location of the socketfile used for communication between runner and monitor
const SOCKET_PATH: &str = "./data/runner/ipc_sock";

lazy_static! {
    static ref TEST_RESULTS: Mutex<Vec<TestResult>> = Mutex::new(vec![]);
}

/// Messages which are passed between the [`std::thread`] used for testing, and the tokio runtime
#[derive(Debug)]
pub(crate) enum Message {
    TestResult(TestResult),
}

/// Struct representing the IPC connection
struct IpcConnection {
    stream: UnixStream,
}

impl AsyncWrite for IpcConnection {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

impl AsyncRead for IpcConnection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl Connection for IpcConnection {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

/// This function is the async entrypoint of tokio. All ipc from and to the monitor application are done here
pub(crate) async fn ipc(
    mut receiver: Receiver<Message>,
    init_data_sender: Sender<(HiveProbeData, HiveTargetData, DefineRegistry)>,
    notify_results_ready: Arc<Notify>,
) {
    let socket_path = Path::new(SOCKET_PATH);

    let mpsc_handler = tokio::spawn(async move {
        let mut shutdown_signal = SHUTDOWN_SIGNAL.subscribe();

        loop {
            tokio::select! {
                msg = receiver.recv() => {
                    if let Some(msg) = msg {
                        match msg {
                            Message::TestResult(result) => {
                                let mut results = TEST_RESULTS.lock().await;
                                results.push(result);
                            }
                        }
                    }else{
                        break;
                    }
                }
                result = shutdown_signal.recv() => {
                    result.expect("Failed to receive global shutdown signal");
                    break;
                }
            }
        }
    });

    let ipc_handler = tokio::spawn(async move {
        let connector = tower::service_fn(move |_: Uri| {
            let path = socket_path;
            Box::pin(async move {
                let stream = UnixStream::connect(path).await?;
                Ok::<_, io::Error>(IpcConnection { stream })
            })
        });

        let client: Client<_, Body> = hyper::Client::builder().build(connector);

        let client_copy = client.clone();

        let initialization = tokio::spawn(async move {
            let probes = retry::try_request(client_copy.clone(), requests::get_probes())
                .await
                .unwrap();

            let targets = retry::try_request(client_copy.clone(), requests::get_targets())
                .await
                .unwrap();

            let defines = retry::try_request(client_copy.clone(), requests::get_defines())
                .await
                .unwrap();

            let probe_data;
            if let IpcMessage::ProbeInitData(data) = probes {
                probe_data = data;
            } else {
                panic!("Received wrong IpcMessage enum variant from the monitor!")
            }

            let target_data;
            if let IpcMessage::TargetInitData(data) = targets {
                target_data = data;
            } else {
                panic!("Received wrong IpcMessage enum variant from the monitor!")
            }

            let define_data;
            if let IpcMessage::HiveDefineData(data) = defines {
                define_data = data;
            } else {
                panic!("Received wrong IpcMessage enum variant from the monitor!")
            }

            // Notify main thread with init data, so it can start with testing
            init_data_sender.send((*probe_data, *target_data, *define_data)).expect("Failed to send init data to main thread. Is the receiver still in scope and the thread still running?");
        });

        let client_copy = client.clone();

        let result_waiter = tokio::spawn(async move {
            let mut shutdown_signal = SHUTDOWN_SIGNAL.subscribe();

            tokio::select! {
                _ = notify_results_ready.notified() => {}
                result = shutdown_signal.recv() => {
                    result.expect("Failed to receive global shutdown signal");
                    return;
                }
            }

            let mut results = TEST_RESULTS.lock().await;
            retry::try_request(
                client_copy,
                requests::post_test_results(TestResults {
                    status: TestRunStatus::Ok,
                    results: Some(mem::take(&mut *results)),
                    error: None,
                }),
            )
            .await
            .unwrap();
        });

        initialization
            .await
            .expect("Failed to get initialization data from monitor");

        result_waiter
            .await
            .expect("Failed to send test results to monitor");
    });

    ipc_handler.await.unwrap();
    mpsc_handler.await.unwrap();
}
