//! Custom middleware used by the server
use axum::async_trait;
use axum::extract::{FromRequest, RequestParts};
use comm_types::cbor::CBOR_MIME;
use hyper::header;
use hyper::StatusCode;

/// Checks if a request has the correct content type of [`CBOR_MIME`]
pub(crate) struct CheckContentType;

#[async_trait]
impl<B> FromRequest<B> for CheckContentType
where
    B: Send,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let content_type = req.headers().get(header::CONTENT_TYPE);

        if let Some(content_type) = content_type {
            if content_type == CBOR_MIME {
                return Ok(Self);
            }
            Err((
                StatusCode::BAD_REQUEST,
                "Wrong content type header provided. Expecting application/cbor",
            ))
        } else {
            Err((
                StatusCode::BAD_REQUEST,
                "Missing content type header. Expecting application/cbor",
            ))
        }
    }
}
