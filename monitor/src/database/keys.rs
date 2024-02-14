//! Contains all valid keys to access the DB values

pub mod config {
    //! All valid keys for config DB Tree
    use comm_types::ipc::{HiveProbeData, HiveTargetData};
    use hive_db::Key;
    use lazy_static::lazy_static;

    use crate::{config::AppConfig, testprogram::Testprogram};

    lazy_static! {
        pub static ref APP_CONFIG: Key<'static, AppConfig> = Key::new("app_config");
        pub static ref TSS: Key<'static, [bool; 8]> = Key::new("tss");
        pub static ref ASSIGNED_TARGETS: Key<'static, HiveTargetData> = Key::new("targets");
        pub static ref ASSIGNED_PROBES: Key<'static, HiveProbeData> = Key::new("probes");
        pub static ref TESTPROGRAMS: Key<'static, Vec<Testprogram>> = Key::new("testprograms");
        pub static ref ACTIVE_TESTPROGRAM: Key<'static, String> = Key::new("active_testprogram");
    }
}

pub mod credentials {
    //! All valid keys for credentials DB Tree
    use comm_types::auth::DbUser;
    use hive_db::Key;
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref USERS: Key<'static, Vec<DbUser>> = Key::new("users");
    }
}
