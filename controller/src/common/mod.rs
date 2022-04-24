//! Common functionalities used in test runner and monitor

use std::mem::MaybeUninit;
use std::sync::Mutex;

use pca9535::{IoExpander, Pca9535Immediate};
use rppal::i2c::I2c;
use shared_bus::BusManagerStd;

use crate::{HiveIoExpander, PCA9535_BASE_ADDR};

pub mod init;
mod target_stack_shield;
mod test_channel;

pub use target_stack_shield::TargetStackShield;
pub use test_channel::CombinedTestChannel;

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
) -> Vec<Mutex<TargetStackShield>> {
    TargetStackShield::create_present_and_init(i2c_bus, io_expander)
}

/// Creates and returns all testchannels which are able to be shared across threads
pub fn create_shareable_testchannels() -> [Mutex<CombinedTestChannel>; 4] {
    CombinedTestChannel::new()
}

/// Detects if a Daugtherboard is present on each connected TSS, is true if present.
///
/// # Failure
/// In case the function fails to determine if a daughterboard is present on a TSS or not, it assumes that none is present.
/// If the false value is wrongly assumed by this function it will later cause a desync error in the initialization functions, which in turn forces the application to resync the hardware configuration.
fn detect_connected_daughterboards(tss: &Vec<Mutex<TargetStackShield>>) -> [bool; 8] {
    let mut detected = [false; 8];
    for tss in tss.iter() {
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
