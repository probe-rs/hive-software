//! Test endpoint
use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Extension, Router};
use tower::ServiceBuilder;

use crate::database::MonitorDb;
use crate::tasks::TaskManager;

mod handlers;

pub(super) fn test_routes(db: Arc<MonitorDb>, task_manager: Arc<TaskManager>) -> Router {
    Router::new()
        .route("/capabilities", get(handlers::capabilities))
        .route("/run", post(handlers::test))
        .route("/socket", get(handlers::ws_handler))
        .layer(
            ServiceBuilder::new()
                .layer(Extension(db))
                .layer(Extension(task_manager)),
        )
}
/*
#[cfg(test)]
mod tests {
    use std::io::Cursor;
    use std::sync::Arc;

    use axum::body::Body;
    use axum::http::{header, Method, Request, StatusCode};
    use comm_types::hardware::{Capabilities, ProbeInfo, ProbeState, TargetInfo, TargetState};
    use comm_types::ipc::{HiveProbeData, HiveTargetData};
    use comm_types::test::{TestOptions, TestResults, TestRunStatus};
    use hive_db::CborDb;
    use hyper::Request as HyperRequest;
    use lazy_static::lazy_static;
    use multipart::client::multipart::{Body as MultipartBody, Form};
    use tokio::sync::mpsc;
    use tokio::sync::mpsc::{Receiver, Sender};
    use tower::ServiceExt;

    use crate::database::{keys, MonitorDb};
    use crate::tasks::TestTask;

    use super::test_routes;

    lazy_static! {
        // We open a temporary test database and initialize it to the test values
        static ref DB: Arc<MonitorDb> = {
            let db = MonitorDb::open_test();

            db.config_tree.c_insert(&keys::config::ASSIGNED_PROBES, &PROBE_DATA).unwrap();
            db.config_tree.c_insert(&keys::config::ASSIGNED_TARGETS, &TARGET_DATA).unwrap();

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
    }

    /// A mock implementation of the actual TestManager struct
    struct TestManagerMock {
        task_sender: Sender<TestTask>,
        task_receiver: Receiver<TestTask>,
    }

    impl TestManagerMock {
        pub fn new() -> Self {
            let (task_sender, task_receiver) = mpsc::channel(1);

            Self {
                task_sender,
                task_receiver,
            }
        }

        pub fn get_task_sender(&self) -> Sender<TestTask> {
            self.task_sender.clone()
        }

        /// Tries to receive a test task and sends an empty [`TestResults`] struct back to the task creator via the provided oneshot channel
        pub async fn receive_test_task(&mut self) {
            if let Some(task) = self.task_receiver.recv().await {
                let dummy_results = TestResults {
                    status: TestRunStatus::Ok,
                    results: Some(vec![]),
                    error: None,
                };
                task.result_sender.send(dummy_results).expect("Failed to send dummy test results via oneshot channel. Has the receiver been dropped?");
            } else {
                panic!("The task receiver failed to receive any value as it was considered closed or no messages can be received anymore because all senders have been dropped.");
            }
        }
    }

    #[tokio::test]
    async fn capabilities_endpoint() {
        let test_manager_mock = TestManagerMock::new();
        let test_routes = test_routes(DB.clone(), test_manager_mock.get_task_sender());

        let res = test_routes
            .oneshot(
                Request::builder()
                    .method(Method::GET)
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
    }

    #[tokio::test]
    async fn test_endpoint_wrong_content_type() {
        let test_manager_mock = TestManagerMock::new();
        let test_routes = test_routes(DB.clone(), test_manager_mock.get_task_sender());

        let res = test_routes
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/run")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::CONTENT_LENGTH, "0")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_endpoint_content_length() {
        let test_manager_mock = TestManagerMock::new();
        let test_routes = test_routes(DB.clone(), test_manager_mock.get_task_sender());

        let res = test_routes
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/run")
                    .header(header::CONTENT_TYPE, "multipart/form-data")
                    .header(header::CONTENT_LENGTH, "100000000")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn test_endpoint_unknown_field_name() {
        let test_manager_mock = TestManagerMock::new();
        let test_routes = test_routes(DB.clone(), test_manager_mock.get_task_sender());

        let mut form = Form::default();
        form.add_text("unknown", "some text");

        // TODO This whole thing looks like a really whack way of transforming the MultipartBody into a Hyper Body.
        // It appears the From impl fails to transform it when directly inserting the initial req object into the oneshot

        let req = form
            .set_body::<MultipartBody>(
                HyperRequest::builder()
                    .method(Method::POST)
                    .uri("/run")
                    .header(header::CONTENT_LENGTH, "0"),
            )
            .unwrap();

        let (parts, body) = req.into_parts();

        let req = HyperRequest::from_parts(parts, body.into());

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
    }

    #[tokio::test]
    async fn test_endpoint_missing_project_field() {
        let test_manager_mock = TestManagerMock::new();
        let test_routes = test_routes(DB.clone(), test_manager_mock.get_task_sender());

        let options = TestOptions {};

        let mut form = Form::default();
        form.add_text("options", serde_json::to_string(&options).unwrap());

        // TODO This whole thing looks like a really whack way of transforming the MultipartBody into a Hyper Body.
        // It appears the From impl fails to transform it when directly inserting the initial req object into the oneshot

        let req = form
            .set_body::<MultipartBody>(
                HyperRequest::builder()
                    .method(Method::POST)
                    .uri("/run")
                    .header(header::CONTENT_LENGTH, "0"),
            )
            .unwrap();

        let (parts, body) = req.into_parts();

        let req = HyperRequest::from_parts(parts, body.into());

        let res = test_routes.oneshot(req).await.unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        assert_eq!(
            "No project tar archive provided to perform the tests on. The field 'project' is missing.",
            String::from_utf8_lossy(
                hyper::body::to_bytes(res.into_body())
                    .await
                    .unwrap()
                    .as_ref()
            )
        );
    }

    #[tokio::test]
    async fn test_endpoint_wrong_project_field_data_type() {
        let test_manager_mock = TestManagerMock::new();
        let test_routes = test_routes(DB.clone(), test_manager_mock.get_task_sender());

        let mut form = Form::default();

        let data = Cursor::new("Some data");

        form.add_reader_file_with_mime(
            "project",
            data,
            "some_tar_file.tar",
            mime::APPLICATION_JSON,
        );

        // TODO This whole thing looks like a really whack way of transforming the MultipartBody into a Hyper Body.
        // It appears the From impl fails to transform it when directly inserting the initial req object into the oneshot

        let req = form
            .set_body::<MultipartBody>(
                HyperRequest::builder()
                    .method(Method::POST)
                    .uri("/run")
                    .header(header::CONTENT_LENGTH, "0"),
            )
            .unwrap();

        let (parts, body) = req.into_parts();

        let req = HyperRequest::from_parts(parts, body.into());

        let res = test_routes.oneshot(req).await.unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        assert_eq!(
            "Invalid file format provided for field 'project'. Expecting tar archive.",
            String::from_utf8_lossy(
                hyper::body::to_bytes(res.into_body())
                    .await
                    .unwrap()
                    .as_ref()
            )
        );
    }

    #[tokio::test]
    async fn test_endpoint_wrong_project_field_file_type() {
        let test_manager_mock = TestManagerMock::new();
        let test_routes = test_routes(DB.clone(), test_manager_mock.get_task_sender());

        let mut form = Form::default();

        let data = Cursor::new("Some data");

        form.add_reader_file_with_mime(
            "project",
            data,
            "some_wrong_file.wrong",
            mime::APPLICATION_OCTET_STREAM,
        );

        // TODO This whole thing looks like a really whack way of transforming the MultipartBody into a Hyper Body.
        // It appears the From impl fails to transform it when directly inserting the initial req object into the oneshot

        let req = form
            .set_body::<MultipartBody>(
                HyperRequest::builder()
                    .method(Method::POST)
                    .uri("/run")
                    .header(header::CONTENT_LENGTH, "0"),
            )
            .unwrap();

        let (parts, body) = req.into_parts();

        let req = HyperRequest::from_parts(parts, body.into());

        let res = test_routes.oneshot(req).await.unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        assert_eq!(
            "Invalid file format provided for field 'project'. Expecting tar archive.",
            String::from_utf8_lossy(
                hyper::body::to_bytes(res.into_body())
                    .await
                    .unwrap()
                    .as_ref()
            )
        );
    }

    #[tokio::test]
    async fn test_endpoint_correct() {
        let mut test_manager_mock = TestManagerMock::new();
        let test_routes = test_routes(DB.clone(), test_manager_mock.get_task_sender());

        let mut form = Form::default();

        let data = Cursor::new("Some data");

        form.add_reader_file_with_mime(
            "project",
            data,
            "some_tar_file.tar",
            mime::APPLICATION_OCTET_STREAM,
        );

        // TODO This whole thing looks like a really whack way of transforming the MultipartBody into a Hyper Body.
        // It appears the From impl fails to transform it when directly inserting the initial req object into the oneshot

        let req = form
            .set_body::<MultipartBody>(
                HyperRequest::builder()
                    .method(Method::POST)
                    .uri("/run")
                    .header(header::CONTENT_LENGTH, "0"),
            )
            .unwrap();

        let (parts, body) = req.into_parts();

        let req = HyperRequest::from_parts(parts, body.into());

        // We instruct the testmanager mock to return data to the request, in case it received a request
        // This needs to happen in a separate task as we only receive a valid response if the testmanager
        // is ready to handle tasks
        tokio::spawn(async move { test_manager_mock.receive_test_task().await });

        let res = test_routes.oneshot(req).await.unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        let results: TestResults = serde_json::from_reader(
            hyper::body::to_bytes(res.into_body())
                .await
                .unwrap()
                .as_ref(),
        )
        .unwrap();

        assert_eq!(results.status, TestRunStatus::Ok);
        assert!(results.results.unwrap().is_empty());
    }
}*/
