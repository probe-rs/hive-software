use std::{error::Error, sync::Mutex};

use ll_api::{RpiTestChannel, TestChannel};
use probe_rs::Probe;

/// A hive testchannel which combines the rpi testpins with the associated probe
#[derive(Debug)]
pub struct CombinedTestChannel {
    channel: TestChannel,
    rpi: RpiTestChannel,
    probe: Option<Probe>,
}

impl CombinedTestChannel {
    pub(crate) fn new() -> [Mutex<Self>; 4] {
        [
            Mutex::new(CombinedTestChannel {
                channel: TestChannel::Channel0,
                rpi: RpiTestChannel::new(TestChannel::Channel0),
                probe: None,
            }),
            Mutex::new(CombinedTestChannel {
                channel: TestChannel::Channel1,
                rpi: RpiTestChannel::new(TestChannel::Channel1),
                probe: None,
            }),
            Mutex::new(CombinedTestChannel {
                channel: TestChannel::Channel2,
                rpi: RpiTestChannel::new(TestChannel::Channel2),
                probe: None,
            }),
            Mutex::new(CombinedTestChannel {
                channel: TestChannel::Channel3,
                rpi: RpiTestChannel::new(TestChannel::Channel3),
                probe: None,
            }),
        ]
    }

    /// Check if testchannel has a probe attached and is ready to be used during testing
    pub fn is_ready(&self) -> bool {
        self.probe.is_some()
    }

    /// Reset the test channel to defaults for use in next test
    pub fn reset(&mut self) -> Result<(), Box<dyn Error>> {
        self.rpi.test_gpio_reset()?;

        if let Some(ref mut probe) = self.probe {
            probe.detach()?;
        }

        Ok(())
    }

    /// Binds the provided probe to the testchannel
    pub fn bind_probe(&mut self, probe: Probe) {
        self.probe = Some(probe);
    }
}
