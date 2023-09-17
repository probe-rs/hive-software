//! IPC between monitor and runner
//!
//! IPC is done using HTTP with CBOR payloads
use std::path::Path;
use std::sync::Arc;
use std::task;
use std::task::Poll;

use axum::extract::connect_info;
use axum::routing::{get, post};
use axum::{middleware, BoxError, Extension, Router, Server};
use comm_types::bincode::CheckContentType;
use comm_types::defines::DefineRegistry;
use comm_types::test::{TestOptions, TestResults};
use futures::ready;
use hyper::server::accept::Accept;
use tokio::net::unix::UCred;
use tokio::net::{unix::SocketAddr, UnixListener, UnixStream};
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tower::ServiceBuilder;

use crate::database::MonitorDb;
use crate::tasks::scheduler::CURRENT_TEST_TASK_OPTIONS;
use crate::testprogram::defines::DEFINE_REGISTRY;
use crate::SHUTDOWN_SIGNAL;

mod handlers;

const SOCKET_PATH: &str = "./data/runner/ipc_sock";

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
pub async fn ipc_server(db: Arc<MonitorDb>, test_result_sender: Sender<TestResults>) {
    let socket_path = Path::new(SOCKET_PATH);

    init_socket_file(socket_path).await;

    let listener = UnixListener::bind(socket_path).expect("TODO");

    let server_handle = tokio::spawn(async move {
        let route = app(
            db,
            test_result_sender,
            &CURRENT_TEST_TASK_OPTIONS,
            &DEFINE_REGISTRY,
        );

        let server = Server::builder(IpcStreamListener { listener })
            .serve(route.into_make_service_with_connect_info::<IpcConnectionInfo>());

        let mut shutdown_signal = SHUTDOWN_SIGNAL.subscribe();

        tokio::select! {
            result = server => {result.expect("Unhandled IPC server error encountered")}
            result = shutdown_signal.recv() => {result.expect("Failed to receive global shutdown signal")}
        }
    });

    server_handle.await.unwrap();
}

/// Builds the IPC server with all endpoints
fn app(
    db: Arc<MonitorDb>,
    test_result_sender: Sender<TestResults>,
    current_test_task_options: &'static Mutex<TestOptions>,
    define_registry: &'static Mutex<DefineRegistry>,
) -> Router {
    Router::new()
        .route("/data/probe", get(handlers::probe_handler))
        .route("/data/target", get(handlers::target_handler))
        .route(
            "/data/defines",
            get(handlers::define_handler).layer(Extension(define_registry)),
        )
        .route(
            "/data/options",
            get(handlers::test_options_handler).layer(Extension(current_test_task_options)),
        )
        .route(
            "/runner/results",
            post(handlers::test_result_handler).layer(Extension(test_result_sender)),
        )
        .layer(
            ServiceBuilder::new()
                .layer(Extension(db))
                .layer(middleware::from_extractor::<CheckContentType>()),
        )
}

