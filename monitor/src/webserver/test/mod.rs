//! Test endpoint
//!
//! The test endpoint is a normal JSON REST API but also allows for websocket connections where JSON is also used.
//! The interface is meant to be pretty standard in order to make it easy for any client applications to create and handle test requests.
//!
//! The test endpoint does not only allow the client to send test requests but it can also call the /capabilities endpoint to find out which probes and targets are connected and ready for testing on the testrack.
//!
//! The general flow to complete a test request is as following:
//! ```text
//! Requesting user                                   Monitor                   
//! +------------------+   POST multipart request     +-----------------------+  
//! |Test Task:        | ---------------------------> |Receive Test task      |  
//! | - Runner binary  |                              | - Generate Ws ticket  |  
//! | - Other options  |   Response with WS ticket    | - Store in test cache |  
//! |                  | <--------------------------- |                       |  
//! +------------------+                              +-----------------------+  
//!                                                                              
//! +------------------+                              +-----------------------+  
//! |Prepare WS        |  GET Ws upgrade + WS ticket  |Receive WS upgrade req |  
//! | - Append received| ---------------------------> | - Check if ticket is  |  
//! |   WS ticket to   |                              |   valid and not timed |  
//! |   Request        |    open websocket if valid   |   out                 |  
//! |                  | <--------------------------- | - Move task into valid|  
//! |                  |                              |   queue if ticket is  |  
//! |                  |                              |   valid               |  
//! +------------------+                              +-----------------------+  
//!                                                                              
//! +------------------+                              +-----------------------+  
//! |WS communication  |         bidirectional        |WS communication       |  
//! | - Receive status | <--------------------------> | - Send status and test|  
//! |   and test       |                              |   results             |  
//! |   results        |                              | - Close WS            |  
//! +------------------+                              +-----------------------+
//! ```
//!
//! The reason on why a websocket is used for sending status and test results is that it is a lot easier to send real-time status messages to the client without having to poll client side or setting up webhooks.
//! Another factor is test execution time which can take quite some time and exceed the normally used http timeouts.
//!
//! To secure the websocket and prevent any unauthorized clients to open websocket connections the monitor issues a WS ticket
//! to the requesting client which grants the client the right to open a websocket connection on the monitor within a certian timeframe.
//! More info on this process can be found in the [`crate::tasks`] module documentation.
use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::middleware::{self};
use axum::routing::{get, post};
use axum::{Extension, Router};
use axum_extra::routing::RouterExt;
use tower::ServiceBuilder;
use tower_http::limit::RequestBodyLimitLayer;

use crate::database::MonitorDb;
use crate::tasks::TaskManager;

use super::api_token::require_api_token;

mod handlers;

const BODY_UPLOAD_LIMIT_BYTES: usize = 400_000_000;

