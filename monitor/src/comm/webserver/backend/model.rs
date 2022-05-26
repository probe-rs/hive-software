//! Data models used for graphql
use async_graphql::{Enum, InputObject, SimpleObject};
use comm_types::{
    auth::{DbUser, Role},
    hardware::{ProbeInfo as CommProbeInfo, ProbeState, TargetInfo, TargetState},
};
use log::Level;
use probe_rs::DebugProbeInfo;
use serde::{Deserialize, Serialize};

/// Flattened version of [`TargetState`] to use it in graphql api
#[derive(SimpleObject, Debug)]
pub(super) struct FlatTargetState {
    state: State,
    data: Option<TargetInfo>,
}

impl From<TargetState> for FlatTargetState {
    fn from(target_state: TargetState) -> Self {
        match target_state {
            TargetState::Known(target_data) => Self {
                state: State::Known,
                data: Some(target_data),
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

/// Flattened version of [`ProbeState`] to use it in graphql api
#[derive(SimpleObject, Debug, InputObject)]
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

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub(super) enum State {
    Known,
    Unknown,
    NotConnected,
}

/// Information on a probe attached to Hive
#[derive(Debug, Clone, SimpleObject, InputObject)]
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

/// The main applications of Hive
#[derive(Enum, PartialEq, Eq, Clone, Copy)]
pub(super) enum Application {
    Monitor,
    Runner,
}

/// Wrapper for [`log::Level`] to use it in graphql api
#[derive(Enum, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
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
