//! Data models required for config which need to implement serde traits
use std::net::{Ipv4Addr, Ipv6Addr};

use serde::{Deserialize, Serialize};
use validators::models::Host as ValidatorHost;

use crate::validate::ValidHost;

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Host {
    address: Address,
    port: Option<u16>,
}

impl Host {
    /// Get the host as https url
    pub fn as_https_url(&self) -> String {
        let address = match &self.address {
            Address::IPv4(addr) => addr.to_string(),
            Address::IPv6(addr) => addr.to_string(),
            Address::Domain(addr) => addr.to_string(),
        };

        match self.port {
            Some(port) => format!("https://{}:{}", address, port),
            None => format!("https://{}", address),
        }
    }
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