pub(super) fn test_routes(db: Arc<MonitorDb>, task_manager: Arc<TaskManager>) -> Router {
    Router::new()
        .route_with_tsr("/capabilities", get(handlers::capabilities))
        .route_with_tsr(
            "/run",
            post(handlers::test).layer(
                ServiceBuilder::new()
                    .layer(RequestBodyLimitLayer::new(BODY_UPLOAD_LIMIT_BYTES))
                    .layer(DefaultBodyLimit::disable()),
            ),
        )
        .route_with_tsr("/socket", get(handlers::ws_handler))
        .layer(
            ServiceBuilder::new()
                .layer(Extension(db))
                .layer(Extension(task_manager))
                .layer(middleware::from_fn(require_api_token)),
        )
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::Arc;
    use std::time::Duration;

    use axum::body::Body;
    use axum::http::{header, Method, Request, StatusCode};
    use comm_types::hardware::{Capabilities, ProbeInfo, ProbeState, TargetInfo, TargetState};
    use comm_types::ipc::{HiveProbeData, HiveTargetData};
    use comm_types::test::TestOptions;
    use comm_types::token::{DbToken, TokenLifetime, API_TOKEN_HEADER};
    use hive_db::{BincodeDb, Key};
    use hyper::Request as HyperRequest;
    use lazy_static::lazy_static;
    use multipart::client::multipart::{Body as MultipartBody, Form};
    use tokio::runtime::Runtime;
    use tower::ServiceExt;

    use crate::database::{keys, MonitorDb};
    use crate::tasks::{TaskManager, WS_CONNECT_TIMEOUT_SECS};

    use super::test_routes;

    lazy_static! {
        // We open a temporary test database and initialize it to the test values
        static ref DB: Arc<MonitorDb> = {
            let db = MonitorDb::open_test();

            db.config_tree.b_insert(&keys::config::ASSIGNED_PROBES, &PROBE_DATA).unwrap();
            db.config_tree.b_insert(&keys::config::ASSIGNED_TARGETS, &TARGET_DATA).unwrap();

            db.token_tree.b_insert(&Key::<DbToken>::new(&API_TOKEN), &API_TOKEN_DATA).unwrap();

            Arc::new(db)
        };
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
        static ref API_TOKEN: &'static str = "myValidApiToken";
        static ref API_TOKEN_DATA: DbToken = DbToken { name: "Nice Token".to_owned(), description: "Nicer description".to_owned(), lifetime: TokenLifetime::Permanent };
    }

    fn with_async_runtime<Fn: FnOnce(Arc<Runtime>)>(function: Fn) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        function(Arc::new(rt));
    }

    #[test]
    fn capabilities_endpoint() {
        with_async_runtime(|rt| {
            let runtime = rt.clone();

            rt.block_on(async {
                let (task_manager, _) = TaskManager::new(DB.clone(), runtime);
                let test_routes = test_routes(DB.clone(), task_manager);

                let res = test_routes
                    .oneshot(
                        Request::builder()
                            .method(Method::GET)
                            .header(API_TOKEN_HEADER, &API_TOKEN as &str)
                            .uri("/capabilities")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(res.status(), StatusCode::OK);

                let capabilities = serde_json::from_slice::<Capabilities>(
                    &hyper::body::to_bytes(res.into_body()).await.unwrap(),
                )
                .unwrap();

                let expected_targets = [
                    "ATSAMD10C13A-SS",
                    "ATSAMD09D14A-M",
                    "ATSAMD51J18A-A",
                    "ATSAMD21E16L-AFT",
                    "LPC1114FDH28_102_5",
                    "LPC1313FBD48_01,15",
                    "nRF5340",
                    "nRF52832-QFAB-T",
                    "nRF52840",
                    "NRF51822-QFAC-R7",
                    "STM32G031F4P6",
                    "STM32L151C8TxA",
                ];
                let expected_probes = ["Curious Probe", "Overpriced Probe"];

                assert_eq!(expected_targets.len(), capabilities.available_targets.len());
                assert_eq!(expected_probes.len(), capabilities.available_probes.len());

                if capabilities
                    .available_targets
                    .iter()
                    .any(|target| !expected_targets.contains(&target.as_str()))
                {
                    panic!("Received targets do not match with expected targets:\nReceived:\n{:#?}\nExpected:\n{:#?}", capabilities.available_targets, expected_targets);
                }

                if capabilities
                    .available_probes
                    .iter()
                    .any(|probe| !expected_probes.contains(&probe.as_str()))
                {
                    panic!("Received probes do not match with expected probes:\nReceived:\n{:#?}\nExpected:\n{:#?}", capabilities.available_probes, expected_probes);
                }
            });
        });
    }

    #[test]
    fn test_endpoint_wrong_content_type() {
        with_async_runtime(|rt| {
            let runtime = rt.clone();

            rt.block_on(async {
                let (task_manager, _) = TaskManager::new(DB.clone(), runtime);
                let test_routes = test_routes(DB.clone(), task_manager);

                let res = test_routes
                    .oneshot(
                        Request::builder()
                            .method(Method::POST)
                            .header(API_TOKEN_HEADER, &API_TOKEN as &str)
                            .uri("/run")
                            .header(header::CONTENT_TYPE, "application/json")
                            .header(header::CONTENT_LENGTH, "0")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(res.status(), StatusCode::BAD_REQUEST);
            });
        });
    }

    #[test]
    fn test_endpoint_content_length() {
        with_async_runtime(|rt| {
            let runtime = rt.clone();

            rt.block_on(async {
                let (task_manager, _) = TaskManager::new(DB.clone(), runtime);
                let test_routes = test_routes(DB.clone(), task_manager);

                let res = test_routes
                    .oneshot(
                        Request::builder()
                            .method(Method::POST)
                            .header(API_TOKEN_HEADER, &API_TOKEN as &str)
                            .uri("/run")
                            .header(header::CONTENT_TYPE, "multipart/form-data")
                            .header(header::CONTENT_LENGTH, "4000000000")
                            .body(Body::from("a".repeat(800_001_000)))
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
            });
        });
    }

    #[test]
    fn test_endpoint_unknown_field_name() {
        with_async_runtime(|rt| {
            let runtime = rt.clone();

            rt.block_on(async {
                let (task_manager, _) = TaskManager::new(DB.clone(), runtime);
                let test_routes = test_routes(DB.clone(), task_manager);

                let mut form = Form::default();
                form.add_text("unknown", "some text");

                let req = form
                    .set_body_convert::<hyper::Body, MultipartBody>(
                        HyperRequest::builder()
                            .method(Method::POST)
                            .header(API_TOKEN_HEADER, &API_TOKEN as &str)
                            .uri("/run"),
                    )
                    .unwrap();

                let res = test_routes.oneshot(req).await.unwrap();

                assert_eq!(res.status(), StatusCode::BAD_REQUEST);

                assert_eq!(
                    "Found unexpected field name: 'unknown'",
                    String::from_utf8_lossy(
                        hyper::body::to_bytes(res.into_body())
                            .await
                            .unwrap()
                            .as_ref()
                    )
                );
            });
        });
    }

    #[test]
    fn test_endpoint_missing_runner_field() {
        with_async_runtime(|rt| {
            let runtime = rt.clone();

            rt.block_on(async {
                let (task_manager, _) = TaskManager::new(DB.clone(), runtime);
                let test_routes = test_routes(DB.clone(), task_manager);

                let options = TestOptions { filter: None };

                let mut form = Form::default();

                let json = Cursor::new(serde_json::to_vec(&options).unwrap());
                form.add_reader_file_with_mime("options", json, "options.json", mime::APPLICATION_JSON);

                let req = form
                    .set_body_convert::<hyper::Body, MultipartBody>(
                        HyperRequest::builder().method(Method::POST).header(API_TOKEN_HEADER, &API_TOKEN as &str).uri("/run"),
                    )
                    .unwrap();

                let res = test_routes.oneshot(req).await.unwrap();

                assert_eq!(res.status(), StatusCode::BAD_REQUEST);

                assert_eq!(
                    "No runner binary provided to perform the tests on. The field 'runner' is missing.",
                    String::from_utf8_lossy(
                        hyper::body::to_bytes(res.into_body())
                            .await
                            .unwrap()
                            .as_ref()
                    )
                );
            });
        });
    }

    #[test]
    fn test_endpoint_wrong_runner_field_data_type() {
        with_async_runtime(|rt| {
            let runtime = rt.clone();

            rt.block_on(async {
                let (task_manager, _) = TaskManager::new(DB.clone(), runtime);
                let test_routes = test_routes(DB.clone(), task_manager);

                let mut form = Form::default();

                let data = Cursor::new("Some data");

                form.add_reader_file_with_mime(
                    "runner",
                    data,
                    "some_runner_file",
                    mime::APPLICATION_JSON,
                );

                let req = form
                    .set_body_convert::<hyper::Body, MultipartBody>(
                        HyperRequest::builder()
                            .method(Method::POST)
                            .header(API_TOKEN_HEADER, &API_TOKEN as &str)
                            .uri("/run"),
                    )
                    .unwrap();

                let res = test_routes.oneshot(req).await.unwrap();

                assert_eq!(res.status(), StatusCode::BAD_REQUEST);

                assert_eq!(
                    "Invalid file format provided for field 'runner'. Expecting binary executable.",
                    String::from_utf8_lossy(
                        hyper::body::to_bytes(res.into_body())
                            .await
                            .unwrap()
                            .as_ref()
                    )
                );
            });
        });
    }

    #[test]
    fn test_endpoint_wrong_options_field_data_type() {
        with_async_runtime(|rt| {
            let runtime = rt.clone();

            rt.block_on(async {
                let (task_manager, _) = TaskManager::new(DB.clone(), runtime);
                let test_routes = test_routes(DB.clone(), task_manager);

                let options = TestOptions { filter: None };

                let mut form = Form::default();

                let json = Cursor::new(serde_json::to_vec(&options).unwrap());
                form.add_reader_file_with_mime(
                    "options",
                    json,
                    "options.json",
                    mime::APPLICATION_JAVASCRIPT,
                );

                let data = Cursor::new("Some data");

                form.add_reader_file_with_mime(
                    "runner",
                    data,
                    "some_runner_file",
                    mime::APPLICATION_OCTET_STREAM,
                );

                let req = form
                    .set_body_convert::<hyper::Body, MultipartBody>(
                        HyperRequest::builder()
                            .method(Method::POST)
                            .header(API_TOKEN_HEADER, &API_TOKEN as &str)
                            .uri("/run"),
                    )
                    .unwrap();

                let res = test_routes.oneshot(req).await.unwrap();

                assert_eq!(res.status(), StatusCode::BAD_REQUEST);

                assert_eq!(
                    "Invalid file format provided for field 'options'. Expecting json data.",
                    String::from_utf8_lossy(
                        hyper::body::to_bytes(res.into_body())
                            .await
                            .unwrap()
                            .as_ref()
                    )
                );
            });
        });
    }

    #[test]
    #[allow(dead_code)]
    fn test_endpoint_correct() {
        with_async_runtime(|rt| {
            let runtime = rt.clone();

            rt.block_on(async {
                let (task_manager, _task_scheduler) = TaskManager::new(DB.clone(), runtime);
                let test_routes = test_routes(DB.clone(), task_manager.clone());

                let mut form = Form::default();

                let data = Cursor::new("Some data");

                form.add_reader_file_with_mime(
                    "runner",
                    data,
                    "some_runner_file",
                    mime::APPLICATION_OCTET_STREAM,
                );

                let req = form
                    .set_body_convert::<hyper::Body, MultipartBody>(
                        HyperRequest::builder()
                            .method(Method::POST)
                            .header(API_TOKEN_HEADER, &API_TOKEN as &str)
                            .uri("/run"),
                    )
                    .unwrap();

                let res = test_routes.oneshot(req).await.unwrap();

                assert_eq!(res.status(), StatusCode::OK);

                let ws_ticket: String = serde_json::from_reader(
                    hyper::body::to_bytes(res.into_body())
                        .await
                        .unwrap()
                        .as_ref(),
                )
                .unwrap();

                assert!(task_manager
                    .validate_test_task_ticket(ws_ticket.into())
                    .await
                    .is_ok());
            });
        });
    }

    #[test]
    fn test_endpoint_timed_out_ticket() {
        with_async_runtime(|rt| {
            let runtime = rt.clone();

            rt.block_on(async {
                let (task_manager, _) = TaskManager::new(DB.clone(), runtime);
                let test_routes = test_routes(DB.clone(), task_manager.clone());

                let mut form = Form::default();

                let data = Cursor::new("Some data");

                form.add_reader_file_with_mime(
                    "runner",
                    data,
                    "some_runner_file",
                    mime::APPLICATION_OCTET_STREAM,
                );

                let req = form
                    .set_body_convert::<hyper::Body, MultipartBody>(
                        HyperRequest::builder()
                            .method(Method::POST)
                            .header(API_TOKEN_HEADER, &API_TOKEN as &str)
                            .uri("/run"),
                    )
                    .unwrap();

                let res = test_routes.oneshot(req).await.unwrap();

                assert_eq!(res.status(), StatusCode::OK);

                let ws_ticket: String = serde_json::from_reader(
                    hyper::body::to_bytes(res.into_body())
                        .await
                        .unwrap()
                        .as_ref(),
                )
                .unwrap();

                // sleep until ws ticket should be invalid
                tokio::time::sleep(Duration::from_secs(WS_CONNECT_TIMEOUT_SECS + 5)).await;

                assert!(task_manager
                    .validate_test_task_ticket(ws_ticket.into())
                    .await
                    .is_err());
            });
        });
    }
}
