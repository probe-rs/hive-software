//! Handles all initialization required by the monitor
use std::fs;
use std::path::Path;
use std::process;
use std::sync::Arc;

use comm_types::hardware::{Architecture, TargetState};
use controller::hardware::{HardwareStatus, HiveHardware, TargetStackShield};
use embedded_hal_bus::i2c::MutexDevice;
use hive_db::{BincodeDb, BincodeTransactional};
use probe_rs::config;
use sled::transaction::UnabortableTransactionError;

use crate::config::{HIVE_GID, HIVE_UID, RUNNER_UID};
use crate::database::{keys, MonitorDb};
use crate::testprogram::{Testprogram, DEFAULT_TESTPROGRAM_NAME, TESTPROGRAM_PATH};
use crate::{database, testprogram};
use crate::{EXPANDERS, HARDWARE, I2C_BUS};

#[cfg(doc)]
use comm_types::hardware::TargetInfo;

pub fn initialize_statics() {
    lazy_static::initialize(&I2C_BUS);
    lazy_static::initialize(&EXPANDERS);
    lazy_static::initialize(&HARDWARE);
    lazy_static::initialize(&HIVE_UID);
    lazy_static::initialize(&RUNNER_UID);
    lazy_static::initialize(&HIVE_GID);
}

/// Checks if there is at least one user registered in the database, otherwise exit the process, as the application has to be run in init-mode first to register a user.
///
/// If no user is found this function exits the process and prompts the user to restart the program in init-mode
///
/// # Termination
/// This function terminates the program by using [`process::exit`]. No values are dropped during exit. Therefore this function should be called as early as possible in the program flow before manipulating DB data.
pub fn check_uninit(db: Arc<MonitorDb>) {
    let users = db
        .credentials_tree
        .b_get(&keys::credentials::USERS)
        .unwrap();

    if users.is_some() && !users.unwrap().is_empty() {
        return;
    }

    println!("Failed to find a user in the DB. Please register the first user by running the program in init-mode: 'monitor init'");
    process::exit(1);
}

/// Initializes the entire testrack hardware with the data contained in the DB
pub fn init_hardware(db: Arc<MonitorDb>, hardware: &mut HiveHardware) {
    init_tss(db.clone());
    init_hardware_from_db_data(db.clone(), hardware);
    init_target_info_from_registry(hardware);

    // Synchronize the target and probe data in the DB with the runtime data in case any data desyncs were encountered
    database::sync::sync_tss_target_data(db.clone(), hardware);
    database::sync::sync_tss_probe_data(db, hardware);

    hardware.hardware_status = HardwareStatus::Ready;
}

