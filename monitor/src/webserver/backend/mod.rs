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

    use comm_types::auth::{DbUser, Role};
    use comm_types::hardware::{ProbeInfo, ProbeState, TargetInfo, TargetState};
    use comm_types::ipc::{HiveProbeData, HiveTargetData};
    use hive_db::CborDb;
    use lazy_static::lazy_static;

    use crate::database::{keys, MonitorDb};

    lazy_static! {
        // We open a temporary test database and initialize it to the test values
        static ref DB: Arc<MonitorDb> = {
            let db = MonitorDb::open_test();

            db.config_tree.c_insert(&keys::config::ASSIGNED_PROBES, &PROBE_DATA).unwrap();
            db.config_tree.c_insert(&keys::config::ASSIGNED_TARGETS, &TARGET_DATA).unwrap();
            db.config_tree.c_insert(&keys::config::TSS, &[true, true, true, true, true, true, false, false]).unwrap();

            db.credentials_tree.c_insert(&keys::credentials::USERS, &DUMMY_USERS).unwrap();
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
        static ref DUMMY_USERS: Vec<DbUser> = {
            let hash = crate::database::hasher::hash_password("Acorn");
            vec![DbUser { username: "Scrat".to_owned(), hash, role: Role::ADMIN }]
        };
    }

    /// Restore the test DB to its default values.
    /// This function must be called in case the testfunction alters contents in the DB, as other testfunctions will assume an unaltered DB state.
    fn restore_db() {
        let db = DB.clone();

        db.config_tree
            .c_insert(&keys::config::ASSIGNED_PROBES, &*PROBE_DATA)
            .unwrap();
        db.config_tree
            .c_insert(&keys::config::ASSIGNED_TARGETS, &*TARGET_DATA)
            .unwrap();
        db.config_tree
            .c_insert(
                &keys::config::TSS,
                &[true, true, true, true, true, true, false, false],
            )
            .unwrap();

        db.credentials_tree
            .c_insert(&keys::credentials::USERS, &*DUMMY_USERS)
            .unwrap();
    }

    mod query {
        use async_graphql::{from_value, value, Request};
        use comm_types::hardware::{TargetInfo, TargetState};
        use serde::Deserialize;

        use crate::webserver::backend::build_schema;
        use crate::webserver::backend::model::{FlatProbeState, State};

        use super::{DB, PROBE_DATA, TARGET_DATA};

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
            let schema = super::super::build_schema();

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
            let schema = build_schema();

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
            let schema = build_schema();

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
            let schema = build_schema();

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

    mod mutation {
        use async_graphql::{value, Request};
        use comm_types::auth::{DbUser, JwtClaims, Role};
        use comm_types::hardware::ProbeState;
        use hive_db::CborDb;
        use serial_test::serial;
        use tower_cookies::Cookies;

        use crate::database::keys;
        use crate::webserver::auth::{self, AUTH_COOKIE_KEY};
        use crate::webserver::backend::build_schema;

        use super::{DB, DUMMY_USERS, PROBE_DATA, TARGET_DATA};

        #[tokio::test]
        async fn assign_target_invalid_pos() {
            let schema = build_schema();

            // Check tssPos field
            let query = r#"mutation{
                assignTarget(tssPos: 9, targetPos: 2, targetName: "Some Target") {
                    tssPos
                    targetPos
                    targetName
                }
            }"#;

            let result = schema.execute(Request::new(query).data(DB.clone())).await;

            assert!(result.is_err());

            assert_eq!(
                result.errors[0].message,
                "Failed to parse \"Int\": the value is 9, must be less than or equal to 7"
            );

            // Check targetPos field
            let query = r#"mutation{
                assignTarget(tssPos: 3, targetPos: 7, targetName: "Some Target") {
                    tssPos
                    targetPos
                    targetName
                }
            }"#;

            let result = schema.execute(Request::new(query).data(DB.clone())).await;

            assert!(result.is_err());

            assert_eq!(
                result.errors[0].message,
                "Failed to parse \"Int\": the value is 7, must be less than or equal to 3"
            );
        }

        #[tokio::test]
        async fn assign_target_no_daughterboard() {
            let schema = build_schema();

            let query = r#"mutation{
                assignTarget(tssPos: 1, targetPos: 2, targetName: "Some Target") {
                    tssPos
                    targetPos
                    targetName
                }
            }"#;

            let result = schema.execute(Request::new(query).data(DB.clone())).await;

            assert!(result.is_err());

            assert_eq!(
                result.errors[0].message,
                "Cannot assign target to tss without daughterboard"
            );
        }

        #[tokio::test]
        #[serial]
        async fn assign_target() {
            let schema = build_schema();

            let query = r#"mutation{
                assignTarget(tssPos: 0, targetPos: 2, targetName: "Some Target") {
                    tssPos
                    targetPos
                    targetName
                }
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
                    "assignTarget": {
                        "tssPos": 0 as u8,
                        "targetPos": 2 as u8,
                        "targetName": "Some Target",
                    }
                })
            );

            let mut expected_assigned_targets = TARGET_DATA.clone();
            let daughterboard = expected_assigned_targets[0].as_mut().unwrap();
            daughterboard[2] = "Some Target".to_owned().into();

            assert_eq!(
                DB.clone()
                    .config_tree
                    .c_get(&keys::config::ASSIGNED_TARGETS)
                    .unwrap()
                    .unwrap(),
                expected_assigned_targets
            );

            super::restore_db();
        }

        #[tokio::test]
        async fn assign_probe_invalid_pos() {
            let schema = build_schema();

            let query = r#"mutation{
                assignProbe(probePos: 5, probeState: {state: NOT_CONNECTED, data: null}) {
                    probePos
                    data {
                        state
                        data {
                            identifier
                            serialNumber
                        }
                    }
                }
            }"#;

            let result = schema.execute(Request::new(query).data(DB.clone())).await;

            assert!(result.is_err());

            assert_eq!(
                result.errors[0].message,
                "Failed to parse \"Int\": the value is 5, must be less than or equal to 3"
            );
        }

        #[tokio::test]
        async fn assign_probe_invalid_reassign() {
            let schema = build_schema();

            let query = r#"mutation{
                assignProbe(probePos: 3, probeState: {state: KNOWN, data: {identifier: "Curious Probe", serialNumber: "abcde1234"}}) {
                    probePos
                    data {
                        state
                        data {
                            identifier
                            serialNumber
                        }
                    }
                }
            }"#;

            let result = schema.execute(Request::new(query).data(DB.clone())).await;

            assert!(result.is_err());

            assert_eq!(
                result.errors[0].message,
                "Cannot reassign a probe which is already assigned to a testchannel"
            );
        }

        #[tokio::test]
        #[serial]
        async fn assign_probe() {
            let schema = build_schema();

            // We do not test adding a known probe, as this would require calling probe-rs and having an actual probe connected during the tests.
            // Instead we just check if the DB is written correctly on a mutation
            let query = r#"mutation{
                assignProbe(probePos: 3, probeState: {state: NOT_CONNECTED, data: null}) {
                    probePos
                    data {
                        state
                        data {
                            identifier
                            serialNumber
                        }
                    }
                }
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
                    "assignProbe": {
                        "probePos": 3 as u8,
                        "data": {
                            "state": "NOT_CONNECTED",
                            "data": null
                        }
                    }
                })
            );

            let mut expected_assigned_probes = PROBE_DATA.clone();

            expected_assigned_probes[3] = ProbeState::NotConnected;

            assert_eq!(
                DB.clone()
                    .config_tree
                    .c_get(&keys::config::ASSIGNED_PROBES)
                    .unwrap()
                    .unwrap(),
                expected_assigned_probes
            );

            super::restore_db();
        }

        #[tokio::test]
        async fn change_username_short_name() {
            let schema = build_schema();

            let query = r#"mutation{
                changeUsername(username: "ab") {
                    username
                    role
                }
            }"#;

            let result = schema.execute(Request::new(query).data(DB.clone())).await;

            assert!(result.is_err());

            assert_eq!(result.errors[0].message, "Failed to parse \"String\": the chars length is 2, must be greater than or equal to 4");
        }

        #[tokio::test]
        async fn change_username_invalid_name() {
            let schema = build_schema();

            let query = r#"mutation{
                changeUsername(username: "abcd efg hij") {
                    username
                    role
                }
            }"#;

            let jwt_claims = JwtClaims {
                iss: "dummy".to_owned(),
                exp: 0,
                username: "Scrat".to_owned(),
                role: Role::ADMIN,
            };

            let cookies = Cookies::default();
            auth::refresh_auth_token(&DUMMY_USERS[0], &cookies);

            let result = schema
                .execute(
                    Request::new(query)
                        .data(DB.clone())
                        .data(jwt_claims)
                        .data(cookies),
                )
                .await;

            assert!(result.is_err());

            assert_eq!(
                result.errors[0].message,
                "Whitespaces are not allowed in the username"
            );
        }

        #[tokio::test]
        async fn change_username_unknown_user() {
            let schema = build_schema();

            let query = r#"mutation{
                changeUsername(username: "Squirrel") {
                    username
                    role
                }
            }"#;

            let jwt_claims = JwtClaims {
                iss: "dummy".to_owned(),
                exp: 0,
                username: "Scratette".to_owned(),
                role: Role::ADMIN,
            };

            let cookies = Cookies::default();
            auth::refresh_auth_token(
                &DbUser {
                    username: "Scratette".to_owned(),
                    hash: "Dummy hash".to_owned(),
                    role: Role::ADMIN,
                },
                &cookies,
            );

            let result = schema
                .execute(
                    Request::new(query)
                        .data(DB.clone())
                        .data(jwt_claims)
                        .data(cookies),
                )
                .await;

            assert!(result.is_err());

            assert_eq!(result.errors[0].message, "Failed to find user");
        }

        #[tokio::test]
        #[serial]
        async fn change_username() {
            let schema = build_schema();

            let query = r#"mutation{
                changeUsername(username: "Squirrel") {
                    username
                    role
                }
            }"#;

            let jwt_claims = JwtClaims {
                iss: "dummy".to_owned(),
                exp: 0,
                username: "Scrat".to_owned(),
                role: Role::ADMIN,
            };

            let cookies = Cookies::default();
            auth::refresh_auth_token(&DUMMY_USERS[0], &cookies);

            let auth_cookie = cookies.get(AUTH_COOKIE_KEY).unwrap();
            let old_jwt = auth_cookie.value().to_string();

            let result = schema
                .execute(
                    Request::new(query)
                        .data(DB.clone())
                        .data(jwt_claims)
                        .data(cookies.clone()),
                )
                .await
                .into_result()
                .unwrap();

            assert_eq!(
                result.data,
                value!({
                    "changeUsername": {
                        "username": "Squirrel",
                        "role": "ADMIN",
                    }
                })
            );

            let auth_cookie = cookies.get(AUTH_COOKIE_KEY).unwrap();
            let new_jwt = auth_cookie.value();

            assert_ne!(old_jwt, new_jwt);

            let mut expected_db_users = DUMMY_USERS.clone();
            expected_db_users[0] = DbUser {
                username: "Squirrel".to_owned(),
                hash: expected_db_users[0].hash.clone(),
                role: expected_db_users[0].role,
            };

            assert_eq!(
                DB.clone()
                    .credentials_tree
                    .c_get(&keys::credentials::USERS)
                    .unwrap()
                    .unwrap(),
                expected_db_users
            );
        }
    }
}