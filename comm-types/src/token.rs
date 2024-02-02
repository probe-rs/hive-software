//! Token based authorization related types
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
