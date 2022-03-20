use embedded_hal::digital::blocking::InputPin;
use embedded_hal::i2c::blocking::{Write, WriteRead};
use pca9535::expander::SyncExpander;
use pca9535::{ExpanderError, ExpanderInputPin};

use crate::StackShieldError;

/// Abstraction struct for the daughterboard detect function
pub(crate) struct DaughterboardDetect<'a, I2C, T>
where
    I2C: Write + WriteRead,
    T: SyncExpander<I2C>,
{
    input: ExpanderInputPin<'a, I2C, T>,
}

impl<'a, I2C, T, E> DaughterboardDetect<'a, I2C, T>
where
    E: std::fmt::Debug,
    I2C: Write<Error = E> + WriteRead<Error = E>,
    T: SyncExpander<I2C>,
{
    /// Creates a new instance of the struct
    pub fn new(input: ExpanderInputPin<'a, I2C, T>) -> Self {
        Self { input }
    }

    /// Checks if the daughterboard is connected or not.
    pub fn is_connected(&mut self) -> Result<bool, StackShieldError<ExpanderError<E>>> {
        self.input
            .is_high()
            .map_err(|err| StackShieldError::DaughterboardDetectError { source: err })
    }
}
