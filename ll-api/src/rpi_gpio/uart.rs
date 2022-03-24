use std::path::Path;
use std::time::Duration;

use rppal::uart::Parity;
use rppal::uart::Uart;

use crate::RpiTestChannelError;
use crate::TestChannel;

const UART_PATHS: [u8; 4] = [1, 0, 2, 3]; // Corresponding UART Linux serial character device number to Channel number
pub const UART_BYTES_READ: u8 = 1; // How many Bytes are read when calling uart read function
const UART_READ_TIMEOUT: u8 = 200; // How many ms read function blocks at max

/// Contains the UART bus per [`TestChannel`]
#[derive(Debug)]
pub(crate) struct TestUart {
    uart: Uart,
}

impl TestUart {
    /// Creates a new instance of the struct and configures the UART peripheral. Both write and read are blocking, as the tests on a single Testchannel are never parallel.
    pub fn new(channel: TestChannel) -> Result<Self, RpiTestChannelError> {
        let mut uart = Uart::with_path(
            Path::new(&format!("/dev/ttyAMA{}", UART_PATHS[channel as usize])),
            115_200,
            Parity::None,
            8,
            1,
        )
        .map_err(|err| RpiTestChannelError::UartInitError { source: err })?;

        uart.set_read_mode(
            UART_BYTES_READ,
            Duration::from_millis(UART_READ_TIMEOUT as u64),
        )
        .map_err(|err| RpiTestChannelError::UartInitError { source: err })?;

        uart.set_write_mode(true)
            .map_err(|err| RpiTestChannelError::UartInitError { source: err })?;

        uart.set_dtr(false)
            .map_err(|err| RpiTestChannelError::UartInitError { source: err })?;

        Ok(Self { uart })
    }

    /// Reads the amount of Bytes specified by [`UART_BYTES_READ`] configuration constant. The function blocks until it can read the specified amount of Bytes or until the timeout [`UART_READ_TIMEOUT`] (ms) runs out.
    pub fn read(&mut self) -> Result<[u8; UART_BYTES_READ as usize], RpiTestChannelError> {
        let mut buffer = [0; UART_BYTES_READ as usize];
        self.uart
            .read(&mut buffer)
            .map_err(|err| RpiTestChannelError::UartError { source: err })?;

        Ok(buffer)
    }

    /// Function writes the content of the provided slice and blocks until the entire data has been copied to the peripherals output queue.
    pub fn write(&mut self, data: &[u8]) -> Result<(), RpiTestChannelError> {
        self.uart
            .write(data)
            .map_err(|err| RpiTestChannelError::UartError { source: err })?;

        Ok(())
    }
}
