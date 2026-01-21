//! This module provides an abstraction on all the functions attached to the IO Expander's GPIO pins.
//!
//! The main functions per target stack shield are:
//! - [`bus_switch`] for controlling the state of the bus switches which connect the targets with the probes
//! - [`status_led`] for controlling the status LED on the target stack shield
//! - [`daughterboard_detect`] for detecting if a daughterboard is mounted on the target stack shield
//!
//! Each of those modules provides basic functions to interact with the hardware.

use embedded_hal::i2c::I2c;
use pca9535::{
    ExpanderError, ExpanderInputPin, ExpanderOutputPin,
    GPIOBank::{Bank0, Bank1},
    PinState::High,
    expander::SyncExpander,
};

use super::StackShieldError;

mod bus_switch;
mod daughterboard_detect;
mod status_led;

use bus_switch::BusSwitch;
use daughterboard_detect::DaughterboardDetect;
use status_led::Led;

/// This struct contains all the devices attached to the IO Expander
pub(crate) struct ExpanderGpio<'a, I2C, T>
where
    I2C: I2c,
    T: SyncExpander<I2C>,
{
    pub status_led: Led<'a, I2C, T>,
    pub bus_switch: BusSwitch<'a, I2C, T>,
    pub daughterboard_detect: DaughterboardDetect<'a, I2C, T>,
}

impl<'a, I2C, T, E> ExpanderGpio<'a, I2C, T>
where
    E: std::fmt::Debug,
    I2C: I2c<Error = E>,
    T: SyncExpander<I2C>,
{
    /// Creates a new instance of the struct. It configures the IO Expander to the required settings. All GPIO pins are initialized into the default mode (eg. All bus switches disconnected, status LED off, daughterboard detect configured as input pin)
    pub fn new(expander: &'a T) -> Result<Self, StackShieldError<ExpanderError<E>>> {
        let detect = ExpanderInputPin::new(expander, Bank0, 3)
            .map_err(|err| StackShieldError::GpioError { source: err })?;

        let daughterboard_detect = DaughterboardDetect::new(detect);

        let red = ExpanderOutputPin::new(expander, Bank0, 0, High)
            .map_err(|err| StackShieldError::GpioError { source: err })?;
        let green = ExpanderOutputPin::new(expander, Bank0, 1, High)
            .map_err(|err| StackShieldError::GpioError { source: err })?;
        let blue = ExpanderOutputPin::new(expander, Bank0, 2, High)
            .map_err(|err| StackShieldError::GpioError { source: err })?;

        let status_led = Led::new(red, green, blue);

        let sw_target_0 = ExpanderOutputPin::new(expander, Bank1, 3, High)
            .map_err(|err| StackShieldError::GpioError { source: err })?;
        let sw_target_1 = ExpanderOutputPin::new(expander, Bank1, 2, High)
            .map_err(|err| StackShieldError::GpioError { source: err })?;
        let sw_target_2 = ExpanderOutputPin::new(expander, Bank1, 1, High)
            .map_err(|err| StackShieldError::GpioError { source: err })?;
        let sw_target_3 = ExpanderOutputPin::new(expander, Bank1, 0, High)
            .map_err(|err| StackShieldError::GpioError { source: err })?;

        let sw_probe_0 = ExpanderOutputPin::new(expander, Bank1, 7, High)
            .map_err(|err| StackShieldError::GpioError { source: err })?;
        let sw_probe_1 = ExpanderOutputPin::new(expander, Bank1, 6, High)
            .map_err(|err| StackShieldError::GpioError { source: err })?;
        let sw_probe_2 = ExpanderOutputPin::new(expander, Bank1, 5, High)
            .map_err(|err| StackShieldError::GpioError { source: err })?;
        let sw_probe_3 = ExpanderOutputPin::new(expander, Bank1, 4, High)
            .map_err(|err| StackShieldError::GpioError { source: err })?;

        let bus_switch = BusSwitch::new(
            [sw_target_0, sw_target_1, sw_target_2, sw_target_3],
            [sw_probe_0, sw_probe_1, sw_probe_2, sw_probe_3],
        );

        Ok(ExpanderGpio {
            status_led,
            bus_switch,
            daughterboard_detect,
        })
    }
}
