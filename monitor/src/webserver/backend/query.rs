//! The graphql query
use std::fs::{self, File};
use std::io::Error as IoError;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use std::{path::Path, sync::Arc};

use anyhow::anyhow;
use async_graphql::{Context, Object, Result as GrapqlResult};
use comm_types::auth::Role;
use controller::logger::LogEntry;
use hive_db::BincodeDb;
use log::Level;
use probe_rs::config::search_chips;
use probe_rs::Probe;
use rppal::system::DeviceInfo;

use crate::database::{keys, MonitorDb};
use crate::testprogram::Testprogram;

use super::model::{
    Application, FlatProbeState, FlatTargetState, FullTestProgramResponse, LogLevel, ProbeInfo,
    SystemInfo, UserResponse,
};

const RUNNER_LOGFILE_PATH: &str = "./data/logs/runner.log";

pub(in crate::webserver) struct BackendQuery;

#[Object]
impl BackendQuery {
    /// The currently connected daughterboards
    async fn connected_daughterboards<'ctx>(&self, ctx: &Context<'ctx>) -> [bool; 8] {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        let targets = db
            .config_tree
            .b_get(&keys::config::ASSIGNED_TARGETS)
            .unwrap()
            .expect("DB not initialized");

        targets
            .into_iter()
            .map(|target| target.is_some())
            .collect::<Vec<bool>>()
            .try_into()
            .unwrap()
    }

    /// The currently connected TSS
    async fn connected_tss<'ctx>(&self, ctx: &Context<'ctx>) -> [bool; 8] {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        db.config_tree
            .b_get(&keys::config::TSS)
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
            .b_get(&keys::config::ASSIGNED_TARGETS)
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
            .b_get(&keys::config::ASSIGNED_PROBES)
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
            let file_reader = BufReader::new(logfile);

            let mut log_entries = vec![];

            let mut entry_count = 0;

            for line in file_reader.lines() {
                let line = line?;
                let entry: LogEntry = serde_json::from_str(&line)?;

                if let Ok(entry_level) = log::Level::from_str(&entry.level) {
                    if entry_level <= <LogLevel as Into<Level>>::into(level) {
                        log_entries.push(line);
                        entry_count += 1;
                    }
                } else {
                    continue;
                }

                if entry_count >= 100 {
                    break;
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
            .b_get(&keys::credentials::USERS)
            .unwrap()
            .expect("DB not initialized")
            .into_iter()
            .map(|user| user.into())
            .collect()
    }

    /// Get all avaialable testprograms
    async fn available_testprograms<'ctx>(&self, ctx: &Context<'ctx>) -> Vec<Testprogram> {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        db.config_tree
            .b_get(&keys::config::TESTPROGRAMS)
            .unwrap()
            .expect("DB not initialized")
    }

    /// Get the currently active testprogram
    async fn active_testprogram<'ctx>(&self, ctx: &Context<'ctx>) -> String {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        db.config_tree
            .b_get(&keys::config::ACTIVE_TESTPROGRAM)
            .unwrap()
            .expect("DB not initialized")
    }

    /// Get the provided testprogram and its sourcecode as base64
    async fn testprogram<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        testprogram_name: String,
    ) -> GrapqlResult<FullTestProgramResponse> {
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();

        let testprograms = db
            .config_tree
            .b_get(&keys::config::TESTPROGRAMS)
            .unwrap()
            .expect("DB not initialized");

        let testprogram = testprograms
            .into_iter()
            .find(|testprogram| testprogram.get_name() == testprogram_name)
            .ok_or_else(|| anyhow!("Failed to find provided testprogram"))?;

        let (code_arm, code_riscv, testprogram) = tokio::task::spawn_blocking(move || {
            let arm_code = base64::encode(
                fs::read(testprogram.get_path().join("arm/main.S"))
                    .expect("Failed to open testprogram ARM assembly source code"),
            );

            let riscv_code = base64::encode(
                fs::read(testprogram.get_path().join("riscv/main.S"))
                    .expect("Failed to open testprogram ARM assembly source code"),
            );

            (arm_code, riscv_code, testprogram)
        })
        .await
        .unwrap();

        Ok(FullTestProgramResponse {
            testprogram,
            code_arm,
            code_riscv,
        })
    }

    /// Get information about the system on which Hive runs
    async fn system_info<'ctx>(&self, _ctx: &Context<'ctx>) -> GrapqlResult<SystemInfo> {
        tokio::task::spawn_blocking(|| {
            let device_info = DeviceInfo::new()?;

            let hostname = sys_info::hostname()?;
            let cores = sys_info::cpu_num()?;
            let os = sys_info::linux_os_release()?;
            let memory = sys_info::mem_info()?;
            let disk = sys_info::disk_info()?;
            let load = sys_info::loadavg()?;

            Ok(SystemInfo {
                controller: device_info.model().to_string(),
                soc: device_info.soc().to_string(),
                hostname,
                cores,
                os: os.pretty_name.unwrap_or_else(|| "Unknown".to_owned()),
                average_load: load.one,
                memory: memory.into(),
                disk: disk.into(),
            })
        })
        .await
        .unwrap()
    }
}
