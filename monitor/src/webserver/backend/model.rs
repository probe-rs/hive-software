//! Data models used for graphql
//!
//! Contains mostly wrappers and flatened versions of comm_types which implement graphql specifics.
use async_graphql::{Enum, InputObject, SimpleObject};
use comm_types::{
    auth::{DbUser, Role},
    hardware::{ProbeInfo as CommProbeInfo, ProbeState, TargetInfo, TargetState},
};
use log::Level;
use probe_rs::DebugProbeInfo;
use serde::{Deserialize, Serialize};

use crate::testprogram::Testprogram;

/// Flattened version of [`TargetState`] to use it in graphql api
#[derive(SimpleObject, Debug)]
pub(super) struct FlatTargetState {
    pub state: State,
    pub data: Option<FlatTargetInfo>,
}

impl From<TargetState> for FlatTargetState {
    fn from(target_state: TargetState) -> Self {
        match target_state {
            TargetState::Known(target_data) => Self {
                state: State::Known,
                data: Some(target_data.into()),
            },
            TargetState::Unknown => Self {
                state: State::Unknown,
                data: None,
            },
            TargetState::NotConnected => Self {
                state: State::NotConnected,
                data: None,
            },
        }
    }
}

/// Flattened version of [`TargetInfo`] to use it in graphql api
#[derive(Debug, SimpleObject)]
pub(super) struct FlatTargetInfo {
    name: String,
    flash_status: ResultEnum,
    flash_message: String,
}

impl From<TargetInfo> for FlatTargetInfo {
    fn from(target_info: TargetInfo) -> Self {
        let flash_status;
        let flash_message;

        match target_info.status {
            Ok(_) => {
                flash_status = ResultEnum::Ok;
                flash_message = "No problems found".to_owned();
            }
            Err(err) => {
                flash_status = ResultEnum::Error;
                flash_message = err;
            }
        }

        Self {
            name: target_info.name,
            flash_status,
            flash_message,
        }
    }
}

/// Flattened version of [`ProbeState`] to use it in graphql api
#[derive(SimpleObject, Debug, InputObject, Deserialize, PartialEq)]
#[graphql(input_name = "FlatProbeStateInput")]
pub(super) struct FlatProbeState {
    pub state: State,
    pub data: Option<ProbeInfo>,
}

impl From<ProbeState> for FlatProbeState {
    fn from(probe_state: ProbeState) -> Self {
        match probe_state {
            ProbeState::Known(probe_info) => Self {
                state: State::Known,
                data: Some(probe_info.into()),
            },
            ProbeState::Unknown => Self {
                state: State::Unknown,
                data: None,
            },
            ProbeState::NotConnected => Self {
                state: State::NotConnected,
                data: None,
            },
        }
    }
}

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")] // Required for tests, to conform with async_graphql Enum derive
pub(super) enum State {
    Known,
    Unknown,
    NotConnected,
}

/// Information on a probe attached to Hive
#[derive(Debug, Clone, SimpleObject, InputObject, Deserialize, PartialEq)]
#[graphql(input_name = "ProbeInfoInput")]
pub(super) struct ProbeInfo {
    pub identifier: String,
    pub serial_number: Option<String>,
}

impl From<DebugProbeInfo> for ProbeInfo {
    fn from(probe_info: DebugProbeInfo) -> Self {
        Self {
            identifier: probe_info.identifier,
            serial_number: probe_info.serial_number,
        }
    }
}

impl From<CommProbeInfo> for ProbeInfo {
    fn from(probe_info: CommProbeInfo) -> Self {
        Self {
            identifier: probe_info.identifier,
            serial_number: probe_info.serial_number,
        }
    }
}

#[derive(Debug, SimpleObject)]
pub(super) struct AssignTargetResponse {
    pub tss_pos: u8,
    pub target_pos: u8,
    pub target_name: String,
}

#[derive(Debug, SimpleObject)]
pub(super) struct AssignProbeResponse {
    pub probe_pos: u8,
    pub data: FlatProbeState,
}

#[derive(Debug, SimpleObject)]
pub(super) struct UserResponse {
    pub username: String,
    pub role: Role,
}

impl From<DbUser> for UserResponse {
    fn from(db_user: DbUser) -> Self {
        Self {
            username: db_user.username,
            role: db_user.role,
        }
    }
}

#[derive(Debug, SimpleObject)]
pub(super) struct FullTestProgramResponse {
    pub testprogram: Testprogram,
    pub code_arm: String,
    pub code_riscv: String,
}

#[derive(Debug, Enum, PartialEq, Eq, Clone, Copy)]
pub(super) enum ResultEnum {
    Ok,
    Error,
}

/// The main applications of Hive
#[derive(Debug, Enum, PartialEq, Eq, Clone, Copy)]
pub(super) enum Application {
    Monitor,
    Runner,
}

/// The supported architectures
#[derive(Debug, Enum, PartialEq, Eq, Clone, Copy)]
pub(super) enum Architecture {
    Arm,
    Riscv,
}

/// Wrapper for [`log::Level`] to use it in graphql api
#[derive(Debug, Enum, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub(super) enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<Level> for LogLevel {
    fn from(level: Level) -> Self {
        match level {
            Level::Error => Self::Error,
            Level::Warn => Self::Warn,
            Level::Info => Self::Info,
            Level::Debug => Self::Debug,
            Level::Trace => Self::Trace,
        }
    }
}

impl Into<Level> for LogLevel {
    fn into(self) -> Level {
        match self {
            LogLevel::Error => Level::Error,
            LogLevel::Warn => Level::Warn,
            LogLevel::Info => Level::Info,
            LogLevel::Debug => Level::Debug,
            LogLevel::Trace => Level::Trace,
        }
    }
}

/// System information of the System running this application
#[derive(Debug, SimpleObject)]
pub(super) struct SystemInfo {
    pub controller: String,
    pub soc: String,
    pub hostname: String,
    pub os: String,
    pub memory: MemoryInfo,
    pub disk: DiskInfo,
    pub average_load: f64,
}

#[derive(Debug, SimpleObject)]
pub(super) struct MemoryInfo {
    total: u64,
    free: u64,
}

impl From<sys_info::MemInfo> for MemoryInfo {
    fn from(info: sys_info::MemInfo) -> Self {
        Self {
            total: info.total,
            free: info.free,
        }
    }
}

#[derive(Debug, SimpleObject)]
pub(super) struct DiskInfo {
    total: u64,
    free: u64,
}

impl From<sys_info::DiskInfo> for DiskInfo {
    fn from(info: sys_info::DiskInfo) -> Self {
        Self {
            total: info.total,
            free: info.free,
        }
    }
}
