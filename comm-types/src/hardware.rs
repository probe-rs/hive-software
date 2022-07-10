//! Contains all Hive hardware shared types
use std::ops::Range;

use serde::{Deserialize, Serialize};

/// The overall capabilities of the tesrack. Contains the most important information such as available probes and targets to test
#[derive(Debug, Serialize, Deserialize)]
pub struct Capabilities {
    pub available_probes: Vec<String>,
    pub available_targets: Vec<String>,
}

/// Holds all information on a testrack instance which needs to be accessible to multiple applications
#[derive(Debug, Serialize, Deserialize)]
pub struct Testrack {
    pub tss: [Option<StackShieldStatus>; 8],
    pub targets: [Option<[TargetState; 4]>; 8],
    pub probes: [Option<ProbeInfo>; 4],
    pub uninitialized_probes: [Option<ProbeInfo>; 4],
}

/// Represents the state of a single MCU target on a daughterboard
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TargetState {
    Known(TargetInfo),
    Unknown,
    NotConnected,
}

impl Default for TargetState {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<String> for TargetState {
    fn from(str: String) -> Self {
        if str == "Unknown" {
            Self::Unknown
        } else if str == "Not Connected" {
            Self::NotConnected
        } else {
            Self::Known(TargetInfo {
                name: str,
                architecture: None,
                memory_address: None,
                status: Err("Not initialized".to_owned()),
            })
        }
    }
}

/// Represents the state of a single probe connected to a testchannel
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProbeState {
    Known(ProbeInfo),
    Unknown,
    NotConnected,
}

impl Default for ProbeState {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Provides information on the currently used target in a Hive testfunction
#[derive(Debug)]
pub struct HiveTargetInfo {
    pub name: String,
    pub architecture: Architecture,
    pub memory_address: Memory,
}

impl From<TargetInfo> for HiveTargetInfo {
    /// # Panics
    /// If either architecture or memory_address is [`None`] at the time of conversion
    fn from(info: TargetInfo) -> Self {
        Self {
            name: info.name,
            architecture: info.architecture.unwrap(),
            memory_address: info.memory_address.unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TargetInfo {
    pub name: String,
    pub architecture: Option<Architecture>,
    pub memory_address: Option<Memory>,
    pub status: Result<(), String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Architecture {
    ARM,
    RISCV,
}

/// The used memory address range of a target
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Memory {
    pub ram: Range<u32>,
    pub nvm: Range<u32>,
}

/// Represents the status of a single target stack shield
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StackShieldStatus {
    Idle,
    Err,
    NoBoard,
    NotInitialized,
}

/// Information on a probe attached to Hive
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct ProbeInfo {
    pub identifier: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
    pub hid_interface: Option<u8>,
}

impl std::fmt::Debug for ProbeInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} (VID: {:04x}, PID: {:04x}, {})",
            self.identifier,
            self.vendor_id,
            self.product_id,
            self.serial_number
                .clone()
                .map_or("".to_owned(), |v| format!("Serial: {}, ", v)),
        )
    }
}
