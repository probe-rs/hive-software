//! This crate provides reexports which are used for writing Hive tests
//!
//! It also contains functionality that has to be isolated from other crates which depend on probe-rs as those would otherwise cause circular dependencies
use ll_api::{RpiTestChannel, RpiTestChannelError, TestInputPin, UART_BYTES_READ};

pub use comm_types::defines;
pub use comm_types::hardware::{Architecture, HiveTargetInfo, Memory};
pub use hive_macro::hive;
pub use hive_macro::hive_test;
pub use inventory;

/// Provides all user facing TestChannel functions inside the testfunctions
pub trait TestChannelHandle {
    /// Checks if provided [`TestInputPin`] is high.
    fn input_is_high(&mut self, pin: TestInputPin) -> Result<bool, RpiTestChannelError>;

    /// Sets the test output pin high
    fn output_set_high(&mut self) -> Result<(), RpiTestChannelError>;

    /// Sets the test output pin low.
    fn output_set_low(&mut self) -> Result<(), RpiTestChannelError>;

    /// Reads [`UART_BYTES_READ`] Bytes from test bus. This function blocks until it can read the specified amount of data or until a preset timeout runs out.
    fn bus_read(&mut self) -> Result<[u8; UART_BYTES_READ as usize], RpiTestChannelError>;

    /// Writes the bytes in the provided data slice. Blocks until all Bytes have been sent to the output queue.
    fn bus_write(&mut self, data: &[u8]) -> Result<(), RpiTestChannelError>;
}

impl TestChannelHandle for RpiTestChannel {
    fn input_is_high(&mut self, pin: TestInputPin) -> Result<bool, RpiTestChannelError> {
        self.test_input_is_high(pin)
    }

    fn output_set_high(&mut self) -> Result<(), RpiTestChannelError> {
        self.test_output_set_high()
    }

    fn output_set_low(&mut self) -> Result<(), RpiTestChannelError> {
        self.test_output_set_low()
    }

    fn bus_read(&mut self) -> Result<[u8; UART_BYTES_READ as usize], RpiTestChannelError> {
        self.test_bus_read()
    }

    fn bus_write(&mut self, data: &[u8]) -> Result<(), RpiTestChannelError> {
        self.test_bus_write(data)
    }
}
