//! Data models required by the app config which need to implement serde traits
use std::net::{Ipv4Addr, Ipv6Addr};

use serde::{Deserialize, Serialize};
use validators::models::Host as ValidatorHost;

use crate::validate::ValidHost;

#[derive(Debug, Serialize, Deserialize)]
pub struct Host {
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

    /// Get the host as wss url
    pub fn as_wss_url(&self) -> String {
        let address = match &self.address {
            Address::IPv4(addr) => addr.to_string(),
            Address::IPv6(addr) => addr.to_string(),
            Address::Domain(addr) => addr.to_string(),
        };

        match self.port {
            Some(port) => format!("wss://{}:{}", address, port),
            None => format!("wss://{}", address),
        }
    }

    /// Return a tuple containing the host and the associated port. If no port is defined in the struct 443 is used
    pub fn as_secure_parts(&self) -> (String, u16) {
        let address = match &self.address {
            Address::IPv4(addr) => addr.to_string(),
            Address::IPv6(addr) => addr.to_string(),
            Address::Domain(addr) => addr.to_string(),
        };

        (address, self.port.unwrap_or(443))
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
pub enum Address {
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
