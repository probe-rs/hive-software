//! Handles the build process of a testprogram (Assembly, Linking)
use std::{path::Path, process::Command};

use thiserror::Error;

use super::MemoryStart;

#[derive(Debug, Clone, Error)]
pub enum BuildError {
    #[error("Failed to assemble the testprogram: {0}")]
    AssemblyError(String),
    #[error("Failed to link the testprogram: {0}")]
    LinkingError(String),
    #[error("Could not find a valid object file for linking the final elf file")]
    ObjectFileNotFound,
}

/// Try to assemble the provided testprogram for ARM cores
pub(super) fn assemble_binary_arm(program_path: &Path) -> Result<(), BuildError> {
    // -g Generate debug info, -mthumb assemble thumb code
    let assemble = Command::new("arm-none-eabi-as")
        .args(["-g", "main.S", "-o", "main.o", "-mthumb"])
        .current_dir(program_path)
        .output()
        .expect("Failed to run the ARM assembly process, is the arm-none-eabi-as command accessible to the application?");

    if !assemble.status.success() {
        let cause = if !assemble.stderr.is_empty() {
            String::from_utf8(assemble.stderr)
                .expect("Failed to parse stderr from ARM assembler to string")
        } else {
            String::from_utf8(assemble.stdout)
                .expect("Failed to parse stdout from ARM assembler to string")
        };

        return Err(BuildError::AssemblyError(cause));
    }

    Ok(())
}

/// Try to assemble the provided testprogram for RISCV cores
pub(super) fn assemble_binary_riscv(program_path: &Path) -> Result<(), BuildError> {
    let assemble = Command::new("riscv-none-embed-as")
        .args(["main.S", "-o", "main.o"])
        .current_dir(program_path)
        .output()
        .expect("Failed to run the RISCV assembly process, is the riscv-none-embed-as command accessible to the application?");

    if !assemble.status.success() {
        let cause = if !assemble.stderr.is_empty() {
            String::from_utf8(assemble.stderr)
                .expect("Failed to parse stderr from RISCV assembler to string")
        } else {
            String::from_utf8(assemble.stdout)
                .expect("Failed to parse stdout from RISCV assembler to string")
        };

        return Err(BuildError::AssemblyError(cause));
    }

    Ok(())
}

/// Try to link the previously generated objectfile to a final ARM elf file with the provided memory address mapping
///
/// The final elf is stored in the following format, to distinguish it between other memory address mappings:
///
/// main_`flash start address`_`ram start address`.elf
pub(super) fn link_binary_arm(
    program_path: &Path,
    arm_start_address: &MemoryStart,
) -> Result<(), BuildError> {
    // Check if object file exists
    if !program_path.join("main.o").exists() {
        return Err(BuildError::ObjectFileNotFound);
    }

    let link = Command::new("arm-none-eabi-ld")
        .args([
            "-b",
            "elf32-littlearm",
            "main.o",
            "-o",
            &format!("main_{:#x}_{:#x}.elf", arm_start_address.nvm, arm_start_address.ram),
            &format!("{}{:#x}", "-Ttext=", arm_start_address.nvm),
            &format!("{}{:#x}", "-Tdata=", arm_start_address.ram),
        ])
        .current_dir(program_path)
        .output()
        .expect("Failed to run the ARM linking process, is the arm-none-eabi-ld command accessible to the application?");

    if !link.status.success() {
        let cause = if !link.stderr.is_empty() {
            String::from_utf8(link.stderr)
                .expect("Failed to parse stderr from ARM linker to string")
        } else {
            String::from_utf8(link.stdout)
                .expect("Failed to parse stdout from ARM linker to string")
        };

        return Err(BuildError::LinkingError(cause));
    }

    Ok(())
}

/// Try to link the previously generated objectfile to a final RISCV elf file with the provided memory address mapping
///
/// The final elf is stored in the following format, to distinguish it between other memory address mappings:
///
/// main_`flash start address`_`ram start address`.elf
pub(super) fn link_binary_riscv(
    program_path: &Path,
    riscv_start_address: &MemoryStart,
) -> Result<(), BuildError> {
    // Check if object file exists
    if !program_path.join("main.o").exists() {
        return Err(BuildError::ObjectFileNotFound);
    }

    let link = Command::new("riscv-none-embed-ld")
        .args([
            "main.o",
            "-o",
            &format!("main_{:#x}_{:#x}.elf", riscv_start_address.nvm, riscv_start_address.ram),
            "-b",
            "elf32-littleriscv",
            "-Ttext",
            &format!("{:#x}", riscv_start_address.nvm),
            "-Tdata",
            &format!("{:#x}", riscv_start_address.ram),
        ])
        .current_dir(program_path)
        .output()
        .expect("Failed to run the RISCV linking process, is the riscv-none-embed-ld command accessible to the application?");

    if !link.status.success() {
        let cause = if !link.stderr.is_empty() {
            String::from_utf8(link.stderr)
                .expect("Failed to parse stderr from RISCV linker to string")
        } else {
            String::from_utf8(link.stdout)
                .expect("Failed to parse stdout from RISCV linker to string")
        };

        return Err(BuildError::LinkingError(cause));
    }

    Ok(())
}
