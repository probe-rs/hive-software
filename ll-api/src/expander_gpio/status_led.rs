use embedded_hal::digital::OutputPin;
use embedded_hal::i2c::I2c;
use pca9535::expander::SyncExpander;
use pca9535::ExpanderError;
use pca9535::ExpanderOutputPin;

use crate::StackShieldError;
use crate::StackShieldStatus;

/// Abstraction struct for the status LED function
pub(crate) struct Led<'a, I2C, T>
where
    I2C: I2c,
    T: SyncExpander<I2C>,
{
    red: ExpanderOutputPin<'a, I2C, T>,
    green: ExpanderOutputPin<'a, I2C, T>,
    blue: ExpanderOutputPin<'a, I2C, T>,
}

impl<'a, I2C, T, E> Led<'a, I2C, T>
where
    E: std::fmt::Debug,
    I2C: I2c<Error = E>,
    T: SyncExpander<I2C>,
{
    /// Creates a new instance of the struct
    pub fn new(
        red: ExpanderOutputPin<'a, I2C, T>,
        green: ExpanderOutputPin<'a, I2C, T>,
        blue: ExpanderOutputPin<'a, I2C, T>,
    ) -> Self {
        Self { red, green, blue }
    }

    /// Controls the status LED according to the provided [`Status`].
    /// The LED lights up as follows
    /// - [`Status::Err`] red
    /// - [`Status::Idle`] green
    /// - [`Status::NoBoard`] blue
    pub fn set_status(
        &mut self,
        status: StackShieldStatus,
    ) -> Result<(), StackShieldError<ExpanderError<E>>> {
        match status {
            StackShieldStatus::Err => {
                self.blue
                    .set_high()
                    .map_err(|err| StackShieldError::LedError { source: err })?;
                self.green
                    .set_high()
                    .map_err(|err| StackShieldError::LedError { source: err })?;
                self.red
                    .set_low()
                    .map_err(|err| StackShieldError::LedError { source: err })
            }
            StackShieldStatus::Idle => {
                self.blue
                    .set_high()
                    .map_err(|err| StackShieldError::LedError { source: err })?;
                self.green
                    .set_low()
                    .map_err(|err| StackShieldError::LedError { source: err })?;
                self.red
                    .set_high()
                    .map_err(|err| StackShieldError::LedError { source: err })
            }
            StackShieldStatus::NoBoard => {
                self.blue
                    .set_low()
                    .map_err(|err| StackShieldError::LedError { source: err })?;
                self.green
                    .set_high()
                    .map_err(|err| StackShieldError::LedError { source: err })?;
                self.red
                    .set_high()
                    .map_err(|err| StackShieldError::LedError { source: err })
            }
            _ => {
                self.blue
                    .set_high()
                    .map_err(|err| StackShieldError::LedError { source: err })?;
                self.green
                    .set_high()
                    .map_err(|err| StackShieldError::LedError { source: err })?;
                self.red
                    .set_high()
                    .map_err(|err| StackShieldError::LedError { source: err })
            }
        }
    }

    /// Switches the status LED off
    pub fn off(&mut self) -> Result<(), StackShieldError<ExpanderError<E>>> {
        self.blue
            .set_low()
            .map_err(|err| StackShieldError::LedError { source: err })?;
        self.green
            .set_low()
            .map_err(|err| StackShieldError::LedError { source: err })?;
        self.red
            .set_low()
            .map_err(|err| StackShieldError::LedError { source: err })
    }
}
