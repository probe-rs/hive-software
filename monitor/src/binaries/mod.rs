//! This module manages all testbinaries which can be flashed onto the targets
//!
//! Generally the user provides a assembly file containing the testprogram, which works for ARM cores as well as one which works for RISC-V.
//! The testprogram can include all sorts of things required by the tests but the tests have to be written specifically to fit the testprogram functionality.
//!
//! As various targets have different flash and ram address spaces the final linking is done by the monitor depending on which targets are currently attached to the Hive Testrack.
//! The final binaries are then flashed onto the connected targets by the monitor before each test run.
use std::sync::Arc;

use comm_types::{hardware::TargetState, ipc::HiveTargetData};

use crate::database::{keys, CborDb, HiveDb};

pub(crate) mod testprogram;

pub(crate) fn sync_binaries(db: Arc<HiveDb>) {
    let target_data: HiveTargetData = db.config_tree.c_get(keys::config::TARGETS).unwrap().expect("Target data not found in DB, syncing binaries can only be done once the DB config data has been initialized.");
    let mut targets = vec![];

    // Get the name of all available targets
    for data in target_data {
        if data.is_some() {
            for target_state in data.unwrap() {
                if let TargetState::Known(name) = target_state {
                    targets.push(name);
                }
            }
        }
    }
}
