use std::{
    error::Error,
    sync::Mutex, 
};

use comm_types::hardware::TargetState;
use ll_api::{RpiTestChannel, Target, TestChannel};
use probe_rs::Probe;
use retry::{delay::Fixed, retry};

use super::TargetStackShield;

const FIXED_RETRY_DELAY_MS: u64 = 10;
const CONNECT_RETRY_LIMIT: usize = 3;

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

    pub fn get_channel(&self) -> TestChannel {
        self.channel
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

    /// Loops through all available TSS and connects the testchannel to each available target, while executing the provided function on each connection.
    pub fn connect_all_available_and_execute<F>(&mut self, tss: &Vec<Mutex<TargetStackShield>>, mut function: F) where F: FnMut(&String, u8) {
        let mut unprocessed_tss_queue: Vec<&Mutex<TargetStackShield>> = tss.iter().collect();

        while unprocessed_tss_queue.len() != 0 {
            match unprocessed_tss_queue[0].try_lock() {
                Ok(tss) => {
                    // do magic
                    if let Some(targets) = tss.get_targets() {
                        for (pos, target) in targets.iter().enumerate() {
                            if let TargetState::Known(target_name) = target {
                                match retry(
                                    Fixed::from_millis(FIXED_RETRY_DELAY_MS)
                                        .take(CONNECT_RETRY_LIMIT),
                                    || {
                                        tss.inner.borrow_mut().connect_test_channel_to_target(
                                            self.channel,
                                            Target::try_from(pos as u8).unwrap(),
                                        )
                                    },
                                ){
                                    Ok(_) => function(target_name, tss.get_position()),
                                    Err(err) => match err {
                                        retry::Error::Operation { error, ..} => {
                                            log::error!(
                                                "Failed to connect testchannel {:?} to target {:?}: {}\nCaused by: {:?}",
                                                self.channel,
                                                Target::try_from(pos as u8).unwrap(),
                                                error,
                                                error.source()
    
                                            );
                                            // handle error, as it might influence other tests and testchannels
                                            todo!();
                                            tss.inner.borrow_mut().disconnect_all().expect("Failed to disconnect tss successfully, this error cannot be recovered, as further operation in such a state may influence other testchannels.");
                                        },
                                        retry::Error::Internal(string) => panic!("Internal library error in retry crate: {}", string),
                                    },
                                }
                            }
                        }
                    }

                    drop(tss);
                    unprocessed_tss_queue.remove(0);
                }
                Err(std::sync::TryLockError::WouldBlock) => {
                    // If the lock is currently held by another testchannel (eg. the tss is currently connected to another testchannel), it is skipped and reinserted at the end of the queue
                    let removed = unprocessed_tss_queue.remove(0);
                    unprocessed_tss_queue.push(removed);
                }
                Err(std::sync::TryLockError::Poisoned(err)) => {
                    panic!("Mutex is poisoned! \n{}", err)
                }
            }
        }
    }
}
