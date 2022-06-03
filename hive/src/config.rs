use std::net::{Ipv4Addr, Ipv6Addr};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use validators::models::Host as ValidatorHost;

use crate::validate::ValidHost;

const CONFIG_NAME: &str = "hive-config";

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct HiveConfig {
    testserver: Option<Host>,
}

impl Default for HiveConfig {
    fn default() -> Self {
        Self { testserver: None }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Host {
    address: Address,
    port: Option<u16>,
}

impl From<ValidHost> for Host {
    fn from(valid: ValidHost) -> Self {
        Self {
            address: valid.host.into(),
            port: valid.port,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) enum Address {
    IPv4(Ipv4Addr),
    IPv6(Ipv6Addr),
    Domain(String),
}

impl From<ValidatorHost> for Address {
    fn from(valid: ValidatorHost) -> Self {
        match valid {
            ValidatorHost::Domain(domain) => Self::Domain(domain),
            ValidatorHost::IPv4(ip) => Self::IPv4(ip),
            ValidatorHost::IPv6(ip) => Self::IPv6(ip),
        }
    }
}

/// Load the configuration file of the cli app
pub(super) fn load_config() -> Result<HiveConfig> {
    let config = confy::load(CONFIG_NAME)?;
    Ok(config)
}

/// Save the provided configuration to disk
pub(super) fn save_config(config: &HiveConfig) -> Result<()> {
    confy::store(CONFIG_NAME, config)?;
    Ok(())
}
