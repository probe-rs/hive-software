//! Handles the flashing of the testbinaries onto the available targets
use std::error::Error;
use std::sync::mpsc;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::sync::RwLock;

use comm_types::hardware::{Architecture, TargetInfo, TargetState};
use controller::common::hardware::{CombinedTestChannel, HardwareStatus};
use hive_db::CborTransactional;
use probe_rs::flashing::{download_file_with_options, DownloadOptions, Format};
use probe_rs::{DebugProbeInfo, Session};
use crossbeam_utils::thread;
use sled::transaction::UnabortableTransactionError;

use crate::database::{keys, MonitorDb};
use crate::{HARDWARE};
use crate::testprogram::Testprogram;

#[derive(Debug)]
struct FlashStatus {
    tss_pos: u8,
    target_name: String,
    result: Result<(), String>,
}

/// Tries to flash the testbinaries onto all available targets.
/// 
/// This function does nothing in case the [`HARDWARE`] static is not [`HardwareStatus::Ready`]
pub(crate) fn flash_testbinaries(db: Arc<MonitorDb>) {
    let hardware = HARDWARE.lock().unwrap();

    if hardware.hardware_status != HardwareStatus::Ready {
        return;
    }

    let active_testprogram = db.config_tree.transaction::<_, _, UnabortableTransactionError>(|tree|{
        let active = tree.c_get(&keys::config::ACTIVE_TESTPROGRAM)?.expect("Failed to get the active testprogram. Flashing the testbinaries can only be performed once the active testprogram is known");

        let mut testprograms = tree.c_get(&keys::config::TESTPROGRAMS)?.expect("DB not initialized");

        for idx in 0..testprograms.len() {
            if active != testprograms[idx].get_name() {
                continue;
            }

            return Ok(testprograms.remove(idx));
        }
        panic!("Failed to find active testprogram in database. This should not happen as it indicates a desync between the active testprogram DB data and the testprogram DB data.");
    }).unwrap();

    let active_testprogram = Arc::new(active_testprogram);

    // A buffersize of 0 enforces that the RwLock flash_results vector does not slowly get out of sync due to read locks.
    // This could lead to situations where a thread checks the FlashStatus on an already invalid flash_results vector thus causing unwanted flashes of already successfully flashed targets.
    // The sync channel forces the sender to block, until the data has been received.
    let (result_sender, result_receiver) = mpsc::sync_channel::<FlashStatus>(0);

    // As we don't know if some probes will work for flashing certain targets, we just try out every available probe until we reach a successful flash or a definitive failure. The logic used here is very similar to the test runner logic.
    let flash_results = Arc::new(RwLock::new(vec![]));

    let tss = &hardware.tss;

    // Spawn scoped threads which can access tss reference. It gurantees that the hardware Mutexguard lives longer than the threads spawned within this scope
    thread::scope(|s|{
        let mut flashing_threads = vec![];

        for (idx, test_channel) in hardware.testchannels.iter().enumerate() {
            let channel = test_channel.lock().unwrap();
    
            if channel.is_ready() {
                drop(channel);
                let result_sender = result_sender.clone();
                let active_testprogram = active_testprogram.clone();
                let flash_results = flash_results.clone();
                
                flashing_threads.push(
                    s.builder()
                        .name(format!("flashing thread {}", idx).to_owned())
                        .spawn(move |_| {
                            let mut channel = test_channel.lock().unwrap();
                            let sender = result_sender;
    
                            channel.connect_all_available_and_execute(
                                tss,
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
    }).unwrap();

    // Update tss targets with the flash results
    for tss in hardware.tss.iter().filter_map(|tss| tss.as_ref()) {
        let mut tss = tss.lock().unwrap();

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
                    // Target is not included in the flash_results due to previous init errors
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

        if targets.is_some() {
            // save updated targets back to tss and ignore any data desyncs
            let _ = tss.set_targets(targets);
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
    testchannel: &mut CombinedTestChannel,
    target_info: &TargetInfo,
    tss_pos: u8,
    result_sender: &SyncSender<FlashStatus>,
    testprogram: &Testprogram,
    flash_results: &Arc<RwLock<Vec<FlashStatus>>>,
) {
    // Check if Testchannel is ready and if the target_info has been successfully initialized.
    if !testchannel.is_ready() || target_info.status.is_err() {
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

    let probe_info = testchannel.get_probe_info().unwrap();

    log::info!(
        "Flashing testbinary onto target {} with probe {}",
        target_info.name,
        probe_info.identifier
    );

    let flash_result = retry_flash(testchannel, target_info, &probe_info, |mut session| {
        let mut download_options = DownloadOptions::default();
        download_options.do_chip_erase = true;

        let path = match target_info.architecture.as_ref().unwrap() {
            Architecture::ARM => {
                testprogram.get_arm().get_elf_path(target_info.memory_address.as_ref().unwrap())
            }
            Architecture::RISCV => {
                testprogram.get_riscv().get_elf_path(target_info.memory_address.as_ref().unwrap())
            }
        };

        download_file_with_options(&mut session, path, Format::Elf, download_options)?;

        Ok(())
    });

    match flash_result {
        Ok(_) => result_sender.send(FlashStatus {
            tss_pos,
            target_name: target_info.name.clone(),
            result: Ok(()),
        }).expect("Failed to send results to main thread, the receiver might have been dropped unexpectedly."),
        Err(err) => {
            let source = match err.source(){
                Some(source) => source.to_string(),
                None => "No source".to_owned(),
            };
            
            result_sender.send(FlashStatus {
            tss_pos,
            target_name: target_info.name.clone(),
            result: Err(format!("{}: {}", err, source)),
        }).expect("Failed to send results to main thread, the receiver might have been dropped unexpectedly.")},
    }

    // reinitialize probe, and transfer ownership back to test_channel
    testchannel.reinitialize_probe().unwrap_or_else(|err|{
        log::warn!(
            "Failed to reinitialize the debug probe connected to {}: {}. Skipping the remaining flash attempts on this Testchannel.",
            testchannel.get_channel(),
            err
        )
    })
}

/// Retries the provided flash function with option attach-under-reset if it fails without
fn retry_flash<F>(
    testchannel: &CombinedTestChannel,
    target_info: &TargetInfo,
    probe_info: &DebugProbeInfo,
    flash: F,
) -> Result<(), Box<dyn Error>>
where
    F: Fn(Session) -> Result<(), Box<dyn Error>>,
{
    let mut probe = testchannel.take_probe_owned();
    let _ = probe.set_speed(8000);
    let session = probe.attach(&target_info.name);

    if let Ok(session) = session {
        let flash_result = flash(session);
        match flash_result {
            Ok(_) => return Ok(()),
            Err(err) => log::warn!(
                "Failed to flash target {} with probe {}: {}\nRetrying with attach-under-reset",
                target_info.name,
                probe_info.identifier,
                err
            ),
        }
    } else {

        let err = session.unwrap_err();
        log::warn!(
            "Failed to flash target {} with probe {}: {}, {:?}\nRetrying with attach-under-reset",
            target_info.name,
            probe_info.identifier,
            err,
            err.source()
        ) 
    }

    let mut probe = probe_info.open()?;
    let _ = probe.set_speed(8000);
    let session = probe.attach_under_reset(&target_info.name)?;

    flash(session)
}
