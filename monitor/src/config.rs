//! Global app configuration data

use std::sync::OnceLock;

use hive_db::BincodeDb;
use lazy_static::lazy_static;
use nix::unistd::{Gid, Group, Uid, User};
use serde::{Deserialize, Serialize};

use crate::database::MonitorDb;
use crate::database::keys::config::APP_CONFIG as APP_CONFIG_KEY;

static APP_CONFIG: OnceLock<AppConfig> = OnceLock::new();

lazy_static! {
    pub static ref HIVE_UID: Uid = get_user_uid(&get_app_config().hive_user);
    pub static ref RUNNER_UID: Uid = get_user_uid(&get_app_config().runner_user);
    pub static ref HIVE_GID: Gid = get_group_gid(&get_app_config().hive_group);
}

/// App configuration data
#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub hive_user: String,
    pub hive_group: String,
    pub runner_user: String,
}

/// Load the app config from DB into the [`APP_CONFIG`] static
pub fn load_app_config_from_db(db: &MonitorDb) {
    let config = db
        .config_tree
        .b_get(&APP_CONFIG_KEY)
        .expect("Failed to read data from the database").expect("Failed to find the app configuration in DB. Please configure the monitor in init-mode first: 'monitor init'");

    APP_CONFIG.set(config).unwrap();
}

/// Returns the global app config
///
/// # Panics
/// If the static [`APP_CONFIG`] has not been initialized with [`load_app_config()`]
pub fn get_app_config() -> &'static AppConfig {
    APP_CONFIG.get().expect("APP_CONFIG static has not been initialized but called get_app_config. This is a bug, please open an issue.")
}

/// Returns the UID of the provided user name
///
/// # Panics
/// In case the users cannot be listed or the given username cannot be found
fn get_user_uid(user_name: &str) -> Uid {
    if let Some(user) = User::from_name(user_name).expect(
        "Failed to search for system users. This is most likely a system configuration issue.",
    ) {
        user.uid
    } else {
        panic!(
            "Failed to find a user named '{}' on this system. This user is required by the monitor. Is the system setup properly?\n\nIn case you changed the usernames for any users used by the testrack you can modify the user names by running the monitor in init mode: 'monitor init'",
            user_name
        );
    }
}

/// Returns the GID of the provided group name
///
/// # Panics
/// In case the groups cannot be listed or the given group cannot be found
fn get_group_gid(group_name: &str) -> Gid {
    if let Some(group) = Group::from_name(group_name).expect(
        "Failed to search for system groups. This is most likely a system configuration issue.",
    ) {
        group.gid
    } else {
        panic!(
            "Failed to find a group named '{}' on this system. This user group is required by the monitor. Is the system setup properly?\n\nIn case you changed the group names for any groups used by the testrack you can modify the group names by running the monitor in init mode: 'monitor init'",
            group_name
        );
    }
}
