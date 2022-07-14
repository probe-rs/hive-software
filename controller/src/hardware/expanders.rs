//! Hive IO-Expanders used in each TSS
use pca9535::{IoExpander, Pca9535Immediate};
use rppal::i2c::I2c;
use shared_bus::BusManagerStd;

use super::{HiveIoExpander, MAX_TSS, PCA9535_BASE_ADDR};

/// Creates and returns all possible IO-Expanders on tss
pub fn create_expanders(i2c_bus: &'static BusManagerStd<I2c>) -> [HiveIoExpander; MAX_TSS] {
    let mut expanders: Vec<HiveIoExpander> = vec![];

    for idx in 0..MAX_TSS {
        expanders.push(IoExpander::new(Pca9535Immediate::new(
            i2c_bus.acquire_i2c(),
            idx as u8 + PCA9535_BASE_ADDR,
        )));
    }

    expanders.try_into().unwrap()
}
