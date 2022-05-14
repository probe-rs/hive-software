//! Data models used for graphql
use async_graphql::{Enum, InputObject, SimpleObject};
use comm_types::hardware::{ProbeInfo as CommProbeInfo, ProbeState, TargetInfo, TargetState};
use probe_rs::DebugProbeInfo;

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
