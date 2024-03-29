//! Traits and respective implementations to allow data (de)serialization before reading/writing from the DB
use bincode::config;
use bincode::serde::{decode_from_slice, encode_to_vec};
use serde::de::DeserializeOwned;
use serde::Serialize;
use sled::transaction::{TransactionalTree, UnabortableTransactionError};
use sled::{Iter, Result as SledResult, Tree};

use crate::keys::Key;

/// Functions which allow the DB to operate on bincode values (Serializing/Deserializing) on each DB call.
pub trait BincodeDb {
    fn b_insert<T>(&self, key: &Key<T>, value: &T) -> SledResult<Option<T>>
    where
        T: Serialize + DeserializeOwned;

    fn b_get<T>(&self, key: &Key<T>) -> SledResult<Option<T>>
    where
        T: Serialize + DeserializeOwned;

    fn b_remove<T>(&self, key: &Key<T>) -> SledResult<Option<T>>
    where
        T: Serialize + DeserializeOwned;

    fn b_iter<T>(&self) -> impl Iterator<Item = SledResult<(String, T)>>
    where
        T: Serialize + DeserializeOwned;
}

/// Functions which allow the user to operate on bincode values (Serializing/Deserializing) within a transaction.
pub trait BincodeTransactional {
    fn b_insert<T>(
        &self,
        key: &Key<T>,
        value: &T,
    ) -> Result<Option<T>, UnabortableTransactionError>
    where
        T: Serialize + DeserializeOwned;

    fn b_get<T>(&self, key: &Key<T>) -> Result<Option<T>, UnabortableTransactionError>
    where
        T: Serialize + DeserializeOwned;

    fn b_remove<T>(&self, key: &Key<T>) -> Result<Option<T>, UnabortableTransactionError>
    where
        T: Serialize + DeserializeOwned;
}

/// Functions which allow the DB to operate on bincode values (Serializing/Deserializing) on each DB iteration.
pub trait BincodeIter {
    fn b_values<T>(self) -> impl DoubleEndedIterator<Item = SledResult<T>> + Send + Sync
    where
        T: Serialize + DeserializeOwned;

    fn b_keys(self) -> impl DoubleEndedIterator<Item = SledResult<String>> + Send + Sync;
}

impl BincodeIter for Iter {
    /// Like [`Iter::values()`] but deserializes the values from bincode
    ///
    /// # Panics
    /// In case the data received from the DB cannot be deserialized to the expected data type.
    fn b_values<T>(self) -> impl DoubleEndedIterator<Item = SledResult<T>> + Send + Sync
    where
        T: Serialize + DeserializeOwned,
    {
        self.values().map(|val| {
            val.map(|val| {
                let (val, _): (T, _) = decode_from_slice(val.as_ref(), config::standard())
                    .unwrap_or_else(|err| {
                        panic!(
                            "Failed to deserialize the existing DB value to bincode: {}",
                            err
                        )
                    });
                val
            })
        })
    }

    /// Like [`Iter::keys()`] but deserializes the keys to [`String`]
    ///
    /// # Panics
    /// In case the keys received from the DB cannot be deserialized to UTF-8 [`String`].
    fn b_keys(self) -> impl DoubleEndedIterator<Item = SledResult<String>> + Send + Sync {
        self.keys().map(|key| {
            key.map(|key| {
                std::str::from_utf8(key.as_ref())
                    .expect("Key received from DB could not be converted to UTF-8")
                    .to_owned()
            })
        })
    }
}

impl BincodeDb for Tree {
    /// Like [`Tree::insert()`], but serializes the value to bincode
    ///
    /// # Panics
    /// In case the provided data cannot be serialized, or the retrieved old data cannot be deserialized into the expected data type of the key
    fn b_insert<T>(&self, key: &Key<T>, value: &T) -> SledResult<Option<T>>
    where
        T: Serialize + DeserializeOwned,
    {
        let bytes = encode_to_vec(value, config::standard()).unwrap_or_else(|err| {
            panic!("Failed to serialize the provided value to bincode: {}", err)
        });

        let prev_val = self.insert(key.get_key(), bytes)?;

        if let Some(val) = prev_val {
            let (prev_val, _): (T, _) = decode_from_slice(val.as_ref(), config::standard())
                .unwrap_or_else(|err| {
                    panic!(
                        "Failed to deserialize the existing DB value to bincode: {}",
                        err
                    )
                });
            return Ok(Some(prev_val));
        }

        Ok(None)
    }

