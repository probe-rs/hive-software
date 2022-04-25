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

mod error;
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
        let route = app();

        Server::builder(IpcStreamListener { listener })
            .serve(route.into_make_service_with_connect_info::<IpcConnectionInfo>())
            .await
            .unwrap();
    });

    server_handle.await.unwrap();
}

/// Builds the IPC server with all endpoints
fn app() -> Router {
    Router::new()
        .route("/data/probe", get(handlers::probe_handler))
        .route("/data/target", get(handlers::target_handler))
        .route("/runner/log", post(handlers::runner_log_handler))
        .route("/runner/results", post(handlers::test_result_handler))
        .layer(extractor_middleware::<middleware::CheckContentType>())
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

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{header, Method, Request, StatusCode};
    use ciborium::de::from_reader;
    use comm_types::hardware::{ProbeInfo, TargetInfo, TargetState};
    use comm_types::ipc::{HiveProbeData, HiveTargetData, IpcMessage};
    use lazy_static::lazy_static;
    use tower::ServiceExt;

    use crate::database::{keys, CborDb, HiveDb};

    use super::app;

    lazy_static! {
        // We open a temporary test database and initialize it to the test values
        static ref DB: HiveDb = {
            let db = HiveDb::open_test();

            db.config_tree.c_insert(keys::config::PROBES, &*PROBE_DATA).unwrap();
            db.config_tree.c_insert(keys::config::TARGETS, &*TARGET_DATA).unwrap();

            db
        };
        static ref PROBE_DATA: HiveProbeData = [
            Some(ProbeInfo {
                identifier: "Curious Probe".to_owned(),
                vendor_id: 42,
                product_id: 920,
                serial_number: Some("abcde1234".to_owned()),
                hid_interface: None,
            }),
            None,
            Some(ProbeInfo {
                identifier: "Overpriced Probe".to_owned(),
                vendor_id: 43,
                product_id: 921,
                serial_number: Some("1234abcde".to_owned()),
                hid_interface: None,
            }),
            None,
        ];
        static ref TARGET_DATA: HiveTargetData = [
            Some([
                TargetState::Known(TargetInfo{
                    name: "ATSAMD10C13A-SS".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::Known(TargetInfo{
                    name: "ATSAMD09D14A-M".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::Known(TargetInfo{
                    name: "ATSAMD51J18A-A".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::Known(TargetInfo{
                    name: "ATSAMD21E16L-AFT".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
            ]),
            None,
            Some([
                TargetState::NotConnected,
                TargetState::Known(TargetInfo{
                    name: "LPC1114FDH28_102_5".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::NotConnected,
                TargetState::Known(TargetInfo{
                    name: "LPC1313FBD48_01,15".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
            ]),
            Some([
                TargetState::Known(TargetInfo{
                    name: "nRF5340".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::Known(TargetInfo{
                    name: "nRF52832-QFAB-T".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::Known(TargetInfo{
                    name: "nRF52840".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::Known(TargetInfo{
                    name: "NRF51822-QFAC-R7".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
            ]),
            None,
            Some([
                TargetState::Known(TargetInfo{
                    name: "STM32G031F4P6".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::NotConnected,
                TargetState::Known(TargetInfo{
                    name: "STM32L151C8TxA".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::NotConnected,
            ]),
            None,
            None,
        ];
    }

    #[tokio::test]
    async fn content_type_middleware_no_content_type() {
        let ipc_server = app();

        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/data/probe")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(
            String::from_utf8(bytes[..].to_vec()).unwrap(),
            "Missing content type header. Expecting application/cbor"
        );
    }

    #[tokio::test]
    async fn content_type_middleware_wrong_content_type() {
        let ipc_server = app();

        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/data/probe")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(
            String::from_utf8(bytes[..].to_vec()).unwrap(),
            "Wrong content type header provided. Expecting application/cbor"
        );
    }

    #[tokio::test]
    async fn content_type_middleware_correct() {
        let ipc_server = app();

        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/data/probe")
                    .header(header::CONTENT_TYPE, "application/cbor")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn wrong_rest_method() {
        let ipc_server = app();

        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/data/target")
                    .header(header::CONTENT_TYPE, "application/cbor")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn probe_endpoint() {
        let ipc_server = app();

        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/data/probe")
                    .header(header::CONTENT_TYPE, "application/cbor")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let data: IpcMessage = from_reader(&bytes[..]).unwrap();

        if let IpcMessage::ProbeInitData(data) = data {
            assert_eq!(
                data.iter().zip(PROBE_DATA.clone()).all(|(a, b)| {
                    match (a, b) {
                        (Some(a), Some(b)) => {
                            if *a == b {
                                return true;
                            }
                            false
                        }
                        (None, None) => true,
                        _ => false,
                    }
                }),
                true
            )
        } else {
            panic!("Expected IpcMessage::ProbeInitData, but found {:?}", data);
        }
    }

    #[tokio::test]
    async fn target_endpoint() {
        let ipc_server = app();

        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/data/target")
                    .header(header::CONTENT_TYPE, "application/cbor")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let data: IpcMessage = from_reader(&bytes[..]).unwrap();

        if let IpcMessage::TargetInitData(data) = data {
            assert_eq!(
                data.iter().zip(TARGET_DATA.clone()).all(|(a, b)| {
                    match (a, b) {
                        (Some(a), Some(b)) => {
                            if *a == b {
                                return true;
                            }
                            false
                        }
                        (None, None) => true,
                        _ => false,
                    }
                }),
                true
            )
        } else {
            panic!("Expected IpcMessage::TargetInitData, but found {:?}", data);
        }
    }
}
