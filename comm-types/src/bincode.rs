//! All bincode helpers and trait implementations used for [`axum`]
use axum::async_trait;
use axum::extract::{FromRequest, FromRequestParts};
use axum::response::{IntoResponse, Response};
use bincode::config;
use bincode::serde::{decode_from_std_read, encode_to_vec};
use http::header::{self, HeaderValue};
use http::request::{Parts, Request};
use hyper::body::Buf;
use hyper::StatusCode;
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;

pub const BINCODE_MIME: &str = "application/bincode";

/// Error type returned by the server if it fails to parse the request body
#[derive(Debug, Error, Serialize)]
pub enum ServerParseError {
    #[error("Wrong content type header provided. Expecting application/bincode")]
    InvalidHeader,
    #[error("Missing content type header. Expecting application/bincode")]
    MissingHeader,
    #[error("Failed to parse bincode body. Are you sending the correct data types?")]
    InvalidBincode,
    #[error("Invalid bincode body")]
    InvalidBody,
}

impl IntoResponse for ServerParseError {
    fn into_response(self) -> Response {
        let bytes = encode_to_vec(&self, config::standard()).expect("failed to serialize the provided response body. Please check your bincode for correctness.");

        let status_code = match self {
            ServerParseError::InvalidHeader => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            ServerParseError::MissingHeader => StatusCode::BAD_REQUEST,
            ServerParseError::InvalidBincode => StatusCode::BAD_REQUEST,
            ServerParseError::InvalidBody => StatusCode::BAD_REQUEST,
        };

        (
            status_code,
            [(header::CONTENT_TYPE, HeaderValue::from_static(BINCODE_MIME))],
            bytes,
        )
            .into_response()
    }
}

/// Struct which represents a bincode value.
///
/// # Parsing
/// The struct implements [`FromRequest`] to parse a request body as bincode into the provided generic type `T`. If parsing fails due to a bad request the server sends back a [`ServerParseError`].
///
/// # Serializing
/// The struct implements [`IntoResponse`] to send a type `T` as bincode value in a response. Serializing can panic. This indicates wrong code or library limitations.
pub struct Bincode<T>(pub T);

impl<T> Into<Vec<u8>> for Bincode<T>
where
    T: Serialize,
{
    /// Converts the provided type into a byte vector
    ///
    /// # Panics
    /// If the provided value cannot be encoded to bincode
    fn into(self) -> Vec<u8> {
        encode_to_vec(&self.0, config::standard()).expect("failed to serialize the provided type into a bincode http body. Please check if the provided type can be encoded using bincode.")
    }
}

#[async_trait]
impl<T, S> FromRequest<S, axum::body::Body> for Bincode<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = ServerParseError;

    async fn from_request(
        req: Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Check and parse body
        let body = hyper::body::aggregate(req.into_body())
            .await
            .map_err(|_| ServerParseError::InvalidBody)?;

        match decode_from_std_read::<T, _, _>(&mut body.reader(), config::standard()) {
            Ok(data) => Ok(Bincode(data)),
            Err(_) => Err(ServerParseError::InvalidBincode),
        }
    }
}

impl<T> IntoResponse for Bincode<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let bytes = encode_to_vec(&self.0, config::standard()).expect("failed to serialize the provided response body. Please check your bincode for correctness.");

        (
            [(header::CONTENT_TYPE, HeaderValue::from_static(BINCODE_MIME))],
            bytes,
        )
            .into_response()
    }
}

/// Checks if a request has the correct content type of [`BINCODE_MIME`]
pub struct CheckContentType;

#[async_trait]
impl<S> FromRequestParts<S> for CheckContentType {
    type Rejection = ServerParseError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let content_type = parts.headers.get(header::CONTENT_TYPE);

        if let Some(content_type) = content_type {
            if content_type == BINCODE_MIME {
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
    use bincode::config;
    use bincode::serde::encode_to_vec;

    use serde::{Deserialize, Serialize};
    use tower::ServiceExt;

    use super::{Bincode, ServerParseError, BINCODE_MIME};

    #[derive(Serialize, Deserialize)]
    enum Animal {
        ZEBRA,
        LION,
        CRAB,
    }

    #[derive(Serialize, Deserialize)]
    struct MockBincodeData {
        username: String,
        favorite_animal: Animal,
        favorite_numbers: Vec<i64>,
    }

    fn app() -> Router {
        Router::new().route("/", post(mock_bincode_request_handler))
    }

    async fn mock_bincode_request_handler(
        Bincode(data): Bincode<MockBincodeData>,
    ) -> Bincode<MockBincodeData> {
        Bincode(data)
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
    async fn invalid_bincode() {
        let mock_server = app();

        let res = mock_server
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/")
                    .header(header::CONTENT_TYPE, BINCODE_MIME)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        let res_bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(
            res_bytes,
            hyper::body::to_bytes(ServerParseError::InvalidBincode.into_response().into_body())
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn valid_request() {
        let mock_server = app();

        let data = MockBincodeData {
            username: "User".to_owned(),
            favorite_animal: Animal::CRAB,
            favorite_numbers: vec![7, 42, 555],
        };

        let data_bytes = encode_to_vec(&data, config::standard()).unwrap();

        let res = mock_server
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/")
                    .header(header::CONTENT_TYPE, BINCODE_MIME)
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
