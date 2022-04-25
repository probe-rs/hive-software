//! Contains all Hive hardware shared types
use std::ops::Range;

use serde::{Deserialize, Serialize};

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
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ProbeInfo {
    pub identifier: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
    pub hid_interface: Option<u8>,
}
