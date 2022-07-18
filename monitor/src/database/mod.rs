//! Handles the sled database
use hive_db::HiveDb;
use sled::Tree;

pub mod hasher;
pub mod keys;
pub mod sync;

/// Interval in which changes of the DB data in RAM are flushed and stored to disk
const FLUSH_INTERVAL_MS: u64 = 60_000;
/// Maximum amount of RAM which can be used to cache the DB data
const CACHE_CAPACITY: u64 = 52_428_800; // 50MB
/// Path to the persistent db files
const DB_PATH: &str = "data/db/";

/// The database and its trees used for this application. Uses a [`HiveDb`] under the hood.
pub struct MonitorDb {
    _db: HiveDb,
    // DB Tree which stores the HW configurations and other Testrack-specifics.
    pub config_tree: Tree,
    /// DB Tree which stores user credentials and credential tokens
    pub credentials_tree: Tree,
}

impl MonitorDb {
    /// Opens the Monitor database. If no database exists, a new one is created.
    ///
    /// # Panics
    /// If any of the open procedures fail.
    pub fn open() -> Self {
        let db = HiveDb::open(DB_PATH, FLUSH_INTERVAL_MS, CACHE_CAPACITY);
        let config_tree = db.open_tree("config");
        let credentials_tree = db.open_tree("credentials");

        Self {
            _db: db,
            config_tree,
            credentials_tree,
        }
    }

    /// Function to open a temporary DB for testing purposes, does not appear in non test builds.
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn open_test() -> Self {
        let db = HiveDb::open_test(FLUSH_INTERVAL_MS, CACHE_CAPACITY);
        let config_tree = db.open_tree("config");
        let credentials_tree = db.open_tree("credentials");

        Self {
            _db: db,
            config_tree,
            credentials_tree,
        }
    }
}
