//! Handles test API tokens which are a means to authorize a token holder to use the underlying API
//!
//! Only to be used to secure the test API. This token does not authenticate the user!
//!
//! # Security
//! The tokens only act as a means to control access to the test API which is used to run tests on the Hive testrack.
//! The tokens are currently stored in plain text inside the Hive DB. This is deemed sufficient for now as in case an attacker
//! is able to obtain the DB they are likely able to cause much more damage to the system already than with the stolen API keys.
//!
//! In case this requirement changes there will still be the possibility to store the keys in hashed form (like the user passwords for example).
//!

use std::sync::Arc;

use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use comm_types::token::{API_TOKEN_HEADER, DbToken, TokenLifetime};
use hive_db::{BincodeTransactional, Key};
use hyper::{Request, StatusCode};
use rand::Rng;
use rand::distributions::Alphanumeric;
use rand_chacha::ChaChaRng;
use rand_chacha::rand_core::SeedableRng;
use sled::transaction::{TransactionError, abort};
use thiserror::Error;

use crate::database::MonitorDb;

const API_TOKEN_LENGTH: usize = 64;

#[derive(Debug, Error)]
pub(super) enum TokenError {
    #[error("No API token provided")]
    MissingToken,
    #[error("Invalid API token")]
    InvalidToken,
}

impl IntoResponse for TokenError {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
    }
}

/// Generates a new secure alphanumeric API token with [`API_TOKEN_LENGTH`]
pub(super) fn generate_token() -> String {
    let mut rng = ChaChaRng::from_entropy();

    (0..API_TOKEN_LENGTH)
        .map(|_| rng.sample(Alphanumeric) as char)
        .collect()
}

/// Checks if the provided API token is valid (exists in DB and has not expired)
///
/// # Token expiration
/// In case the token is valid but has a [`TokenLifetime::Temporary`] this function checks against the system time if the token already expired.
/// In that case the token is removed from the DB and the function returns a [`TokenError::InvalidToken`]
fn check_token(db: &MonitorDb, token: &str) -> Result<(), TokenError> {
    let token_valid = db
        .token_tree
        .transaction(|tree| {
            let token_db_key = Key::new(token);

            let db_token: Option<DbToken> = tree.b_get(&token_db_key)?;

            if let Some(db_token) = db_token {
                // Check token lifetime
                match db_token.lifetime {
                    TokenLifetime::Permanent => Ok(true),
                    TokenLifetime::Temporary(expiration_time) => {
                        if expiration_time <= chrono::offset::Utc::now() {
                            // Token expired
                            tree.b_remove(&token_db_key)?;

                            Ok(false)
                        } else {
                            Ok(true)
                        }
                    }
                }
            } else {
                // Token was not found in DB
                abort(TokenError::InvalidToken)
            }
        })
        .map_err(|err| match err {
            TransactionError::Abort(err) => err,
            TransactionError::Storage(err) => {
                panic!("Failed to apply DB transaction to storage: {}", err)
            }
        })?;

    match token_valid {
        true => Ok(()),
        false => Err(TokenError::InvalidToken),
    }
}

