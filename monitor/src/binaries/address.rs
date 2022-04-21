//! Handlers to find all ram/flash address ranges which need to be provided for the building of the binaries
use std::{ops::Range, sync::Arc};

use comm_types::{hardware::TargetState, ipc::HiveTargetData};
use probe_rs::{
    config::{self, MemoryRegion},
    Architecture, Target,
};

use crate::database::{keys, CborDb, HiveDb};

pub(crate) struct BaseAddressRanges {
    pub arm: Vec<Memory>,
    pub riscv: Vec<Memory>,
}

#[derive(PartialEq)]
pub struct Memory {
    pub ram: Range<u32>,
    pub nvm: Range<u32>,
}

/// Returns all required address ranges for the currently connected targets.
pub(crate) fn get_target_address_ranges(db: Arc<HiveDb>) -> BaseAddressRanges {
    let target_data: HiveTargetData = db.config_tree.c_get(keys::config::TARGETS).unwrap().expect("Target data not found in DB, getting target address ranges can only be done once the DB config data has been initialized.");

    let mut targets = vec![];

    let mut addresses = BaseAddressRanges {
        arm: vec![],
        riscv: vec![],
    };

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

    // Get all required flash/ram starting addresses
    for target_name in targets.iter() {
        let target = match config::get_target_by_name(target_name) {
            Ok(target) => target,
            Err(err) => {
                log::warn!("Could not find target '{}' in the probe-rs registry. Failed to determine flash/ram addresses for this target: {}", target_name, err);
                continue;
            }
        };

        let nvm_address_space = match get_nvm_address(target.clone()) {
            Ok(address_space) => address_space,
            Err(_) => {
                log::warn!(
                    "Failed to determine the NVM address space to use for target '{}'.",
                    target_name
                );
                continue;
            }
        };

        let ram_address_space = match get_ram_address(target.clone()) {
            Ok(address_space) => address_space,
            Err(_) => {
                log::warn!(
                    "Failed to determine the RAM address space to use for target '{}'.",
                    target_name
                );
                continue;
            }
        };

        let new_address = Memory {
            ram: ram_address_space,
            nvm: nvm_address_space,
        };

        if target.architecture() == Architecture::Arm {
            if addresses
                .arm
                .iter()
                .find(|address_range| **address_range == new_address)
                .is_none()
            {
                addresses.arm.push(new_address);
            }
        } else if target.architecture() == Architecture::Riscv {
            if addresses
                .riscv
                .iter()
                .find(|address_range| **address_range == new_address)
                .is_none()
            {
                addresses.riscv.push(new_address);
            }
        }
    }

    addresses
}

/// Tries to get the NVM address space into which the program can be loaded.
/// As targets might have multiple NVM memory instances, this function returns the address space of the bootable NVM.
/// In multicore targets the address space of the first core is returned.
fn get_nvm_address(target: Target) -> Result<Range<u32>, ()> {
    let cores = target.cores;

    // Get the boot memory
    let bootable_nvm: Vec<&MemoryRegion> = target
        .memory_map
        .iter()
        .filter(|region| {
            if let MemoryRegion::Nvm(nvm) = region {
                if nvm.is_boot_memory {
                    if nvm.cores[0] == cores[0].name {
                        return true;
                    }
                }
            }
            false
        })
        .collect();

    if bootable_nvm.is_empty() {
        // Failed to determine NVM address
        return Err(());
    } else if bootable_nvm.len() == 1 {
        if let MemoryRegion::Nvm(region) = bootable_nvm[0] {
            return Ok(region.range.clone());
        }
        unreachable!();
    } else {
        // Choose the range with the most space
        let mut biggest_idx = 0;
        for (idx, nvm) in bootable_nvm.iter().enumerate() {
            if let MemoryRegion::Nvm(region) = nvm {
                if let MemoryRegion::Nvm(biggest_region) = bootable_nvm[biggest_idx] {
                    if biggest_region.range.len() < region.range.len() {
                        biggest_idx = idx;
                    }
                } else {
                    unreachable!();
                }
            } else {
                unreachable!();
            }
        }

        if let MemoryRegion::Nvm(region) = bootable_nvm[biggest_idx] {
            return Ok(region.range.clone());
        }
        unreachable!();
    }
}

