//! All requests that can be sent to monitor from runner
//!
//! Each function in this module represents a valid HTTP request which can be sent over IPC to the runner.
//! In case any payload is sent in the body it needs to be serialized as CBOR using [`into_writer()`] function in the respective request function
use axum::http::{Method, Request};
use comm_types::{
    bincode::{Bincode, BINCODE_MIME},
    ipc::IpcMessage,
    test::TestResults,
};
use hyper::{body::Body, header};

pub fn get_probes<B: Body>() -> (Request<B>, Option<Vec<u8>>) {
    (
        Request::builder()
            .method(Method::GET)
            .header(header::CONTENT_TYPE, BINCODE_MIME)
            .uri("http://monitor.sock/data/probe")
            .body(Body::empty())
            .unwrap(),
        None,
    )
}

pub fn get_targets<B: Body>() -> (Request<B>, Option<Vec<u8>>) {
    (
        Request::builder()
            .method(Method::GET)
            .header(header::CONTENT_TYPE, BINCODE_MIME)
            .uri("http://monitor.sock/data/target")
            .body(Body::empty())
            .unwrap(),
        None,
    )
}

pub fn get_defines<B: Body>() -> (Request<B>, Option<Vec<u8>>) {
    (
        Request::builder()
            .method(Method::GET)
            .header(header::CONTENT_TYPE, BINCODE_MIME)
            .uri("http://monitor.sock/data/defines")
            .body(Body::empty())
            .unwrap(),
        None,
    )
}

pub fn get_options<B: Body>() -> (Request<B>, Option<Vec<u8>>) {
    (
        Request::builder()
            .method(Method::GET)
            .header(header::CONTENT_TYPE, BINCODE_MIME)
            .uri("http://monitor.sock/data/options")
            .body(Body::empty())
            .unwrap(),
        None,
    )
}

pub fn post_test_results<B: Body>(results: TestResults) -> (Request<B>, Option<Vec<u8>>) {
    let body: Vec<u8> = Bincode(IpcMessage::TestResults(Box::new(results))).into();

    (
        Request::builder()
            .method(Method::POST)
            .header(header::CONTENT_TYPE, BINCODE_MIME)
            .uri("http://monitor.sock/runner/results")
            .body(Body::from(body.clone()))
            .unwrap(),
        Some(body),
    )
}
