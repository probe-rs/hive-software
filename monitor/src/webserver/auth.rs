//! Handles user authentication
//!
//! The authentication flow of Hive is quite simple and standard using JWT (JSON web token). The user provides password and username in the request.
//! If the provided CSRF token is correct the provided password is hashed and compared to the user entry in the DB.
//! If the hashes match the server creates a JWT and appends it to the response as a http-only cookie. This is important as this apporach prevents any malicious scripts on the user webpage side to steal the JWT.
//! A small downside of this approach is that the user cannot delete the JWT client side but needs to call an API endpoint on the server so the server can delete it.
//!
//! The provided JWT contains authorization in form of the Role which the user belongs to. This way any subsequent requests with the JWT token can also be authorized based on the  user role stored in the JWT.
//! The JWT is set to expire after [`TOKEN_EXPIRE_TIME_SEC`]
use std::sync::Arc;

use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::{async_trait, extract};
use comm_types::auth::{DbUser, JwtClaims};
use cookie::time::Duration;
use cookie::{Cookie, SameSite};
use hyper::StatusCode;
use jsonwebtoken::{get_current_timestamp, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaChaRng;
use thiserror::Error;
use tower_cookies::Cookies;

use crate::database::{hasher, MonitorDb};

use super::csrf;

#[cfg(doc)]
use comm_types::auth::Role;

const ISSUER: &str = "probe-rs hive";
/// Expire time of the jwt
const TOKEN_EXPIRE_TIME_SEC: u64 = 1800; // 30min
pub const AUTH_COOKIE_KEY: &str = "AUTH";

lazy_static! {
    static ref JWT_SECRET: [u8; 64] = {
        let mut secret: [u8; 64] = [0; 64];
        let mut rng = ChaChaRng::from_entropy();
        rng.fill_bytes(&mut secret);

        secret
    };
}

#[derive(Error, Debug)]
pub(super) enum AuthError {
    #[error("No auth cookie provided")]
    MissingCookie,
    #[error("Invalid auth token")]
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        (StatusCode::UNAUTHORIZED, self.to_string()).into_response()
    }
}

/// Authenticates a user and sets the auth cookie and a new csrf cookie on success and returns [`Ok`]
///
/// Retuns an [`Err`] if authentication fails because of wrong credentials
///
/// # JWT
/// The expire time of the jwt is set to [`TOKEN_EXPIRE_TIME_SEC`]
///
/// # CSRF
/// The expire time of the csrf cookie is set to [`csrf::COOKIE_TTL`]
pub(super) async fn authenticate_user(
    db: Arc<MonitorDb>,
    username: &str,
    password: &str,
    cookies: &Cookies,
) -> Result<DbUser, ()> {
    let username = username.to_string();
    let password = password.to_string();
    let user =
        tokio::task::spawn_blocking(move || hasher::check_password(db, &username, &password))
            .await
            .unwrap()?;

    csrf::add_new_csrf_cookie(cookies).await;

    set_auth_cookie(cookies, generate_jwt(&user, TOKEN_EXPIRE_TIME_SEC));

    Ok(user)
}

/// Refreshes the jwt auth cookie for the provided user.
///
/// # Security
/// This function does not check user credentials at all and simply generates a valid jwt for the provided user. It is intended to be used for authenticated users only, in case they change data which affects the [`JwtClaims`]. For example a username change.
///
/// # JWT
/// The expire time of the jwt is set to [`TOKEN_EXPIRE_TIME_SEC`]
pub(super) fn refresh_auth_token(user: &DbUser, cookies: &Cookies) {
    set_auth_cookie(cookies, generate_jwt(user, TOKEN_EXPIRE_TIME_SEC));
}

/// Sets the auth cookie with the provided jwt.
///
/// # Cookie settings
/// The cookie is `http-only`, `secure` and `same-site strict`. It has a session lifetime and is deleted once the browser closes or the client logs out using [`logout`].
fn set_auth_cookie(cookies: &Cookies, jwt: String) {
    let auth_cookie = Cookie::build(AUTH_COOKIE_KEY, jwt)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Strict)
        .path("/")
        .finish();

    cookies.add(auth_cookie);
}

