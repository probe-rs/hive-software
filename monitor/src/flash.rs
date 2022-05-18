//! Handles the flashing of the testbinaries onto the available targets
use std::error::Error;
use std::sync::mpsc;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

use comm_types::hardware::{Architecture, TargetInfo, TargetState};
use controller::common::CombinedTestChannel;
use probe_rs::flashing::Format;
use probe_rs::flashing::{download_file_with_options, DownloadOptions};
use probe_rs::DebugProbeInfo;
use probe_rs::Session;

use crate::database::{keys, CborDb, HiveDb};
use crate::{TESTCHANNELS, TSS};
use crate::testprogram::TestProgram;

#[derive(Debug)]
struct FlashStatus {
    tss_pos: u8,
    target_name: String,
    result: Result<(), String>,
}

/// Tries to flash the testbinaries onto all available targets.
pub(crate) fn flash_testbinaries(db: Arc<HiveDb>) {
    let active_testprogram: Arc<TestProgram> = Arc::new(db.config_tree.c_get(keys::config::ACTIVE_TESTPROGRAM).unwrap().expect("Failed to get the active testprogram. Flashing the testbinaries can only be performed once the active testprogram is known"));

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

        if let Some(targets) = targets {
            // save updated targets back to tss
            tss.set_targets(targets);
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
    testprogram: &TestProgram,
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

    let probe_info_lock = testchannel.get_probe_info().lock();
    let probe_info = probe_info_lock.as_ref().unwrap();

    log::info!(
        "Flashing testbinary onto target {} with probe {}",
        target_info.name,
        probe_info.identifier
    );

    let flash_result = retry_flash(testchannel, target_info, probe_info, |mut session| {
        let mut download_options = DownloadOptions::default();
        download_options.do_chip_erase = true;

        let path = match target_info.architecture.as_ref().unwrap() {
            Architecture::ARM => {
                testprogram.get_elf_path_arm(target_info.memory_address.as_ref().unwrap())
            }
            Architecture::RISCV => {
                testprogram.get_elf_path_riscv(target_info.memory_address.as_ref().unwrap())
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
    match probe_info.open() {
        Ok(probe) => testchannel.return_probe(probe),
        Err(err) => {
            log::warn!(
                "Failed to reinitialize the debug probe connected to {}: {}. Skipping the remaining flash attempts on this Testchannel.",
                testchannel.get_channel(),
                err
            )
        }
    }
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
