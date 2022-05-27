//! Hive IO-Expanders used in each TSS
use std::mem::MaybeUninit;

use pca9535::{IoExpander, Pca9535Immediate};
use rppal::i2c::I2c;
use shared_bus::BusManagerStd;

use crate::{HiveIoExpander, PCA9535_BASE_ADDR};

/// Creates and returns all possible IO-Expanders on tss
pub fn create_expanders(i2c_bus: &'static BusManagerStd<I2c>) -> [HiveIoExpander; 8] {
    let mut expanders: [MaybeUninit<HiveIoExpander>; 8] =
        unsafe { std::mem::MaybeUninit::uninit().assume_init() };

    for (idx, e) in &mut expanders.iter_mut().enumerate() {
        e.write(IoExpander::new(Pca9535Immediate::new(
            i2c_bus.acquire_i2c(),
            idx as u8 + PCA9535_BASE_ADDR,
        )));
    }

    unsafe { std::mem::transmute(expanders) }
}
