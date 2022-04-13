//! Handles all ipc communications
use std::path::Path;
use std::task::{Context, Poll};
use std::{io, pin::Pin};

use axum::http::Uri;
use comm_types::results::TestResults;
use hyper::client::connect::{Connected, Connection};
use hyper::{Body, Client};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::{net::UnixStream, sync::mpsc::Receiver};

mod requests;
mod retry;

/// The location of the socketfile used for communication between runner and monitor
const SOCKET_PATH: &str = "/tmp/hive/monitor/ipc_sock";

/// Messages which are passed between the [`std::thread`] used for testing, and the tokio runtime
#[derive(Debug)]
pub(crate) enum Message {
    //Error(String),
    Message(String),
    TestResult(String),
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
pub(crate) async fn ipc(mut receiver: Receiver<Message>) {
    let socket_path = Path::new(SOCKET_PATH);

    let mpsc_handler = tokio::spawn(async move {
        while let Some(msg) = receiver.recv().await {
            println!("{:?}", msg);
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

        retry::try_request(client, requests::post_test_results(TestResults {}))
            .await
            .unwrap();
    });

    ipc_handler.await.unwrap();
    mpsc_handler.await.unwrap();
}
