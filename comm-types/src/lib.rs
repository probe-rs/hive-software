//! # Comm-Types
//! This crate contains all shared types used for communication between Hive applications. All types in this crate implement the serde [`Serialize`] and [`Deserialize`]
#[cfg(doc)]
use serde::{Deserialize, Serialize};

#[cfg(feature = "hardware")]
pub mod hardware;
