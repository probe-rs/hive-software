use embedded_hal::digital::blocking::OutputPin;
use pca9535::expander::SyncExpander;
use pca9535::ExpanderOutputPin;

use crate::StackShieldError;
use crate::StackShieldStatus;

/// Abstraction struct for the status LED function
pub(crate) struct Led<'a, T>
where
    T: SyncExpander,
{
    red: ExpanderOutputPin<'a, T>,
    green: ExpanderOutputPin<'a, T>,
    blue: ExpanderOutputPin<'a, T>,
}

impl<'a, T: SyncExpander> Led<'a, T> {
    /// Creates a new instance of the struct
    pub fn new(
        red: ExpanderOutputPin<'a, T>,
        green: ExpanderOutputPin<'a, T>,
        blue: ExpanderOutputPin<'a, T>,
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
    ) -> Result<(), StackShieldError<<T as SyncExpander>::Error>> {
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
    pub fn off(&mut self) -> Result<(), StackShieldError<<T as SyncExpander>::Error>> {
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
