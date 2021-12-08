use hal::digital::blocking::InputPin;
use pca9535::expander::SyncExpander;
use pca9535::ExpanderInputPin;

use crate::StackShieldError;

/// Abstraction struct for the daughterboard detect function
pub(crate) struct DaughterboardDetect<'a, T>
where
    T: SyncExpander,
{
    input: ExpanderInputPin<'a, T>,
}

impl<'a, T: SyncExpander> DaughterboardDetect<'a, T> {
    /// Creates a new instance of the struct
    pub fn new(input: ExpanderInputPin<'a, T>) -> Self {
        Self { input }
    }

    /// Checks if the daughterboard is connected or not.
    pub fn is_connected(&mut self) -> Result<bool, StackShieldError<<T as SyncExpander>::Error>> {
        self.input
            .is_high()
            .map_err(|err| StackShieldError::DaughterboardDetectError(err))
    }
}
