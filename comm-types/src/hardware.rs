//! Contains all Hive hardware shared types
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetState {
    Known(String),
    Unknown,
    NotConnected,
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
#[derive(Debug, Serialize, Deserialize)]
pub struct ProbeInfo {
    pub identifier: String,
    pub vendor_id: u16,
    pub product_id: u16,
    pub serial_number: Option<String>,
    pub hid_interface: Option<u8>,
}
