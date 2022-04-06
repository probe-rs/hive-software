use std::sync::Mutex;

use pca9535::{IoExpander, Pca9535Immediate};
use rppal::i2c::I2c;
use shared_bus::I2cProxy;

const PCA9535_BASE_ADDR: u8 = 32;

pub type ShareableI2c = I2cProxy<'static, Mutex<I2c>>;
pub type HiveIoExpander =
    IoExpander<ShareableI2c, Pca9535Immediate<ShareableI2c>, Mutex<Pca9535Immediate<ShareableI2c>>>;

pub mod common;
#[cfg(feature = "runner")]
pub mod runner;