/// Logs the user out by resetting the jwt auth cookie. This does not invalidate the original jwt in any way. As it is a stateless implementation the jwt is invalidated once its expire time is reached.
pub(super) fn logout(cookies: &Cookies) {
    let expire_cookie = Cookie::build(AUTH_COOKIE_KEY, "")
        .max_age(Duration::seconds(0))
        .path("/")
        .http_only(true)
        .finish();
    cookies.add(expire_cookie)
}

/// Implements custom jwt authentication in [`tower_http`] auth middleware.
///
/// The jwt needs to be supplied in a http-only, secure cookie with key [`AUTH_COOKIE_KEY`]
#[derive(Clone, Copy)]
pub(super) struct HiveAuth;

#[async_trait]
impl<S> extract::FromRequestParts<S> for HiveAuth
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let req_cookies = parts.extensions.get::<Cookies>().expect("Failed to get extracted cookies. This middleware can only be called after the request cookies have been extracted.");

        let auth_cookie = match req_cookies.get(AUTH_COOKIE_KEY) {
            Some(cookie) => cookie,
            None => return Err(AuthError::MissingCookie),
        };

        match check_jwt(auth_cookie.value()) {
            Ok(claims) => {
                parts.extensions.insert(claims);

                Ok(Self)
            }
            Err(_) => Err(AuthError::InvalidToken),
        }
    }
}

/// Generates a new JWT for the provided user which expires after the provided amount in seconds
pub fn generate_jwt(user: &DbUser, expires_in_secs: u64) -> String {
    let claims = JwtClaims {
        iss: ISSUER.to_owned(),
        exp: (get_current_timestamp() + expires_in_secs) as usize,
        username: user.username.to_owned(),
        role: user.role,
    };

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET.as_ref()),
    )
    .unwrap()
}

