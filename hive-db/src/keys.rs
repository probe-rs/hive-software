//! Typed keys which are used to access values stored inside the DB
//!
//! The keys are designed as such that they do not only hold the key itself but also the type of the stored value.
//! This allows for a safer API as it can guarantee that there are no attempts to read or write invalid types on the DB.
use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Serialize};

/// A key which is used to access data in the database
pub struct Key<T> {
    key: &'static str,
    phantom_data: PhantomData<T>,
}

impl<T> Key<T>
where
    T: Serialize + DeserializeOwned,
{
    pub fn new(key: &'static str) -> Self {
        Self {
            key,
            phantom_data: PhantomData,
        }
    }

    /// Get the inner key
    pub(crate) fn get_key(&self) -> &str {
        self.key
    }
}
