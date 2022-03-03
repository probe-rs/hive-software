extern crate embedded_hal as hal;
extern crate pca9535;
extern crate retry;

use std::convert::TryFrom;

use expander_gpio::ExpanderGpio;
use pca9535::expander::SyncExpander;
use rpi_gpio::TestChannelGpio;
use rppal::gpio::Gpio;
use thiserror::Error;

mod expander_gpio;
mod rpi_gpio;

pub use crate::rpi_gpio::uart::UART_BYTES_READ;
pub use rpi_gpio::gpio::TestInputPin;

#[derive(Debug, Clone, Copy)]
pub enum StackShieldStatus {
    Idle,
    Err,
    NoBoard,
    NotInitialized,
}

#[derive(Debug, Clone, Copy)]
pub enum TestChannelStatus {
    Idle,
    Connected,
    Err,
    NotInitialized,
}

#[derive(Debug, Clone, Copy)]
pub enum Target {
    Target0 = 0,
    Target1 = 1,
    Target2 = 2,
    Target3 = 3,
}

impl TryFrom<u8> for Target {
    type Error = ApiError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Target::Target0),
            1 => Ok(Target::Target1),
            2 => Ok(Target::Target2),
            3 => Ok(Target::Target3),
            _ => Err(ApiError::ConversionError),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TestChannel {
    Channel0 = 0,
    Channel1 = 1,
    Channel2 = 2,
    Channel3 = 3,
}

impl TryFrom<u8> for TestChannel {
    type Error = ApiError;

    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(TestChannel::Channel0),
            1 => Ok(TestChannel::Channel1),
            2 => Ok(TestChannel::Channel2),
            3 => Ok(TestChannel::Channel3),
            _ => Err(ApiError::ConversionError),
        }
    }
}

#[derive(Error, Debug)]
pub enum StackShieldError<ERR>
where
    ERR: core::fmt::Debug,
{
    #[error("Failed to control target stack shield LED: {:?}", .0)]
    LedError(ERR),
    #[error("Failed to control target stack shield GPIO: {:?}", .0)]
    GpioError(ERR),
    #[error("Failed to control target stack shield bus switches: {:?}", .0)]
    BusSwitchError(ERR),
    #[error("Failed to detect if daugherboard is present or not: {:?}", .0)]
    DaughterboardDetectError(ERR),
    #[error("Target stack shield not initialized")]
    NotInitialized,
}

#[derive(Error, Debug)]
pub enum RpiTestChannelError {
    #[error("Failed to initialize the Raspberry Pi UART: {:?}", .0)]
    UartInitError(rppal::uart::Error),
    #[error("Failed to initialize the Raspberry Pi GPIO: {:?}", .0)]
    GpioInitError(rppal::gpio::Error),
    #[error("Failed to control Raspberry Pi UART: {:?}", .0)]
    UartError(rppal::uart::Error),
    #[error("Failed to control Raspberry Pi GPIO: {:?}", .0)]
    GpioError(rppal::gpio::Error),
    #[error("Test channel not initialized")]
    NotInitialized,
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Failed to convert provided value to enum")]
    ConversionError,
}

/// Representation of a physical target stack shield of Hive
pub struct TargetStackShield<'a, T>
where
    T: SyncExpander,
{
    pub expander: &'a T,
    status: StackShieldStatus,
    pins: Option<ExpanderGpio<'a, T>>,
}

impl<'a, T: SyncExpander> TargetStackShield<'a, T> {
    /// Creates a new instance of the struct.
    /// This function does not issue any i2c bus transaction to the respective IO expander! The state of the struct remains in [`StackShieldStatus::NotInitialized`] until the init_pins() function has been called successfully. Only then the shield is fully usable and functional.
    pub fn new(expander: &'a T) -> Self {
        Self {
            expander,
            status: StackShieldStatus::NotInitialized,
            pins: Option::None,
        }
    }

    /// Initializes all the pins of the IO Expander on the target stack shield and updates the status of the struct.
    pub fn init_pins(&mut self) -> Result<(), StackShieldError<<T as SyncExpander>::Error>> {
        let gpio = ExpanderGpio::new(self.expander)?;

        self.pins = Some(gpio);

        let daughterboard = self.daughterboard_is_connected()?;
        if daughterboard {
            self.set_status(StackShieldStatus::Idle)?;
        } else {
            self.set_status(StackShieldStatus::NoBoard)?;
        }

        Ok(())
    }

    /// Gets the status of the target stack shield
    pub fn get_status(&self) -> StackShieldStatus {
        self.status
    }

    /// Sets the status of the target stack shield
    pub fn set_status(
        &mut self,
        status: StackShieldStatus,
    ) -> Result<(), StackShieldError<<T as SyncExpander>::Error>> {
        self.status = status;
        self.get_gpio_and_try(|gpio| gpio.status_led.set_status(status))
    }

