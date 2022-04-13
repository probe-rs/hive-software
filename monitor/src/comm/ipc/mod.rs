//! IPC between monitor and runner
use std::path::Path;
use std::sync::Arc;
use std::task;
use std::task::Poll;

use axum::extract::{connect_info, extractor_middleware};
use axum::routing::{get, post};
use axum::{BoxError, Router, Server};
use futures::ready;
use hyper::server::accept::Accept;
use tokio::net::unix::UCred;
use tokio::net::{unix::SocketAddr, UnixListener, UnixStream};

mod extractors;
mod handlers;
mod middleware;

const SOCKET_PATH: &str = "/tmp/hive/monitor/ipc_sock";

struct IpcStreamListener {
    listener: UnixListener,
}

impl Accept for IpcStreamListener {
    type Conn = UnixStream;

    type Error = BoxError;

    fn poll_accept(
        self: std::pin::Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> std::task::Poll<Option<Result<Self::Conn, Self::Error>>> {
        let (stream, _) = ready!(self.listener.poll_accept(cx))?;
        Poll::Ready(Some(Ok(stream)))
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct IpcConnectionInfo {
    runner_address: Arc<SocketAddr>,
    runner_credentials: UCred,
}

impl connect_info::Connected<&UnixStream> for IpcConnectionInfo {
    fn connect_info(target: &UnixStream) -> Self {
        IpcConnectionInfo {
            runner_address: Arc::new(target.peer_addr().unwrap()),
            runner_credentials: target.peer_cred().unwrap(),
        }
    }
}

/// Starts the IPC server and listens for incoming connections
pub(crate) async fn ipc_server() {
    let socket_path = Path::new(SOCKET_PATH);

    init_socket_file(socket_path).await;

    let listener = UnixListener::bind(socket_path).expect("TODO");

    let server_handle = tokio::spawn(async {
        let route = Router::new()
            .route("/data/probe", get(handlers::probe_handler))
            .route("/data/target", get(handlers::target_handler))
            .route("/runner/log", post(handlers::runner_log_handler))
            .route("/runner/results", post(handlers::test_result_handler))
            .layer(extractor_middleware::<middleware::CheckContentType>());

        Server::builder(IpcStreamListener { listener })
            .serve(route.into_make_service_with_connect_info::<IpcConnectionInfo>())
            .await
            .unwrap();
    });

    server_handle.await.unwrap();
}

/// Creates the folders required by the path, if not existing. Removes previous socket file if existing.
async fn init_socket_file(socket_path: &Path) {
    tokio::fs::create_dir_all(
        socket_path
            .parent()
            .expect("Provided socket path is already root or prefix"),
    )
    .await
    .expect(&format!(
        "Failed to create missing folders in path: {:?} Please check the permissions.",
        socket_path
    ));
    let _ = tokio::fs::remove_file(socket_path).await;
}
