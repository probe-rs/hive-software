use std::{
    error::Error,
    sync::Mutex,  
};
use std::mem;

use comm_types::hardware::{TargetState, TargetInfo};
use ll_api::{RpiTestChannel, Target, TestChannel};
use retry::{delay::Fixed, retry};
use antidote::Mutex as PoisonFreeMutex;

// Depending on the usecase, the probe-rs dependency is either stable, or the one being tested by Hive
#[cfg(not(feature = "runner"))]
use probe_rs::{Probe, DebugProbeInfo, DebugProbeError};
#[cfg(feature = "runner")]
use probe_rs_test::{Probe, DebugProbeInfo, DebugProbeError};

use super::{TargetStackShield, MAX_DAUGHTERBOARD_TARGETS};

const FIXED_RETRY_DELAY_MS: u64 = 10;
const CONNECT_RETRY_LIMIT: usize = 3;

/// A hive testchannel which combines the rpi testpins with the associated probe
#[derive(Debug)]
pub struct CombinedTestChannel {
    channel: TestChannel,
    rpi: PoisonFreeMutex<RpiTestChannel>,
    probe: PoisonFreeMutex<Option<Probe>>,
    probe_info: PoisonFreeMutex<Option<DebugProbeInfo>>
}

impl CombinedTestChannel {
    /// Creates and returns all testchannels which are able to be shared across threads
    pub(crate) fn new() -> [Mutex<Self>; MAX_DAUGHTERBOARD_TARGETS] {
        [
            Mutex::new(CombinedTestChannel {
                channel: TestChannel::Channel0,
                rpi: PoisonFreeMutex::new(RpiTestChannel::new(TestChannel::Channel0)),
                probe: PoisonFreeMutex::new(None),
                probe_info: PoisonFreeMutex::new(None),
            }),
            Mutex::new(CombinedTestChannel {
                channel: TestChannel::Channel1,
                rpi: PoisonFreeMutex::new(RpiTestChannel::new(TestChannel::Channel1)),
                probe: PoisonFreeMutex::new(None),
                probe_info: PoisonFreeMutex::new(None),
            }),
            Mutex::new(CombinedTestChannel {
                channel: TestChannel::Channel2,
                rpi: PoisonFreeMutex::new(RpiTestChannel::new(TestChannel::Channel2)),
                probe: PoisonFreeMutex::new(None),
                probe_info: PoisonFreeMutex::new(None),
            }),
            Mutex::new(CombinedTestChannel {
                channel: TestChannel::Channel3,
                rpi: PoisonFreeMutex::new(RpiTestChannel::new(TestChannel::Channel3)),
                probe: PoisonFreeMutex::new(None),
                probe_info: PoisonFreeMutex::new(None),
            }),
        ]
    }

    /// Binds the provided probe to the testchannel
    pub fn bind_probe(&self, probe: Probe, probe_info: DebugProbeInfo) {
        *self.probe.lock() = Some(probe);
        *self.probe_info.lock() = Some(probe_info);
    }

    /// Removes a probe and the associated probe_info from this testchannel, if existing
    pub fn remove_probe(&self) {
        *self.probe.lock() = None;
        *self.probe_info.lock() = None;
    }

    /// Drops the probe instance to unlock it for other programs. This leaves the probe_info in place which can later be used to reinstantiate the probe.
    pub fn unlock_probe(&self) {
        *self.probe.lock() = None;
    }

    /// Reinitializes the probe based on the stored probe_info, and adds the newly initialized probe to the struct.
    /// 
    /// If probe_info is none, the function does nothing
    pub fn reinitialize_probe(&self) -> Result<(), DebugProbeError>{
        let probe_info = self.probe_info.lock();

        if let Some(probe_info) = probe_info.as_ref(){
            *self.probe.lock() = Some(probe_info.open()?);
        }

        Ok(())
    }

    /// Returns a cloned instance of the currently stored probe_info
    pub fn get_probe_info(&self) -> Option<DebugProbeInfo> {
        let probe_info = &self.probe_info.lock();

        match probe_info.as_ref() {
            Some(probe_info) => Some(probe_info.clone()),
            None => None,
        }
    }

    /// Check if testchannel has a probe attached and is ready to be used during testing
    pub fn is_ready(&self) -> bool {
        self.probe.lock().is_some()
    }

    pub fn get_channel(&self) -> TestChannel {
        self.channel
    }

    /// Returns a owned instance of the [`Probe`] which is currently held by this struct. The probe field of this struct is replaced with [`Option::None`], until [`Probe`] ownership is returned to this struct by calling [`Self::return_probe()`].
    /// 
    /// # Panics
    /// If the current probe field of the struct is [Option::None]
    pub fn take_probe_owned(&self) -> Probe{
        let mut probe = self.probe.lock();
        mem::take(&mut *probe).expect("Tried to take owned instance of Probe struct but found None, make sure to call bind_probe before taking the Probe out of the struct.")
    }

    /// Used to return the owned [`Probe`] to the struct after it has been taken with [`Self::take_probe_owned()`]
    pub fn return_probe(&self, probe: Probe) {
        *self.probe.lock() = Some(probe);
    }

    pub fn get_rpi(&self) -> &PoisonFreeMutex<RpiTestChannel> {
        &self.rpi
    }

    /*/// Reset the test channel to defaults for use in next test
    pub fn reset(&self) -> Result<(), Box<dyn Error>> {
        self.rpi.lock().test_gpio_reset()?;

        if let Some(ref mut probe) = *self.probe.lock() {
            probe.detach()?;
        }

        Ok(())
    }*/

    /// Loops through all available TSS and connects the testchannel to each available target, while executing the provided function on each connection.
    pub fn connect_all_available_and_execute<F>(&mut self, tss: &[Option<Mutex<TargetStackShield>>], mut function: F) where F: FnMut(&mut Self, &TargetInfo, u8) {
        let mut unprocessed_tss_queue: Vec<&Mutex<TargetStackShield>> = tss.iter().filter_map(|tss| tss.as_ref()).collect();

        while !unprocessed_tss_queue.is_empty() {
            match unprocessed_tss_queue[0].try_lock() {
                Ok(tss) => {
                    log::trace!("{}: locked tss {}.", self.get_channel(), tss.get_position());

                    if let Some(targets) = tss.get_targets() {
                        for (pos, target) in targets.iter().enumerate() {
                            if let TargetState::Known(ref target_info) = target {
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
                                    Ok(_) => function(self, target_info, tss.get_position()),
                                    Err(err) => match err {
                                        retry::Error::Operation { error, ..} => {
                                            log::error!(
                                                "Failed to connect testchannel {:?} to target {:?}: {}\nCaused by: {:?}",
                                                self.channel,
                                                Target::try_from(pos as u8).unwrap(),
                                                error,
                                                error.source()
    
                                            );
                                            // At this point it is uncertain in which state the busswitches are. Therefore we try to disconnect all affected switches, so any remaining operations are not influenced by this error.
                                            // If disconnecting fails the testrack hardware is in an undefined and unrecoverable state, therefore the application panics as such errors need manual power reset and are likely caused by faulty hardware
                                            tss.inner.borrow_mut().disconnect_all().expect("Failed to disconnect tss successfully, this error cannot be recovered, as further operation in such a state may influence other testchannels.\n This is likely caused by a hardware issue in the I2C communication, please verify that your hardware is working correctly.");
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