/// Checks if the provided jwt is valid and returns the contained [`Role`] if it is.
fn check_jwt(token: &str) -> Result<JwtClaims, ()> {
    let mut validator = Validation::new(jsonwebtoken::Algorithm::HS256);
    validator.set_issuer(&[ISSUER]);
    validator.leeway = 0;
    validator.set_required_spec_claims(&["exp", "iss"]);
    validator.validate_exp = true;

    let payload = jsonwebtoken::decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &validator,
    )
    .map_err(|_| ())?;

    Ok(payload.claims)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::middleware::from_extractor;
    use axum::routing::get;
    use axum::Router;
    use comm_types::auth::DbUser;
    use comm_types::auth::JwtClaims;
    use comm_types::auth::Role;
    use cookie::time::Duration;
    use cookie::SameSite;
    use hive_db::BincodeDb;
    use hyper::header;
    use hyper::Body;
    use hyper::Method;
    use hyper::Request;
    use hyper::StatusCode;
    use jsonwebtoken::{get_current_timestamp, EncodingKey, Header};
    use lazy_static::lazy_static;
    use serde::Deserialize;
    use serde::Serialize;
    use tower::{ServiceBuilder, ServiceExt};
    use tower_cookies::Cookie;
    use tower_cookies::CookieManagerLayer;
    use tower_cookies::Cookies;

    use crate::database::{keys, MonitorDb};
    use crate::webserver::csrf;

    use super::check_jwt;
    use super::generate_jwt;
    use super::AuthError;
    use super::HiveAuth;
    use super::AUTH_COOKIE_KEY;
    use super::ISSUER;
    use super::JWT_SECRET;
    use super::TOKEN_EXPIRE_TIME_SEC;

    lazy_static! {
        // We open a temporary test database and initialize it to the test values
        static ref DB: Arc<MonitorDb> = {
            let db = MonitorDb::open_test();

            db.credentials_tree.b_insert(&keys::credentials::USERS, &DUMMY_USERS).unwrap();

            Arc::new(db)
        };

        static ref DUMMY_USERS: Vec<DbUser> = {
            let hash = crate::database::hasher::hash_password("fancy password");
            vec![DbUser { username: "TeyKey1".to_owned(), hash, role: Role::ADMIN }]
        };
    }

    fn app() -> Router {
        Router::new().route("/", get(get_handler)).layer(
            ServiceBuilder::new()
                .layer(CookieManagerLayer::new())
                .layer(from_extractor::<HiveAuth>()),
        )
    }

    async fn get_handler() -> String {
        "successfully passed auth".to_owned()
    }

    #[test]
    fn jwt_expired() {
        let claims = JwtClaims {
            iss: ISSUER.to_owned(),
            exp: (get_current_timestamp() - TOKEN_EXPIRE_TIME_SEC) as usize,
            username: "SomeUser".to_owned(),
            role: Role::ADMIN,
        };

        let new_token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&*JWT_SECRET),
        )
        .unwrap();

        assert!(check_jwt(&new_token).is_err());
    }

    #[test]
    fn jwt_wrong_claims() {
        #[derive(Serialize, Deserialize)]
        struct WrongClaims {
            exp: usize,
            role: Role,
        }

        let claims = WrongClaims {
            exp: (get_current_timestamp() + TOKEN_EXPIRE_TIME_SEC) as usize,
            role: Role::ADMIN,
        };

        let new_token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&*JWT_SECRET),
        )
        .unwrap();

        assert!(check_jwt(&new_token).is_err());
    }

    #[test]
    fn jwt_correct() {
        let claims = JwtClaims {
            iss: ISSUER.to_owned(),
            exp: (get_current_timestamp() + TOKEN_EXPIRE_TIME_SEC) as usize,
            username: "SomeUser".to_owned(),
            role: Role::ADMIN,
        };

        let new_token = jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(&*JWT_SECRET),
        )
        .unwrap();

        let result = check_jwt(&new_token);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), claims);
    }

    #[tokio::test]
    async fn auth_middleware_no_cookie() {
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
        assert_eq!(body_text, AuthError::MissingCookie.to_string().as_bytes());
    }

    #[tokio::test]
    async fn auth_middleware_invalid_jwt() {
        let auth_server = app();

        let jwt = generate_jwt(
            &DbUser {
                username: "Thor".to_owned(),
                hash: "dummy hash".to_owned(),
                role: Role::ADMIN,
            },
            60,
        );

        let auth_cookie = Cookie::build(AUTH_COOKIE_KEY, jwt.to_ascii_uppercase())
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Strict)
            .finish();

        let res = auth_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .header(header::COOKIE, auth_cookie.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        let body_text = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(body_text, AuthError::InvalidToken.to_string().as_bytes());
    }

    #[tokio::test]
    async fn auth_middleware_correct() {
        let auth_server = app();

        let jwt = generate_jwt(
            &DbUser {
                username: "Thor".to_owned(),
                hash: "dummy hash".to_owned(),
                role: Role::ADMIN,
            },
            60,
        );

        let auth_cookie = Cookie::build(AUTH_COOKIE_KEY, jwt).finish();

        let res = auth_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .header(header::COOKIE, auth_cookie.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn authenticate_user_csrf_refresh() {
        let cookie_jar = Cookies::default();
        csrf::add_new_csrf_cookie(&cookie_jar).await;

        let csrf_cookie = cookie_jar.get("CSRF-TOKEN").unwrap();
        let old_csrf_value = csrf_cookie.value();

        let authentification_result =
            super::authenticate_user(DB.clone(), "TeyKey1", "fancy password", &cookie_jar).await;

        assert!(authentification_result.is_ok());

        // After successful auth the old csrf cookie should be replaced by a new one
        assert_ne!(
            old_csrf_value,
            cookie_jar.get("CSRF-TOKEN").unwrap().value()
        );
    }

    #[tokio::test]
    async fn logout_cookie_deletion() {
        let cookie_jar = Cookies::default();

        let jwt = generate_jwt(
            &DbUser {
                username: "Thor".to_owned(),
                hash: "dummy hash".to_owned(),
                role: Role::ADMIN,
            },
            60,
        );

        let auth_cookie = Cookie::build(AUTH_COOKIE_KEY, jwt).finish();
        cookie_jar.add(auth_cookie);

        super::logout(&cookie_jar);

        let expired_cookie = cookie_jar.get(AUTH_COOKIE_KEY).unwrap();
        assert_eq!(expired_cookie.max_age().unwrap(), Duration::new(0, 0));
        assert_eq!(expired_cookie.value(), "");
    }
}
