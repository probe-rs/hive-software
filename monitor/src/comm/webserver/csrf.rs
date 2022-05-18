//! Handles CSRF tokens
//!
//! The pattern used to provide csrf protection is stateless and uses a cookie/header approach as used in AngularJs (https://docs.angularjs.org/api/ng/service/$http#cross-site-request-forgery-xsrf-protection)
//!
//! In this approach a csrf token is sent by the server as a cookie. The csrf token in the cookie is then being read by the client-side javascript and appended to each request made to the server as a http header.
//! The Keys where the cookie and the http header csrf token are stored are [`COOKIE_CSRF_TOKEN_KEY`] and [`HEADER_CSRF_TOKEN_KEY`] respectively.
//!
//! If the csrf tokens in the header and the cookie match, the request is considered valid. If not, it is rejected.
//!
//! The csrf cookie is signed by the server to avoid any clientside manipulation. It has a lifetime of [`COOKIE_TTL`] after which a new csrf token has to be issued.
//!
//! # Obtaining a csrf token
//! A csrf token is automatically obtained by requesting a resource on a route protected by this middleware. In this case the first request will always fail, as no cookie with a csrf token is present.
//! Each time a request fails (Except for a bad request, in case the csrf header is missing completely) the middleware sets a new csrf cookie which can then be used by the client to populate the csrf header.
//! It is up to the client to ensure that requests are retried in order to obtain a token before getting an actual response.
use axum::{
    http::{Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use base64::{decode, encode};
use cookie::{time::Duration, SameSite};
use lazy_static::lazy_static;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaChaRng;
use ring::{
    hmac::{self, Key, HMAC_SHA256},
    rand,
};
use thiserror::Error;
use tokio::sync::Mutex;
use tower_cookies::{Cookie, Cookies};

const COOKIE_CSRF_TOKEN_KEY: &str = "CSRF-TOKEN";
const HEADER_CSRF_TOKEN_KEY: &str = "X-CSRF-TOKEN";
/// Cookie lifetime in seconds
const COOKIE_TTL: u64 = 600; // 10min

lazy_static! {
    /// Random cryptographically secure key which is generated during runtime to sign cookies
    static ref COOKIE_SIGNING_KEY: Key = {
        let rng = rand::SystemRandom::new();
        Key::generate(HMAC_SHA256, &rng).unwrap()
    };
    /// ChaCha20 rng which is seeded by os rng
    static ref CHACHA_RNG: Mutex<ChaChaRng> = Mutex::new(ChaChaRng::from_entropy());
}

#[derive(Error, Debug)]
pub(super) enum CsrfError {
    #[error("No csrf cookie provided, reissuing a new csrf cookie")]
    MissingCsrfCookie,
    #[error("Missing csrf header")]
    MissingCsrfHeader,
    #[error("The signature of the csrf cookie is invalid")]
    InvalidCsrfCookie,
    #[error("Csrf tokens do not match")]
    InvalidCsrfToken,
    #[error("Failed to parse csrf header value")]
    InvalidCsrfHeader,
}

impl IntoResponse for CsrfError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            CsrfError::MissingCsrfCookie => StatusCode::FORBIDDEN,
            CsrfError::MissingCsrfHeader => StatusCode::BAD_REQUEST,
            CsrfError::InvalidCsrfToken => StatusCode::FORBIDDEN,
            CsrfError::InvalidCsrfHeader => StatusCode::BAD_REQUEST,
            CsrfError::InvalidCsrfCookie => StatusCode::FORBIDDEN,
        };

        (status, self.to_string()).into_response()
    }
}