    /// Like [`Tree::get()`], but deserializes the value from bincode
    ///
    /// # Panics
    /// In case the data received from the DB cannot be deserialized to the expected data type of the key.
    fn b_get<T>(&self, key: &Key<T>) -> SledResult<Option<T>>
    where
        T: Serialize + DeserializeOwned,
    {
        let val = self.get(key.get_key())?;

        if let Some(val) = val {
            let (val, _) =
                decode_from_slice(val.as_ref(), config::standard()).unwrap_or_else(|err| {
                    panic!("Failed to deserialize the DB value to bincode: {}", err)
                });
            return Ok(Some(val));
        }

        Ok(None)
    }

    /// Like [`Tree::remove()`], but deserializes the value from bincode
    ///
    /// # Panics
    /// In case the data received from the DB cannot be deserialized to the expected data type of the key.
    fn b_remove<T>(&self, key: &Key<T>) -> SledResult<Option<T>>
    where
        T: Serialize + DeserializeOwned,
    {
        let val = self.remove(key.get_key())?;

        if let Some(val) = val {
            let (val, _) =
                decode_from_slice(val.as_ref(), config::standard()).unwrap_or_else(|err| {
                    panic!("Failed to deserialize the DB value to bincode: {}", err)
                });
            return Ok(Some(val));
        }

        Ok(None)
    }

    /// Like [`Tree::iter()`], but deserializes the key to [`String`] and the value from bincode
    ///
    /// # Panics
    /// In case the data received from the DB cannot be deserialized to the expected data type of the key.
    /// In case the keys received from the DB cannot be deserialized into UTF-8 [`String`]
    fn b_iter<T>(&self) -> impl Iterator<Item = SledResult<(String, T)>>
    where
        T: Serialize + DeserializeOwned,
    {
        self.iter().map(|val| {
            val.map(|(key, val)| {
                let (val, _): (T, _) = decode_from_slice(val.as_ref(), config::standard())
                    .unwrap_or_else(|err| {
                        panic!(
                            "Failed to deserialize the existing DB value to bincode: {}",
                            err
                        )
                    });

                let key = std::str::from_utf8(key.as_ref())
                    .expect("Key received from DB could not be converted to UTF-8")
                    .to_owned();

                (key, val)
            })
        })
    }
}

impl BincodeTransactional for TransactionalTree {
    /// Like [`TransactionalTree::insert()`], but serializes the value to bincode
    ///
    /// # Panics
    /// In case the provided data cannot be serialized, or the retrieved old data cannot be deserialized into the expected data type of the key
    fn b_insert<T>(&self, key: &Key<T>, value: &T) -> Result<Option<T>, UnabortableTransactionError>
    where
        T: Serialize + DeserializeOwned,
    {
        let bytes = encode_to_vec(value, config::standard()).unwrap_or_else(|err| {
            panic!("Failed to serialize the provided value to bincode: {}", err)
        });

        let prev_val = self.insert(key.get_key(), bytes)?;

        if let Some(val) = prev_val {
            let (prev_val, _): (T, _) = decode_from_slice(val.as_ref(), config::standard())
                .unwrap_or_else(|err| {
                    panic!(
                        "Failed to deserialize the existing DB value to bincode: {}",
                        err
                    )
                });
            return Ok(Some(prev_val));
        }

        Ok(None)
    }

    /// Like [`TransactionalTree::get()`], but deserializes the value to bincode
    ///
    /// # Panics
    /// In case the data received from the DB cannot be deserialized to the expected data type of the key.
    fn b_get<T>(&self, key: &Key<T>) -> Result<Option<T>, UnabortableTransactionError>
    where
        T: Serialize + DeserializeOwned,
    {
        let val = self.get(key.get_key())?;

        if let Some(val) = val {
            let (val, _) =
                decode_from_slice(val.as_ref(), config::standard()).unwrap_or_else(|err| {
                    panic!("Failed to deserialize the DB value to bincode: {}", err)
                });
            return Ok(Some(val));
        }

        Ok(None)
    }

    /// Like [`TransactionalTree::remove()`], but deserializes the value to bincode
    ///
    /// # Panics
    /// In case the data received from the DB cannot be deserialized to the expected data type of the key.
    fn b_remove<'de, T>(&self, key: &Key<T>) -> Result<Option<T>, UnabortableTransactionError>
    where
        T: Serialize + DeserializeOwned,
    {
        let val = self.remove(key.get_key())?;

        if let Some(val) = val {
            let (val, _) =
                decode_from_slice(val.as_ref(), config::standard()).unwrap_or_else(|err| {
                    panic!("Failed to deserialize the DB value to bincode: {}", err)
                });
            return Ok(Some(val));
        }

        Ok(None)
    }
}
