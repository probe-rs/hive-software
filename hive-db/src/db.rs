//! Traits and respective implementations to allow data (de)serialization before reading/writing from the DB
use ciborium::de::from_reader;
use ciborium::ser::into_writer;
use serde::{Deserialize, Serialize};
use sled::transaction::{TransactionalTree, UnabortableTransactionError};
use sled::{Result as SledResult, Tree};

use crate::keys::Key;

/// Functions which allow the DB to operate on CBOR values (Serializing/Deserializing) on each DB call.
pub trait CborDb {
    fn c_insert<'de, T>(&self, key: &Key<'de, T>, value: &'de T) -> SledResult<Option<T>>
    where
        T: Serialize + Deserialize<'de>;

    fn c_get<'de, T>(&self, key: &Key<'de, T>) -> SledResult<Option<T>>
    where
        T: Serialize + Deserialize<'de>;

    fn c_remove<'de, T>(&self, key: &Key<'de, T>) -> SledResult<Option<T>>
    where
        T: Serialize + Deserialize<'de>;
}

/// Functions which allow the user to operate on CBOR values (Serializing/Deserializing) within a transaction.
pub trait CborTransactional {
    fn c_insert<'de, T>(
        &self,
        key: &Key<'de, T>,
        value: &'de T,
    ) -> Result<Option<T>, UnabortableTransactionError>
    where
        T: Serialize + Deserialize<'de>;

    fn c_get<'de, T>(&self, key: &Key<'de, T>) -> Result<Option<T>, UnabortableTransactionError>
    where
        T: Serialize + Deserialize<'de>;

    fn c_remove<'de, T>(&self, key: &Key<'de, T>) -> Result<Option<T>, UnabortableTransactionError>
    where
        T: Serialize + Deserialize<'de>;
}

impl CborDb for Tree {
    /// Like [`Tree::insert()`], but serializes the value to CBOR
    ///
    /// # Panics
    /// In case the provided data cannot be serialized, or the retrieved old data cannot be deserialized into the expected data type of the key
    fn c_insert<'de, T>(&self, key: &Key<'de, T>, value: &'de T) -> SledResult<Option<T>>
    where
        T: Serialize + Deserialize<'de>,
    {
        let mut bytes: Vec<u8> = vec![];
        into_writer(value, &mut bytes).unwrap_or_else(|err| {
            panic!("Failed to serialize the provided value to CBOR: {}", err)
        });

        let prev_val = self.insert(key.get_key(), bytes)?;

        if let Some(val) = prev_val {
            let prev_val: T = from_reader(&*val).unwrap_or_else(|err| {
                panic!(
                    "Failed to deserialize the existing DB value to CBOR: {}",
                    err
                )
            });
            return Ok(Some(prev_val));
        }

        Ok(None)
    }

    /// Like [`Tree::get()`], but deserializes the value to CBOR
    ///
    /// # Panics
    /// In case the data received from the DB cannot be deserialized to the expected data type of the key.
    fn c_get<'de, T>(&self, key: &Key<'de, T>) -> SledResult<Option<T>>
    where
        T: Serialize + Deserialize<'de>,
    {
        let val = self.get(key.get_key())?;

        if let Some(val) = val {
            let val = from_reader(&*val).unwrap_or_else(|err| {
                panic!("Failed to deserialize the DB value to CBOR: {}", err)
            });
            return Ok(Some(val));
        }

        Ok(None)
    }

    /// Like [`Tree::remove()`], but deserializes the value to CBOR
    ///
    /// # Panics
    /// In case the data received from the DB cannot be deserialized to the expected data type of the key.
    fn c_remove<'de, T>(&self, key: &Key<'de, T>) -> SledResult<Option<T>>
    where
        T: Serialize + Deserialize<'de>,
    {
        let val = self.remove(key.get_key())?;

        if let Some(val) = val {
            let val = from_reader(&*val).unwrap_or_else(|err| {
                panic!("Failed to deserialize the DB value to CBOR: {}", err)
            });
            return Ok(Some(val));
        }

        Ok(None)
    }
}

impl CborTransactional for TransactionalTree {
    /// Like [`TransactionalTree::insert()`], but serializes the value to CBOR
    ///
    /// # Panics
    /// In case the provided data cannot be serialized, or the retrieved old data cannot be deserialized into the expected data type of the key
    fn c_insert<'de, T>(
        &self,
        key: &Key<'de, T>,
        value: &'de T,
    ) -> Result<Option<T>, UnabortableTransactionError>
    where
        T: Serialize + Deserialize<'de>,
    {
        let mut bytes: Vec<u8> = vec![];
        into_writer(value, &mut bytes).unwrap_or_else(|err| {
            panic!("Failed to serialize the provided value to CBOR: {}", err)
        });

        let prev_val = self.insert(key.get_key(), bytes)?;

        if let Some(val) = prev_val {
            let prev_val: T = from_reader(&*val).unwrap_or_else(|err| {
                panic!(
                    "Failed to deserialize the existing DB value to CBOR: {}",
                    err
                )
            });
            return Ok(Some(prev_val));
        }

        Ok(None)
    }

    /// Like [`TransactionalTree::get()`], but deserializes the value to CBOR
    ///
    /// # Panics
    /// In case the data received from the DB cannot be deserialized to the expected data type of the key.
    fn c_get<'de, T>(&self, key: &Key<'de, T>) -> Result<Option<T>, UnabortableTransactionError>
    where
        T: Serialize + Deserialize<'de>,
    {
        let val = self.get(key.get_key())?;

        if let Some(val) = val {
            let val = from_reader(&*val).unwrap_or_else(|err| {
                panic!("Failed to deserialize the DB value to CBOR: {}", err)
            });
            return Ok(Some(val));
        }

        Ok(None)
    }

    /// Like [`TransactionalTree::remove()`], but deserializes the value to CBOR
    ///
    /// # Panics
    /// In case the data received from the DB cannot be deserialized to the expected data type of the key.
    fn c_remove<'de, T>(&self, key: &Key<'de, T>) -> Result<Option<T>, UnabortableTransactionError>
    where
        T: Serialize + Deserialize<'de>,
    {
        let val = self.remove(key.get_key())?;

        if let Some(val) = val {
            let val = from_reader(&*val).unwrap_or_else(|err| {
                panic!("Failed to deserialize the DB value to CBOR: {}", err)
            });
            return Ok(Some(val));
        }

        Ok(None)
    }
}
