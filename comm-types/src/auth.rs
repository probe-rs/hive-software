//! Authentication related types
use serde::{Deserialize, Serialize};

/// The representation of a user in the servers database
#[derive(Deserialize, Serialize)]
pub struct DbUser {
    pub username: String,
    pub hash: String,
}

/// The authentication request sent to the server
#[derive(Deserialize, Serialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}
