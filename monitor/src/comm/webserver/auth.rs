//! Handles user authentication
use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::response::Response;
use axum::Extension;
use comm_types::auth::{AuthRequest, AuthResponse, DbUser, JwtClaims};
use comm_types::cbor::Cbor;
use http_body::combinators::UnsyncBoxBody;
use hyper::{Request, StatusCode};
use jsonwebtoken::{get_current_timestamp, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use rand_chacha::rand_core::{RngCore, SeedableRng};
use rand_chacha::ChaChaRng;
use tower_cookies::Cookies;
use tower_http::auth::AuthorizeRequest;

use crate::database::{keys, CborDb, HiveDb};

const ISSUER: &str = "probe-rs hive";
const TOKEN_EXPIRE_TIME: u64 = 30; // 30s
pub(crate) const AUTH_COOKIE_KEY: &str = "AUTH";

lazy_static! {
    static ref JWT_SECRET: [u8; 64] = {
        let mut secret: [u8; 64] = [0; 64];
        let mut rng = ChaChaRng::from_entropy();
        rng.fill_bytes(&mut secret);

        secret
    };
}

/// Handles user authentication requests to obtain ws connection.
///
/// If authentication is successful it returns a JWT which contains the user role and is valid for [`TOKEN_EXPIRE_TIME`]. Which is then used by the client to open a websocket connection.
pub(super) async fn ws_auth_handler(
    Extension(db): Extension<Arc<HiveDb>>,
    Cbor(request): Cbor<AuthRequest>,
) -> Result<Cbor<AuthResponse>, StatusCode> {
    let user = check_password(db, request.username, request.password)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let token = generate_jwt(user, TOKEN_EXPIRE_TIME);

    Ok(Cbor(AuthResponse { token }))
}

/// Implements custom jwt authentication in [`tower_http`] auth middleware.
///
/// The jwt needs to be supplied in a http-only, secure cookie with key [`AUTH_COOKIE_KEY`]
#[derive(Clone, Copy)]
pub(super) struct HiveAuth;

impl<B> AuthorizeRequest<B> for HiveAuth {
    type ResponseBody = UnsyncBoxBody<axum::body::Bytes, axum::Error>;

    fn authorize(&mut self, request: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>> {
        let req_cookies = request
        .extensions()
        .get::<Cookies>()
        .expect("Failed to get extracted cookies. This middleware can only be called after the request cookies have been extracted.");

        let auth_cookie = match req_cookies.get(AUTH_COOKIE_KEY) {
            Some(cookie) => cookie,
            None => {
                return Err(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body(UnsyncBoxBody::default())
                    .unwrap())
            }
        };

        match check_jwt(auth_cookie.value()) {
            Ok(claims) => {
                request.extensions_mut().insert(claims);

                return Ok(());
            }
            Err(_) => {
                return Err(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(UnsyncBoxBody::default())
                    .unwrap());
            }
        }
    }
}

/// Generates a new JWT for the provided user which expires after the provided amount in seconds
pub(crate) fn generate_jwt(user: DbUser, expires_in_secs: u64) -> String {
    let claims = JwtClaims {
        iss: ISSUER.to_owned(),
        exp: (get_current_timestamp() + expires_in_secs) as usize,
        username: user.username,
        role: user.role,
    };

    jsonwebtoken::encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&*JWT_SECRET),
    )
    .unwrap()
}

