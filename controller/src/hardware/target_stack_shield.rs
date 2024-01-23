use std::{cell::RefCell, error::Error, sync::Mutex};

use comm_types::hardware::{TargetInfo, TargetState};
use embedded_hal::i2c::I2c as HalI2c;
use embedded_hal_bus::i2c::MutexDevice;
use ll_api::TargetStackShield as Shield;
use pca9535::Register;
use rppal::i2c::I2c;

use super::{HiveIoExpander, ShareableI2c, MAX_DAUGHTERBOARD_TARGETS, MAX_TSS, PCA9535_BASE_ADDR};

pub struct TargetStackShield {
    pub inner: RefCell<Shield<'static, ShareableI2c, HiveIoExpander>>,
    position: u8,
    targets: Option<[TargetState; MAX_DAUGHTERBOARD_TARGETS]>,
}

impl TargetStackShield {
    /// Creates and returns all tss, and initializes the connected tss
    pub fn new(
        i2c_bus: &'static Mutex<I2c>,
        io_expander: &'static [HiveIoExpander; MAX_TSS],
    ) -> [Option<Mutex<Self>>; MAX_TSS] {
        let i2c: ShareableI2c = MutexDevice::new(i2c_bus);
        let detected_tss = Self::detect_connected_tss(i2c);

        let mut created = [None, None, None, None, None, None, None, None];

        for (idx, detected_addr) in detected_tss.iter().enumerate() {
            if let Some(addr) = detected_addr {
                let new_tss = Self {
                    inner: RefCell::new(Shield::new(
                        &io_expander[(*addr - PCA9535_BASE_ADDR) as usize],
                    )),
                    position: *addr - PCA9535_BASE_ADDR,
                    targets: None,
                };

                // Try to initialize tss, and add to created array if successful
                let mut init_success = true;
                new_tss
                    .inner
                    .borrow_mut()
                    .init_pins()
                    .unwrap_or_else(|err| {
                        if let Some(source) = err.source() {
                            log::warn!(
                                "Failed to initialize TSS at position {}: {}\nCaused by:\n{}",
                                new_tss.position,
                                err,
                                source
                            );
                        } else {
                            log::warn!(
                                "Failed to initialize TSS at position {}: {}",
                                new_tss.position,
                                err
                            );
                        }

                        init_success = false;
                    });

                if init_success {
                    created[idx] = Some(Mutex::new(new_tss));
                }
            }
        }

        created
    }

    /// Sets the currently connected targets. [`None`] means that no daughterboard is connected.
    ///
    /// # Data desync
    /// This function internally checks if a daughterboard is actually connected or not. In case the user input differs from the actually detected state on the hardware
    /// (For example if the user provides [`None`] but there is a daughterboard connected) this function fails with an [`Err`] and resets the targets field of the struct to the appropriate default value.
    pub fn set_targets(
        &mut self,
        targets: Option<[TargetState; MAX_DAUGHTERBOARD_TARGETS]>,
    ) -> Result<(), ()> {
        let daughterboard_is_connected = match self.inner.borrow_mut().daughterboard_is_connected()
        {
            Ok(connected) => connected,
            Err(err) => {
                if let Some(source) = err.source() {
                    log::warn!(
                        "Failed to determine if a daughterboard is connected to tss {}: {}\nCaused by:\n{}",
                        self.position,
                        err,
                        source
                    );
                } else {
                    log::warn!(
                        "Failed to determine if a daughterboard is connected to tss {}: {}",
                        self.position,
                        err
                    );
                }

                false
            }
        };

        // Check for data desync
        if daughterboard_is_connected && targets.is_none()
            || !daughterboard_is_connected && targets.is_some()
        {
            // Apply defaults
            match daughterboard_is_connected {
                true => {
                    self.targets = Some([
                        TargetState::Unknown,
                        TargetState::Unknown,
                        TargetState::Unknown,
                        TargetState::Unknown,
                    ])
                }
                false => self.targets = None,
            }

            return Err(());
        }

        self.targets = targets;

        Ok(())
    }

    /// Sets the target info of given target index.
    ///
    /// # Panics
    /// If there is no daughterboard connected or if the current target state is not [`TargetState::Known`]
    pub fn set_target_info(&mut self, pos: usize, info: TargetInfo) {
        let mut is_known = false;
        if let TargetState::Known(_) = &self.targets.as_ref().unwrap()[pos] {
            is_known = true;
        }

        if is_known {
            self.targets.as_mut().unwrap()[pos] = TargetState::Known(info);
        } else {
            panic!("The target state on pos {} is not TargetState::Known, cannot set TargetInfo on an unknown target", pos);
        }
    }

    pub fn get_position(&self) -> u8 {
        self.position
    }

    pub fn get_targets(&self) -> &Option<[TargetState; MAX_DAUGHTERBOARD_TARGETS]> {
        &self.targets
    }

    pub fn get_targets_mut(&mut self) -> &mut Option<[TargetState; MAX_DAUGHTERBOARD_TARGETS]> {
        &mut self.targets
    }

    /// Detects all connected TSS by trying to read an IO-Expander register on each possible i2c address. Returns the detected i2c addresses or [`None`]
    pub fn detect_connected_tss(mut i2c: ShareableI2c) -> [Option<u8>; MAX_TSS] {
        let mut detected: [Option<u8>; MAX_TSS] = Default::default();
        for (i, item) in detected.iter_mut().enumerate() {
            match i2c.write_read(
                PCA9535_BASE_ADDR + i as u8,
                &[Register::ConfigurationPort0 as u8],
                &mut [0],
            ) {
                Err(err) => {
                    log::warn!(
                        "Failed to detect TSS {}, assuming none is connected. \n\nCaused by: {}",
                        i,
                        err
                    );

                    *item = None;
                }
                Ok(_) => *item = Some(PCA9535_BASE_ADDR + i as u8),
            }
        }

        log::debug!("Detected TSS: {:#?}", detected);

        detected
    }
}
