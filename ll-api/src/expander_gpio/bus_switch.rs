use embedded_hal::digital::OutputPin;
use embedded_hal::i2c::I2c;
use pca9535::expander::SyncExpander;
use pca9535::ExpanderError;
use pca9535::ExpanderOutputPin;
use retry::{delay::Fixed, retry};

use crate::StackShieldError;
use crate::Target;
use crate::TestChannel;

const FIXED_RETRY_DELAY_MS: u64 = 10;
const RETRY_LIMIT: usize = 3;

/// Abstraction struct for the bus switch function
pub(crate) struct BusSwitch<'a, I2C, T>
where
    I2C: I2c,
    T: SyncExpander<I2C>,
{
    sw_target: [ExpanderOutputPin<'a, I2C, T>; 4],
    sw_test_channel: [ExpanderOutputPin<'a, I2C, T>; 4],
}

impl<'a, I2C, T, E> BusSwitch<'a, I2C, T>
where
    E: std::fmt::Debug,
    I2C: I2c<Error = E>,
    T: SyncExpander<I2C>,
{
    /// Creates a new instance of the struct
    pub fn new(
        sw_target: [ExpanderOutputPin<'a, I2C, T>; 4],
        sw_test_channel: [ExpanderOutputPin<'a, I2C, T>; 4],
    ) -> Self {
        Self {
            sw_target,
            sw_test_channel,
        }
    }

    /// Connects the provided [`TestChannel`] with the provided [`Target`].
    ///
    /// # Disconnect
    /// Before the connection is made all switches are disconnected to prevent any short circuits. In case the disconnect fails the function automatically retries to disconnect. If the amount of allowed retries are exhausted and the disconnect still fails the function returns an error.
    /// To prevent any short circuits the function returns an Error on each call before connecting any other bus switches to prevent short circuits and other undefined behavior.
    ///
    /// # Panics
    /// The function panics in case the [`retry`] crate encounters an internal error [`retry::Error::Internal`].
    pub fn connect(
        &mut self,
        channel: TestChannel,
        target: Target,
    ) -> Result<(), StackShieldError<ExpanderError<E>>> {
        retry(
            Fixed::from_millis(FIXED_RETRY_DELAY_MS).take(RETRY_LIMIT),
            || self.disconnect_all(),
        )
        .map_err(|err| err.error)?;

        self.sw_test_channel[channel as usize]
            .set_low()
            .map_err(|err| StackShieldError::BusSwitchError { source: err })?;
        self.sw_target[target as usize]
            .set_low()
            .map_err(|err| StackShieldError::BusSwitchError { source: err })
    }

    /// Disconnects all the bus switches on the target stack shield.
    ///
    /// # Error
    /// In case this function returns an error, disconnecting all bus switches has failed. In that case the operation should be retried until it is successful before any other bus switches on other target stack shields are connected in order to avoid short circuits and undefined behavior.
    pub fn disconnect_all(&mut self) -> Result<(), StackShieldError<ExpanderError<E>>> {
        for sw in &mut self.sw_target {
            sw.set_high()
                .map_err(|err| StackShieldError::BusSwitchError { source: err })?;
        }

        for sw in &mut self.sw_test_channel {
            sw.set_high()
                .map_err(|err| StackShieldError::BusSwitchError { source: err })?;
        }

        Ok(())
    }
}
