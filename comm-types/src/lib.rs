//! # Comm-Types
//! This crate contains all shared types used for communication between Hive applications, as well as communication helpers and trait impls. All communication types in this crate implement the serde [`Serialize`] and [`Deserialize`]

#[cfg(doc)]
use serde::{Deserialize, Serialize};

#[cfg(feature = "hardware")]
pub mod hardware;

#[cfg(feature = "cbor")]
pub mod cbor;

#[cfg(feature = "ipc")]
pub mod ipc;

#[cfg(feature = "results")]
pub mod results;

#[cfg(feature = "auth")]
pub mod auth;
