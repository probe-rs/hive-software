use std::path::Path;

use sled::{Config, Db, Mode, Tree};

mod db;
mod keys;

pub use db::BincodeDb;
pub use db::BincodeIter;
pub use db::BincodeTransactional;
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
        IVec, Tree,
        transaction::{TransactionalTree, UnabortableTransactionError},
    };

    use crate::{BincodeDb, BincodeTransactional, HiveDb, Key, db::BincodeIter};

    lazy_static! {
        static ref DB: HiveDb = HiveDb::open_test(100, 256);
        static ref KEY: Key<'static, SerDeData> = Key::new("data");
        static ref KEY_1: Key<'static, SerDeData> = Key::new("key1");
        static ref KEY_2: Key<'static, SerDeData> = Key::new("key2");
        static ref KEY_3: Key<'static, SerDeData> = Key::new("key3");
        static ref DATA: SerDeData = SerDeData {
            bool: false,
            number: 1873945,
            string: "some text".to_owned()
        };
        static ref DATA_1: SerDeData = SerDeData {
            bool: false,
            number: 1,
            string: "key1".to_owned()
        };
        static ref DATA_2: SerDeData = SerDeData {
            bool: false,
            number: 2,
            string: "key2".to_owned()
        };
        static ref DATA_3: SerDeData = SerDeData {
            bool: false,
            number: 3,
            string: "key3".to_owned()
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

    /// Creates multiple entries in the tree to test iterators
    fn setup_test_tree_multiple(tree: &Tree) {
        tree.b_insert(&KEY_1, &DATA_1).unwrap();
        tree.b_insert(&KEY_2, &DATA_2).unwrap();
        tree.b_insert(&KEY_3, &DATA_3).unwrap();
    }

    /// Removes all entries inserted by calling [`setup_test_tree_multiple()`]
    fn reset_test_tree_multiple(tree: Tree) {
        tree.b_remove(&KEY_1).unwrap();
        tree.b_remove(&KEY_2).unwrap();
        tree.b_remove(&KEY_3).unwrap();
    }

    #[test]
    #[serial]
    fn b_get() {
        let test_tree = DB.open_tree("test");

        let option = test_tree.b_get(&KEY).unwrap();

        assert!(option.is_none());

        test_tree.b_insert(&KEY, &DATA).unwrap();

        let option = test_tree.b_get(&KEY).unwrap();

        assert_eq!(option, Some(DATA.clone()));

        reset_test_tree(test_tree);
    }

    #[test]
    #[serial]
    fn b_insert() {
        let test_tree = DB.open_tree("test");

        let option = test_tree.b_insert(&KEY, &DATA).unwrap();

        assert!(option.is_none());

        let option = test_tree.b_insert(&KEY, &DATA).unwrap();

        assert_eq!(option, Some(DATA.clone()));

        reset_test_tree(test_tree);
    }

    #[test]
    #[serial]
    fn b_remove() {
        let test_tree = DB.open_tree("test");

        let option = test_tree.b_remove(&KEY).unwrap();

        assert!(option.is_none());

        test_tree.b_insert(&KEY, &DATA).unwrap();

        let option = test_tree.b_remove(&KEY).unwrap();

        assert_eq!(option, Some(DATA.clone()));

        reset_test_tree(test_tree);
    }

    #[test]
    #[serial]
    fn b_iter() {
        let test_tree = DB.open_tree("test");

        setup_test_tree_multiple(&test_tree);

        let mut iterator = test_tree.b_iter::<SerDeData>();

        assert_eq!(
            iterator.next().unwrap(),
            Ok((KEY_1.get_key().to_owned(), DATA_1.clone()))
        );
        assert_eq!(
            iterator.next().unwrap(),
            Ok((KEY_2.get_key().to_owned(), DATA_2.clone()))
        );
        assert_eq!(
            iterator.next().unwrap(),
            Ok((KEY_3.get_key().to_owned(), DATA_3.clone()))
        );
        assert_eq!(iterator.next(), None);

        drop(iterator);

        reset_test_tree_multiple(test_tree);
    }

    #[test]
    #[serial]
    fn b_values() {
        let test_tree = DB.open_tree("test");

        setup_test_tree_multiple(&test_tree);

        let mut iterator = test_tree.iter().b_values::<SerDeData>();

        assert_eq!(iterator.next().unwrap(), Ok(DATA_1.clone()));
        assert_eq!(iterator.next().unwrap(), Ok(DATA_2.clone()));
        assert_eq!(iterator.next().unwrap(), Ok(DATA_3.clone()));
        assert_eq!(iterator.next(), None);

        reset_test_tree_multiple(test_tree);
    }

    #[test]
    #[serial]
    fn b_keys() {
        let test_tree = DB.open_tree("test");

        setup_test_tree_multiple(&test_tree);

        let mut iterator = test_tree.iter().b_keys();

        assert_eq!(iterator.next().unwrap(), Ok(KEY_1.get_key().to_owned()));
        assert_eq!(iterator.next().unwrap(), Ok(KEY_2.get_key().to_owned()));
        assert_eq!(iterator.next().unwrap(), Ok(KEY_3.get_key().to_owned()));
        assert_eq!(iterator.next(), None);

        reset_test_tree_multiple(test_tree);
    }

    #[test]
    #[parallel]
    fn transaction_get() {
        let test_tree = DB.open_tree("test");

        test_tree
            .transaction::<_, _, UnabortableTransactionError>(|transactional| {
                let option = transactional.b_get(&KEY)?;

                assert!(option.is_none());

                transactional.b_insert(&KEY, &DATA)?;

                let option = transactional.b_get(&KEY)?;

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
                let option = transactional.b_insert(&KEY, &DATA).unwrap();

                assert!(option.is_none());

                let option = transactional.b_insert(&KEY, &DATA).unwrap();

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
                let option = transactional.b_remove(&KEY).unwrap();

                assert!(option.is_none());

                transactional.b_insert(&KEY, &DATA).unwrap();

                let option = transactional.b_remove(&KEY).unwrap();

                assert_eq!(option, Some(DATA.clone()));

                reset_test_transactional_tree(transactional)?;
                Ok(())
            })
            .unwrap();
    }
}
