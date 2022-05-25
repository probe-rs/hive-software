//! Password hashing functions used to check and create new hashes
use std::sync::Arc;

use argon2::{
    password_hash::SaltString, Algorithm, Argon2, Params, PasswordHash, PasswordHasher,
    PasswordVerifier, Version,
};
use comm_types::auth::DbUser;
use lazy_static::lazy_static;
use rand_chacha::rand_core::OsRng;

use super::{keys, CborDb, HiveDb};

lazy_static! {
    static ref HASHER_SETTINGS: Params =
        Params::new(8192, 3, 1, Some(Params::DEFAULT_OUTPUT_LEN)).unwrap();
}

/// Re-hashes the provided password and checks it against the userdata in the DB, if the user exists.
///
/// This function only returns an [`Result::Ok`] value with the authenticated user if the provided user exists and the provided password is correct.
pub(crate) fn check_password(
    db: Arc<HiveDb>,
    username: &str,
    password: &str,
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
            .find(|user| user.username == username)
            .unwrap();

        let hasher = Argon2::new(
            Algorithm::default(),
            Version::default(),
            HASHER_SETTINGS.clone(),
        );

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

/// Hashes the provided password and returns the PHC-String containing the hashed password
pub(crate) fn hash_password(password: &str) -> String {
    let hasher = Argon2::new(
        Algorithm::default(),
        Version::default(),
        HASHER_SETTINGS.clone(),
    );

    let salt = SaltString::generate(&mut OsRng);

    hasher
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use argon2::password_hash::SaltString;
    use argon2::Argon2;
    use argon2::PasswordHasher;
    use comm_types::auth::DbUser;
    use comm_types::auth::Role;
    use lazy_static::lazy_static;
    use rand_chacha::rand_core::OsRng;

    use crate::database::{keys, CborDb, HiveDb};

    use super::check_password;

    lazy_static! {
        static ref DB: Arc<HiveDb> = Arc::new(HiveDb::open_test());
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

        assert!(check_password(DB.clone(), "Yarpen", "dummy password").is_err());
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

        assert!(check_password(DB.clone(), "Aloy", "dummy password").is_err());
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

        assert!(check_password(DB.clone(), "Arthur Morgan", "Very wrong password").is_err());
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

        assert!(check_password(DB.clone(), "Arthur Morgan", "Very strong password").is_ok());
    }
}
