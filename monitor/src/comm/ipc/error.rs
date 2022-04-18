//! Error types returned by server as a response in case something went wrong
use axum::response::IntoResponse;
use ciborium::cbor;
use comm_types::cbor::CborValue;
use hyper::StatusCode;

pub(crate) enum ServerError {
    WrongMessageType,
    WrongContentType,
    MissingContentType,
}

impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ServerError::WrongMessageType => (
                StatusCode::BAD_REQUEST,
                CborValue(
                    cbor!("Received an unexpected IpcMessage type on this endpoint").unwrap(),
                ),
            )
                .into_response(),
            ServerError::WrongContentType => (
                StatusCode::BAD_REQUEST,
                CborValue(
                    cbor!("Received a wrong content-type header. Expecting application/cbor")
                        .unwrap(),
                ),
            )
                .into_response(),
            ServerError::MissingContentType => (
                StatusCode::BAD_REQUEST,
                CborValue(
                    cbor!(
                        "Request has no content-type header specified. Expecting application/cbor"
                    )
                    .unwrap(),
                ),
            )
                .into_response(),
        }
    }
}
