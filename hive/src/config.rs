use std::fs::{self, File};

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::models::Host;

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct HiveConfig {
    /// The currently stored testserver address
    pub testserver: Option<Host>,
}

impl Default for HiveConfig {
    fn default() -> Self {
        Self { testserver: None }
    }
}

impl HiveConfig {
    /// Load the configuration file of the cli app
    pub(super) fn load_config() -> Result<HiveConfig> {
        let project_path = ProjectDirs::from("rs", "probe-rs", "hive")
            .expect("Failed to determine a directory to store the application configuration");

        fs::create_dir_all(project_path.config_dir())?;

        let config_path = project_path.config_dir().join("config.json");

        let is_new = !config_path.is_file();

        let config_file = File::options()
            .write(true)
            .read(true)
            .create(true)
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
            .open(project_path.config_dir().join("config.json"))?;

        serde_json::to_writer_pretty(config_file, self)?;

        Ok(())
    }
}
