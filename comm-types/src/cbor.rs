//! All CBOR helpers and trait implementations used for [`axum`]
use axum::response::{IntoResponse, Response};
use ciborium::ser::into_writer;
use ciborium::value::Value;
use http::header;
use http::header::HeaderValue;

pub const CBOR_MIME: &str = "application/cbor";

/// A wrapper for [`ciborium::value::Value`] in order to implement axum traits
pub struct CborValue(pub Value);

impl IntoResponse for CborValue {
    fn into_response(self) -> Response {
        let mut bytes: Vec<u8> = vec![];
        into_writer(&self.0, &mut bytes).expect("failed to serialize the provided response body. Please check your cbor for correctness.");

        (
            [(header::CONTENT_TYPE, HeaderValue::from_static(CBOR_MIME))],
            bytes,
        )
            .into_response()
    }
}
