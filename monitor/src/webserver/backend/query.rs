//! The graphql query
use std::fs::File;
use std::io::Error as IoError;
use std::{path::Path, sync::Arc};

use async_graphql::{Context, Object, Result as GrapqlResult};
use ciborium::de::from_reader;
use comm_types::auth::Role;
use controller::common::logger::LogEntry;
use hive_db::CborDb;
use log::Level;
use probe_rs::config::search_chips;
use probe_rs::Probe;

use crate::database::{keys, MonitorDb};

use super::model::{
    Application, FlatProbeState, FlatTargetState, LogLevel, ProbeInfo, UserResponse,
};

const RUNNER_LOGFILE_PATH: &str = "/mnt/hivetmp/runner.log";

pub(in crate::webserver) struct BackendQuery;

#[Object]
impl BackendQuery {
    /// The currently connected daughterboards
    async fn connected_daughterboards<'ctx>(&self, ctx: &Context<'ctx>) -> [bool; 8] {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        let targets = db
            .config_tree
            .c_get(&keys::config::ASSIGNED_TARGETS)
            .unwrap()
            .expect("DB not initialized");

        let connected = targets
            .into_iter()
            .map(|target| target.is_some())
            .collect::<Vec<bool>>()
            .try_into()
            .unwrap();

        connected
    }

    /// The currently connected TSS
    async fn connected_tss<'ctx>(&self, ctx: &Context<'ctx>) -> [bool; 8] {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        db.config_tree
            .c_get(&keys::config::TSS)
            .unwrap()
            .expect("DB not initialized")
    }

    /// The current targets assigned to connected daughterboards
    async fn assigned_targets<'ctx>(
        &self,
        ctx: &Context<'ctx>,
    ) -> [Option<[FlatTargetState; 4]>; 8] {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        let target_data = db
            .config_tree
            .c_get(&keys::config::ASSIGNED_TARGETS)
            .unwrap()
            .expect("DB not initialized");

        target_data
            .into_iter()
            .map(|target_data| {
                if let Some(target_data) = target_data {
                    let flat_data: [FlatTargetState; 4] = target_data
                        .into_iter()
                        .map(FlatTargetState::from)
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

    /// The current probes assigned to testchannels
    async fn assigned_probes<'ctx>(&self, ctx: &Context<'ctx>) -> [FlatProbeState; 4] {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        db.config_tree
            .c_get(&keys::config::ASSIGNED_PROBES)
            .unwrap()
            .expect("DB not initialized")
            .into_iter()
            .map(FlatProbeState::from)
            .collect::<Vec<FlatProbeState>>()
            .try_into()
            .unwrap()
    }

    /// Search the supported targets by Hive
    async fn search_supported_targets(&self, search: String) -> Vec<String> {
        tokio::task::spawn_blocking(|| {
            // We limit the max amount of returned targets to 30 in order to not send monster responses which could hang the frontend
            let mut chips = search_chips(search).unwrap();
            chips.truncate(30);
            chips
        })
        .await
        .unwrap()
    }

    /// The currently connected debug probes
    async fn connected_probes(&self) -> Vec<ProbeInfo> {
        tokio::task::spawn_blocking(|| {
            Probe::list_all()
                .into_iter()
                .map(|probe| probe.into())
                .collect()
        })
        .await
        .unwrap()
    }

    /// Return the log data of the provided application (either runner or monitor)
    async fn application_log(
        &self,
        application: Application,
        level: LogLevel,
    ) -> GrapqlResult<Vec<String>> {
        let filepath = match application {
            Application::Monitor => Path::new(crate::LOGFILE_PATH),
            Application::Runner => Path::new(RUNNER_LOGFILE_PATH),
        };

        if !filepath.exists() {
            return Ok(vec![]);
        }

        let log_entries = tokio::task::spawn_blocking::<_, Result<_, IoError>>(move || {
            let logfile = File::open(filepath)?;

            let mut log_entries = vec![];

            let mut entry_count = 0;
            while let Ok(entry) = from_reader::<LogEntry, _>(&logfile) {
                if entry.level <= <LogLevel as Into<Level>>::into(level) && entry_count < 100 {
                    log_entries.push(entry.message);
                    entry_count += 1;
                }
            }

            Ok(log_entries)
        })
        .await
        .unwrap()?;

        Ok(log_entries)
    }

    /// List the registered users
    #[graphql(guard = "Role::ADMIN")]
    async fn registered_users<'ctx>(&self, ctx: &Context<'ctx>) -> Vec<UserResponse> {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        db.credentials_tree
            .c_get(&keys::credentials::USERS)
            .unwrap()
            .expect("DB not initialized")
            .into_iter()
            .map(|user| user.into())
            .collect()
    }
}