    /// Switches the status LED off
    pub fn status_led_off(&mut self) -> Result<(), StackShieldError<<T as SyncExpander>::Error>> {
        self.get_gpio_and_try(|gpio| gpio.status_led.off())
    }

    /// Checks if a daughterboard is connected
    pub fn daughterboard_is_connected(
        &mut self,
    ) -> Result<bool, StackShieldError<<T as SyncExpander>::Error>> {
        self.get_gpio_and_try(|gpio| gpio.daughterboard_detect.is_connected())
    }

    /// Connects provided [`TestChannel`] to [`Target`]
    pub fn connect_test_channel_to_target(
        &mut self,
        channel: TestChannel,
        target: Target,
    ) -> Result<(), StackShieldError<<T as SyncExpander>::Error>> {
        self.get_gpio_and_try(|gpio| gpio.bus_switch.connect(channel, target))
    }

    /// Disconnects all targets and test channels from target stack shield
    pub fn disconnect_all(&mut self) -> Result<(), StackShieldError<<T as SyncExpander>::Error>> {
        self.get_gpio_and_try(|gpio| gpio.bus_switch.disconnect_all())
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

/// Representation of a physical Raspberry Pi Testchannel
pub struct RpiTestChannel {
    channel: TestChannel,
    status: TestChannelStatus,
    pins: Option<TestChannelGpio>,
}

impl RpiTestChannel {
    /// Creates a new instance of the struct.
    /// This function does not initialize the hardware. The struct needs to be initialized by calling init_pins function before it is fully usable. If any functions are called which require initialization prior to initialization they will return a [`RpiTestChannelError::NotInitialized`]
    pub fn new(channel: TestChannel) -> Self {
        Self {
            channel,
            status: TestChannelStatus::NotInitialized,
            pins: None,
        }
    }

    /// Initializes all the required pins of the Raspberry Pi and updates the status of the struct.
    pub fn init_pins(&mut self, rpi_gpio: &mut Gpio) -> Result<(), RpiTestChannelError> {
        let gpio = TestChannelGpio::new(self.channel, rpi_gpio)?;

        self.pins = Some(gpio);

        self.status = TestChannelStatus::Idle;

        Ok(())
    }

    /// Checks if provided [`TestInputPin`] is high.
    pub fn test_input_is_high(&mut self, pin: TestInputPin) -> Result<bool, RpiTestChannelError> {
        self.get_gpio_and_try(|gpio| Ok(gpio.gpio.input_is_high(pin)))
    }

    /// Sets the test output pin high
    pub fn test_output_set_high(&mut self) -> Result<(), RpiTestChannelError> {
        self.get_gpio_and_try(|gpio| {
            gpio.gpio.output_set_high();
            Ok(())
        })
    }

    /// Sets the test output pin low.
    pub fn test_output_set_low(&mut self) -> Result<(), RpiTestChannelError> {
        self.get_gpio_and_try(|gpio| {
            gpio.gpio.output_set_low();
            Ok(())
        })
    }

    /// Resets the test gpio to its default state.
    pub fn test_gpio_reset(&mut self) -> Result<(), RpiTestChannelError> {
        self.get_gpio_and_try(|gpio| {
            gpio.gpio.reset();
            Ok(())
        })
    }

    /// Reads [`UART_BYTES_READ`] Bytes from test bus. This function blocks until it can read the specified amount of data or until a preset timeout runs out.
    pub fn test_bus_read(&mut self) -> Result<[u8; UART_BYTES_READ as usize], RpiTestChannelError> {
        self.get_gpio_and_try(|gpio| gpio.uart.read())
    }

    /// Writes the bytes in the provided data slice. Blocks until all Bytes have been sent to the output queue.
    pub fn test_bus_write(&mut self, data: &[u8]) -> Result<(), RpiTestChannelError> {
        self.get_gpio_and_try(|gpio| gpio.uart.write(data))
    }

    /// Tries to get the [`TestChannelGpio`] struct and execute the provided closure. It automatically unwraps the Option and provides the gpio for use in the closure.
    ///
    /// If the struct was not initialized this function returns a [RpiTestChannelError::NotInitialized]
    ///
    /// # Example
    ///
    /// ```rust
    /// //inside some impl function of RpiTestChannel struct
    /// self.get_gpio_and_try(|gpio| gpio.gpio.input_is_high(TestInputPin::Pin0))
    /// ```
    fn get_gpio_and_try<
        RetVal,
        F: FnOnce(&mut TestChannelGpio) -> Result<RetVal, RpiTestChannelError>,
    >(
        &mut self,
        op: F,
    ) -> Result<RetVal, RpiTestChannelError> {
        if let Some(ref mut gpio) = self.pins {
            op(gpio)
        } else {
            Err(RpiTestChannelError::NotInitialized)
        }
    }
}
