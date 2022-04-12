//! Contains extractors used to extract values out of IPC requests
use axum::async_trait;
use axum::extract::{FromRequest, RequestParts};
use ciborium::de::from_reader;
use comm_types::ipc::IpcMessage;
use hyper::body::Buf;
use hyper::StatusCode;

/// Parses the CBOR body into either of the [`IpcMessage`] variants
pub(crate) struct Cbor(pub IpcMessage);

#[async_trait]
impl FromRequest<axum::body::Body> for Cbor {
    type Rejection = (StatusCode, &'static str);

    async fn from_request(
        req: &mut RequestParts<axum::body::Body>,
    ) -> Result<Self, Self::Rejection> {
        if req.body().is_some() {
            let body = hyper::body::aggregate(req.take_body().unwrap())
                .await
                .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid cbor body"))?;

            match from_reader::<IpcMessage, _>(body.reader()) {
                Ok(message) => Ok(Cbor(message)),
                Err(_) => Err((
                    StatusCode::BAD_REQUEST,
                    "Failed to parse data as IpcMessage",
                )),
            }
        } else {
            Err((
                StatusCode::BAD_REQUEST,
                "Expecting IpcMessage in message body",
            ))
        }
    }
}
