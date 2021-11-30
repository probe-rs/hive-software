extern crate embedded_hal as hal;
extern crate pca9535;

use pca9535::expander::SyncExpander;
use thiserror::Error;

mod pins;

const EXPANDER_BASE_ADDRESS: u8 = 32;

#[derive(Debug)]
pub enum Status {
    Idle,
    Err,
    NoBoard,
}

pub enum Target {
    Target0,
    Target1,
    Target2,
    Target3,
}

pub enum Probe {
    Probe0,
    Probe1,
    Probe2,
    Probe3,
}

#[derive(Error, Debug)]
pub enum StackShieldError<E> {
    #[error("Failed to control stack shield LED")]
    LedError(E),
    #[error("Failed to control stack shield GPIOs")]
    GpioError {
        #[from]
        source: E,
    },
}

struct StackShield<'a, T>
where
    T: SyncExpander,
{
    pub position: u8,
    pub status: Status,
    pins: ExpanderPins<'a, T>,
}
