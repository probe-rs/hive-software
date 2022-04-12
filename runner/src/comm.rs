//! Handles all ipc communications
use std::path::Path;
use std::task::{Context, Poll};
use std::{io, pin::Pin};

use axum::body::Body;
use axum::http::{Method, Request, Uri};
use ciborium::de::from_reader;
use ciborium::value::Value;
use comm_types::cbor::CBOR_MIME;
use hyper::client::connect::{Connected, Connection};
use hyper::header;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::{net::UnixStream, sync::mpsc::Receiver};

const SOCKET_PATH: &str = "/tmp/hive/monitor/ipc_sock";

/// Messages which are passed between the [`std::thread`] used for testing, and the tokio runtime
#[derive(Debug)]
pub(crate) enum Message {
    //Error(String),
    Message(String),
    TestResult(String),
}

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

        let client = hyper::Client::builder().build(connector);

        let request = Request::builder()
            .method(Method::GET)
            .header(header::CONTENT_TYPE, CBOR_MIME)
            .uri("https://monitor.sock/data/probe")
            .body(Body::empty())
            .unwrap();

        let response = client.request(request).await.unwrap();

        if response.status().is_success() {
            let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
            let body: Value = from_reader(body.as_ref()).unwrap();
            log::info!("Received from monitor: {:?}", body);
        } else {
            log::error!(
                "Received error response with status: {} and body {:?}",
                response.status(),
                response.body()
            );
        }
    });

    ipc_handler.await.unwrap();
    mpsc_handler.await.unwrap();
}
