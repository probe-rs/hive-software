//! Handles user authentication
use std::sync::Arc;

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::http::{header, HeaderValue};
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
use tower_http::auth::AuthorizeRequest;

use crate::database::{keys, CborDb, HiveDb};

const ISSUER: &str = "probe-rs hive";
const TOKEN_EXPIRE_TIME: u64 = 30; // 30s

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
/// The jwt needs to be supplied in the authorization header in the form of: `Bearer <jwt>`
#[derive(Clone, Copy)]
pub(super) struct HiveAuth;

impl<B> AuthorizeRequest<B> for HiveAuth {
    type ResponseBody = UnsyncBoxBody<axum::body::Bytes, axum::Error>;

    fn authorize(&mut self, request: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>> {
        let auth_header = request.headers().get(header::AUTHORIZATION);

        if auth_header.is_none() {
            return Err(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(UnsyncBoxBody::default())
                .unwrap());
        }

        let auth_header = auth_header.unwrap();

        match check_jwt(auth_header) {
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

/// Implements custom jwt authentication in [`tower_http`] auth middleware which includes a csrf token check using "double cookie" method.
///
/// The jwt needs to be supplied in the authorization header in the form of: `Bearer <jwt>`
#[derive(Clone, Copy)]
pub(super) struct HiveAuthCsrf;

impl<B> AuthorizeRequest<B> for HiveAuthCsrf {
    type ResponseBody = UnsyncBoxBody<axum::body::Bytes, axum::Error>;

    fn authorize(&mut self, request: &mut Request<B>) -> Result<(), Response<Self::ResponseBody>> {
        let auth_header = request.headers().get(header::AUTHORIZATION);

        if auth_header.is_none() {
            return Err(Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(UnsyncBoxBody::default())
                .unwrap());
        }

        let auth_header = auth_header.unwrap();

        match check_jwt(auth_header) {
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
fn check_jwt(auth_header: &HeaderValue) -> Result<JwtClaims, ()> {
    let auth_header = auth_header.to_str().map_err(|_| ())?;

    let mut parts = auth_header.split_ascii_whitespace();

    let bearer = parts.next();
    let token = parts.next();

    if bearer.is_some() && token.is_some() {
        if bearer.unwrap() == "Bearer" {
            let mut validator = Validation::new(jsonwebtoken::Algorithm::HS256);
            validator.set_issuer(&[ISSUER]);
            validator.leeway = 0;
            validator.set_required_spec_claims(&["exp", "iss"]);
            validator.validate_exp = true;

            let payload = jsonwebtoken::decode::<JwtClaims>(
                token.unwrap(),
                &DecodingKey::from_secret(&*JWT_SECRET),
                &validator,
            )
            .map_err(|_| ())?;

            return Ok(payload.claims);
        }
    }

    Err(())
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;
    use comm_types::auth::JwtClaims;
    use comm_types::auth::Role;
    use jsonwebtoken::{get_current_timestamp, EncodingKey, Header};
    use serde::Deserialize;
    use serde::Serialize;

    use super::check_jwt;
    use super::ISSUER;
    use super::JWT_SECRET;
    use super::TOKEN_EXPIRE_TIME;

    #[test]
    fn jwt_wrong_auth_header() {
        let auth_header = HeaderValue::from_str("Basic ABCDE").unwrap();
        assert!(check_jwt(&auth_header).is_err());

        let auth_header = HeaderValue::from_str("").unwrap();
        assert!(check_jwt(&auth_header).is_err());

        let auth_header = HeaderValue::from_str("Bearer ABCDE").unwrap();
        assert!(check_jwt(&auth_header).is_err());
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

        let auth_header = HeaderValue::from_str(&format!("Bearer {}", new_token)).unwrap();

        assert!(check_jwt(&auth_header).is_err());
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

        let auth_header = HeaderValue::from_str(&format!("Bearer {}", new_token)).unwrap();

        assert!(check_jwt(&auth_header).is_err());
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

        let auth_header = HeaderValue::from_str(&format!("Bearer {}", new_token)).unwrap();

        let result = check_jwt(&auth_header);

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), claims);
    }
}