/// Checks if existing testprograms in the DB are still available on the disk and ready for use, removes them otherwise
///
/// # Panics
/// In case the default test program is not (or only partially) found on the disk. This might indicate a corrupted monitor install.
pub fn init_testprograms(db: Arc<MonitorDb>, hardware: &HiveHardware) {
    log::debug!("Initializing testprograms");

    db.config_tree.transaction::<_, _, UnabortableTransactionError>(|tree|{
        match tree
        .b_get(&keys::config::TESTPROGRAMS)?
    {
        Some(mut programs) => {
            if programs.is_empty() {
                panic!("Could not find the default testprogram. The installation might be corrupted, please reinstall the program.");
            }

            let mut idx = 0;
            while programs.len() > idx {
                if !programs[idx].get_path().join("arm/main.S").exists()
                    || !programs[idx].get_path().join("riscv/main.S").exists()
                {
                    if programs[idx].get_name() == DEFAULT_TESTPROGRAM_NAME {
                        panic!("The files of the default testprogram are incomplete. The installation might be corrupted, please reinstall the program.");
                    }

                    log::warn!("Found testprogram '{}' in DB but failed to locate the complete program files on the disk. Removing corrupted testprogram...", programs[idx].get_name());

                    // try to remove the program folder (in case only parts of the testprogram folder structure were missing)
                    let _ = fs::remove_dir_all(programs[idx].get_path());

                    programs.remove(idx);
                    tree
                        .b_insert(&keys::config::TESTPROGRAMS, &programs)?;
                } else {
                    idx += 1;
                }
            }

            Ok(())
        }
        None => {
            // As this might be the first run of the monitor the default testprogram has to be registered in the DB first
            if !Path::new(&format!("{}{}", TESTPROGRAM_PATH, "default/arm/main.S")).exists()
                || !Path::new(&format!("{}{}", TESTPROGRAM_PATH, "default/riscv/main.S")).exists()
            {
                panic!("Could not find the default testprogram. The installation might be corrupted, please reinstall the program.");
            } else {
                let mut testprograms = vec![];
                let default_testprogram = Testprogram::create_default();

                tree.b_insert(&keys::config::ACTIVE_TESTPROGRAM, &default_testprogram.get_name().to_owned())?;

                testprograms.push(default_testprogram);

                tree.b_insert(&keys::config::TESTPROGRAMS, &testprograms)?;

                Ok(())
            }
        }
    }
    }).unwrap();

    // Sync binaries from cleaned DB data
    testprogram::sync_binaries(db, hardware);
}

/// Detect all connected TSS and update DB data
fn init_tss(db: Arc<MonitorDb>) {
    let detected = TargetStackShield::detect_connected_tss(MutexDevice::new(&I2C_BUS));

    let detected = detected.map(|e| e.is_some());

    db.config_tree
        .b_insert(&keys::config::TSS, &detected)
        .unwrap();
}

/// Initializes the provided [`HiveHardware`] according to the data provided by the DB. If no data is found in the DB the data is initialized from default values.
///
/// # Panics
/// If the data in the DB has not been initialized.
fn init_hardware_from_db_data(db: Arc<MonitorDb>, hardware: &HiveHardware) {
    let target_data = db
        .config_tree
        .b_get(&keys::config::ASSIGNED_TARGETS)
        .unwrap()
        .unwrap_or_default();
    let probe_data = db
        .config_tree
        .b_get(&keys::config::ASSIGNED_PROBES)
        .unwrap()
        .unwrap_or_default();

    // Ignore desync error as it is autoresolved by the function
    let _ = hardware.initialize_target_data(target_data);
    let _ = hardware.initialize_probe_data(probe_data);
}

/// Initializes [`TargetInfo`] on each known target connected to the tss using the [`probe_rs::config::get_target_by_name()`] function. If the target is not found in the probe-rs registry, its [`TargetInfo`] status field is set to a [`Result::Err`] value.
/// Targets which are not found in the registry are thus being ignored for any subsequent initialization steps, such as flashing the testbinaries for example.
fn init_target_info_from_registry(hardware: &HiveHardware) {
    for tss in hardware.tss.iter() {
        if let Some(tss) = tss.as_ref() {
            let mut tss = tss.lock().unwrap();
            let targets = tss.get_targets_mut();

            if targets.is_some() {
                for target in targets.as_mut().unwrap().iter_mut() {
                    if let TargetState::Known(target_info) = target {
                        match config::get_target_by_name(&target_info.name) {
                            Ok(probe_rs_target) => {
                                // Set the architecture field
                                let architecture = match probe_rs_target.architecture() {
                                    probe_rs::Architecture::Arm => Architecture::ARM,
                                    probe_rs::Architecture::Riscv => Architecture::RISCV,
                                    probe_rs::Architecture::Xtensa => unreachable!("Hive does currently not support Xtensa targets. This is a bug, users should not be able to set Xtensa targets in the backend UI.")
                                };
                                target_info.architecture = Some(architecture);

                                target_info.status = Ok(());
                            }
                            Err(err) => target_info.status = Err(err.to_string()),
                        }
                    }
                }
            }
        }
    }
}
