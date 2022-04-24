//! Common initialization functions
use std::sync::Mutex;

use comm_types::ipc::{HiveProbeData, HiveTargetData};
use thiserror::Error;

// Depending on the usecase, the probe-rs dependency is either stable, or the one being tested by Hive
#[cfg(not(feature = "runner"))]
use probe_rs::Probe;
#[cfg(feature = "runner")]
use probe_rs_test::Probe;

use super::{detect_connected_daughterboards, CombinedTestChannel, TargetStackShield};

#[derive(Debug, Error)]
pub enum InitError {
    #[error("The probe hardware which was detected by the runner does not match with the data provided by the monitor")]
    ProbeInitDesync,
    #[error("The target hardware which was detected by the runner does not match with the data provided by the monitor")]
    TargetInitDesync,
}

/// Initializes TSS with target data and checks for data desyncs between the currently detected hardware and the provided data
pub fn initialize_target_data(
    tss: &Vec<Mutex<TargetStackShield>>,
    data: HiveTargetData,
) -> Result<(), InitError> {
    // Check for data desync
    let detected_daughterboards = detect_connected_daughterboards(tss);
    if data
        .iter()
        .enumerate()
        .filter(|(idx, data)| {
            if data.is_some() == detected_daughterboards[*idx] {
                return false;
            }
            true
        })
        .count()
        != 0
    {
        log::error!("Encountered data desync during target data initialization.\nData received:\n{:#?}\nDaughterboards detected by the application:\n{:#?}", data, detected_daughterboards);
        return Err(InitError::TargetInitDesync);
    }

    for (idx, targets) in data.into_iter().enumerate().filter_map(|(idx, data)| {
        if data.is_some() {
            return Some((idx, data.unwrap()));
        }
        None
    }) {
        let mut tss = tss[idx].lock().unwrap();
        tss.set_targets(targets);
    }

    Ok(())
}

/// Initializes Testchannels with the debug probes and checks for data desyncs between the currently detected probe hardware and the provided data
pub fn initialize_probe_data(
    testchannels: &[Mutex<CombinedTestChannel>; 4],
    data: HiveProbeData,
) -> Result<(), InitError> {
    let mut found_probes = Probe::list_all();

    log::debug!(
        "Found {} attached probes:\n{:#?}",
        found_probes.len(),
        found_probes
    );

    for (channel_idx, probe_info) in data.iter().enumerate().filter_map(|(channel_idx, data)| {
        if data.is_some() {
            return Some((channel_idx, data.as_ref().unwrap()));
        }
        None
    }) {
        let mut found_probes_idx = 0;
        while found_probes_idx < found_probes.len() {
            if found_probes[found_probes_idx].identifier == probe_info.identifier
                && found_probes[found_probes_idx].vendor_id == probe_info.vendor_id
                && found_probes[found_probes_idx].product_id == probe_info.product_id
                && found_probes[found_probes_idx].serial_number == probe_info.serial_number
                && found_probes[found_probes_idx].hid_interface == probe_info.hid_interface
            {
                let tss = testchannels[channel_idx].lock().unwrap();

                let probe_info = found_probes.remove(found_probes_idx);
                let probe = probe_info.open().expect("TODO either skip probe or panic");

                tss.bind_probe(probe, probe_info);
                break;
            } else {
                found_probes_idx += 1;
            }
        }
    }

    // Check for data desync
    if !found_probes.is_empty() {
        log::error!("Encountered data desync during probe data initialization.\nData received:\n{:#?}\nProbes detected by the application, which are not found in the received data:\n{:#?}", data, found_probes);
        return Err(InitError::ProbeInitDesync);
    }

    Ok(())
}
