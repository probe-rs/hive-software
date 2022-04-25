//! This module manages all testbinaries which can be flashed onto the targets
//!
//! Generally the user provides an assembly file containing the testprogram, which works for ARM cores as well as one which works for RISC-V.
//! The testprogram can include all sorts of things required by the tests but the tests have to be written specifically to fit the testprogram functionality.
//!
//! As various targets have different flash and ram address spaces the final linking is done by the monitor depending on which targets are currently attached to the Hive Testrack.
//! The final binaries are then flashed onto the connected targets by the monitor before each test run.
use std::sync::mpsc;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

use comm_types::hardware::{Architecture, TargetInfo, TargetState};
use controller::common::CombinedTestChannel;
use probe_rs::flashing::Format;
use probe_rs::flashing::{download_file_with_options, DownloadOptions};
use testprogram::TestProgram;

use crate::database::{keys, CborDb};
use crate::{DB, TESTCHANNELS, TSS};

mod address;
mod build;
mod hive_defines;
pub(crate) mod testprogram;

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

    hive_defines::insert_hive_defines(&active_testprogram);

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

#[derive(Debug)]
struct FlashStatus {
    tss_pos: u8,
    target_name: String,
    result: Result<(), String>,
}

/// Tries to flash the testbinaries onto all available targets.
pub(crate) fn flash_testbinaries() {
    let active_testprogram: Arc<TestProgram> = Arc::new(DB.config_tree.c_get(keys::config::ACTIVE_TESTPROGRAM).unwrap().expect("Failed to get the active testprogram. Flashing the testbinaries can only be performed once the active testprogram is known"));

    // A buffersize of 0 enforces that the RwLock flash_results vector does not slowly get out of sync due to read locks.
    // This could lead to situations where a thread checks the FlashStatus on an already invalid flash_results vector thus causing unwanted flashes of already successfully flashed targets.
    // The sync channel forces the sender to block, until the data has been received.
    let (result_sender, result_receiver) = mpsc::sync_channel::<FlashStatus>(0);

    // As we don't know if some probes will work for flashing certain targets, we just try out every available probe until we reach a successful flash or a definitive failure. The logic used here is very similar to the test runner logic.
    let mut flashing_threads = vec![];
    let flash_results = Arc::new(RwLock::new(vec![]));

    for (idx, test_channel) in TESTCHANNELS.iter().enumerate() {
        let channel = test_channel.lock().unwrap();

        if channel.is_ready() {
            drop(channel);
            let result_sender = result_sender.clone();
            let active_testprogram = active_testprogram.clone();
            let flash_results = flash_results.clone();

            flashing_threads.push(
                thread::Builder::new()
                    .name(format!("flashing thread {}", idx).to_owned())
                    .spawn(move || {
                        let mut channel = test_channel.lock().unwrap();
                        let sender = result_sender;

                        channel.connect_all_available_and_execute(
                            &TSS,
                            |test_channel, target_info, tss_pos| {
                                flash_target(
                                    test_channel,
                                    target_info,
                                    tss_pos,
                                    &sender,
                                    &*active_testprogram,
                                    &flash_results,
                                );
                            },
                        );
                    })
                    .unwrap(),
            );
        }
    }

    // Drop local owned sender, so the while loop exits once all senders in the flashing thread have been dropped
    drop(result_sender);

    while let Ok(received) = result_receiver.recv() {
        let mut flash_results = flash_results.write().unwrap();
        flash_results.push(received);
    }

    for thread in flashing_threads {
        thread.join().unwrap();
    }

    // Update tss targets with the flash results
    for tss in TSS.iter() {
        let tss = tss.lock().unwrap();

        if tss.get_targets().is_none() {
            // No daughterboard attached
            continue;
        }

        let mut targets = tss.get_targets().clone();

        for target in targets.as_mut().unwrap() {
            if let TargetState::Known(target_info) = target {
                let flash_results = flash_results.read().unwrap();

                if flash_results
                    .iter()
                    .filter(|result| {
                        result.tss_pos == tss.get_position()
                            && result.target_name == target_info.name
                    })
                    .count()
                    == 0
                {
                    // Target is not included in the flash_results
                    continue;
                }

                if flash_results
                    .iter()
                    .filter(|result| {
                        result.tss_pos == tss.get_position()
                            && result.target_name == target_info.name
                            && result.result.is_ok()
                    })
                    .count()
                    != 0
                {
                    target_info.status = Ok(());
                } else {
                    target_info.status =
                        Err("Failed to flash testbinary prior to testing".to_owned());
                }
            }
        }
    }

    log::info!(
        "Following results were pushed by the flashing threads: {:#?}",
        flash_results
    );
}

/// Flashes a testbinary onto the provided target.
///
/// # Panics
/// If the provided [`TargetInfo`] struct fields `architecture` and/or `memory_address` are not initialized
fn flash_target(
    test_channel: &mut CombinedTestChannel,
    target_info: &TargetInfo,
    tss_pos: u8,
    result_sender: &SyncSender<FlashStatus>,
    testprogram: &TestProgram,
    flash_results: &Arc<RwLock<Vec<FlashStatus>>>,
) {
    // Check if Testchannel is ready and if the target_info has been successfully initialized.
    if !test_channel.is_ready() || target_info.status.is_err() {
        return;
    }

    // Check if this target was already flashed successfully
    let flash_results = flash_results.read().unwrap();
    if flash_results
        .iter()
        .filter(|result| {
            result.tss_pos == tss_pos
                && result.target_name == target_info.name
                && result.result.is_ok()
        })
        .count()
        != 0
    {
        return;
    }
    drop(flash_results); // Return lock

    let probe_info_lock = test_channel.get_probe_info().lock();
    let probe_info = probe_info_lock.as_ref().unwrap();

    match test_channel.take_probe_owned().attach(&target_info.name) {
        Ok(mut session) => {
            let mut download_options = DownloadOptions::default();
            download_options.do_chip_erase = true;

            let path = match target_info.architecture.as_ref().unwrap() {
                Architecture::ARM => {
                    testprogram.get_elf_path_arm(&target_info.memory_address.as_ref().unwrap())
                }
                Architecture::RISCV => {
                    testprogram.get_elf_path_riscv(&target_info.memory_address.as_ref().unwrap())
                }
            };

            match download_file_with_options(
                &mut session,
                path,
                Format::Elf,
                download_options,
            ) {
                Ok(_) => result_sender.send(FlashStatus {
                    tss_pos,
                    target_name: target_info.name.clone(),
                    result: Ok(()),
                }).expect("Failed to send results to main thread, the receiver might have been dropped unexpectedly."),
                Err(err) => result_sender.send(FlashStatus {
                    tss_pos,
                    target_name: target_info.name.clone(),
                    result: Err(err.to_string()),
                }).expect("Failed to send results to main thread, the receiver might have been dropped unexpectedly."),
            }
        }
        Err(err) => {
            result_sender.send(FlashStatus {
                tss_pos,
                target_name: target_info.name.clone(),
                result: Err(err.to_string()),
            }).expect("Failed to send results to main thread, the receiver might have been dropped unexpectedly.");
        }
    }

    // reinitialize probe, and transfer ownership back to test_channel
    match probe_info.open() {
        Ok(probe) => test_channel.return_probe(probe),
        Err(err) => {
            log::warn!(
                "Failed to reinitialize the debug probe connected to {}: {}. Skipping the remaining flash attempts on this Testchannel.",
                test_channel.get_channel(),
                err
            )
        }
    }
}
