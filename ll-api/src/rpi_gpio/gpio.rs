use crate::{RpiTestChannelError, TestChannel};
use rppal::gpio::Gpio;
use rppal::gpio::{InputPin, OutputPin};

const BCM_TO_HIVE_PINS: [[u8; 4]; 4] = [
    [17, 27, 22, 10], //Channel 0, GPIO 0-3
    [18, 23, 24, 25], //Channel 1, GPIO 0-3
    [11, 7, 6, 19],   //Channel 2, GPIO 0-3
    [16, 26, 20, 21], //Channel 3, GPIO 0-3
];

/// All available input pins per [`TestChannel`]
#[derive(Debug, Clone, Copy)]
pub enum TestInputPin {
    Pin0,
    Pin1,
    Pin2,
}

/// Contains all the gpio Inputs and Outputs per [`TestChannel`]
pub(crate) struct TestGpio {
    pin_0: InputPin,
    pin_1: InputPin,
    pin_2: InputPin,
    pin_3: OutputPin,
}

impl TestGpio {
    /// Creates a new instance of the struct
    pub fn new(channel: TestChannel, rpi_gpio: &mut Gpio) -> Result<Self, RpiTestChannelError> {
        let mut pin_0 = rpi_gpio
            .get(BCM_TO_HIVE_PINS[channel as usize][0])
            .map_err(|err| RpiTestChannelError::GpioInitError { source: err })?
            .into_input_pullup();
        let mut pin_1 = rpi_gpio
            .get(BCM_TO_HIVE_PINS[channel as usize][1])
            .map_err(|err| RpiTestChannelError::GpioInitError { source: err })?
            .into_input_pullup();
        let mut pin_2 = rpi_gpio
            .get(BCM_TO_HIVE_PINS[channel as usize][2])
            .map_err(|err| RpiTestChannelError::GpioInitError { source: err })?
            .into_input_pullup();
        let mut pin_3 = rpi_gpio
            .get(BCM_TO_HIVE_PINS[channel as usize][3])
            .map_err(|err| RpiTestChannelError::GpioInitError { source: err })?
            .into_output_low();

        pin_0.set_reset_on_drop(false);
        pin_1.set_reset_on_drop(false);
        pin_2.set_reset_on_drop(false);
        pin_3.set_reset_on_drop(false);

        Ok(Self {
            pin_0,
            pin_1,
            pin_2,
            pin_3,
        })
    }

    /// Checks if provided input pin is high
    pub fn input_is_high(&self, pin: TestInputPin) -> bool {
        match pin {
            TestInputPin::Pin0 => self.pin_0.is_high(),
            TestInputPin::Pin1 => self.pin_1.is_high(),
            TestInputPin::Pin2 => self.pin_2.is_high(),
        }
    }

    /// Sets output high
    pub fn output_set_high(&mut self) {
        self.pin_3.set_high();
    }

    /// Sets output low
    pub fn output_set_low(&mut self) {
        self.pin_3.set_low();
    }

    /// Resets gpio struct to its default state by setting output low
    pub fn reset(&mut self) {
        self.pin_3.set_low();
    }
}
