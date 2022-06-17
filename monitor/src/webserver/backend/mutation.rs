//! The graphql mutation
use std::sync::Arc;

use anyhow::anyhow;
use async_graphql::{Context, Object, Result as GraphQlResult};
use comm_types::auth::{DbUser, JwtClaims, Role};
use comm_types::hardware::{ProbeInfo, ProbeState, TargetState};
use hive_db::CborTransactional;
use probe_rs::Probe;
use sled::transaction::{abort, TransactionError};
use tokio::sync::mpsc::Sender;
use tower_cookies::Cookies;

use crate::{
    database::{hasher, keys, MonitorDb},
    testmanager::ReinitializationTask,
    webserver::auth,
    HARDWARE_DB_DATA_CHANGED,
};

use super::model::{
    AssignProbeResponse, AssignTargetResponse, FlatProbeState, State, UserResponse,
};

pub(in crate::webserver) struct BackendMutation;

#[Object]
impl BackendMutation {
    /// Assigns a target to a given position. This does only update the data in the DB. To apply the changes into the runtime use the update mutation to reinitialize the testrack
    async fn assign_target<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(validator(maximum = 7))] tss_pos: usize,
        #[graphql(validator(maximum = 3))] target_pos: usize,
        target_name: String,
    ) -> GraphQlResult<AssignTargetResponse> {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        db.config_tree
            .transaction(|tree| {
                let mut assigned = tree
                    .c_get(&keys::config::ASSIGNED_TARGETS)?
                    .expect("DB not initialized");

                if assigned[tss_pos].is_none() {
                    abort(anyhow!("Cannot assign target to tss without daughterboard"))?;
                }

                let target_state: TargetState = target_name.clone().into();

                assigned[tss_pos].as_mut().unwrap()[target_pos] = target_state;

                tree.c_insert(&keys::config::ASSIGNED_TARGETS, &assigned)?;

                Ok(())
            })
            .map_err(|err| match err {
                TransactionError::Abort(err) => err,
                TransactionError::Storage(err) => {
                    panic!("Failed to apply DB transaction to storage: {}", err)
                }
            })?;

        // Set the data changed flag to notify the test manager to reinitialize the hardware before the next test run
        let mut data_changed = HARDWARE_DB_DATA_CHANGED.lock().await;
        *data_changed = true;
        drop(data_changed);

        Ok(AssignTargetResponse {
            tss_pos: tss_pos as u8,
            target_pos: target_pos as u8,
            target_name,
        })
    }

    /// Assigns a probe to a given position. This does only update the data in the DB. To apply the changes into the runtime use the update mutation to reinitialize the testrack
    async fn assign_probe<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(validator(maximum = 3))] probe_pos: usize,
        probe_state: FlatProbeState,
    ) -> GraphQlResult<AssignProbeResponse> {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        db.config_tree
            .transaction(|tree| {
                let mut assigned = tree
                    .c_get(&keys::config::ASSIGNED_PROBES)?
                    .expect("DB not initialized");

                if probe_state.state == State::Known {
                    if assigned.iter().any(|assigned_probe| {
                        if let ProbeState::Known(assigned_probe_info) = assigned_probe {
                            return assigned_probe_info.identifier
                                == probe_state.data.as_ref().unwrap().identifier
                                && assigned_probe_info.serial_number
                                    == probe_state.data.as_ref().unwrap().serial_number;
                        }
                        false
                    }) {
                        abort(anyhow!(
                            "Cannot reassign a probe which is already assigned to a testchannel"
                        ))?;
                    }

                    let probes = Probe::list_all();

                    let mut probe_found = false;

                    for probe in probes.into_iter() {
                        if probe.identifier == probe_state.data.as_ref().unwrap().identifier {
                            probe_found = true;
                            assigned[probe_pos] = ProbeState::Known(ProbeInfo {
                                identifier: probe.identifier,
                                vendor_id: probe.vendor_id,
                                product_id: probe.product_id,
                                serial_number: probe.serial_number,
                                hid_interface: probe.hid_interface,
                            });
                        }
                    }

                    if !probe_found {
                        abort(anyhow!("Could not detect the provided probe"))?;
                    }
                } else {
                    assigned[probe_pos] = match probe_state.state {
                        State::Known => unreachable!(),
                        State::Unknown => ProbeState::Unknown,
                        State::NotConnected => ProbeState::NotConnected,
                    };
                }

                tree.c_insert(&keys::config::ASSIGNED_PROBES, &assigned)?;

                Ok(())
            })
            .map_err(|err| match err {
                TransactionError::Abort(err) => err,
                TransactionError::Storage(err) => {
                    panic!("Failed to apply DB transaction to storage: {}", err)
                }
            })?;

        // Set the data changed flag to notify the test manager to reinitialize the hardware before the next test run
        let mut data_changed = HARDWARE_DB_DATA_CHANGED.lock().await;
        *data_changed = true;
        drop(data_changed);

        Ok(AssignProbeResponse {
            probe_pos: probe_pos as u8,
            data: probe_state,
        })
    }

    /// Manually reinitialize the hardware in the runtime
    async fn reinitialize_hardware<'ctx>(&self, ctx: &Context<'ctx>) -> GraphQlResult<bool> {
        let reinit_task_sender = ctx.data::<Sender<ReinitializationTask>>().unwrap();

        let (task, completed_receiver) = ReinitializationTask::new();

        reinit_task_sender.send(task).await?;

        completed_receiver.await?;

        Ok(true)
    }

    /// Change the username of the authenticated user
    async fn change_username<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(validator(chars_min_length = 4))] username: String,
    ) -> GraphQlResult<UserResponse> {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();
        let cookies = ctx.data::<Cookies>().unwrap();
        let claims = ctx.data::<JwtClaims>().unwrap();

        if username.contains(' ') {
            return Err(anyhow!("Whitespaces are not allowed in the username").into());
        }

        let user = db
            .credentials_tree
            .transaction(|tree| {
                let mut users: Vec<DbUser> = tree
                    .c_get(&keys::credentials::USERS)?
                    .expect("DB not initialized");

                let user = users
                    .iter_mut()
                    .enumerate()
                    .find(|(_, user)| user.username == claims.username);

                if let Some((idx, _)) = user {
                    users[idx].username = username.clone();

                    tree.c_insert(&keys::credentials::USERS, &users)?;

                    Ok(users.remove(idx))
                } else {
                    abort(anyhow!("Failed to find user"))
                }
            })
            .map_err(|err| match err {
                TransactionError::Abort(err) => err,
                TransactionError::Storage(err) => {
                    panic!("Failed to apply DB transaction to storage: {}", err)
                }
            })?;

        // As jwt claims changed due to the username change we refresh the jwt auth cookie
        auth::refresh_auth_token(&user, cookies);

        Ok(UserResponse {
            username: user.username,
            role: claims.role,
        })
    }

    /// Change the password of the authenticated user
    async fn change_password<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        old_password: String,
        #[graphql(validator(chars_min_length = 6))] new_password: String,
    ) -> GraphQlResult<bool> {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();
        let claims = ctx.data::<JwtClaims>().unwrap();

        let blocking_claims = claims.clone();
        let blocking_old_password = old_password.clone();
        let blocking_db = db.clone();

        tokio::task::spawn_blocking(move || {
            hasher::check_password(
                blocking_db,
                &blocking_claims.username,
                &blocking_old_password,
            )
            .map_err(|_| anyhow!("Old password is incorrect"))
        })
        .await
        .unwrap()?;

        let blocking_claims = claims.clone();
        let blocking_new_password = new_password.clone();
        let blocking_db = db.clone();
        tokio::task::spawn_blocking(move || {
            blocking_db
                .credentials_tree
                .transaction(|tree| {
                    let mut users: Vec<DbUser> = tree
                        .c_get(&keys::credentials::USERS)?
                        .expect("DB not initialized");

                    let user = users
                        .iter_mut()
                        .enumerate()
                        .find(|(_, user)| user.username == blocking_claims.username);

                    if let Some((idx, _)) = user {
                        // TODO: Find a way to only hash password once, even if the transaction closure is executed multiple times.
                        users[idx].hash = hasher::hash_password(&blocking_new_password);

                        tree.c_insert(&keys::credentials::USERS, &users)?;

                        Ok(true)
                    } else {
                        abort(anyhow!("Failed to find user").into())
                    }
                })
                .map_err(|err| match err {
                    TransactionError::Abort(err) => err,
                    TransactionError::Storage(err) => {
                        panic!("Failed to apply DB transaction to storage: {}", err)
                    }
                })
        })
        .await
        .unwrap()
    }

    /// Create a new user
    #[graphql(guard = "Role::ADMIN")]
    async fn create_user<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(validator(chars_min_length = 4))] username: String,
        #[graphql(validator(chars_min_length = 6))] password: String,
        role: Role,
    ) -> GraphQlResult<UserResponse> {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        if username.contains(' ') {
            return Err(anyhow!("Whitespaces are not allowed in the username").into());
        }

        let hash = tokio::task::spawn_blocking(move || hasher::hash_password(&password))
            .await
            .unwrap();

        let new_user = DbUser {
            username,
            hash,
            role,
        };

        db.credentials_tree
            .transaction(|tree| {
                let mut users = tree
                    .c_get(&keys::credentials::USERS)?
                    .expect("DB not initialized");

                if users.iter().any(|user| user.username == new_user.username) {
                    abort(anyhow!("User already exists"))?;
                }

                users.push(new_user.clone());

                tree.c_insert(&keys::credentials::USERS, &users)?;

                Ok(())
            })
            .map_err(|err| match err {
                TransactionError::Abort(err) => err,
                TransactionError::Storage(err) => {
                    panic!("Failed to apply DB transaction to storage: {}", err)
                }
            })?;

        Ok(new_user.into())
    }

    /// Delete a user
    #[graphql(guard = "Role::ADMIN")]
    async fn delete_user<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(validator(chars_min_length = 4))] username: String,
    ) -> GraphQlResult<UserResponse> {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();
        let claims = ctx.data::<JwtClaims>().unwrap();

        if username.contains(' ') {
            return Err(anyhow!("Whitespaces are not allowed in the username").into());
        }

        if username == claims.username {
            return Err(anyhow!("Cannot delete own user").into());
        }

        let deleted_user = db
            .credentials_tree
            .transaction(|tree| {
                let mut users = tree
                    .c_get(&keys::credentials::USERS)?
                    .expect("DB not initialized");

                for idx in 0..users.len() {
                    if username == users[idx].username {
                        let deleted_user = users.remove(idx);

                        tree.c_insert(&keys::credentials::USERS, &users)?;

                        return Ok(deleted_user);
                    }
                }

                abort(anyhow!("No user with the provided username found in DB."))
            })
            .map_err(|err| match err {
                TransactionError::Abort(err) => err,
                TransactionError::Storage(err) => {
                    panic!("Failed to apply DB transaction to storage: {}", err)
                }
            })?;

        Ok(deleted_user.into())
    }

    /// Log the currently authenticated user out by deleting the auth jwt cookie
    async fn logout<'ctx>(&self, ctx: &Context<'ctx>) -> GraphQlResult<bool> {
        let cookies = ctx.data::<Cookies>().unwrap();

        auth::logout(cookies);

        Ok(true)
    }
}
