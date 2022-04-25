//! Handles the sled database
use std::path::Path;

use ciborium::de::from_reader;
use ciborium::ser::into_writer;
use serde::{Deserialize, Serialize};
use sled::{Config, Db, Mode, Result as SledResult, Tree};

pub(crate) mod keys;
pub(crate) mod sync;

const FLUSH_INTERVAL_MS: u64 = 60_000;
const CACHE_CAPACITY: u64 = 52_428_800; // 50MB
const DB_PATH: &str = "data/db/";

pub(crate) struct HiveDb {
    db: Db,
    // DB Tree which stores the HW configurations and other Testrack-specifics.
    pub config_tree: Tree,
    /// DB Tree which stores user credentials and credential tokens
    pub credentials_tree: Tree,
}

impl HiveDb {
    /// Opens the Hive database. If no database exists, a new one is created.
    ///
    /// # Panics
    /// If any of the open procedures fail.
    pub fn open() -> Self {
        let db = Config::default()
            .path(Path::new(DB_PATH))
            .cache_capacity(CACHE_CAPACITY)
            .mode(Mode::HighThroughput)
            .flush_every_ms(Some(FLUSH_INTERVAL_MS))
            .open()
            .expect("Failed to open the database");
        let config_tree = db
            .open_tree("config")
            .expect("Failed to open or create the config DB tree");
        let credentials_tree = db
            .open_tree("credentials")
            .expect("Failed to open or create the credentials DB tree");

        Self {
            db,
            config_tree,
            credentials_tree,
        }
    }

    /// Function to open a temporary DB for testing purposes, does not appear in non test builds.
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn open_test() -> Self {
        let db = Config::default()
            .temporary(true)
            .cache_capacity(CACHE_CAPACITY)
            .mode(Mode::HighThroughput)
            .flush_every_ms(Some(FLUSH_INTERVAL_MS))
            .open()
            .expect("Failed to open the database");
        let config_tree = db
            .open_tree("config")
            .expect("Failed to open or create the config DB tree");
        let credentials_tree = db
            .open_tree("credentials")
            .expect("Failed to open or create the credentials DB tree");

        Self {
            db,
            config_tree,
            credentials_tree,
        }
    }
}

impl Drop for HiveDb {
    /// Flushes the remaining buffers and makes them persistent on the disk before dropping the struct.
    ///
    /// # Panics
    /// If the flushing of the buffers fails.
    fn drop(&mut self) {
        self.db.flush().expect("Failed to flush the DB during drop");
    }
}

/// Functions which allow the DB to operate on CBOR values (Serializing/Deserializing) on each DB call.
pub(crate) trait CborDb {
    /// Like [`Tree::insert()`], but serializes value to CBOR
    fn c_insert<'de, T>(&self, key: &str, value: &T) -> SledResult<Option<T>>
    where
        T: Serialize + Deserialize<'de>;
    /// Like [`Tree::get()`], but deserializes value to CBOR
    fn c_get<'de, T>(&self, key: &str) -> SledResult<Option<T>>
    where
        T: Deserialize<'de>;
}

impl CborDb for Tree {
    fn c_insert<'de, T>(&self, key: &str, value: &T) -> SledResult<Option<T>>
    where
        T: Serialize + Deserialize<'de>,
    {
        let mut bytes: Vec<u8> = vec![];
        into_writer(value, &mut bytes).expect("Failed to serialize the provided value to CBOR");

        let prev_val = self.insert(key, bytes)?;

        if prev_val.is_some() {
            let prev_val: T = from_reader(&*prev_val.unwrap())
                .expect("Failed to deserialize the existing DB value to CBOR");
            return Ok(Some(prev_val));
        }

        Ok(None)
    }

    fn c_get<'de, T>(&self, key: &str) -> SledResult<Option<T>>
    where
        T: Deserialize<'de>,
    {
        let val = self.get(key)?;

        if val.is_some() {
            let val =
                from_reader(&*val.unwrap()).expect("Failed to deserialize the DB value to CBOR");
            return Ok(Some(val));
        }

        Ok(None)
    }
}
