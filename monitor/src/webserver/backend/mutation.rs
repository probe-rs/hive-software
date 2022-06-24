//! The graphql mutation
use std::fs;
use std::io::Read;
use std::sync::Arc;

use anyhow::anyhow;
use async_graphql::{Context, Object, Result as GraphQlResult, Upload};
use comm_types::auth::{DbUser, JwtClaims, Role};
use comm_types::hardware::{ProbeInfo, ProbeState, TargetState};
use hive_db::{CborTransactional};
use probe_rs::Probe;
use sled::transaction::{abort, TransactionError};
use tower_cookies::Cookies;

use crate::ACTIVE_TESTPROGRAM_CHANGED;
use crate::tasks::TaskManager;
use crate::testprogram::{Testprogram, DEFAULT_TESTPROGRAM_NAME};
use crate::{
    database::{hasher, keys, MonitorDb},
    tasks::ReinitializationTask,
    webserver::auth,
    HARDWARE_DB_DATA_CHANGED,
};

use super::model::{
    Architecture, AssignProbeResponse, AssignTargetResponse, FlatProbeState, State, UserResponse,
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
        let task_manager = ctx.data::<Arc<TaskManager>>().unwrap();

        let (task, completed_receiver) = ReinitializationTask::new();

        task_manager.register_reinit_task(task).await;

        completed_receiver.await??;

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

    /// Modify a user
    #[graphql(guard = "Role::ADMIN")]
    async fn modify_user<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        #[graphql(validator(chars_min_length = 4))] old_username: String,
        new_role: Option<Role>,
        #[graphql(validator(chars_min_length = 4))] new_username: Option<String>,
        #[graphql(validator(chars_min_length = 6))] new_password: Option<String>,
    ) -> GraphQlResult<UserResponse> {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        if old_username.contains(' ') || new_username.clone().unwrap_or_default().contains(' ') {
            return Err(anyhow!("Whitespaces are not allowed in the username").into());
        }

        let hash = match new_password {
            Some(password) => Some(
                tokio::task::spawn_blocking(move || hasher::hash_password(&password))
                    .await
                    .unwrap(),
            ),
            None => None,
        };

        let new_user = db
            .credentials_tree
            .transaction(|tree| {
                let mut users: Vec<DbUser> = tree
                    .c_get(&keys::credentials::USERS)?
                    .expect("DB not initialized");

                for idx in 0..users.len() {
                    if old_username == users[idx].username {
                        let mut user = users.remove(idx);

                        user = DbUser {
                            username: new_username
                                .as_ref()
                                .unwrap_or(&user.username)
                                .to_owned(),
                            hash: hash.as_ref().unwrap_or(&user.hash).to_owned(),
                            role: new_role.unwrap_or(user.role),
                        };

                        users.push(user.clone());

                        tree.c_insert(&keys::credentials::USERS, &users)?;

                        return Ok(user);
                    }
                }

                abort(anyhow!("Failed to find user"))
            })
            .map_err(|err| match err {
                TransactionError::Abort(err) => err,
                TransactionError::Storage(err) => {
                    panic!("Failed to apply DB transaction to storage: {}", err)
                }
            })?;

        Ok(new_user.into())
    }

    /// Modify a testprogram
    async fn modify_testprogram<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        testprogram_name: String,
        #[graphql(validator(max_items = 2, min_items = 1))] code_files: Vec<Upload>,
    ) -> GraphQlResult<Testprogram> {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        let mut verified_code_files: Vec<(String, Architecture, Vec<u8>)> = vec![];

        // validate files
        for file in code_files.into_iter(){
            let mut file = file.value(ctx).unwrap();

            if file.filename == "arm_main.S" || file.filename == "riscv_main.S"{
                if verified_code_files.iter().any(|(filename, _, _)| *filename == file.filename){
                    return Err(anyhow!("Received upload of the file '{}' twice", file.filename).into());
                }

                let mut bytes = vec![];
                file.content.read_to_end(&mut bytes).unwrap();

                let architecture = match file.filename.as_str() {
                    "arm_main.S" => Architecture::Arm,
                    "riscv_main.S" => Architecture::Riscv,
                    _ => unreachable!(),
                };

                verified_code_files.push((file.filename, architecture, bytes));
                continue;
            }

            return Err(anyhow!("Found invalid filename '{}', expecting 'arm_main.S' or 'riscv_main.S'", file.filename).into());
        }

        let blocking_db = db.clone();
        let (modified_testprogram, is_active) =  tokio::task::spawn_blocking(move || { 
            blocking_db.config_tree
            .transaction(|tree| {
                let mut is_active = false;

                let mut testprograms = tree
                    .c_get(&keys::config::TESTPROGRAMS)?
                    .expect("DB not initialized");

                for idx in 0..testprograms.len(){
                    if testprograms[idx].get_name() != testprogram_name {
                        continue;
                    }

                    let active_testprogram = tree.c_get(&keys::config::ACTIVE_TESTPROGRAM)?.expect("DB not initialized");
                    let mut testprogram = testprograms.remove(idx);

                    if testprogram.get_name() == active_testprogram {
                        is_active = true;
                    }

                    for (_, architecture, bytes) in verified_code_files.iter() {
                        match architecture {
                            Architecture::Arm => testprogram.get_arm_mut().check_source_code(bytes),
                            Architecture::Riscv => testprogram.get_riscv_mut().check_source_code(bytes),
                        }
                    }

                    testprograms.insert(idx, testprogram);

                    tree.c_insert(&keys::config::TESTPROGRAMS, &testprograms)?;

                    return Ok((testprograms.remove(idx), is_active));
                }

                abort(anyhow!("Failed to find provided testprogram"))
            })
            .map_err(|err| match err {
                TransactionError::Abort(err) => err,
                TransactionError::Storage(err) => {
                    panic!("Failed to apply DB transaction to storage: {}", err)
                }
            })

        })
        .await
        .unwrap()?;

        if is_active {
            *ACTIVE_TESTPROGRAM_CHANGED.lock().await = true;
        }

        Ok(modified_testprogram)
    }

    /// Delete a testprogram
    async fn delete_testprogram<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        testprogram_name: String,
    ) -> GraphQlResult<String> {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        if testprogram_name == DEFAULT_TESTPROGRAM_NAME {
            return Err(anyhow!("Cannot delete the default testprogram").into());
        }

        let (delete_path, was_active) = db.config_tree
            .transaction(|tree| {
                let mut was_active = false;

                let active_testprogram = tree
                    .c_get(&keys::config::ACTIVE_TESTPROGRAM)?
                    .expect("DB not initialized");

                let mut testprograms = tree
                    .c_get(&keys::config::TESTPROGRAMS)?
                    .expect("DB not initialized");

                for idx in 0..testprograms.len() {
                    if testprograms[idx].get_name() != testprogram_name{
                        continue;
                    }

                    if active_testprogram == testprograms[idx].get_name() {
                        was_active = true;

                        let default_testprogram = testprograms.iter().find(|program| program.get_name() == DEFAULT_TESTPROGRAM_NAME).expect("Failed to find default testprogram in DB. This should not happen as it is not allowed to delete the default testprogram.");
    
                        tree.c_insert(&keys::config::ACTIVE_TESTPROGRAM, &default_testprogram.get_name().to_owned())?;
                    }
    
                    let deleted = testprograms.swap_remove(idx);

                    tree.c_insert(&keys::config::TESTPROGRAMS, &testprograms)?;

                    return Ok((deleted.get_path().to_path_buf(), was_active));
                }

                abort(anyhow!("Failed to find provided testprogram"))
            })
            .map_err(|err| match err {
                TransactionError::Abort(err) => err,
                TransactionError::Storage(err) => {
                    panic!("Failed to apply DB transaction to storage: {}", err)
                }
            })?;

        if was_active {
            *ACTIVE_TESTPROGRAM_CHANGED.lock().await = true;
        }

        fs::remove_dir_all(delete_path).unwrap();

        Ok(testprogram_name)
    }

    /// Create a testprogram
    async fn create_testprogram<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        testprogram_name: String,
    ) -> GraphQlResult<Testprogram> {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        if testprogram_name == DEFAULT_TESTPROGRAM_NAME {
            return Err(anyhow!("Cannot create the default testprogram").into());
        }

        let new_testprogram = db.config_tree.transaction(|tree|{
            let mut testprograms = tree.c_get(&keys::config::TESTPROGRAMS)?.expect("DB not initialized");

            if testprograms.iter().any(|testprogram| testprogram.get_name() == testprogram_name) {
                abort(anyhow!("Testprogram already exists"))?;
            }

            let new_testprogram = Testprogram::new(testprogram_name.clone());

            testprograms.push(new_testprogram);

            tree.c_insert(&keys::config::TESTPROGRAMS, &testprograms)?;

            Ok(testprograms.pop().unwrap())
        }).map_err(|err| match err {
            TransactionError::Abort(err) => err,
            TransactionError::Storage(err) => {
                panic!("Failed to apply DB transaction to storage: {}", err)
            }
        })?;

        Ok(new_testprogram)
    }

    /// Set a testprogram as active testprogram
    async fn set_active_testprogram<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        testprogram_name: String,
    ) -> GraphQlResult<String> {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        db.config_tree
            .transaction(|tree| {
                let active_testprogram = tree
                    .c_get(&keys::config::ACTIVE_TESTPROGRAM)?
                    .expect("DB not initialized");

                if active_testprogram == testprogram_name {
                    abort(anyhow!("Testprogram is already active"))?;
                }

                let testprograms = tree
                    .c_get(&keys::config::TESTPROGRAMS)?
                    .expect("DB not initialized");

                let testprogram = testprograms.iter().find(|testprogram| testprogram.get_name() == testprogram_name);

                if testprogram.is_none(){
                    abort(anyhow!("Failed to find provided testprogram"))?;
                }

                let testprogram = testprogram.unwrap();

                tree.c_insert(&keys::config::ACTIVE_TESTPROGRAM, &testprogram.get_name().to_owned())?;

                Ok(())
            })
            .map_err(|err| match err {
                TransactionError::Abort(err) => err,
                TransactionError::Storage(err) => {
                    panic!("Failed to apply DB transaction to storage: {}", err)
                }
            })?;

            *ACTIVE_TESTPROGRAM_CHANGED.lock().await = true;

        Ok(testprogram_name)
    }
}
