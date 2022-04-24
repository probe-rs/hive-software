use std::{cell::RefCell, error::Error, sync::Mutex};

use comm_types::hardware::{TargetInfo, TargetState};
use embedded_hal::i2c::blocking::WriteRead;
use ll_api::TargetStackShield as Shield;
use pca9535::Register;
use rppal::i2c::I2c;
use shared_bus::BusManagerStd;

use crate::{HiveIoExpander, ShareableI2c, PCA9535_BASE_ADDR};

pub struct TargetStackShield {
    pub inner: RefCell<Shield<'static, ShareableI2c, HiveIoExpander>>,
    position: u8,
    targets: Option<[TargetState; 4]>,
}

impl TargetStackShield {
    /// Creates and returns all tss which are connected and successfully initialized
    pub fn create_present_and_init(
        i2c_bus: &'static BusManagerStd<I2c>,
        io_expander: &'static [HiveIoExpander; 8],
    ) -> Vec<Mutex<Self>> {
        let i2c: ShareableI2c = i2c_bus.acquire_i2c();
        let detected_tss = Self::detect_connected_tss(i2c);

        let mut created = vec![];

        for detected_addr in detected_tss.iter().filter_map(|tss| tss.as_ref()) {
            let tss = Self {
                inner: RefCell::new(Shield::new(
                    &io_expander[(*detected_addr - PCA9535_BASE_ADDR) as usize],
                )),
                position: *detected_addr - PCA9535_BASE_ADDR,
                targets: None,
            };

            created.push(Mutex::new(tss));
        }

        let mut i = 0;
        while i < created.len() {
            let tss = created[i].lock().unwrap();
            let mut inner = tss.inner.borrow_mut();
            match inner.init_pins() {
                Ok(_) => i += 1,
                Err(err) => {
                    if let Some(source) = err.source() {
                        log::warn!(
                            "Failed to initialize TSS at position {}: {}\nCaused by:\n{}",
                            tss.position,
                            err,
                            source
                        );
                    } else {
                        log::warn!(
                            "Failed to initialize TSS at position {}: {}",
                            tss.position,
                            err
                        );
                    }
                    drop(inner);
                    drop(tss); // unlock mutex

                    created.remove(i);
                }
            }
        }

        created
    }

    /// Sets the currently connected target states, if a daughterboard is connected
    pub fn set_targets(&mut self, targets: [TargetState; 4]) {
        let is_connected = match self.inner.borrow_mut().daughterboard_is_connected() {
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

        if is_connected {
            self.targets = Some(targets);
        }
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

    pub fn get_targets(&self) -> &Option<[TargetState; 4]> {
        &self.targets
    }

    /// Detects all connected TSS by trying to read an IO-Expander register on each possible i2c address
    fn detect_connected_tss(mut i2c: ShareableI2c) -> Vec<Option<u8>> {
        let mut detected = vec![];
        for i in 0..=7 {
            match i2c.write_read(
                PCA9535_BASE_ADDR + i,
                &[Register::ConfigurationPort0 as u8],
                &mut [0],
            ) {
                Err(_) => {
                    detected.push(None);
                }
                Ok(_) => detected.push(Some(PCA9535_BASE_ADDR + i)),
            }
        }

        log::debug!("Detected TSS: {:#?}", detected);

        detected
    }
}
