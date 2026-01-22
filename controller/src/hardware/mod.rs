//! Handles the entire Hive hardware
use std::sync::Mutex;

use comm_types::{
    hardware::ProbeState,
    ipc::{HiveProbeData, HiveTargetData},
};
use embedded_hal_bus::i2c::MutexDevice;
use pca9535::{IoExpander, Pca9535Immediate};
use rppal::i2c::I2c;

use thiserror::Error;

// Depending on the usecase, the probe-rs dependency is either stable, or the one being tested by Hive
#[cfg(feature = "monitor")]
use probe_rs::probe::list::Lister;
#[cfg(feature = "runner")]
use probe_rs_test::probe::list::Lister;

mod expanders;
mod probe;
mod target_stack_shield;
mod test_channel;

pub use expanders::create_expanders;
pub use probe::reset_probe_usb;
pub use probe::try_attach;
pub use target_stack_shield::TargetStackShield;
pub use test_channel::CombinedTestChannel;

/// The base address of the IO Expander used on each TSS
const PCA9535_BASE_ADDR: u8 = 32;
/// The max amount of TSS which can be attached to a single Hive testrack
pub const MAX_TSS: usize = 8;
/// The max amount of targets a single daughterboard can contain
pub const MAX_DAUGHTERBOARD_TARGETS: usize = 4;

pub type ShareableI2c = MutexDevice<'static, I2c>;
pub type HiveIoExpander =
    IoExpander<ShareableI2c, Pca9535Immediate<ShareableI2c>, Mutex<Pca9535Immediate<ShareableI2c>>>;

#[derive(Debug, Error)]
pub enum InitError {
    #[error(
        "The probe hardware which was detected by the runner does not match with the data provided by the monitor"
    )]
    ProbeInitDesync,
    #[error(
        "The target hardware which was detected by the runner does not match with the data provided by the monitor"
    )]
    TargetInitDesync,
}

/// The top level struct which contains the entire Hive testrack hardware handlers
pub struct HiveHardware {
    pub tss: [Option<Mutex<TargetStackShield>>; MAX_TSS],
    pub testchannels: [Mutex<CombinedTestChannel>; MAX_DAUGHTERBOARD_TARGETS],
    pub hardware_status: HardwareStatus,
}

impl HiveHardware {
    pub fn new(
        i2c_bus: &'static Mutex<I2c>,
        io_expander: &'static [HiveIoExpander; MAX_TSS],
    ) -> Self {
        Self {
            tss: TargetStackShield::new(i2c_bus, io_expander),
            testchannels: CombinedTestChannel::new(),
            hardware_status: HardwareStatus::Uninitialized,
        }
    }

    /// Initializes TSS with target data and checks for data desyncs between the currently detected hardware and the provided data.
    ///
    /// In case of a data desync the target data is set to the appropriate default automatically before an [`Err`] is returned.
    pub fn initialize_target_data(&self, data: HiveTargetData) -> Result<(), InitError> {
        let mut data_desync = false;

        for (idx, targets) in data.into_iter().enumerate() {
            if self.tss[idx].is_none() && targets.is_some() {
                data_desync = true;
                continue;
            }

            if let Some(tss) = self.tss[idx].as_ref() {
                let mut tss = tss.lock().unwrap();
                tss.set_targets(targets)
                    .unwrap_or_else(|_| data_desync = true);
            }
        }

        if data_desync {
            return Err(InitError::TargetInitDesync);
        }

        Ok(())
    }

    /// Initializes Testchannels with the debug probes and checks for data desyncs between the currently detected probe hardware and the provided data
    ///
    /// In case of a data desync the probe data associated to the testchannel is reset to [`None`]
    pub fn initialize_probe_data(&self, data: HiveProbeData) -> Result<(), InitError> {
        // Remove all probes which are still registered or opened in the struct
        for testchannel in self.testchannels.iter() {
            let testchannel = testchannel.lock().unwrap();

            testchannel.remove_probe();
        }

        let probe_lister = Lister::new();

        let mut found_probes = probe_lister.list_all();
        let mut data_desync = false;

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
            let mut found_probe = false;
            while found_probes_idx < found_probes.len() {
                if found_probes[found_probes_idx].vendor_id == probe_info.vendor_id
                    && found_probes[found_probes_idx].product_id == probe_info.product_id
                    && found_probes[found_probes_idx].serial_number == probe_info.serial_number
                    && found_probes[found_probes_idx].interface == probe_info.hid_interface
                {
                    let tss = self.testchannels[channel_idx].lock().unwrap();

                    let probe_info = found_probes.remove(found_probes_idx);
                    let probe = probe_info.open().expect("TODO either skip probe or panic");

                    tss.bind_probe(probe, probe_info);
                    found_probe = true;
                    break;
                } else {
                    found_probes_idx += 1;
                }
            }

            if !found_probe {
                data_desync = true;
                let tss = self.testchannels[channel_idx].lock().unwrap();

                tss.remove_probe();
            }
        }

        if data_desync {
            log::warn!(
                "Encountered data desync during probe data initialization.\nData received:\n{:#?}\nProbes detected by the application, which are not found in the received data:\n{:#?}",
                data,
                found_probes
            );
            return Err(InitError::ProbeInitDesync);
        }

        Ok(())
    }
}

/// Global status of the [`HiveHardware`]
#[derive(Debug, PartialEq, Eq)]
pub enum HardwareStatus {
    /// Hardware is not initialized
    Uninitialized,
    /// Hardware was initialized with invalid data which is out of sync with the actual detected hardware
    DataDesync,
    /// Hardware is ready for standard operation
    Ready,
}