/// Creates the folders required by the path, if not existing. Removes previous socket file if existing.
async fn init_socket_file(socket_path: &Path) {
    tokio::fs::create_dir_all(
        socket_path
            .parent()
            .expect("Provided socket path is already root or prefix"),
    )
    .await
    .unwrap_or_else(|_| {
        panic!(
            "Failed to create missing folders in path: {:?} Please check the permissions.",
            socket_path
        )
    });
    let _ = tokio::fs::remove_file(socket_path).await;
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{header, Method, Request, StatusCode};
    use bincode::config;
    use bincode::serde::{decode_from_slice, encode_to_vec};
    use comm_types::bincode::BINCODE_MIME;
    use comm_types::defines::DefineRegistry;
    use comm_types::hardware::{ProbeInfo, ProbeState, TargetInfo, TargetState};
    use comm_types::ipc::{HiveProbeData, HiveTargetData, IpcMessage};
    use comm_types::test::{Filter, TestFilter, TestOptions, TestResults, TestRunStatus};
    use hive_db::BincodeDb;
    use lazy_static::lazy_static;
    use tokio::sync::mpsc::{Receiver, Sender};
    use tokio::sync::Mutex;
    use tower::ServiceExt;

    use crate::database::{keys, MonitorDb};

    use super::app;

    lazy_static! {
        // We open a temporary test database and initialize it to the test values
        static ref DB: Arc<MonitorDb> = {
            let db = MonitorDb::open_test();

            db.config_tree.b_insert(&keys::config::ASSIGNED_PROBES, &*PROBE_DATA).unwrap();
            db.config_tree.b_insert(&keys::config::ASSIGNED_TARGETS, &*TARGET_DATA).unwrap();

            Arc::new(db)
        };
        static ref TEST_OPTIONS: Mutex<TestOptions> = Mutex::new(
            TestOptions {
                filter: Some(TestFilter {
                    probes: None,
                    targets: Some(Filter::Include(vec!["STM32*".to_owned(), "NRF*".to_owned()]))
                })
            }
        );
        static ref DEFINE_REGISTRY: Mutex<DefineRegistry> = Mutex::new(DefineRegistry::new());
        static ref PROBE_DATA: HiveProbeData = [
            ProbeState::Known(ProbeInfo {
                identifier: "Curious Probe".to_owned(),
                vendor_id: 42,
                product_id: 920,
                serial_number: Some("abcde1234".to_owned()),
                hid_interface: None,
            }),
            ProbeState::Unknown,
            ProbeState::Known(ProbeInfo {
                identifier: "Overpriced Probe".to_owned(),
                vendor_id: 43,
                product_id: 921,
                serial_number: Some("1234abcde".to_owned()),
                hid_interface: None,
            }),
            ProbeState::Unknown,
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

    /// Small mock interface which mimics the function of the TaskRunner
    struct MockTestResultManager {
        sender: Sender<TestResults>,
        receiver: Receiver<TestResults>,
    }

    impl MockTestResultManager {
        pub fn new() -> Self {
            let (sender, receiver) = tokio::sync::mpsc::channel(1);
            Self { sender, receiver }
        }

        pub fn get_sender(&self) -> Sender<TestResults> {
            self.sender.clone()
        }

        /// Try to receive the next TestResults
        ///
        /// # Panics
        /// In case receiving fails, or no TestResults are available
        pub fn receive(&mut self) -> TestResults {
            self.receiver.try_recv().unwrap()
        }
    }

    #[tokio::test]
    async fn wrong_rest_method() {
        let mock_test_result_manager = MockTestResultManager::new();
        let ipc_server = app(
            DB.clone(),
            mock_test_result_manager.get_sender(),
            &TEST_OPTIONS,
            &DEFINE_REGISTRY,
        );

        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::PUT)
                    .uri("/data/target")
                    .header(header::CONTENT_TYPE, BINCODE_MIME)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn wrong_content_type() {
        let mock_test_result_manager = MockTestResultManager::new();
        let ipc_server = app(
            DB.clone(),
            mock_test_result_manager.get_sender(),
            &TEST_OPTIONS,
            &DEFINE_REGISTRY,
        );

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

        assert_eq!(res.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn probe_endpoint() {
        let mock_test_result_manager = MockTestResultManager::new();
        let ipc_server = app(
            DB.clone(),
            mock_test_result_manager.get_sender(),
            &TEST_OPTIONS,
            &DEFINE_REGISTRY,
        );

        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/data/probe")
                    .header(header::CONTENT_TYPE, BINCODE_MIME)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let (data, _): (IpcMessage, _) = decode_from_slice(&bytes[..], config::standard()).unwrap();

        if let IpcMessage::ProbeInitData(data) = data {
            assert!(data.iter().zip(PROBE_DATA.clone()).all(|(a, b)| {
                match (a, b) {
                    (ProbeState::Known(a), ProbeState::Known(b)) => {
                        if *a == b {
                            return true;
                        }
                        false
                    }
                    (ProbeState::Unknown, ProbeState::Unknown) => true,
                    (ProbeState::NotConnected, ProbeState::NotConnected) => true,
                    _ => false,
                }
            }))
        } else {
            panic!("Expected IpcMessage::ProbeInitData, but found {:?}", data);
        }
    }

    #[tokio::test]
    async fn target_endpoint() {
        let mock_test_result_manager = MockTestResultManager::new();
        let ipc_server = app(
            DB.clone(),
            mock_test_result_manager.get_sender(),
            &TEST_OPTIONS,
            &DEFINE_REGISTRY,
        );

        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/data/target")
                    .header(header::CONTENT_TYPE, BINCODE_MIME)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let (data, _): (IpcMessage, _) = decode_from_slice(&bytes[..], config::standard()).unwrap();

        if let IpcMessage::TargetInitData(data) = data {
            assert!(data.iter().zip(TARGET_DATA.clone()).all(|(a, b)| {
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
            }))
        } else {
            panic!("Expected IpcMessage::TargetInitData, but found {:?}", data);
        }
    }

    #[tokio::test]
    async fn result_endpoint() {
        let mut mock_test_result_manager = MockTestResultManager::new();
        let ipc_server = app(
            DB.clone(),
            mock_test_result_manager.get_sender(),
            &TEST_OPTIONS,
            &DEFINE_REGISTRY,
        );

        let dummy_test_results = TestResults {
            status: TestRunStatus::Error,
            results: None,
            error: None,
        };

        let bytes = encode_to_vec(
            &IpcMessage::TestResults(Box::new(dummy_test_results)),
            config::standard(),
        )
        .unwrap();

        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/runner/results")
                    .header(header::CONTENT_TYPE, BINCODE_MIME)
                    .body(Body::from(bytes))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let (data, _): (IpcMessage, _) = decode_from_slice(&bytes[..], config::standard()).unwrap();

        if let IpcMessage::Empty = data {
            let received = mock_test_result_manager.receive();

            assert_eq!(received.status, TestRunStatus::Error);
            assert!(received.error.is_none());
            assert!(received.results.is_none());
        } else {
            panic!("Expected IpcMessage::Empty, but found {:?}", data);
        }
    }

    #[tokio::test]
    async fn define_endpoint() {
        let mock_test_result_manager = MockTestResultManager::new();
        let ipc_server = app(
            DB.clone(),
            mock_test_result_manager.get_sender(),
            &TEST_OPTIONS,
            &DEFINE_REGISTRY,
        );

        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/data/defines")
                    .header(header::CONTENT_TYPE, BINCODE_MIME)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let (data, _): (IpcMessage, _) = decode_from_slice(&bytes[..], config::standard()).unwrap();

        let define_registry_message =
            IpcMessage::HiveDefineData(Box::new(DEFINE_REGISTRY.lock().await.clone()));
        let define_registry_message_bytes =
            encode_to_vec(&define_registry_message, config::standard()).unwrap();

        if let IpcMessage::HiveDefineData(_) = data {
            assert_eq!(bytes, define_registry_message_bytes);
        } else {
            panic!("Expected IpcMessage::HiveDefineData, but found {:?}", data);
        }
    }

    #[tokio::test]
    async fn test_options_endpoint() {
        let mock_test_result_manager = MockTestResultManager::new();
        let ipc_server = app(
            DB.clone(),
            mock_test_result_manager.get_sender(),
            &TEST_OPTIONS,
            &DEFINE_REGISTRY,
        );

        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/data/options")
                    .header(header::CONTENT_TYPE, BINCODE_MIME)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let (data, _): (IpcMessage, _) = decode_from_slice(&bytes[..], config::standard()).unwrap();

        let test_options = TEST_OPTIONS.lock().await;

        if let IpcMessage::TestOptionData(data) = data {
            assert_eq!(*data, *test_options);
        } else {
            panic!("Expected IpcMessage::TestOptionData, but found {:?}", data);
        }
    }
}
