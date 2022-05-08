//! This module manages all testbinaries which can be flashed onto the targets
//!
//! Generally the user provides an assembly file containing the testprogram, which works for ARM cores as well as one which works for RISC-V.
//! The testprogram can include all sorts of things required by the tests but the tests have to be written specifically to fit the testprogram functionality.
//!
//! As various targets have different flash and ram address spaces the final linking is done by the monitor depending on which targets are currently attached to the Hive Testrack.
//! The final binaries are then flashed onto the connected targets by the monitor before each test run.
use crate::database::{keys, CborDb};
use crate::testprogram::TestProgram;
use crate::DB;

mod address;
mod build;

/// Builds binaries out of the currently available Testprograms. Automatically builds multiple binaries with different flash/ram start addresses according to the needs of the currently connected targets.
///
/// # Panics
/// If the target or testprogram configuration data in the DB has not been initialized.
pub(crate) fn sync_binaries() {
    let active_testprogram: TestProgram = DB
        .config_tree
        .c_get(keys::config::ACTIVE_TESTPROGRAM)
        .unwrap().expect("Failed to get active testprogram in DB. The DB needs to be initialized before this function can be called");

    let addresses = address::get_and_init_target_address_ranges();

    active_testprogram.insert_hive_defines();

    for address in addresses.arm.iter() {
        match build::assemble_and_link_arm(&active_testprogram, address) {
            Ok(_) => (),
            Err(err) => {
                log::error!("Failed to assemble and link active ARM testbinary '{}' with memory addresses: {:#?}\n Caused by: {}", active_testprogram.name, address, err);
                todo!()
            }
        }
    }

    for address in addresses.riscv.iter() {
        match build::assemble_and_link_riscv(&active_testprogram, address) {
            Ok(_) => (),
            Err(err) => {
                log::error!("Failed to assemble and link active RISCV testbinary '{}' with memory addresses: {:#?}\n Caused by: {}", active_testprogram.name, address, err);
                todo!()
            }
        }
    }
}
