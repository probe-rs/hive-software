//! Contains all Hive hardware shared types
use std::ops::Range;

use async_graphql::SimpleObject;
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SimpleObject)]
pub struct TargetInfo {
    pub name: String,
    #[graphql(skip)]
    pub architecture: Option<Architecture>,
    #[graphql(skip)]
    pub memory_address: Option<Memory>,
    #[graphql(skip)]
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
