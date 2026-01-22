//! All requests that can be sent to monitor from runner
//!
//! Each function in this module represents a valid HTTP request which can be sent over IPC to the runner.
//! In case any payload is sent in the body it needs to be serialized as CBOR using [`into_writer()`] function in the respective request function
use axum::body::Bytes;
use axum::http::{Method, Request};
use comm_types::ipc::RUNNER_SOCKET_PATH;
use comm_types::{
    bincode::{BINCODE_MIME, Bincode},
    ipc::IpcMessage,
    test::TestResults,
};
use http_body_util::Full;
use hyper::header;
use hyperlocal::Uri;

use crate::comm::IpcRequest;

pub fn get_probes() -> (IpcRequest, Option<Vec<u8>>) {
    (
        Request::builder()
            .method(Method::GET)
            .header(header::CONTENT_TYPE, BINCODE_MIME)
            .uri(Uri::new(RUNNER_SOCKET_PATH, "/data/probe"))
            .body(Full::new(Bytes::new()))
            .unwrap(),
        None,
    )
}

pub fn get_targets() -> (IpcRequest, Option<Vec<u8>>) {
    (
        Request::builder()
            .method(Method::GET)
            .header(header::CONTENT_TYPE, BINCODE_MIME)
            .uri(Uri::new(RUNNER_SOCKET_PATH, "/data/target"))
            .body(Full::new(Bytes::new()))
            .unwrap(),
        None,
    )
}

pub fn get_defines() -> (IpcRequest, Option<Vec<u8>>) {
    (
        Request::builder()
            .method(Method::GET)
            .header(header::CONTENT_TYPE, BINCODE_MIME)
            .uri(Uri::new(RUNNER_SOCKET_PATH, "/data/defines"))
            .body(Full::new(Bytes::new()))
            .unwrap(),
        None,
    )
}

pub fn get_options() -> (IpcRequest, Option<Vec<u8>>) {
    (
        Request::builder()
            .method(Method::GET)
            .header(header::CONTENT_TYPE, BINCODE_MIME)
            .uri(Uri::new(RUNNER_SOCKET_PATH, "/data/options"))
            .body(Full::new(Bytes::new()))
            .unwrap(),
        None,
    )
}

pub fn post_test_results(results: TestResults) -> (IpcRequest, Option<Vec<u8>>) {
    let body: Vec<u8> = Bincode(IpcMessage::TestResults(Box::new(results))).into();

    (
        Request::builder()
            .method(Method::POST)
            .header(header::CONTENT_TYPE, BINCODE_MIME)
            .uri(Uri::new(RUNNER_SOCKET_PATH, "/runner/results"))
            .body(Full::from(body.clone()))
            .unwrap(),
        Some(body),
    )
}
