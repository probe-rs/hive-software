//! Authentication related types
use async_graphql::{Context, Enum, Guard, Result};
use serde::{Deserialize, Serialize};

/// The possible roles a user can have
#[derive(Debug, Clone, Copy, Eq, Deserialize, Serialize, PartialEq, Enum, PartialOrd)]
pub enum Role {
    /// Can create / delete users
    ADMIN = 0,
    MAINTAINER = 1,
}

impl Guard for Role {
    async fn check(&self, ctx: &Context<'_>) -> Result<()> {
        if let Some(claims) = ctx.data_opt::<JwtClaims>() {
            if self >= &claims.role {
                return Ok(());
            }
        }

        Err("Insufficient permission".into())
    }
}

/// The representation of a user in the servers database
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct DbUser {
    pub username: String,
    pub hash: String,
    pub role: Role,
}

/// The authentication request sent to the server
#[derive(Deserialize, Serialize)]
pub struct AuthRequest {
    pub username: String,
    pub password: String,
}

/// The authentication response sent by the server if the [`AuthRequest`] was valid and accepted
#[derive(Deserialize, Serialize)]
pub struct AuthResponse {
    /// The JWT generated by the server
    pub token: String,
}

/// Claims used in a Hive JWT
#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
pub struct JwtClaims {
    pub iss: String,
    pub exp: usize,
    pub username: String,
    pub role: Role,
}
