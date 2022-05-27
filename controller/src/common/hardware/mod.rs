//! Handles the entire Hive hardware
use std::sync::Mutex;

use comm_types::{
    hardware::ProbeState,
    ipc::{HiveProbeData, HiveTargetData},
};
use rppal::i2c::I2c;
use shared_bus::BusManager;
use thiserror::Error;

// Depending on the usecase, the probe-rs dependency is either stable, or the one being tested by Hive
#[cfg(not(feature = "runner"))]
use probe_rs::Probe;
#[cfg(feature = "runner")]
use probe_rs_test::Probe;

mod expanders;
mod target_stack_shield;
mod test_channel;

pub use expanders::create_expanders;
pub use target_stack_shield::TargetStackShield;
pub use test_channel::CombinedTestChannel;

use crate::HiveIoExpander;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("The probe hardware which was detected by the runner does not match with the data provided by the monitor")]
    ProbeInitDesync,
    #[error("The target hardware which was detected by the runner does not match with the data provided by the monitor")]
    TargetInitDesync,
}

/// The top level struct which contains the entire Hive testrack hardware handlers
pub struct HiveHardware {
    pub tss: Vec<Mutex<TargetStackShield>>,
    pub testchannels: [Mutex<CombinedTestChannel>; 4],
    pub hardware_status: HardwareStatus,
}

impl HiveHardware {
    pub fn new(
        i2c_bus: &'static BusManager<Mutex<I2c>>,
        io_expander: &'static [HiveIoExpander; 8],
    ) -> Self {
        Self {
            tss: TargetStackShield::create_present_and_init(i2c_bus, io_expander),
            testchannels: CombinedTestChannel::new(),
            hardware_status: HardwareStatus::Uninitialized,
        }
    }

    /// Initializes TSS with target data and checks for data desyncs between the currently detected hardware and the provided data
    pub fn initialize_target_data(&self, data: HiveTargetData) -> Result<(), InitError> {
        // Check for data desync
        let detected_daughterboards = self.detect_connected_daughterboards();
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
            if let Some(data) = data {
                return Some((idx, data));
            }
            None
        }) {
            let mut tss = self.tss[idx].lock().unwrap();
            tss.set_targets(targets);
        }

        Ok(())
    }

    /// Initializes Testchannels with the debug probes and checks for data desyncs between the currently detected probe hardware and the provided data
    pub fn initialize_probe_data(&self, data: HiveProbeData) -> Result<(), InitError> {
        let mut found_probes = Probe::list_all();

        log::debug!(
            "Found {} attached probes:\n{:#?}",
            found_probes.len(),
            found_probes
        );

        for (channel_idx, probe_info) in
            data.iter().enumerate().filter_map(|(channel_idx, data)| {
                if let ProbeState::Known(probe_info) = data {
                    return Some((channel_idx, probe_info));
                }
                None
            })
        {
            let mut found_probes_idx = 0;
            while found_probes_idx < found_probes.len() {
                if found_probes[found_probes_idx].identifier == probe_info.identifier
                    && found_probes[found_probes_idx].vendor_id == probe_info.vendor_id
                    && found_probes[found_probes_idx].product_id == probe_info.product_id
                    && found_probes[found_probes_idx].serial_number == probe_info.serial_number
                    && found_probes[found_probes_idx].hid_interface == probe_info.hid_interface
                {
                    let tss = self.testchannels[channel_idx].lock().unwrap();

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

    /// Detects if a Daugtherboard is present on each connected TSS, is true if present.
    ///
    /// # Failure
    /// In case the function fails to determine if a daughterboard is present on a TSS or not, it assumes that none is present.
    /// If the false value is wrongly assumed by this function it will later cause a desync error in the initialization functions, which in turn forces the application to resync the hardware configuration.
    fn detect_connected_daughterboards(&self) -> [bool; 8] {
        let mut detected = [false; 8];
        for tss in self.tss.iter() {
            let mut tss = tss.lock().unwrap();

            match tss.inner.get_mut().daughterboard_is_connected() {
                Ok(is_connected) => {
                    detected[tss.get_position() as usize] = is_connected;
                }
                Err(err) => {
                    log::warn!(
                    "Failed to detect daughterboard on TSS {}, assuming none is connected. \n\nCaused by: {}",
                    tss.get_position(),
                    err
                );
                }
            }
        }
        detected
    }
}

/// Global status of the [`HiveHardware`]
pub enum HardwareStatus {
    /// Hardware is not initialized
    Uninitialized,
    /// Hardware was initialized with invalid data which is out of sync with the actual detected hardware
    DataDesync,
    /// Hardware is ready for standard operation
    Ready,
}