/// Middleware function which checks the provided csrf token validity and rejects the request in case it is not valid. In that case a new token is added as a cookie.
pub(super) async fn require_csrf_token<B>(
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, CsrfError> {
    let csrf_header = req.headers().get(HEADER_CSRF_TOKEN_KEY);
    let req_cookies = req
        .extensions()
        .get::<Cookies>()
        .expect("Failed to get extracted cookies. This middleware can only be called after the request cookies have been extracted.");

    let cookie_csrf_token = req_cookies.get(COOKIE_CSRF_TOKEN_KEY);

    let cookie_csrf_token = match cookie_csrf_token {
        Some(cookie) => verify_csrf_cookie(&cookie)?,
        None => {
            // No csrf cookie has been provided, therefore we set a new valid csrf cookie and reject the request.
            add_new_csrf_cookie(req_cookies).await;

            return Err(CsrfError::MissingCsrfCookie);
        }
    };

    match csrf_header {
        Some(csrf_header) => {
            if csrf_header
                .to_str()
                .map_err(|_| CsrfError::InvalidCsrfHeader)?
                == cookie_csrf_token
            {
                // Csrf tokens match
                return Ok(next.run(req).await);
            } else {
                // Csrf tokens do not match. Add a new csrf cookie and reject the request
                add_new_csrf_cookie(req_cookies).await;

                Err(CsrfError::InvalidCsrfToken)
            }
        }
        None => {
            // No csrf header is provided, this is considered a bad request as the header is mandatory in csrf protected routes
            Err(CsrfError::MissingCsrfHeader)
        }
    }
}

/// Adds a new csrf cookie (containing a new csrf token) to the provided cookie jar
async fn add_new_csrf_cookie(cookie_jar: &Cookies) {
    let new_csrf_cookie = Cookie::build(
        COOKIE_CSRF_TOKEN_KEY,
        sign_csrf_token(generate_csrf_token().await),
    )
    .max_age(Duration::seconds(COOKIE_TTL as i64))
    .secure(true)
    .same_site(SameSite::Strict)
    .finish();

    cookie_jar.add(new_csrf_cookie);
}

/// Generates a 32 Byte base64 encoded csrf token, by using [`ChaChaRng`]
async fn generate_csrf_token() -> String {
    let mut csrf_token: [u8; 32] = Default::default();

    let mut rng = CHACHA_RNG.lock().await;
    rng.fill_bytes(&mut csrf_token);
    drop(rng);

    encode(csrf_token)
}

/// Signs the csrf token with HS256 in the format: `<csrf_token>.<tag>`
fn sign_csrf_token(token: String) -> String {
    let tag = hmac::sign(&*COOKIE_SIGNING_KEY, token.as_bytes());
    return format!("{}.{}", token, encode(tag.as_ref()));
}

/// Verifies the csrf cookie value signature and returns the csrf token, if valid
fn verify_csrf_cookie(csrf_cookie: &Cookie) -> Result<String, CsrfError> {
    let signed_token = csrf_cookie.value();

    let parts: Vec<&str> = signed_token.split('.').collect();

    if parts.len() != 2 {
        return Err(CsrfError::InvalidCsrfCookie);
    }

    let token = parts[0];
    let tag = decode(parts[1]).map_err(|_| CsrfError::InvalidCsrfCookie)?;

    hmac::verify(&COOKIE_SIGNING_KEY, token.as_bytes(), &tag)
        .map_err(|_| CsrfError::InvalidCsrfCookie)?;

    Ok(token.to_owned())
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{header, Method, Request, StatusCode};
    use axum::middleware;
    use axum::routing::get;
    use axum::Router;
    use cookie::{Cookie, SameSite};
    use tower::{ServiceBuilder, ServiceExt};
    use tower_cookies::CookieManagerLayer;

    use super::{CsrfError, COOKIE_CSRF_TOKEN_KEY, HEADER_CSRF_TOKEN_KEY};

    fn app() -> Router {
        Router::new().route("/", get(get_handler)).layer(
            ServiceBuilder::new()
                .layer(CookieManagerLayer::new())
                .layer(middleware::from_fn(super::require_csrf_token)),
        )
    }

    async fn get_handler() -> String {
        "passed csrf check".to_owned()
    }

    #[tokio::test]
    async fn signing_cookie() {
        let csrf_token = super::generate_csrf_token().await;

        let signed_token = super::sign_csrf_token(csrf_token.clone());

        let signed_cookie = Cookie::build("signed", signed_token).finish();

        let retrieved_token = super::verify_csrf_cookie(&signed_cookie).unwrap();

        assert_eq!(csrf_token, retrieved_token);
    }

    #[tokio::test]
    async fn signing_cookie_modified() {
        let csrf_token = super::generate_csrf_token().await;
        let csrf_token_modified = super::generate_csrf_token().await;

        let signed_token = super::sign_csrf_token(csrf_token.clone());

        let parts: Vec<&str> = signed_token.split(".").collect();

        let modified_cookie =
            Cookie::build("signed", format!("{}.{}", csrf_token_modified, parts[1])).finish();

        assert!(super::verify_csrf_cookie(&modified_cookie).is_err());
    }

    #[tokio::test]
    async fn missing_cookie() {
        let ipc_server = app();

        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        let body_text = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(
            body_text,
            CsrfError::MissingCsrfCookie.to_string().as_bytes()
        );
    }

    #[tokio::test]
    async fn csrf_cookie_settings() {
        let ipc_server = app();

        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let cookies = res.headers().get(header::SET_COOKIE).unwrap();
        let csrf_cookie = Cookie::parse(cookies.to_str().unwrap()).unwrap();

        assert_eq!(csrf_cookie.same_site(), Some(SameSite::Strict));
        assert!(csrf_cookie.secure().unwrap());
        assert_eq!(COOKIE_CSRF_TOKEN_KEY, csrf_cookie.name());
    }

    #[tokio::test]
    async fn modified_cookie() {
        let ipc_server = app();

        // Initial request which should fail and contain a new csrf cookie
        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let cookies = res.headers().get(header::SET_COOKIE).unwrap().clone();
        let mut csrf_cookie = Cookie::parse(cookies.to_str().unwrap()).unwrap();

        let csrf_token = csrf_cookie.value().to_string();
        // modify the signed cookie value... (In fact we modify the signed value and not the token directly, which is why this test only tests a modified cookie and not a modified token)
        csrf_cookie.set_value(csrf_token.to_ascii_uppercase());

        let ipc_server = app();
        // Second request with appended and modified csrf cookie
        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .header(header::COOKIE, csrf_cookie.to_string())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        let body_text = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(
            body_text,
            CsrfError::InvalidCsrfCookie.to_string().as_bytes()
        );
    }

    #[tokio::test]
    async fn missing_header() {
        let ipc_server = app();

        // Initial request which should fail and contain a new csrf cookie
        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let cookies = res.headers().get(header::SET_COOKIE).unwrap().clone();

        let ipc_server = app();
        // Second request with appended csrf cookie but missing csrf header
        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .header(header::COOKIE, cookies)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
        let body_text = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(
            body_text,
            CsrfError::MissingCsrfHeader.to_string().as_bytes()
        );
    }

    #[tokio::test]
    async fn modified_csrf_token() {
        let ipc_server = app();

        // Initial request which should fail and contain a new csrf cookie
        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let cookies = res.headers().get(header::SET_COOKIE).unwrap().clone();
        let csrf_cookie = Cookie::parse(cookies.to_str().unwrap()).unwrap();

        let ipc_server = app();
        // Second request to get another signed csrf cookie
        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let cookies = res.headers().get(header::SET_COOKIE).unwrap().clone();
        let modified_token_parts: Vec<&str> = csrf_cookie.value().split(".").collect();
        let modified_token = modified_token_parts[0];

        let ipc_server = app();
        // Second request with appended csrf cookie containing the modified token as header
        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    // Cookies from new request
                    .header(header::COOKIE, cookies)
                    // Signed and unmodified csrf token from old cookie
                    .header(HEADER_CSRF_TOKEN_KEY, modified_token)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        let body_text = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(
            body_text,
            CsrfError::InvalidCsrfToken.to_string().as_bytes()
        );
    }

    #[tokio::test]
    async fn correct_csrf_token() {
        let ipc_server = app();

        // Initial request which should fail and contain a new csrf cookie
        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let cookies = res.headers().get(header::SET_COOKIE).unwrap().clone();
        let csrf_cookie = Cookie::parse(cookies.to_str().unwrap()).unwrap();

        let token_parts: Vec<&str> = csrf_cookie.value().split(".").collect();
        let token = token_parts[0];

        let ipc_server = app();
        // Second request with appended csrf cookie containing the signed token as header
        let res = ipc_server
            .oneshot(
                Request::builder()
                    .method(Method::GET)
                    .uri("/")
                    .header(header::COOKIE, cookies.clone())
                    .header(HEADER_CSRF_TOKEN_KEY, token)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(res.status(), StatusCode::OK);
        let body_text = hyper::body::to_bytes(res.into_body()).await.unwrap();
        assert_eq!(body_text, "passed csrf check".to_owned().as_bytes());
    }
}
