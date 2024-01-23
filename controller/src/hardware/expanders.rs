//! Hive IO-Expanders used in each TSS
use std::sync::Mutex;

use embedded_hal_bus::i2c::MutexDevice;
use pca9535::{IoExpander, Pca9535Immediate};
use rppal::i2c::I2c;

use super::{HiveIoExpander, MAX_TSS, PCA9535_BASE_ADDR};

/// Creates and returns all possible IO-Expanders on tss
pub fn create_expanders(i2c_bus: &'static Mutex<I2c>) -> [HiveIoExpander; MAX_TSS] {
    let mut expanders: Vec<HiveIoExpander> = vec![];

    for idx in 0..MAX_TSS {
        expanders.push(IoExpander::new(Pca9535Immediate::new(
            MutexDevice::new(i2c_bus),
            idx as u8 + PCA9535_BASE_ADDR,
        )));
    }

    expanders.try_into().unwrap_or_else(|_| {
        panic!("Failed to turn vec into array. This is a bug, please file an issue")
    })
}
