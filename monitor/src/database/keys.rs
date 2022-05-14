//! Contains all valid keys to access the DB values

pub(crate) mod config {
    //! All valid keys for config DB Tree
    pub(crate) const TSS: &str = "tss";
    pub(crate) const ASSIGNED_TARGETS: &str = "targets";
    pub(crate) const ASSIGNED_PROBES: &str = "probes";
    pub(crate) const TESTPROGRAMS: &str = "testprograms";
    pub(crate) const ACTIVE_TESTPROGRAM: &str = "active_testprogram";
}

pub(crate) mod credentials {
    //! All valid keys for credentials DB Tree
    pub(crate) const USERS: &str = "users";
}
