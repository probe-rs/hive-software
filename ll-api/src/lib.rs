extern crate embedded_hal as hal;
extern crate pca9535;
extern crate rppal;
extern crate shared_bus;

use hal::digital::blocking::{InputPin, IoPin, OutputPin};
use pca9535::{
    expander::SyncExpander,
    ExpanderInputPin, ExpanderOutputPin,
    GPIOBank::{Bank0, Bank1},
    PinState::{High, Low},
};

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

#[derive(Debug)]
pub enum StackShieldError<GPIO> {
    LedError(GPIO),
    GpioError(GPIO),
}

impl<GPIO> StackShieldError<GPIO> {
    fn from_led(err: GPIO) -> Self {
        Self::LedError(err)
    }

    fn from_gpio(err: GPIO) -> Self {
        Self::GpioError(err)
    }
}

struct StackShield<'a, T>
where
    T: SyncExpander,
{
    pub position: u8,
    pub status: Status,
    pins: ExpanderPins<'a, T>,
}

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
        let red = ExpanderOutputPin::new(expander, Bank0, 0, Low);
        let green = ExpanderOutputPin::new(expander, Bank0, 1, Low);
        let blue = ExpanderOutputPin::new(expander, Bank0, 2, Low);

        let led = LED { red, green, blue };

        let sw_target_0 = ExpanderOutputPin::new(expander, Bank1, 3, Low);
        let sw_target_1 = ExpanderOutputPin::new(expander, Bank1, 2, Low);
        let sw_target_2 = ExpanderOutputPin::new(expander, Bank1, 1, Low);
        let sw_target_3 = ExpanderOutputPin::new(expander, Bank1, 0, Low);

        let sw_probe_0 = ExpanderOutputPin::new(expander, Bank1, 7, Low);
        let sw_probe_1 = ExpanderOutputPin::new(expander, Bank1, 6, Low);
        let sw_probe_2 = ExpanderOutputPin::new(expander, Bank1, 5, Low);
        let sw_probe_3 = ExpanderOutputPin::new(expander, Bank1, 4, Low);

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

impl<'a, T: SyncExpander, GPIO> LED<'a, T> {
    fn setStatus(&mut self, status: Status) -> Result<(), StackShieldError<GPIO>> {
        match status {
            Status::Err => {
                self.blue
                    .set_low()
                    .map_err(|err| StackShieldError::from_led(err))?;
                self.green
                    .set_low()
                    .map_err(|err| StackShieldError::from_led(err))?;
                self.red
                    .set_high()
                    .map_err(|err| StackShieldError::from_led(err))?;
            }
            Status::Idle => {
                self.blue
                    .set_low()
                    .map_err(|err| StackShieldError::from_led(err))?;
                self.green
                    .set_high()
                    .map_err(|err| StackShieldError::from_led(err))?;
                self.red
                    .set_low()
                    .map_err(|err| StackShieldError::from_led(err))?;
            }
            Status::NoBoard => {
                self.blue
                    .set_high()
                    .map_err(|err| StackShieldError::from_led(err))?;
                self.green
                    .set_low()
                    .map_err(|err| StackShieldError::from_led(err))?;
                self.red
                    .set_low()
                    .map_err(|err| StackShieldError::from_led(err))?;
            }
        }
    }

    fn off(&mut self) -> Result<(), StackShieldError<GPIO>> {
        self.blue
            .set_low()
            .map_err(|err| StackShieldError::from_led(err))?;
        self.green
            .set_low()
            .map_err(|err| StackShieldError::from_led(err))?;
        self.red
            .set_low()
            .map_err(|err| StackShieldError::from_led(err))?;
    }
}
