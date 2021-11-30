use hal::digital::blocking::{InputPin, IoPin, OutputPin};
use pca9535::{
    expander::{ExpanderError, SyncExpander},
    ExpanderInputPin, ExpanderOutputPin,
    GPIOBank::{Bank0, Bank1},
    IoExpander,
    PinState::{High, Low},
};

use super::StackShieldError;
use super::Status;

struct ExpanderPins<'a, T>
where
    T: SyncExpander,
{
    led: LED<'a, T>,
}

impl<'a, T: SyncExpander> ExpanderPins<'a, T> {
    pub fn new<Ex: SyncExpander, GPIO>(
        expander: &'a Ex,
        position: u8,
    ) -> Result<Self, StackShieldError<GPIO>> {
        assert!(position > 8);

        let board_detect = ExpanderInputPin::new(expander, Bank0, 3);
        let red = ExpanderOutputPin::new(expander, Bank0, 0, Low)?;
        let green = ExpanderOutputPin::new(expander, Bank0, 1, Low)?;
        let blue = ExpanderOutputPin::new(expander, Bank0, 2, Low)?;

        let led = LED { red, green, blue };

        let sw_target_0 = ExpanderOutputPin::new(expander, Bank1, 3, Low)?;
        let sw_target_1 = ExpanderOutputPin::new(expander, Bank1, 2, Low)?;
        let sw_target_2 = ExpanderOutputPin::new(expander, Bank1, 1, Low)?;
        let sw_target_3 = ExpanderOutputPin::new(expander, Bank1, 0, Low)?;

        let sw_probe_0 = ExpanderOutputPin::new(expander, Bank1, 7, Low)?;
        let sw_probe_1 = ExpanderOutputPin::new(expander, Bank1, 6, Low)?;
        let sw_probe_2 = ExpanderOutputPin::new(expander, Bank1, 5, Low)?;
        let sw_probe_3 = ExpanderOutputPin::new(expander, Bank1, 4, Low)?;

        Ok(ExpanderPins { led })
    }
}

struct LED<'a, T>
where
    T: SyncExpander,
{
    red: ExpanderOutputPin<'a, T>,
    green: ExpanderOutputPin<'a, T>,
    blue: ExpanderOutputPin<'a, T>,
}

impl<'a, T: SyncExpander> LED<'a, T> {
    fn setStatus<GPIO>(&mut self, status: Status) -> Result<(), StackShieldError<GPIO>> {
        match status {
            Status::Err => {
                self.blue
                    .set_low()
                    .map_err(|err| StackShieldError::LedError(err))?;
                self.green
                    .set_low()
                    .map_err(|err| StackShieldError::LedError(err))?;
                self.red
                    .set_high()
                    .map_err(|err| StackShieldError::LedError(err))?;
            }
            Status::Idle => {
                self.blue
                    .set_low()
                    .map_err(|err| StackShieldError::LedError(err))?;
                self.green
                    .set_high()
                    .map_err(|err| StackShieldError::LedError(err))?;
                self.red
                    .set_low()
                    .map_err(|err| StackShieldError::LedError(err))?;
            }
            Status::NoBoard => {
                self.blue
                    .set_high()
                    .map_err(|err| StackShieldError::LedError(err))?;
                self.green
                    .set_low()
                    .map_err(|err| StackShieldError::LedError(err))?;
                self.red
                    .set_low()
                    .map_err(|err| StackShieldError::LedError(err))?;
            }
        }
    }

    fn off(&mut self) -> Result<(), StackShieldError<GPIO>> {
        self.blue
            .set_low()
            .map_err(|err| StackShieldError::LedError(err))?;
        self.green
            .set_low()
            .map_err(|err| StackShieldError::LedError(err))?;
        self.red
            .set_low()
            .map_err(|err| StackShieldError::LedError(err))?;
    }
}
