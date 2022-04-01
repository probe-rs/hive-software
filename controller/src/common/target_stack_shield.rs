use std::{error::Error, sync::Mutex};

use embedded_hal::i2c::blocking::WriteRead;
use ll_api::TargetStackShield as Shield;
use pca9535::Register;
use rppal::i2c::I2c;
use shared_bus::BusManagerStd;

use crate::{HiveIoExpander, ShareableI2c, PCA9535_BASE_ADDR};

use super::TargetState;

pub struct TargetStackShield {
    inner: Shield<'static, ShareableI2c, HiveIoExpander>,
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
                inner: Shield::new(&io_expander[(*detected_addr - PCA9535_BASE_ADDR) as usize]),
                position: *detected_addr - PCA9535_BASE_ADDR,
                targets: None,
            };

            created.push(Mutex::new(tss));
        }

        let mut i = 0;
        while i < created.len() {
            let mut tss = created[i].lock().unwrap();
            match tss.inner.init_pins() {
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
                    drop(tss); // unlock mutex

                    created.remove(i);
                }
            }
        }

        created
    }

    pub fn set_targets(&mut self) {}

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
