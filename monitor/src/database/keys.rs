//! Contains all valid keys to access the DB values

pub mod config {
    //! All valid keys for config DB Tree
    use comm_types::ipc::{HiveProbeData, HiveTargetData};
    use hive_db::Key;
    use lazy_static::lazy_static;

    use crate::testprogram::Testprogram;

    lazy_static! {
        pub static ref TSS: Key<[bool; 8]> = Key::new("tss");
        pub static ref ASSIGNED_TARGETS: Key<HiveTargetData> = Key::new("targets");
        pub static ref ASSIGNED_PROBES: Key<HiveProbeData> = Key::new("probes");
        pub static ref TESTPROGRAMS: Key<Vec<Testprogram>> = Key::new("testprograms");
        pub static ref ACTIVE_TESTPROGRAM: Key<String> = Key::new("active_testprogram");
    }
}

pub mod credentials {
    //! All valid keys for credentials DB Tree
    use comm_types::auth::DbUser;
    use hive_db::Key;
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref USERS: Key<Vec<DbUser>> = Key::new("users");
    }
}
