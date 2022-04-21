//! Handles all initialization required to start testing
use std::fs;
use std::path::Path;
use std::sync::Arc;

use comm_types::hardware::{ProbeInfo, TargetState};
use comm_types::ipc::{HiveProbeData, HiveTargetData};
use probe_rs::Probe;

use crate::binaries;
use crate::binaries::testprogram::{TestProgram, TESTPROGRAM_PATH};
use crate::database::{keys, CborDb, HiveDb};
use crate::{EXPANDERS, SHARED_I2C, TESTCHANNELS, TSS};

pub(crate) fn initialize_statics() {
    lazy_static::initialize(&SHARED_I2C);
    lazy_static::initialize(&EXPANDERS);
    lazy_static::initialize(&TSS);
    lazy_static::initialize(&TESTCHANNELS);
}

/// Current dummy implementation of the configuration data initialization. Later this will be done by the user in the configuration backend UI
pub(crate) fn dummy_init_config_data(db: Arc<HiveDb>) {
    let target_data: HiveTargetData = [
        // atsamd daughterboard
        Some([
            TargetState::Known("ATSAMD10C13A-SS".to_owned()),
            TargetState::Known("ATSAMD09D14A-M".to_owned()),
            TargetState::Known("ATSAMD51J18A-A".to_owned()),
            TargetState::Known("ATSAMD21E16L-AFT".to_owned()),
        ]),
        None,
        // lpc daughterboard
        Some([
            TargetState::NotConnected,
            TargetState::Known("LPC1114FDH28_102_5".to_owned()),
            TargetState::NotConnected,
            TargetState::Known("LPC1313FBD48_01,15".to_owned()),
        ]),
        // nrf daughterboard
        Some([
            TargetState::Known("nRF5340".to_owned()),
            TargetState::Known("nRF52832-QFAB-T".to_owned()),
            TargetState::Known("nRF52840".to_owned()),
            TargetState::Known("NRF51822-QFAC-R7".to_owned()),
        ]),
        None,
        // stm32 daughterboard
        Some([
            TargetState::Known("STM32G031F4P6".to_owned()),
            TargetState::NotConnected,
            TargetState::Known("STM32L151C8TxA".to_owned()),
            TargetState::NotConnected,
        ]),
        None,
        None,
    ];

    db.config_tree
        .c_insert(keys::config::TARGETS, &target_data)
        .unwrap();

    let probes = Probe::list_all();

    let probe_data: HiveProbeData = [
        Some(ProbeInfo {
            identifier: probes[0].identifier.clone(),
            vendor_id: probes[0].vendor_id,
            product_id: probes[0].product_id,
            serial_number: probes[0].serial_number.clone(),
            hid_interface: probes[0].hid_interface,
        }),
        Some(ProbeInfo {
            identifier: probes[1].identifier.clone(),
            vendor_id: probes[1].vendor_id,
            product_id: probes[1].product_id,
            serial_number: probes[1].serial_number.clone(),
            hid_interface: probes[1].hid_interface,
        }),
        None,
        None,
    ];

    db.config_tree
        .c_insert(keys::config::PROBES, &probe_data)
        .unwrap();
}

/// Checks if existing testprograms in the DB are still available on the disk and ready for use, removes them otherwise
///
/// # Panics
/// In case the default test program is not (or only partially) found on the disk. This might indicate a corrupted monitor install.
pub(crate) fn init_testprograms(db: Arc<HiveDb>) {
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
            binaries::sync_binaries(db);
        }
        None => {
            // As this might be the first run of the monitor the default testprogram has to be registered in the DB first
            if !Path::new(&format!("{}{}", TESTPROGRAM_PATH, "default/arm/main.S")).exists()
                || !Path::new(&format!("{}{}", TESTPROGRAM_PATH, "default/riscv/main.S")).exists()
            {
                panic!("Could not find the default testprogram. The installation might be corrupted, please reinstall the program.");
            } else {
                let mut testprograms = vec![];

                testprograms.push(TestProgram {
                    name: "Default".to_owned(),
                    path: Path::new(&format!("{}{}", TESTPROGRAM_PATH, "default/")).to_path_buf(),
                });

                db.config_tree
                    .c_insert(keys::config::TESTPROGRAMS, &testprograms)
                    .unwrap();

                // Sync binaries after testprograms have been checked and cleaned
                binaries::sync_binaries(db);
            }
        }
    }
}
