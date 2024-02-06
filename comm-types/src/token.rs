//! Token based authorization related types
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Header key value for the API token
pub const API_TOKEN_HEADER: &str = "Authorization";

/// Lifetime of a token
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TokenLifetime {
    Permanent,
    Temporary(DateTime<Utc>),
}

/// Database entry of an API token
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DbToken {
    pub name: String,
    pub description: String,
    pub lifetime: TokenLifetime,
}
