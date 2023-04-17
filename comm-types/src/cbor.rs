//! All CBOR helpers and trait implementations used for [`axum`]
use axum::async_trait;
use axum::extract::{FromRequest, FromRequestParts};
use axum::response::{IntoResponse, Response};
use ciborium::de::from_reader;
use ciborium::ser::into_writer;
use http::header::{self, HeaderValue};
use http::request::{Parts, Request};
use hyper::body::Buf;
use hyper::StatusCode;
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

pub const CBOR_MIME: &str = "application/cbor";

/// Error type returned by the server if it fails to parse the request body
#[derive(Debug, Error, Serialize)]
pub enum ServerParseError {
    #[error("Wrong content type header provided. Expecting application/cbor")]
    InvalidHeader,
    #[error("Missing content type header. Expecting application/cbor")]
    MissingHeader,
    #[error("Failed to parse cbor body. Are you sending the correct data types?")]
    InvalidCbor,
    #[error("Invalid cbor body")]
    InvalidBody,
}

impl IntoResponse for ServerParseError {
    fn into_response(self) -> Response {
        let mut bytes: Vec<u8> = vec![];
        into_writer(&self, &mut bytes).expect("failed to serialize the provided response body. Please check your cbor for correctness.");

        (
            StatusCode::BAD_REQUEST,
            [(header::CONTENT_TYPE, HeaderValue::from_static(CBOR_MIME))],
            bytes,
        )
            .into_response()
    }
}

/// Struct which represents a cbor value.
///
/// # Parsing
/// The struct implements [`FromRequest`] to parse a request body as cbor into the provided generic type `T`. If parsing fails due to a bad request the server sends back a [`ServerParseError`]. As this extractor consumes the body it panics if another extractor already consumed the body before.
///
/// # Serializing
/// The struct implements [`IntoResponse`] to send a type `T` as cbor value in a response. Serializing can panic, as this indicates wrong code or library limitations.
pub struct Cbor<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S, axum::body::Body> for Cbor<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ServerParseError;

    async fn from_request(
        req: Request<axum::body::Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();

        // Check content type headers
        CheckContentType::from_request_parts(&mut parts, state).await?;

        // Check and parse body
        let body = hyper::body::aggregate(body)
            .await
            .map_err(|_| ServerParseError::InvalidBody)?;

        match from_reader::<T, _>(body.reader()) {
            Ok(data) => Ok(Cbor(data)),
            Err(_) => Err(ServerParseError::InvalidCbor),
        }
    }
}

impl<T> IntoResponse for Cbor<T>
where
    T: Serialize,
{
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

/// Checks if a request has the correct content type of [`CBOR_MIME`]
struct CheckContentType;

#[async_trait]
impl<S> FromRequestParts<S> for CheckContentType {
    type Rejection = ServerParseError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let content_type = parts.headers.get(header::CONTENT_TYPE);

        if let Some(content_type) = content_type {
            if content_type == CBOR_MIME {
                return Ok(Self);
            }
            Err(ServerParseError::InvalidHeader)
        } else {
            Err(ServerParseError::MissingHeader)
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{header, Method, Request, StatusCode};
    use axum::response::IntoResponse;
    use axum::routing::post;
    use axum::Router;
    use ciborium::ser::into_writer;
    use serde::{Deserialize, Serialize};
    use tower::ServiceExt;

    use crate::cbor::ServerParseError;

    use super::Cbor;

    #[derive(Serialize, Deserialize)]
    enum Animal {
        ZEBRA,
        LION,
        CRAB,
    }

    #[derive(Serialize, Deserialize)]
    struct MockCborData {
        username: String,
        favorite_animal: Animal,
        favorite_numbers: Vec<i64>,
    }

    fn app() -> Router {
        Router::new().route("/", post(mock_cbor_request_handler))
    }

    async fn mock_cbor_request_handler(Cbor(data): Cbor<MockCborData>) -> Cbor<MockCborData> {
        Cbor(data)
    }

    #[tokio::test]
    async fn missing_content_type() {
        let mock_server = app();

        let res = mock_server
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        let res_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(
            res_bytes,
            hyper::body::to_bytes(ServerParseError::MissingHeader.into_response().into_body())
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn wrong_content_type() {
        let mock_server = app();

        let res = mock_server
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        let res_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(
            res_bytes,
            hyper::body::to_bytes(ServerParseError::InvalidHeader.into_response().into_body())
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn invalid_cbor() {
        let mock_server = app();

        let res = mock_server
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/")
                    .header(header::CONTENT_TYPE, "application/cbor")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        let res_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(
            res_bytes,
            hyper::body::to_bytes(ServerParseError::InvalidCbor.into_response().into_body())
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn valid_request() {
        let mock_server = app();

        let data = MockCborData {
            username: "User".to_owned(),
            favorite_animal: Animal::CRAB,
            favorite_numbers: vec![7, 42, 555],
        };

        let mut data_bytes = vec![];
        into_writer(&data, &mut data_bytes).unwrap();

        let res = mock_server
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/")
                    .header(header::CONTENT_TYPE, "application/cbor")
                    .body(Body::from(data_bytes.clone()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        let res_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(res_bytes, data_bytes);
    }
}
