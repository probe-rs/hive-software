//! Webserver request handlers
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use comm_types::auth::{AuthRequest, DbUser};
use comm_types::cbor::Cbor;

use crate::database::keys;
use crate::{database::CborDb, DB};

/// Handles user authentication requests
pub(super) async fn auth_handler(Cbor(request): Cbor<AuthRequest>) {
    todo!();
    check_password(request);
}

/// Re-hashes the provided password and checks it against the userdata in the DB, if the user exists.
///
/// This function only returns an [`Result::Ok`] value if the provided user exists and the provided password is correct.
fn check_password(request: AuthRequest) -> Result<(), ()> {
    let users: Vec<DbUser> = DB
        .credentials_tree
        .c_get(keys::credentials::USERS)
        .unwrap()
        .unwrap();

    if users
        .iter()
        .filter(|user| user.username == request.username)
        .count()
        == 1
    {
        let user = users
            .iter()
            .filter(|user| user.username == request.username)
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

        match hasher.verify_password(request.password.as_bytes(), &db_password_hash) {
            Ok(_) => return Ok(()),
            Err(_) => return Err(()),
        };
    }

    Err(())
}

pub(super) async fn backend_ws_handler() {
    todo!()
}
