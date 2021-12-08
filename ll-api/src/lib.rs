extern crate embedded_hal as hal;
extern crate pca9535;
extern crate retry;

use expander_gpio::ExpanderGpio;
use pca9535::expander::SyncExpander;
use thiserror::Error;

mod expander_gpio;

const EXPANDER_BASE_ADDRESS: u8 = 32;

#[derive(Debug)]
pub enum Status {
    Idle,
    Err,
    NoBoard,
}

pub enum Target {
    Target0 = 0,
    Target1 = 1,
    Target2 = 2,
    Target3 = 3,
}

pub enum Probe {
    Probe0 = 0,
    Probe1 = 1,
    Probe2 = 2,
    Probe3 = 3,
}

#[derive(Error, Debug)]
pub enum StackShieldError<ERR>
where
    ERR: core::fmt::Debug,
{
    #[error("Failed to control stack shield LED")]
    LedError(ERR),
    #[error("Failed to control stack shield GPIOs")]
    GpioError(ERR),
    #[error("Failed to control bus switches")]
    BusSwitchError(ERR),
    #[error("Failed to detect if daugherboard is present or not")]
    DaughterboardDetectError(ERR),
}

struct StackShield<'a, T>
where
    T: SyncExpander,
{
    pub position: u8,
    pub status: Status,
    pins: ExpanderGpio<'a, T>,
}
