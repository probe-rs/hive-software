//! All requests that can be sent to monitor from runner
use axum::http::{Method, Request};
use ciborium::ser::into_writer;
use comm_types::{cbor::CBOR_MIME, ipc::IpcMessage, test::TestResults};
use hyper::{header, Body};

pub(crate) fn get_probes() -> (Request<Body>, Option<Vec<u8>>) {
    (
        Request::builder()
            .method(Method::GET)
            .header(header::CONTENT_TYPE, CBOR_MIME)
            .uri("http://monitor.sock/data/probe")
            .body(Body::empty())
            .unwrap(),
        None,
    )
}

pub(crate) fn get_targets() -> (Request<Body>, Option<Vec<u8>>) {
    (
        Request::builder()
            .method(Method::GET)
            .header(header::CONTENT_TYPE, CBOR_MIME)
            .uri("http://monitor.sock/data/target")
            .body(Body::empty())
            .unwrap(),
        None,
    )
}

pub(crate) fn get_defines() -> (Request<Body>, Option<Vec<u8>>) {
    (
        Request::builder()
            .method(Method::GET)
            .header(header::CONTENT_TYPE, CBOR_MIME)
            .uri("http://monitor.sock/data/defines")
            .body(Body::empty())
            .unwrap(),
        None,
    )
}

pub(crate) fn post_test_results(results: TestResults) -> (Request<Body>, Option<Vec<u8>>) {
    let mut bytes: Vec<u8> = vec![];
    into_writer(&IpcMessage::TestResults(Box::new(results)), &mut bytes)
        .expect("Failed to serialize TestResults, please check for format correctness.");

    (
        Request::builder()
            .method(Method::POST)
            .header(header::CONTENT_TYPE, CBOR_MIME)
            .uri("http://monitor.sock/runner/results")
            .body(Body::from(bytes.clone()))
            .unwrap(),
        Some(bytes),
    )
}
