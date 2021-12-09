use hal::digital::blocking::OutputPin;
use pca9535::expander::SyncExpander;
use pca9535::ExpanderOutputPin;

use crate::StackShieldError;
use crate::Status;

/// Abstraction struct for the status LED function
pub(crate) struct LED<'a, T>
where
    T: SyncExpander,
{
    red: ExpanderOutputPin<'a, T>,
    green: ExpanderOutputPin<'a, T>,
    blue: ExpanderOutputPin<'a, T>,
}

impl<'a, T: SyncExpander> LED<'a, T> {
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
        status: Status,
    ) -> Result<(), StackShieldError<<T as SyncExpander>::Error>> {
        match status {
            Status::Err => {
                self.blue
                    .set_high()
                    .map_err(|err| StackShieldError::LedError(err))?;
                self.green
                    .set_high()
                    .map_err(|err| StackShieldError::LedError(err))?;
                self.red
                    .set_low()
                    .map_err(|err| StackShieldError::LedError(err))
            }
            Status::Idle => {
                self.blue
                    .set_high()
                    .map_err(|err| StackShieldError::LedError(err))?;
                self.green
                    .set_low()
                    .map_err(|err| StackShieldError::LedError(err))?;
                self.red
                    .set_high()
                    .map_err(|err| StackShieldError::LedError(err))
            }
            Status::NoBoard => {
                self.blue
                    .set_low()
                    .map_err(|err| StackShieldError::LedError(err))?;
                self.green
                    .set_high()
                    .map_err(|err| StackShieldError::LedError(err))?;
                self.red
                    .set_high()
                    .map_err(|err| StackShieldError::LedError(err))
            }
            _ => {
                self.blue
                    .set_high()
                    .map_err(|err| StackShieldError::LedError(err))?;
                self.green
                    .set_high()
                    .map_err(|err| StackShieldError::LedError(err))?;
                self.red
                    .set_high()
                    .map_err(|err| StackShieldError::LedError(err))
            }
        }
    }

    /// Switches the status LED off
    pub fn off(&mut self) -> Result<(), StackShieldError<<T as SyncExpander>::Error>> {
        self.blue
            .set_low()
            .map_err(|err| StackShieldError::LedError(err))?;
        self.green
            .set_low()
            .map_err(|err| StackShieldError::LedError(err))?;
        self.red
            .set_low()
            .map_err(|err| StackShieldError::LedError(err))
    }
}
