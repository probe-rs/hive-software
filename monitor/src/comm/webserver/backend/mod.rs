//! Backend graphql schemas
use async_graphql::{EmptyMutation, EmptySubscription, Enum, Object, Schema, SimpleObject};
use comm_types::{
    hardware::{TargetInfo, TargetState},
    ipc::HiveTargetData,
};
use probe_rs::config::search_chips;

use crate::{
    database::{keys, CborDb},
    DB,
};

pub(super) type BackendSchema = Schema<BackendQuery, EmptyMutation, EmptySubscription>;

pub(super) fn build_schema() -> BackendSchema {
    Schema::build(BackendQuery, EmptyMutation, EmptySubscription).finish()
}

pub(super) struct BackendQuery;

#[Object]
impl BackendQuery {
    /// The currently connected daughterboards
    async fn connected_daughterboards(&self) -> [bool; 8] {
        let targets: HiveTargetData = DB
            .config_tree
            .c_get(keys::config::TARGETS)
            .unwrap()
            .expect("DB not initialized");

        let connected: [bool; 8] = targets
            .into_iter()
            .map(|target| target.is_some())
            .collect::<Vec<bool>>()
            .try_into()
            .unwrap();

        connected
    }

    /// The currently connected TSS
    async fn connected_tss(&self) -> [bool; 8] {
        todo!()
    }

    async fn target_data<'a>(&self) -> [Option<[FlatTargetState; 4]>; 8] {
        let target_data = DB
            .config_tree
            .c_get::<HiveTargetData>(keys::config::TARGETS)
            .unwrap()
            .expect("DB not initialized");

        target_data
            .into_iter()
            .map(|target_data| {
                if target_data.is_some() {
                    let flat_data: [FlatTargetState; 4] = target_data
                        .unwrap()
                        .into_iter()
                        .map(|target_state| FlatTargetState::from(target_state))
                        .collect::<Vec<FlatTargetState>>()
                        .try_into()
                        .unwrap();

                    return Some(flat_data);
                }

                None
            })
            .collect::<Vec<Option<[FlatTargetState; 4]>>>()
            .try_into()
            .unwrap()
    }

    /// All supported targets
    /*async fn supported_targets(&self) -> Vec<String> {
        let mut supported_targets = vec![];

        for family in families().unwrap().into_iter() {
            for chip in family.variants.into_iter() {
                supported_targets.push(chip.name);
            }
        }

        supported_targets
    }*/

    async fn search_supported_targets(&self, search: String) -> Vec<String> {
        // We limit the max amount of returned targets to 30 in order to not send monster responses which could hang the frontend
        let mut chips = search_chips(search).unwrap();
        chips.truncate(30);
        chips
    }
}

/// Flattened version of [`TargetState`] to use it in graphql api
#[derive(SimpleObject, Debug)]
struct FlatTargetState {
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

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
enum State {
    Known,
    Unknown,
    NotConnected,
}
