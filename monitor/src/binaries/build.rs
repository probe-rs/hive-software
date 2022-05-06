//! Handles the build process of a testinary (Assembly, Linking)
use std::process::Command;

use comm_types::hardware::Memory;
use thiserror::Error;

use super::testprogram::TestProgram;

#[derive(Debug, Error)]
pub(super) enum BuildError {
    #[error("Failed to assemble the testprogram: {0}")]
    AssemblyError(String),
    #[error("Failed to link the testprogram: {0}")]
    LinkingError(String),
    #[error("Could not find a valid object file for linking the final elf file")]
    ObjectFileNotFound,
}

/// Assembles and links the provided testprogram for ARM with the provided memory address range
pub(super) fn assemble_and_link_arm(
    testprogram: &TestProgram,
    arm_address: &Memory,
) -> Result<(), BuildError> {
    assemble_binary_arm(testprogram)?;
    link_binary_arm(testprogram, arm_address)
}

/// Assembles and links the provided testprogram for RISCV with the provided memory address range
pub(super) fn assemble_and_link_riscv(
    testprogram: &TestProgram,
    arm_address: &Memory,
) -> Result<(), BuildError> {
    assemble_binary_riscv(testprogram)?;
    link_binary_riscv(testprogram, arm_address)
}

/// Try to assemble the provided testprogram for ARM cores
pub(super) fn assemble_binary_arm(testprogram: &TestProgram) -> Result<(), BuildError> {
    let working_dir = testprogram.path.to_owned().join("arm/");

    // -g Generate debug info, -mthumb assemble thumb code
    let assemble = Command::new("arm-none-eabi-as")
        .args(["-g", "main.S", "-o", "main.o", "-mthumb"])
        .current_dir(working_dir.clone())
        .output()
        .expect("Failed to run the ARM assembly process, is the arm-none-eabi-as command accessible to the application?");

    if !assemble.status.success() {
        let cause;

        if !assemble.stderr.is_empty() {
            cause = String::from_utf8(assemble.stderr)
                .expect("Failed to parse stderr from ARM assembler to string");
        } else {
            cause = String::from_utf8(assemble.stdout)
                .expect("Failed to parse stdout from ARM assembler to string");
        }

        return Err(BuildError::AssemblyError(cause));
    }

    Ok(())
}

/// Try to assemble the provided testprogram for RISCV cores
pub(super) fn assemble_binary_riscv(testprogram: &TestProgram) -> Result<(), BuildError> {
    let working_dir = testprogram.path.to_owned().join("riscv/");

    let assemble = Command::new("riscv-none-embed-as")
        .args(["main.S", "-o", "main.o"])
        .current_dir(working_dir.clone())
        .output()
        .expect("Failed to run the RISCV assembly process, is the riscv-none-embed-as command accessible to the application?");

    if !assemble.status.success() {
        let cause;

        if !assemble.stderr.is_empty() {
            cause = String::from_utf8(assemble.stderr)
                .expect("Failed to parse stderr from RISCV assembler to string");
        } else {
            cause = String::from_utf8(assemble.stdout)
                .expect("Failed to parse stdout from RISCV assembler to string");
        }

        return Err(BuildError::AssemblyError(cause));
    }

    Ok(())
}

/// Try to link the previously generated objectfile to a final ARM elf file with the provided memory address mapping
///
/// The final elf is stored in the following format, to distinguish it between other memory address mappings:
///
/// main_`flash start address`_`ram start address`.elf
fn link_binary_arm(testprogram: &TestProgram, arm_address: &Memory) -> Result<(), BuildError> {
    let working_dir = testprogram.path.to_owned().join("arm/");

    // Check if object file exists
    if !working_dir.to_owned().join("main.o").exists() {
        return Err(BuildError::ObjectFileNotFound);
    }

    let link = Command::new("arm-none-eabi-ld")
        .args([
            "-b",
            "elf32-littlearm",
            "main.o",
            "-o",
            &format!("main_{:#x}_{:#x}.elf", arm_address.nvm.start, arm_address.ram.start),
            &format!("{}{:#x}", "-Ttext=", arm_address.nvm.start),
            &format!("{}{:#x}", "-Tdata=", arm_address.ram.start),
        ])
        .current_dir(working_dir)
        .output()
        .expect("Failed to run the ARM linking process, is the arm-none-eabi-ld command accessible to the application?");

    if !link.status.success() {
        let cause;

        if !link.stderr.is_empty() {
            cause = String::from_utf8(link.stderr)
                .expect("Failed to parse stderr from ARM linker to string");
        } else {
            cause = String::from_utf8(link.stdout)
                .expect("Failed to parse stdout from ARM linker to string");
        }

        return Err(BuildError::LinkingError(cause));
    }

    Ok(())
}

/// Try to link the previously generated objectfile to a final RISCV elf file with the provided memory address mapping
///
/// The final elf is stored in the following format, to distinguish it between other memory address mappings:
///
/// main_`flash start address`_`ram start address`.elf
fn link_binary_riscv(testprogram: &TestProgram, riscv_address: &Memory) -> Result<(), BuildError> {
    let working_dir = testprogram.path.to_owned().join("riscv/");

    // Check if object file exists
    if !working_dir.to_owned().join("main.o").exists() {
        return Err(BuildError::ObjectFileNotFound);
    }

    let link = Command::new("riscv-none-embed-ld")
        .args([
            "main.o",
            "-o",
            &format!("main_{:#x}_{:#x}.elf", riscv_address.nvm.start, riscv_address.ram.start),
            "-b",
            "elf32-littleriscv",
            "-Ttext",
            &format!("{:#x}", riscv_address.nvm.start),
            "-Tdata",
            &format!("{:#x}", riscv_address.ram.start),
        ])
        .current_dir(working_dir)
        .output()
        .expect("Failed to run the RISCV linking process, is the riscv-none-embed-ld command accessible to the application?");

    if !link.status.success() {
        let cause;

        if !link.stderr.is_empty() {
            cause = String::from_utf8(link.stderr)
                .expect("Failed to parse stderr from RISCV linker to string");
        } else {
            cause = String::from_utf8(link.stdout)
                .expect("Failed to parse stdout from RISCV linker to string");
        }

        return Err(BuildError::LinkingError(cause));
    }

    Ok(())
}
