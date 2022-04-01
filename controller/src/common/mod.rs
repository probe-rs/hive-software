use std::mem::MaybeUninit;
use std::sync::{Arc, Mutex};

use pca9535::{IoExpander, Pca9535Immediate};
use rppal::i2c::I2c;
use shared_bus::BusManagerStd;

use target_stack_shield::TargetStackShield;
use test_channel::CombinedTestChannel;

use crate::{HiveIoExpander, PCA9535_BASE_ADDR};

pub mod target_stack_shield;
pub mod test_channel;

/// Represents the state of a single MCU target on a daughterboard
#[derive(Debug, Clone)]
pub enum TargetState {
    Known(String),
    Unknown,
    NotConnected,
}

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

/// creates, initializes and returns all connected tss which are able to be shared across threads
pub fn create_shareable_tss(
    i2c_bus: &'static BusManagerStd<I2c>,
    io_expander: &'static [HiveIoExpander; 8],
) -> Arc<Vec<Mutex<TargetStackShield>>> {
    Arc::new(TargetStackShield::create_present_and_init(
        i2c_bus,
        io_expander,
    ))
}

/// Creates and returns all testchannels which are able to be shared across threads
pub fn create_shareable_testchannels() -> Arc<[Mutex<CombinedTestChannel>; 4]> {
    Arc::new(CombinedTestChannel::new())
}
