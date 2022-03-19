use self::{gpio::TestGpio, uart::TestUart};
use crate::{RpiTestChannelError, TestChannel};
use rppal::gpio::Gpio;

pub mod gpio;
pub mod uart;

/// This struct contains all the devices per [`TestChannel`]
pub(crate) struct TestChannelGpio {
    pub gpio: TestGpio,
    pub uart: TestUart,
}

impl TestChannelGpio {
    /// Creates a new instance of the struct. Configures all the required peripherals of this channel.
    pub fn new(channel: TestChannel, rpi_gpio: &mut Gpio) -> Result<Self, RpiTestChannelError> {
        let gpio = TestGpio::new(channel, rpi_gpio)?;

        let uart = TestUart::new(channel)?;

        Ok(Self { gpio, uart })
    }
}