/// Implements custom token based authorisation in [`tower_http`] auth middleware.
///
/// The auth token needs to be supplied in the [`API_TOKEN_HEADER`]
pub(super) async fn require_api_token<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, TokenError> {
    let token_header = req
        .headers()
        .get(API_TOKEN_HEADER)
        .ok_or(TokenError::MissingToken)?;

    let api_token = token_header
        .to_str()
        .map_err(|_| TokenError::InvalidToken)?;

    let db = req
        .extensions()
        .get::<Arc<MonitorDb>>()
        .expect("Failed to get monitor DB. This middleware can only be called in routes that provide the MonitorDB via extension. This is a bug, please open an issue.");

    match check_token(db, api_token) {
        Ok(_) => Ok(next.run(req).await),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::Extension;
    use axum::Router;
    use axum::middleware::from_fn;
    use axum::routing::get;
    use comm_types::token::DbToken;
    use comm_types::token::TokenLifetime;
    use hive_db::BincodeDb;
    use hive_db::Key;
    use hyper::Body;
    use hyper::Method;
    use hyper::Request;
    use hyper::StatusCode;
    use lazy_static::lazy_static;
    use tower::{ServiceBuilder, ServiceExt};

    use crate::database::MonitorDb;

    use super::{API_TOKEN_HEADER, TokenError, require_api_token};

    lazy_static! {
        // We open a temporary test database and initialize it to the test values
        static ref DB: Arc<MonitorDb> = {
            let db = MonitorDb::open_test();

            db.token_tree.b_insert(&API_TOKEN_KEY, &DUMMY_TOKEN_DATA).unwrap();

            Arc::new(db)
        };

        static ref API_TOKEN: &'static str = "secretTokenValue";

        static ref API_TOKEN_KEY: Key<'static, DbToken> = Key::new(*API_TOKEN);

        static ref DUMMY_TOKEN_DATA: DbToken = DbToken { name: "my token".to_owned(), description: "some descriptive description".to_owned(), lifetime: TokenLifetime::Permanent };
    }

    fn app() -> Router {
        Router::new().route("/", get(get_handler)).layer(
            ServiceBuilder::new()
                .layer(Extension(DB.clone()))
                .layer(from_fn(require_api_token)),
        )
    }

    async fn get_handler() -> String {
        "successfully passed API token verification".to_owned()
    }

    #[tokio::test]
    async fn token_middleware_valid_permanent() {
        let auth_server = app();

        let res = auth_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .header(API_TOKEN_HEADER, &API_TOKEN as &str)
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        let body_text = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(
            body_text,
            "successfully passed API token verification".as_bytes()
        );
    }

    #[tokio::test]
    async fn token_middleware_valid_temporary() {
        let temp_token = "temporaryToken";
        let expiration_date = chrono::DateTime::parse_from_rfc3339("2077-02-02T15:25:58.000+01:00")
            .map(|dt| dt.into())
            .unwrap();
        let temp_token_data = DbToken {
            name: "I am temporary".to_owned(),
            description: "should get deleted in 2077".to_owned(),
            lifetime: TokenLifetime::Temporary(expiration_date),
        };

        // Insert temporary token into DB
        DB.token_tree
            .b_insert(&Key::new(temp_token), &temp_token_data)
            .unwrap();

        let auth_server = app();

        let res = auth_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .header(API_TOKEN_HEADER, temp_token)
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);

        let body_text = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(
            body_text,
            "successfully passed API token verification".as_bytes()
        );

        // Make sure temp token has not been deleted
        let db_temp_token_data = DB.token_tree.b_get(&Key::new(temp_token)).unwrap();

        assert_eq!(db_temp_token_data, Some(temp_token_data));

        // Cleanup
        DB.token_tree.remove(temp_token).unwrap();
    }

    #[tokio::test]
    async fn token_middleware_no_header() {
        let auth_server = app();

        let res = auth_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        let body_text = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(body_text, TokenError::MissingToken.to_string().as_bytes());
    }

    #[tokio::test]
    async fn token_middleware_invalid_token() {
        let auth_server = app();

        let res = auth_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .header(API_TOKEN_HEADER, "invalidToken")
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        let body_text = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(body_text, TokenError::InvalidToken.to_string().as_bytes());
    }

    #[tokio::test]
    async fn token_middleware_expired_token() {
        let auth_server = app();

        let expired_token = "expiredToken";
        let expiration_date = chrono::DateTime::parse_from_rfc3339("1990-02-02T15:25:58.000+01:00")
            .map(|dt| dt.into())
            .unwrap();
        let expired_token_data = DbToken {
            name: "I am expired".to_owned(),
            description: "should get deleted".to_owned(),
            lifetime: TokenLifetime::Temporary(expiration_date),
        };

        // Insert expired token into DB
        DB.token_tree
            .b_insert(&Key::new(expired_token), &expired_token_data)
            .unwrap();

        let res = auth_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .header(API_TOKEN_HEADER, expired_token)
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        let body_text = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(body_text, TokenError::InvalidToken.to_string().as_bytes());

        // Check if expired token has been deleted from DB
        let expired_token_data = DB
            .token_tree
            .b_get::<DbToken>(&Key::new(expired_token))
            .unwrap();

        assert_eq!(expired_token_data, None);
    }
}
