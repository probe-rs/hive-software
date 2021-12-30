use hal::digital::blocking::OutputPin;
use pca9535::expander::SyncExpander;
use pca9535::ExpanderOutputPin;
use retry::{delay::Fixed, retry, Error};

use crate::StackShieldError;
use crate::Target;
use crate::TestChannel;

const FIXED_RETRY_DELAY_MS: u64 = 10;
const RETRY_LIMIT: usize = 3;

/// Abstraction struct for the bus switch function
pub(crate) struct BusSwitch<'a, T>
where
    T: SyncExpander,
{
    sw_target: [ExpanderOutputPin<'a, T>; 4],
    sw_test_channel: [ExpanderOutputPin<'a, T>; 4],
}

impl<'a, T: SyncExpander> BusSwitch<'a, T> {
    /// Creates a new instance of the struct
    pub fn new(
        sw_target: [ExpanderOutputPin<'a, T>; 4],
        sw_test_channel: [ExpanderOutputPin<'a, T>; 4],
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
    ) -> Result<(), StackShieldError<<T as SyncExpander>::Error>> {
        retry(
            Fixed::from_millis(FIXED_RETRY_DELAY_MS).take(RETRY_LIMIT),
            || self.disconnect_all(),
        )
        .map_err(|err| match err {
            Error::Internal(string) => {
                panic!("Internal library error in retry crate: {}", string)
            }
            Error::Operation { error, .. } => error,
        })?;

        self.sw_test_channel[channel as usize]
            .set_low()
            .map_err(StackShieldError::BusSwitchError)?;
        self.sw_target[target as usize]
            .set_low()
            .map_err(StackShieldError::BusSwitchError)
    }

    /// Disconnects all the bus switches on the target stack shield.
    ///
    /// # Error
    /// In case this function returns an error, disconnecting all bus switches has failed. In that case the operation should be retried until it is successful before any other bus switches on other target stack shields are connected in order to avoid short circuits and undefined behavior.
    pub fn disconnect_all(&mut self) -> Result<(), StackShieldError<<T as SyncExpander>::Error>> {
        for sw in &mut self.sw_target {
            sw.set_high().map_err(StackShieldError::BusSwitchError)?;
        }

        for sw in &mut self.sw_test_channel {
            sw.set_high().map_err(StackShieldError::BusSwitchError)?;
        }

        Ok(())
    }
}
