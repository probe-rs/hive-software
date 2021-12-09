extern crate embedded_hal as hal;
extern crate pca9535;
extern crate retry;

use expander_gpio::ExpanderGpio;
use pca9535::expander::SyncExpander;
use thiserror::Error;

mod expander_gpio;

#[derive(Debug, Clone, Copy)]
pub enum Status {
    Idle,
    Err,
    NoBoard,
    NotInitialized,
}

#[derive(Debug, Clone, Copy)]
pub enum Target {
    Target0 = 0,
    Target1 = 1,
    Target2 = 2,
    Target3 = 3,
}

#[derive(Debug, Clone, Copy)]
pub enum Probe {
    Probe0 = 0,
    Probe1 = 1,
    Probe2 = 2,
    Probe3 = 3,
}

#[derive(Error, Debug)]
pub enum StackShieldError<ERR>
where
    ERR: core::fmt::Debug,
{
    #[error("Failed to control target stack shield LED")]
    LedError(ERR),
    #[error("Failed to control target stack shield GPIOs")]
    GpioError(ERR),
    #[error("Failed to control target stack shield bus switches")]
    BusSwitchError(ERR),
    #[error("Failed to detect if daugherboard is present or not")]
    DaughterboardDetectError(ERR),
    #[error("Target stack shield not initialized")]
    NotInitialized,
}

/// Representation of a physical target stack shield of Hive
pub struct TargetStackShield<'a, T>
where
    T: SyncExpander,
{
    pub expander: &'a T,
    pub position: u8,
    pub status: Status,
    pins: Option<ExpanderGpio<'a, T>>,
}

impl<'a, T: SyncExpander> TargetStackShield<'a, T> {
    /// Creates a new instance of the struct.
    /// This function does not issue any i2c bus transaction to the respective IO expander! The state of the struct remains in [`Status::NotInitialized`] until the init_pins() function has been called successfully. Only then the shield is fully usable and functional.
    ///
    /// # Panics
    /// If provided position is > 8 (as the maximum of allowed target stack shields per assembly is 8 with numbering from 0-7)
    pub fn new(expander: &'a T, position: u8) -> Self {
        assert!(position < 8);

        Self {
            expander,
            position,
            status: Status::NotInitialized,
            pins: Option::None,
        }
    }

    /// Initializes all the pins of the IO Expander on the target stack shield and updates the status of the struct.
    pub fn init_pins(&mut self) -> Result<(), StackShieldError<<T as SyncExpander>::Error>> {
        let mut gpio = ExpanderGpio::new(self.expander, self.position)?;

        let daughterboard = gpio.daughterboard_detect.is_connected()?;
        if daughterboard {
            self.set_status(Status::Idle);
        } else {
            self.set_status(Status::NoBoard);
        }

        self.pins = Some(gpio);

        Ok(())
    }

    /// Sets the status of the target stack shield
    pub fn set_status(
        &mut self,
        status: Status,
    ) -> Result<(), StackShieldError<<T as SyncExpander>::Error>> {
        self.status = status;
        self.get_gpio_and_try(|gpio| gpio.status_led.set_status(status))
    }

    /// Tries to get the [`ExpanderGpio`] struct and execute the provided closure. It automatically unwraps the Option and provides the gpio for use in the closure.
    ///
    /// If the struct was not initialized this function returns a [StackShieldError::NotInitialized]
    ///
    /// # Example
    ///
    /// ```rust
    /// //inside some impl function of TargetStackShield struct
    /// self.get_gpio_and_try(|gpio| gpio.status_led.set_status(Status::Err))
    /// ```
    fn get_gpio_and_try<
        RetVal,
        F: FnOnce(
            &mut ExpanderGpio<'a, T>,
        ) -> Result<RetVal, StackShieldError<<T as SyncExpander>::Error>>,
    >(
        &mut self,
        op: F,
    ) -> Result<RetVal, StackShieldError<<T as SyncExpander>::Error>> {
        if let Some(ref mut gpio) = self.pins {
            op(gpio)
        } else {
            Err(StackShieldError::NotInitialized)
        }
    }
}
