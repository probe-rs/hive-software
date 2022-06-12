//! Backend graphql schemas
use async_graphql::{EmptySubscription, Schema};

pub(super) mod auth;
mod model;
mod mutation;
mod query;

pub(super) type BackendSchema =
    Schema<query::BackendQuery, mutation::BackendMutation, EmptySubscription>;

pub(super) fn build_schema() -> BackendSchema {
    Schema::build(
        query::BackendQuery,
        mutation::BackendMutation,
        EmptySubscription,
    )
    .finish()
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_graphql::{from_value, value, Request};
    use comm_types::hardware::{ProbeInfo, ProbeState, TargetInfo, TargetState};
    use comm_types::ipc::{HiveProbeData, HiveTargetData};
    use lazy_static::lazy_static;
    use serde::Deserialize;

    use crate::database::{keys, CborDb, HiveDb};
    use crate::webserver::backend::model::State;

    use super::model::FlatProbeState;

    lazy_static! {
        // We open a temporary test database and initialize it to the test values
        static ref DB: Arc<HiveDb> = {
            let db = HiveDb::open_test();

            db.config_tree.c_insert(keys::config::ASSIGNED_PROBES, &*PROBE_DATA).unwrap();
            db.config_tree.c_insert(keys::config::ASSIGNED_TARGETS, &*TARGET_DATA).unwrap();
            db.config_tree.c_insert(keys::config::TSS, &[true, true, true, true, true, true, false, false]).unwrap();

            Arc::new(db)
        };
        static ref PROBE_DATA: HiveProbeData = [
            ProbeState::Known(ProbeInfo {
                identifier: "Curious Probe".to_owned(),
                vendor_id: 42,
                product_id: 920,
                serial_number: Some("abcde1234".to_owned()),
                hid_interface: None,
            }),
            ProbeState::Unknown,
            ProbeState::Known(ProbeInfo {
                identifier: "Overpriced Probe".to_owned(),
                vendor_id: 43,
                product_id: 921,
                serial_number: Some("1234abcde".to_owned()),
                hid_interface: None,
            }),
            ProbeState::Unknown,
        ];
        static ref TARGET_DATA: HiveTargetData = [
            Some([
                TargetState::Known(TargetInfo{
                    name: "ATSAMD10C13A-SS".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::Known(TargetInfo{
                    name: "ATSAMD09D14A-M".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::Known(TargetInfo{
                    name: "ATSAMD51J18A-A".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::Known(TargetInfo{
                    name: "ATSAMD21E16L-AFT".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
            ]),
            None,
            Some([
                TargetState::NotConnected,
                TargetState::Known(TargetInfo{
                    name: "LPC1114FDH28_102_5".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::NotConnected,
                TargetState::Known(TargetInfo{
                    name: "LPC1313FBD48_01,15".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
            ]),
            Some([
                TargetState::Known(TargetInfo{
                    name: "nRF5340".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::Known(TargetInfo{
                    name: "nRF52832-QFAB-T".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::Known(TargetInfo{
                    name: "nRF52840".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::Known(TargetInfo{
                    name: "NRF51822-QFAC-R7".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
            ]),
            None,
            Some([
                TargetState::Known(TargetInfo{
                    name: "STM32G031F4P6".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::NotConnected,
                TargetState::Known(TargetInfo{
                    name: "STM32L151C8TxA".to_owned(),
                    architecture: None,
                    memory_address: None,
                    status: Err("Not initialized".to_owned()),
                }),
                TargetState::NotConnected,
            ]),
            None,
            None,
        ];
    }

    /// Wrapper for [`TargetState`] for use in the unit tests
    #[derive(Debug, Deserialize, PartialEq)]
    struct TestFlatTargetState {
        state: State,
        data: Option<TestTargetInfo>,
    }

    impl From<TargetState> for TestFlatTargetState {
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

    /// Wrapper for [`TargetInfo`] for use in the unit tests
    #[derive(Debug, Deserialize, PartialEq)]
    struct TestTargetInfo {
        pub name: String,
    }

    impl From<TargetInfo> for TestTargetInfo {
        fn from(info: TargetInfo) -> Self {
            Self { name: info.name }
        }
    }

    #[tokio::test]
    async fn connected_daughterboards() {
        let schema = super::build_schema();

        let query = r#"{
                connectedDaughterboards
            }"#;

        let result = schema
            .execute(Request::new(query).data(DB.clone()))
            .await
            .into_result()
            .unwrap()
            .data;

        assert_eq!(
            result,
            value!({
                "connectedDaughterboards": [true, false, true, true, false, true, false, false],
            })
        );
    }

    #[tokio::test]
    async fn connected_tss() {
        let schema = super::build_schema();

        let query = r#"{
                connectedTss
            }"#;

        let result = schema
            .execute(Request::new(query).data(DB.clone()))
            .await
            .into_result()
            .unwrap()
            .data;

        assert_eq!(
            result,
            value!({
                "connectedTss": [true, true, true, true, true, true, false, false],
            })
        );
    }

    #[tokio::test]
    async fn assigned_targets() {
        let schema = super::build_schema();

        let query = r#"{
                assignedTargets {
                    state
                    data {
                        name
                    }
                }
            }"#;

        let result = schema
            .execute(Request::new(query).data(DB.clone()))
            .await
            .into_result()
            .unwrap()
            .data;

        let mut flat_assigned_targets: [Option<[TestFlatTargetState; 4]>; 8] =
            [None, None, None, None, None, None, None, None];

        for (idx, targets) in TARGET_DATA.clone().into_iter().enumerate() {
            if targets.is_none() {
                continue;
            }

            let targets = targets.unwrap();

            flat_assigned_targets[idx] = Some([
                targets[0].clone().into(),
                targets[1].clone().into(),
                targets[2].clone().into(),
                targets[3].clone().into(),
            ]);
        }

        #[allow(non_snake_case)]
        #[derive(Debug, Deserialize)]
        struct ExpectedValue {
            assignedTargets: [Option<[TestFlatTargetState; 4]>; 8],
        }

        assert_eq!(
            from_value::<ExpectedValue>(result).unwrap().assignedTargets,
            flat_assigned_targets
        );
    }

    #[tokio::test]
    async fn assigned_probes() {
        let schema = super::build_schema();

        let query = r#"{
                assignedProbes {
                    state
                    data {
                        identifier
                        serialNumber
                    }
                }
            }"#;

        let result = schema
            .execute(Request::new(query).data(DB.clone()))
            .await
            .into_result()
            .unwrap()
            .data;

        let mut flat_assigned_probes: [FlatProbeState; 4] = [
            FlatProbeState {
                state: State::Unknown,
                data: None,
            },
            FlatProbeState {
                state: State::Unknown,
                data: None,
            },
            FlatProbeState {
                state: State::Unknown,
                data: None,
            },
            FlatProbeState {
                state: State::Unknown,
                data: None,
            },
        ];

        for (idx, probe) in PROBE_DATA.clone().into_iter().enumerate() {
            flat_assigned_probes[idx] = probe.into();
        }

        #[allow(non_snake_case)]
        #[derive(Debug, Deserialize)]
        struct ExpectedValue {
            assignedProbes: [FlatProbeState; 4],
        }

        assert_eq!(
            from_value::<ExpectedValue>(result).unwrap().assignedProbes,
            flat_assigned_probes
        );
    }
}