/// Tries to get the RAM address space into which the program can be loaded.
/// As targets might have multiple RAM memory instances, this function returns the address space of the largest RAM.
/// In multicore targets the address space of the first core is returned.
fn get_ram_address(target: Target) -> Result<Range<u32>, ()> {
    let cores = target.cores;

    // Get the boot memory
    let available_ram: Vec<&MemoryRegion> = target
        .memory_map
        .iter()
        .filter(|region| {
            if let MemoryRegion::Ram(ram) = region {
                if ram.cores[0] == cores[0].name {
                    return true;
                }
            }
            false
        })
        .collect();

    if available_ram.is_empty() {
        // Failed to determine RAM address
        return Err(());
    } else if available_ram.len() == 1 {
        if let MemoryRegion::Ram(region) = available_ram[0] {
            return Ok(region.range.clone());
        }
        unreachable!();
    } else {
        // Choose the range with the most space
        let mut biggest_idx = 0;
        for (idx, ram) in available_ram.iter().enumerate() {
            if let MemoryRegion::Ram(region) = ram {
                if let MemoryRegion::Ram(biggest_region) = available_ram[biggest_idx] {
                    if biggest_region.range.len() < region.range.len() {
                        biggest_idx = idx;
                    }
                } else {
                    unreachable!();
                }
            } else {
                unreachable!();
            }
        }

        if let MemoryRegion::Ram(region) = available_ram[biggest_idx] {
            return Ok(region.range.clone());
        }
        unreachable!();
    }
}

#[cfg(test)]
mod tests {
    use probe_rs::config;
    use serial_test::serial;

    use super::{get_nvm_address, get_ram_address};

    // Running tests in series to avoid trylock panics due to probe-rs registry using trylock instead of lock

    #[test]
    #[serial]
    fn singlecore_ram_nvm_address_single_mem() {
        let target = config::get_target_by_name("STM32F030C6Tx").unwrap();

        let nvm_range = get_nvm_address(target.clone()).unwrap();
        let ram_range = get_ram_address(target).unwrap();

        assert_eq!(nvm_range, 0x8000000..0x8008000);
        assert_eq!(ram_range, 0x20000000..0x20001000);
    }

    #[test]
    #[serial]
    fn singlecore_ram_nvm_address_multi_ram() {
        let target = config::get_target_by_name("esp32c3").unwrap();

        let nvm_range = get_nvm_address(target.clone()).unwrap();
        let ram_range = get_ram_address(target).unwrap();

        assert_eq!(nvm_range, 0x0..0x4000000);
        assert_eq!(ram_range, 0x40380000..0x403E0000);
    }

    #[test]
    #[serial]
    fn singlecore_ram_nvm_address_multi_nvm() {
        let target = config::get_target_by_name("nRF52805_xxAA").unwrap();

        let nvm_range = get_nvm_address(target.clone()).unwrap();
        let ram_range = get_ram_address(target).unwrap();

        assert_eq!(nvm_range, 0x0..0x30000);
        assert_eq!(ram_range, 0x20000000..0x20006000);
    }

    #[test]
    #[serial]
    fn multicore_ram_nvm_address_multi_mem() {
        let target = config::get_target_by_name("nRF5340_xxAA").unwrap();

        let nvm_range = get_nvm_address(target.clone()).unwrap();
        let ram_range = get_ram_address(target).unwrap();

        assert_eq!(nvm_range, 0x0..0x100000);
        assert_eq!(ram_range, 0x20000000..0x20040000);
    }
}
