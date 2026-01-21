//! Handles the application config file
use std::fs::{self, File};

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::models::Host;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct HiveConfig {
    /// The currently stored testserver address
    pub testserver: Option<Host>,
}

impl HiveConfig {
    /// Load the configuration file of the cli app
    pub(super) fn load_config() -> Result<HiveConfig> {
        let project_path = get_project_dirs();

        fs::create_dir_all(project_path.config_dir())?;

        let config_path = project_path.config_dir().join("config.json");

        let is_new = !config_path.is_file();

        let config_file = File::options()
            .write(true)
            .read(true)
            .create(true)
            .truncate(false)
            .open(config_path)?;

        let config: HiveConfig;

        if is_new {
            config = HiveConfig::default();
            serde_json::to_writer_pretty(config_file, &config)?;
        } else {
            config = serde_json::from_reader(config_file)?;
        }

        Ok(config)
    }

    /// Save the provided configuration to disk
    pub(super) fn save_config(&self) -> Result<()> {
        let project_path = ProjectDirs::from("rs", "probe-rs", "hive")
            .expect("Failed to determine a directory to store the application configuration");

        let config_file = File::options()
            .write(true)
            .create(true)
            .truncate(true)
            .open(project_path.config_dir().join("config.json"))?;

        serde_json::to_writer_pretty(config_file, self)?;

        Ok(())
    }
}

/// Return the project dirs of this application which are used to determine where the cache and config data resides on the system
///
/// # Panics
/// In case the function is not able to determine the appropriate directories on the system
pub fn get_project_dirs() -> ProjectDirs {
    ProjectDirs::from("rs", "probe-rs", "hive")
        .expect("Failed to determine a directory to store the application configuration and cache")
}
