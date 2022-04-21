//! This module manages all testbinaries which can be flashed onto the targets
//!
//! Generally the user provides a assembly file containing the testprogram, which works for ARM cores as well as one which works for RISC-V.
//! The testprogram can include all sorts of things required by the tests but the tests have to be written specifically to fit the testprogram functionality.
//!
//! As various targets have different flash and ram address spaces the final linking is done by the monitor depending on which targets are currently attached to the Hive Testrack.
//! The final binaries are then flashed onto the connected targets by the monitor before each test run.
use std::process::Command;
use std::sync::Arc;

use thiserror::Error;

use self::address::Memory;
use crate::database::{keys, CborDb, HiveDb};
use testprogram::TestProgram;

mod address;
pub(crate) mod testprogram;

#[derive(Debug, Error)]
enum BuildError {
    #[error("Failed to assemble the testprogram: {0}")]
    AssemblyError(String),
    #[error("Failed to link the testprogram: {0}")]
    LinkingError(String),
}

/// Builds binaries out of the currently available Testprograms. Automatically builds multiple binaries with different flash/ram start addresses according to the needs of the currently connected targets.
///
/// # Panics
/// If the target or testprogram configuration data in the DB has not been initialized.
pub(crate) fn sync_binaries(db: Arc<HiveDb>) {
    let testprograms: Vec<TestProgram> = db.config_tree.c_get(keys::config::TESTPROGRAMS).unwrap().expect("Testprogram data not found in DB, syncing binaries can only be done once the DB config data has been initialized.");

    let addresses = address::get_target_address_ranges(db.clone());

    insert_hive_defines();

    for testprogram in testprograms.iter() {
        for address in addresses.arm.iter() {
            build_binary_arm(testprogram, address);
        }
    }
}

/// Inserts the newest hive_defines.S file into all available testprograms
fn insert_hive_defines() {
    todo!();
}

/// Builds and saves an ARM binary of a testprogram with the provided options
fn build_binary_arm(testprogram: &TestProgram, arm_address: &Memory) -> Result<(), BuildError> {
    let working_dir = testprogram.path.to_owned().join("arm/");

    // -g Generate debug info, -mthumb assemble thumb code
    let assemble = Command::new("arm-none-eabi-as")
        .args(["-g", "main.S", "-o", "main.o", "-mthumb"])
        .current_dir(working_dir.clone())
        .spawn()
        .expect("Failed to run the ARM assembly process");

    let assemble_status = assemble.wait_with_output().unwrap();

    if !assemble_status.status.success() {
        if !assemble_status.stderr.is_empty() {
            let cause = String::from_utf8(assemble_status.stderr)
                .expect("Failed to parse stderr from ARM assembler to string");

            log::error!(
                "Failed to assemble the testprogram '{}' for ARM cores: {}",
                testprogram.name,
                cause,
            );

            return Err(BuildError::AssemblyError(cause));
        } else {
            let cause = String::from_utf8(assemble_status.stdout)
                .expect("Failed to parse stdout from ARM assembler to string");

            log::error!(
                "Failed to assemble the testprogram '{}' for ARM cores: {}",
                testprogram.name,
                cause,
            );

            return Err(BuildError::AssemblyError(cause));
        }
    }

    let link = Command::new("arm-none-eabi-ld")
        .args([
            "-b",
            "elf32-littlearm",
            "main.o",
            "-o",
            "main.elf",
            &format!("{}{}", "-Ttext=", arm_address.nvm.start),
            &format!("{}{}", "-Tdata=", arm_address.ram.start),
        ])
        .current_dir(working_dir)
        .spawn()
        .expect("Failed to run the ARM linking process");

    let link_status = link.wait_with_output().unwrap();

    if !link_status.status.success() {
        if !link_status.stderr.is_empty() {
            let cause = String::from_utf8(link_status.stderr)
                .expect("Failed to parse stderr from ARM linker to string");

            log::error!(
                "Failed to link the testprogram '{}' for ARM cores: {}",
                testprogram.name,
                cause,
            );

            return Err(BuildError::LinkingError(cause));
        } else {
            let cause = String::from_utf8(link_status.stdout)
                .expect("Failed to parse stdout from ARM linker to string");

            log::error!(
                "Failed to link the testprogram '{}' for ARM cores: {}",
                testprogram.name,
                cause,
            );

            return Err(BuildError::LinkingError(cause));
        }
    }

    Ok(())
}