/// Re-hashes the provided password and checks it against the userdata in the DB, if the user exists.
///
/// This function only returns an [`Result::Ok`] value with the authenticated user if the provided user exists and the provided password is correct.
pub(crate) fn check_password(
    db: Arc<HiveDb>,
    username: String,
    password: String,
) -> Result<DbUser, ()> {
    let users: Vec<DbUser> = db
        .credentials_tree
        .c_get(keys::credentials::USERS)
        .unwrap()
        .unwrap();

    if users
        .iter()
        .filter(|user| user.username == username)
        .count()
        == 1
    {
        let user = users
            .into_iter()
            .filter(|user| user.username == username)
            .next()
            .unwrap();

        let hasher = Argon2::default();

        // Parse PHC string from DB
        let db_password_hash = match PasswordHash::new(&user.hash) {
            Ok(hash) => hash,
            Err(err) => {
                log::warn!("Failed to parse the user password hash from the DB. This might be caused by a corrupted DB: {}", err);
                return Err(());
            }
        };

        match hasher.verify_password(password.as_bytes(), &db_password_hash) {
            Ok(_) => return Ok(user),
            Err(_) => return Err(()),
        };
    }

    Err(())
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
        &DecodingKey::from_secret(&*JWT_SECRET),
        &validator,
    )
    .map_err(|_| ())?;

    Ok(payload.claims)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use argon2::password_hash::SaltString;
    use argon2::Argon2;
    use argon2::PasswordHasher;
    use axum::routing::get;
    use axum::Router;
    use comm_types::auth::DbUser;
    use comm_types::auth::JwtClaims;
    use comm_types::auth::Role;
    use cookie::SameSite;
    use hyper::header;
    use hyper::Body;
    use hyper::Method;
    use hyper::Request;
    use hyper::StatusCode;
    use jsonwebtoken::{get_current_timestamp, EncodingKey, Header};
    use lazy_static::lazy_static;
    use rand_chacha::rand_core::OsRng;
    use serde::Deserialize;
    use serde::Serialize;
    use tower::{ServiceBuilder, ServiceExt};
    use tower_cookies::Cookie;
    use tower_cookies::CookieManagerLayer;
    use tower_http::auth::RequireAuthorizationLayer;

    use crate::database::{keys, CborDb, HiveDb};

    use super::check_jwt;
    use super::check_password;
    use super::generate_jwt;
    use super::HiveAuth;
    use super::AUTH_COOKIE_KEY;
    use super::ISSUER;
    use super::JWT_SECRET;
    use super::TOKEN_EXPIRE_TIME;

    lazy_static! {
        static ref DB: Arc<HiveDb> = Arc::new(HiveDb::open_test());
    }

    fn app() -> Router {
        Router::new().route("/", get(get_handler)).layer(
            ServiceBuilder::new()
                .layer(CookieManagerLayer::new())
                .layer(RequireAuthorizationLayer::custom(HiveAuth)),
        )
    }

    async fn get_handler() -> String {
        "successfully passed auth".to_owned()
    }

    #[test]
    fn jwt_expired() {
        let claims = JwtClaims {
            iss: ISSUER.to_owned(),
            exp: (get_current_timestamp() - TOKEN_EXPIRE_TIME) as usize,
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
            exp: (get_current_timestamp() + TOKEN_EXPIRE_TIME) as usize,
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
            exp: (get_current_timestamp() + TOKEN_EXPIRE_TIME) as usize,
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

    #[test]
    fn check_password_wrong_username() {
        let dummy_users = vec![
            DbUser {
                username: "Geralt".to_owned(),
                hash: "dummy hash".to_owned(),
                role: Role::ADMIN,
            },
            DbUser {
                username: "Ciri".to_owned(),
                hash: "dummy hash".to_owned(),
                role: Role::ADMIN,
            },
            DbUser {
                username: "Vesemir".to_owned(),
                hash: "dummy hash".to_owned(),
                role: Role::ADMIN,
            },
        ];

        DB.credentials_tree
            .c_insert(keys::credentials::USERS, &dummy_users)
            .unwrap();

        assert!(
            check_password(DB.clone(), "Yarpen".to_owned(), "dummy password".to_owned()).is_err()
        );
    }

    #[test]
    fn check_password_invalid_hash() {
        let dummy_users = vec![DbUser {
            username: "Aloy".to_owned(),
            hash: "This is not a PHC hash string".to_owned(),
            role: Role::ADMIN,
        }];

        DB.credentials_tree
            .c_insert(keys::credentials::USERS, &dummy_users)
            .unwrap();

        assert!(
            check_password(DB.clone(), "Aloy".to_owned(), "dummy password".to_owned()).is_err()
        );
    }

    #[test]
    fn check_password_wrong_password() {
        let password = "Very strong password";

        let hasher = Argon2::default();

        let salt = SaltString::generate(&mut OsRng);

        let hash = hasher.hash_password(password.as_bytes(), &salt).unwrap();

        let dummy_users = vec![DbUser {
            username: "Arthur Morgan".to_owned(),
            hash: hash.to_string(),
            role: Role::ADMIN,
        }];

        DB.credentials_tree
            .c_insert(keys::credentials::USERS, &dummy_users)
            .unwrap();

        assert!(check_password(
            DB.clone(),
            "Arthur Morgan".to_owned(),
            "Very wrong password".to_owned()
        )
        .is_err());
    }

    #[test]
    fn check_password_correct() {
        let password = "Very strong password";

        let hasher = Argon2::default();

        let salt = SaltString::generate(&mut OsRng);

        let hash = hasher.hash_password(password.as_bytes(), &salt).unwrap();

        let dummy_users = vec![DbUser {
            username: "Arthur Morgan".to_owned(),
            hash: hash.to_string(),
            role: Role::ADMIN,
        }];

        DB.credentials_tree
            .c_insert(keys::credentials::USERS, &dummy_users)
            .unwrap();

        assert!(check_password(
            DB.clone(),
            "Arthur Morgan".to_owned(),
            "Very strong password".to_owned()
        )
        .is_ok());
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

        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn auth_middleware_invalid_jwt() {
        let auth_server = app();

        let jwt = generate_jwt(
            DbUser {
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
    }

    #[tokio::test]
    async fn auth_middleware_correct() {
        let auth_server = app();

        let jwt = generate_jwt(
            DbUser {
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
}
