use std::path::Path;

use sled::{Config, Db, Mode, Tree};

mod db;
mod keys;

pub use db::CborDb;
pub use db::CborTransactional;
pub use keys::Key;

/// The Hive Database
///
/// This is a thin wrapper over [`sled::Db`] to implement the traits defined in the [`db`] module.
pub struct HiveDb {
    inner: Db,
}

impl HiveDb {
    /// Opens the Hive database. If no database exists a new one is created.
    ///
    /// # Panics
    /// If opening the DB fails, which is a fatal error that cannot be recovered and needs human intervention
    pub fn open(path: &str, flush_interval_ms: u64, cache_capacity_bytes: u64) -> Self {
        let db = Config::default()
            .path(Path::new(path))
            .cache_capacity(cache_capacity_bytes)
            .mode(Mode::HighThroughput)
            .flush_every_ms(Some(flush_interval_ms))
            .open()
            .expect("Failed to open the database");

        Self { inner: db }
    }

    /// Open a specific tree on the database
    ///
    /// # Panics
    /// If opening the specific tree fails, which is a fatal error that cannot be recovered and needs human intervention
    pub fn open_tree(&self, name: &str) -> Tree {
        self.inner
            .open_tree(name)
            .unwrap_or_else(|err| panic!("Failed to open DB tree '{}'. Caused by: {}", name, err))
    }

    /// Function to open a temporary DB for testing purposes, does not appear in non test builds.
    ///
    /// # Panics
    /// If opening the DB fails, which is a fatal error that cannot be recovered and needs human intervention
    #[cfg(any(test, feature = "test"))]
    #[allow(dead_code)]
    pub fn open_test(flush_interval_ms: u64, cache_capacity_bytes: u64) -> Self {
        let db = Config::default()
            .temporary(true)
            .cache_capacity(cache_capacity_bytes)
            .mode(Mode::HighThroughput)
            .flush_every_ms(Some(flush_interval_ms))
            .open()
            .expect("Failed to open the database");

        Self { inner: db }
    }
}

impl Drop for HiveDb {
    /// Flushes the remaining buffers and makes them persistent on the disk before dropping the DB.
    ///
    /// # Panics
    /// If the flushing of the buffers fails.
    fn drop(&mut self) {
        self.inner
            .flush()
            .expect("Failed to flush the DB during drop");
    }
}

#[cfg(test)]
mod tests {
    use lazy_static::lazy_static;
    use serde::{Deserialize, Serialize};
    use serial_test::{parallel, serial};
    use sled::{
        transaction::{TransactionalTree, UnabortableTransactionError},
        IVec, Tree,
    };

    use crate::{CborDb, CborTransactional, HiveDb, Key};

    lazy_static! {
        static ref DB: HiveDb = HiveDb::open_test(100, 256);
        static ref KEY: Key<'static, SerDeData> = Key::new("data");
        static ref DATA: SerDeData = SerDeData {
            bool: false,
            number: 1873945,
            string: "some text".to_owned()
        };
    }

    /// (De)serializable dummy data for testing the DB operations
    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct SerDeData {
        bool: bool,
        number: u64,
        string: String,
    }

    /// Remove any data saved at [`KEY`] in the DB
    fn reset_test_tree(tree: Tree) {
        tree.remove(KEY.get_key()).unwrap();
    }

    /// Remove any data saved at [`KEY`] in the transactional
    fn reset_test_transactional_tree(
        transactional: &TransactionalTree,
    ) -> Result<Option<IVec>, UnabortableTransactionError> {
        transactional.remove(KEY.get_key())
    }

    #[test]
    #[serial]
    fn c_get() {
        let test_tree = DB.open_tree("test");

        let option = test_tree.c_get(&KEY).unwrap();

        assert!(option.is_none());

        test_tree.c_insert(&KEY, &DATA).unwrap();

        let option = test_tree.c_get(&KEY).unwrap();

        assert_eq!(option, Some(DATA.clone()));

        reset_test_tree(test_tree);
    }

    #[test]
    #[serial]
    fn c_insert() {
        let test_tree = DB.open_tree("test");

        let option = test_tree.c_insert(&KEY, &DATA).unwrap();

        assert!(option.is_none());

        let option = test_tree.c_insert(&KEY, &DATA).unwrap();

        assert_eq!(option, Some(DATA.clone()));

        reset_test_tree(test_tree);
    }

    #[test]
    #[serial]
    fn c_remove() {
        let test_tree = DB.open_tree("test");

        let option = test_tree.c_remove(&KEY).unwrap();

        assert!(option.is_none());

        test_tree.c_insert(&KEY, &DATA).unwrap();

        let option = test_tree.c_remove(&KEY).unwrap();

        assert_eq!(option, Some(DATA.clone()));

        reset_test_tree(test_tree);
    }

    #[test]
    #[parallel]
    fn transaction_get() {
        let test_tree = DB.open_tree("test");

        test_tree
            .transaction::<_, _, UnabortableTransactionError>(|transactional| {
                let option = transactional.c_get(&KEY)?;

                assert!(option.is_none());

                transactional.c_insert(&KEY, &DATA)?;

                let option = transactional.c_get(&KEY)?;

                assert_eq!(option, Some(DATA.clone()));

                reset_test_transactional_tree(transactional)?;
                Ok(())
            })
            .unwrap();
    }

    #[test]
    #[parallel]
    fn transaction_insert() {
        let test_tree = DB.open_tree("test");

        test_tree
            .transaction::<_, _, UnabortableTransactionError>(|transactional| {
                let option = transactional.c_insert(&KEY, &DATA).unwrap();

                assert!(option.is_none());

                let option = transactional.c_insert(&KEY, &DATA).unwrap();

                assert_eq!(option, Some(DATA.clone()));

                reset_test_transactional_tree(transactional)?;
                Ok(())
            })
            .unwrap();
    }

    #[test]
    #[parallel]
    fn transaction_remove() {
        let test_tree = DB.open_tree("test");

        test_tree
            .transaction::<_, _, UnabortableTransactionError>(|transactional| {
                let option = transactional.c_remove(&KEY).unwrap();

                assert!(option.is_none());

                transactional.c_insert(&KEY, &DATA).unwrap();

                let option = transactional.c_remove(&KEY).unwrap();

                assert_eq!(option, Some(DATA.clone()));

                reset_test_transactional_tree(transactional)?;
                Ok(())
            })
            .unwrap();
    }
}
