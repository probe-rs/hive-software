//! Handlers to find all ram/flash address ranges which need to be provided for the building of the testprogram binaries.
//!
//! As various targets have specific ram and flash addresses the assembled object files of a testprogram need to be linked to support all those address ranges.
//! The functions inside this module determine all address ranges that need to be supported based on the currently connected targets.
use std::ops::Range;

use comm_types::hardware::{Memory, TargetState};
use controller::hardware::HiveHardware;
use probe_rs::{
    config::{MemoryRegion, Registry},
    Architecture, Target,
};

pub struct BaseAddressRanges {
    pub arm: Vec<Memory>,
    pub riscv: Vec<Memory>,
}

/// Returns all required address ranges for the currently connected targets and updates the TargetInfo of each individual target to the correct range
pub fn get_and_init_target_address_ranges(hardware: &HiveHardware) -> BaseAddressRanges {
    let mut addresses = BaseAddressRanges {
        arm: vec![],
        riscv: vec![],
    };
    let probe_rs_registry = Registry::from_builtin_families();

    for tss in hardware.tss.iter().filter_map(|tss| tss.as_ref()) {
        let mut tss = tss.lock().unwrap();

        if tss.get_targets().is_none() {
            // No daughterboard connected
            continue;
        }

        let targets = tss.get_targets().as_ref().unwrap().clone();

        for (idx, target) in targets.iter().enumerate() {
            if let TargetState::Known(target_info) = target {
                let target = match probe_rs_registry.get_target_by_name(&target_info.name) {
                    Ok(target) => target,
                    Err(err) => {
                        log::warn!("Could not find target '{}' in the probe-rs registry. Failed to determine flash/ram addresses for this target: {}", target_info.name, err);
                        continue;
                    }
                };

                let nvm_address_space = match get_nvm_address(target.clone()) {
                    Ok(address_space) => address_space,
                    Err(_) => {
                        log::warn!(
                            "Failed to determine the NVM address space to use for target '{}'.",
                            target_info.name
                        );
                        continue;
                    }
                };

                let ram_address_space = match get_ram_address(target.clone()) {
                    Ok(address_space) => address_space,
                    Err(_) => {
                        log::warn!(
                            "Failed to determine the RAM address space to use for target '{}'.",
                            target_info.name
                        );
                        continue;
                    }
                };

                let new_address = Memory {
                    ram: ram_address_space,
                    nvm: nvm_address_space,
                };

                let mut new_target_info = target_info.clone();
                new_target_info.memory_address = Some(new_address.clone());
                new_target_info.status = Ok(());
                tss.set_target_info(idx, new_target_info);

                if target.architecture() == Architecture::Arm {
                    if !addresses
                        .arm
                        .iter()
                        .any(|address_range| *address_range == new_address)
                    {
                        addresses.arm.push(new_address);
                    }
                } else if target.architecture() == Architecture::Riscv
                    && !addresses
                        .riscv
                        .iter()
                        .any(|address_range| *address_range == new_address)
                {
                    addresses.riscv.push(new_address);
                }
            }
        }
    }

    addresses
}

/// Tries to get the NVM address space into which the program can be loaded.
/// As targets might have multiple NVM memory instances, this function returns the address space of the bootable NVM.
/// In multicore targets the address space of the first core is returned.
fn get_nvm_address(target: Target) -> Result<Range<u64>, ()> {
    let cores = target.cores;

    // Get the boot memory
    let bootable_nvm: Vec<&MemoryRegion> = target
        .memory_map
        .iter()
        .filter(|region| {
            if let MemoryRegion::Nvm(nvm) = region {
                if nvm.is_boot_memory() && nvm.cores[0] == cores[0].name {
                    return true;
                }
            }
            false
        })
        .collect();

    if bootable_nvm.is_empty() {
        // Failed to determine NVM address
        Err(())
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
                    if biggest_region.range.end - biggest_region.range.start
                        < region.range.end - region.range.start
                    {
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
fn get_ram_address(target: Target) -> Result<Range<u64>, ()> {
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
        Err(())
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
                    if biggest_region.range.end - biggest_region.range.start
                        < region.range.end - region.range.start
                    {
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
    use probe_rs::config::{self, Registry};

    use super::{get_nvm_address, get_ram_address};

    #[test]
    fn singlecore_ram_nvm_address_single_mem() {
        let target = Registry::from_builtin_families()
            .get_target_by_name("STM32F030C6Tx")
            .unwrap();

        let nvm_range = get_nvm_address(target.clone()).unwrap();
        let ram_range = get_ram_address(target).unwrap();

        assert_eq!(nvm_range, 0x8000000..0x8008000);
        assert_eq!(ram_range, 0x20000000..0x20001000);
    }

    #[test]
    fn singlecore_ram_nvm_address_multi_ram() {
        let target = Registry::from_builtin_families()
            .get_target_by_name("esp32c3")
            .unwrap();

        let nvm_range = get_nvm_address(target.clone()).unwrap();
        let ram_range = get_ram_address(target).unwrap();

        assert_eq!(nvm_range, 0x0..0x01000000);
        assert_eq!(ram_range, 0x40380000..0x403E0000);
    }

    #[test]
    fn singlecore_ram_nvm_address_multi_nvm() {
        let target = Registry::from_builtin_families()
            .get_target_by_name("nRF52805_xxAA")
            .unwrap();

        let nvm_range = get_nvm_address(target.clone()).unwrap();
        let ram_range = get_ram_address(target).unwrap();

        assert_eq!(nvm_range, 0x0..0x30000);
        assert_eq!(ram_range, 0x20000000..0x20006000);
    }

    #[test]
    fn multicore_ram_nvm_address_multi_mem() {
        let target = Registry::from_builtin_families()
            .get_target_by_name("nRF5340_xxAA")
            .unwrap();

        let nvm_range = get_nvm_address(target.clone()).unwrap();
        let ram_range = get_ram_address(target).unwrap();

        assert_eq!(nvm_range, 0x0..0x100000);
        assert_eq!(ram_range, 0x20000000..0x20040000);
    }
}
