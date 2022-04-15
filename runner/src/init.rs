//! Handles all initialization required to run the tests
use comm_types::ipc::{HiveProbeData, HiveTargetData};
use probe_rs_test::Probe;
use thiserror::Error;

use crate::{detect_connected_daughterboards, EXPANDERS, SHARED_I2C, TESTCHANNELS, TSS};

#[derive(Debug, Error)]
pub(crate) enum InitError {
    #[error("The probe hardware which was detected by the runner does not match with the data provided by the monitor")]
    ProbeInitDesync,
    #[error("The target hardware which was detected by the runner does not match with the data provided by the monitor")]
    TargetInitDesync,
    #[error("Failed to reinitialize the probe during testing")]
    ProbeReinitError,
}

pub(crate) fn initialize_statics() {
    lazy_static::initialize(&SHARED_I2C);
    lazy_static::initialize(&EXPANDERS);
    lazy_static::initialize(&TSS);
    lazy_static::initialize(&TESTCHANNELS);
}

/// Initializes TSS with target data and checks for data desyncs between the currently detected hardware and the data from the monitor
pub(crate) fn initialize_target_data(data: HiveTargetData) -> Result<(), InitError> {
    // Check for data desync
    let detected_daughterboards = detect_connected_daughterboards();
    if data
        .iter()
        .enumerate()
        .filter(|(idx, data)| {
            if data.is_some() && detected_daughterboards[*idx] {
                return false;
            }
            true
        })
        .count()
        != 0
    {
        log::error!("Encountered data desync during target data initialization.\nData received from monitor:\n{:#?}\nDaughterboards detected by runner:\n{:#?}", data, detected_daughterboards);
        return Err(InitError::TargetInitDesync);
    }

    for (idx, targets) in data.into_iter().enumerate().filter_map(|(idx, data)| {
        if data.is_some() {
            return Some((idx, data.unwrap()));
        }
        None
    }) {
        let mut tss = TSS[idx].lock().unwrap();
        tss.set_targets(targets);
    }

    Ok(())
}

/// Initializes Testchannels with the debug probes and checks for data desyncs between the currently detected probe hardware and the data from the monitor
pub(crate) fn initialize_probe_data(data: HiveProbeData) -> Result<(), InitError> {
    let mut found_probes = Probe::list_all();
    let found_probes_len = found_probes.len();

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
                && found_probes[found_probes_idx].hid_interface == Some(probe_info.usb_port)
            {
                let tss = TESTCHANNELS[channel_idx].lock().unwrap();

                let probe_info = found_probes.remove(found_probes_idx);
                let probe = probe_info
                    .open()
                    .expect("TODO either skip probe for test run or panic");

                tss.bind_probe(probe, probe_info);
                break;
            } else {
                found_probes_idx += 1;
            }
        }
    }

    // Check for data desync
    if !found_probes.is_empty() || data.len() != found_probes_len {
        log::error!("Encountered data desync during probe data initialization.\nData received from monitor:\n{:#?}\nProbes detected by runner, which are not found in monitor data:\n{:#?}", data, found_probes);
        return Err(InitError::ProbeInitDesync);
    }

    Ok(())
}
