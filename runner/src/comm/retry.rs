//! Wrappers to make hyper requests retryable
use axum::http::Request;
use comm_types::ipc::{ClientParseError, IpcMessage};
use hyper::{Body, StatusCode, header};
use hyper::{Client, Error as HyperError};
use tokio_retry::RetryIf;
use tokio_retry::strategy::{FibonacciBackoff, jitter};

/// How many times a request using [`try_request()`] can be retried until failure.
///
/// Please note that this is not the only limitation on whether a test is retried or not.
/// The function [`is_retryable_error()`] can abort retries before reaching this limit
const REQ_RETRY_LIMIT: usize = 3;

#[derive(Debug)]
pub enum RequestError {
    BadStatus(StatusCode),
    Parse(ClientParseError),
    Network(HyperError),
}

/// Determines if the provided error should trigger a retry or not.
fn is_retryable_error(err: &RequestError) -> bool {
    match err {
        RequestError::BadStatus(status) => {
            if status.is_server_error() {
                return true;
            }
        }
        RequestError::Network(err) => {
            if err.is_incomplete_message()
                || err.is_parse()
                || err.is_canceled()
                || err.is_connect()
            {
                return true;
            }
        }
        RequestError::Parse(err) => {
            if let ClientParseError::InvalidBody = err {
                return true;
            }
        }
    }

    false
}

/// Tries to request provided resource. This function retries requests if it makes sense and ultimatively returns the parsed [`IpcMessage`] if it succeeds.
///
/// As the requests are consumed, this function internally clones a provided request, and internally calls [`dispatch_request`] which then sends the request and handles the parsing of the response.
///
/// This function should be used in combination with the requests provided in the [`super::requests`] module like follows:
/// ```rust
/// retry::try_request(client, requests::post_test_results(TestResults {}))
/// .await
/// .unwrap();
/// ```
///
/// # Unwrapping
/// As this function already internally retries failed requests the ultimative result should be unwrapped, as the underlying error is likely not recoverable by the application at runtime.
pub async fn try_request<T>(
    client: Client<T, Body>,
    request: (Request<Body>, Option<Vec<u8>>),
) -> Result<IpcMessage, RequestError>
where
    T: 'static + hyper::client::connect::Connect + Clone + Sync + Send,
{
    let retry_strategy = FibonacciBackoff::from_millis(10)
        .map(jitter)
        .take(REQ_RETRY_LIMIT);

    RetryIf::spawn(
        retry_strategy,
        || dispatch_request(&client, &request),
        is_retryable_error,
    )
    .await
}

/// Internal request handler, which might fail and can be retried, as the request is being cloned on every call
async fn dispatch_request<T>(
    client: &Client<T, Body>,
    request: &(Request<Body>, Option<Vec<u8>>),
) -> Result<IpcMessage, RequestError>
where
    T: 'static + hyper::client::connect::Connect + Clone + Sync + Send,
{
    let response = client
        .clone()
        .request(clone_request(&request.0, request.1.as_ref()))
        .await
        .map_err(RequestError::Network)?;

    if response.status().is_success() {
        let message = IpcMessage::from_response(response)
            .await
            .map_err(RequestError::Parse)?;
        Ok(message)
    } else {
        Err(RequestError::BadStatus(response.status()))
    }
}

/// Clones a request including the body, if existing.
///
/// # Panics
/// If the provided request does not have the [`header::CONTENT_TYPE`] set.
fn clone_request(request: &Request<Body>, body: Option<&Vec<u8>>) -> Request<Body> {
    Request::builder()
        .method(request.method())
        .header(
            header::CONTENT_TYPE,
            request
                .headers()
                .get(header::CONTENT_TYPE)
                .expect("Cannot clone request without a content-type header."),
        )
        .uri(request.uri().clone())
        .body((|| {
            if let Some(body) = body {
                return Body::from(body.clone());
            }
            Body::empty()
        })())
        .unwrap()
}
