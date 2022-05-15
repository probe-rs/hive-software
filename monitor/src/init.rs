//! Handles all initialization required to start testing
use std::fs;
use std::path::Path;
use std::process;
use std::sync::Arc;

use comm_types::auth::DbUser;
use comm_types::hardware::{Architecture, TargetState};
use controller::common::init;
use controller::common::TargetStackShield;
use probe_rs::config;

use crate::binaries;
use crate::database::{keys, CborDb, HiveDb};
use crate::testprogram::{TestProgram, TESTPROGRAM_PATH};
use crate::{EXPANDERS, SHARED_I2C, TESTCHANNELS, TSS};

pub(crate) fn initialize_statics() {
    lazy_static::initialize(&SHARED_I2C);
    lazy_static::initialize(&EXPANDERS);
    lazy_static::initialize(&TSS);
    lazy_static::initialize(&TESTCHANNELS);
}

/// Checks if there is at least one user registered in the database, otherwise exit the process, as the application has to be run in init-mode first to register a user.
///
/// If no user is found this function exits the process and prompts the user to restart the program in init-mode
pub(crate) fn check_uninit(db: Arc<HiveDb>) {
    let users = db
        .credentials_tree
        .c_get::<Vec<DbUser>>(keys::credentials::USERS)
        .unwrap();

    if users.is_some() {
        if !users.unwrap().is_empty() {
            return;
        }
    }

    println!("Failed to find a user in the DB. Please register the first user by running the program in init-mode: 'monitor init'");
    process::exit(1);
}

/// Detect all connected TSS and update DB data
pub(crate) fn init_tss(db: Arc<HiveDb>) {
    let detected = TargetStackShield::detect_connected_tss(SHARED_I2C.acquire_i2c());

    let detected = detected.map(|e| e.is_some());

    db.config_tree
        .c_insert(keys::config::TSS, &detected)
        .unwrap();
}

/// Checks if existing testprograms in the DB are still available on the disk and ready for use, removes them otherwise
///
/// # Panics
/// In case the default test program is not (or only partially) found on the disk. This might indicate a corrupted monitor install.
pub(crate) fn init_testprograms(db: Arc<HiveDb>) {
    log::debug!("Initializing testprograms");
    match db
        .config_tree
        .c_get::<Vec<TestProgram>>(keys::config::TESTPROGRAMS)
        .unwrap()
    {
        Some(mut programs) => {
            if programs.is_empty() {
                panic!("Could not find the default testprogram. The installation might be corrupted, please reinstall the program.");
            }

            let mut idx = 0;
            while programs.len() > idx {
                if !programs[idx].path.join("arm/main.S").exists()
                    || !programs[idx].path.join("riscv/main.S").exists()
                {
                    if programs[idx].name == "Default" {
                        panic!("The files of the default testprogram are incomplete. The installation might be corrupted, please reinstall the program.");
                    }

                    log::warn!("Found testprogram '{}' in DB but failed to locate the complete program files on the disk. Removing corrupted testprogram...", programs[idx].name);

                    // try to remove the program folder (in case only parts of the testprogram folder structure were missing)
                    let _ = fs::remove_dir_all(programs[idx].path.to_owned());

                    programs.remove(idx);
                    db.config_tree
                        .c_insert(keys::config::TESTPROGRAMS, &programs)
                        .unwrap();
                } else {
                    idx += 1;
                }
            }

            // Sync binaries after testprograms have been checked and cleaned
            binaries::sync_binaries(db.clone());
        }
        None => {
            // As this might be the first run of the monitor the default testprogram has to be registered in the DB first
            if !Path::new(&format!("{}{}", TESTPROGRAM_PATH, "default/arm/main.S")).exists()
                || !Path::new(&format!("{}{}", TESTPROGRAM_PATH, "default/riscv/main.S")).exists()
            {
                panic!("Could not find the default testprogram. The installation might be corrupted, please reinstall the program.");
            } else {
                let mut testprograms = vec![];
                let default_testprogram = TestProgram {
                    name: "Default".to_owned(),
                    path: Path::new(&format!("{}{}", TESTPROGRAM_PATH, "default/")).to_path_buf(),
                };

                db.config_tree
                    .c_insert(keys::config::ACTIVE_TESTPROGRAM, &default_testprogram)
                    .unwrap();

                testprograms.push(default_testprogram);

                db.config_tree
                    .c_insert(keys::config::TESTPROGRAMS, &testprograms)
                    .unwrap();

                // Sync binaries after testprograms have been checked and cleaned
                binaries::sync_binaries(db.clone());
            }
        }
    }
}

/// Initializes the TSS and TESTCHANNELS statics according to the data provided by the DB. This function fails if the data in the DB is not in sync with the detected hardware.
///
/// # Panics
/// If the data in the DB has not been initialized.
pub(crate) fn init_hardware_from_db_data(db: Arc<HiveDb>) -> Result<(), init::InitError> {
    let target_data = db.config_tree.c_get(keys::config::ASSIGNED_TARGETS).unwrap().expect("Failed to get the target data in the DB. This function can only be called once the target data has been initialized in the DB.");
    let probe_data = db.config_tree.c_get(keys::config::ASSIGNED_PROBES).unwrap().expect("Failed to get the probe data in the DB. This function can only be called once the probe data has been initialized in the DB.");

    init::initialize_target_data(&TSS, target_data)?;
    init::initialize_probe_data(&TESTCHANNELS, probe_data)
}

/// Initializes [`TargetInfo`] on each known target connected to the tss using the [`probe_rs::config::get_target_by_name()`] function. If the target is not found in the probe-rs registry, its [`TargetInfo`] status field is set to a [`Result::Err`] value.
/// Targets which are not found in the registry are thus being ignored for any subsequent initialization steps, such as flashing the testbinaries for example.
pub(crate) fn init_target_info_from_registry() {
    for tss in TSS.iter() {
        let mut tss = tss.lock().unwrap();

        let mut targets = tss.get_targets().clone();

        if targets.is_some() {
            for target in targets.as_mut().unwrap().iter_mut() {
                if let TargetState::Known(target_info) = target {
                    match config::get_target_by_name(&target_info.name) {
                        Ok(probe_rs_target) => {
                            // Set the architecture field
                            let architecture = match probe_rs_target.architecture() {
                                probe_rs::Architecture::Arm => Architecture::ARM,
                                probe_rs::Architecture::Riscv => Architecture::RISCV,
                            };
                            target_info.architecture = Some(architecture);

                            target_info.status = Ok(());
                        }
                        Err(err) => target_info.status = Err(err.to_string()),
                    }
                }
            }

            tss.set_targets(targets.unwrap());
        }
    }
}
