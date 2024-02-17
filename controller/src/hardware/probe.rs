//! Functions used to manage debug probe related functionality
use std::error::Error;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use std::time::Duration;

use comm_types::hardware::TargetInfo;
use thiserror::Error;

// Depending on the usecase, the probe-rs dependency is either stable, or the one being tested by Hive
#[cfg(feature = "monitor")]
use probe_rs::{
    probe::{list::Lister, DebugProbeInfo},
    Permissions, Session,
};
#[cfg(feature = "runner")]
use probe_rs_test::{
    probe::{list::Lister, DebugProbeInfo},
    Permissions, Session,
};

use super::CombinedTestChannel;

#[derive(Debug, Error)]
pub enum ProbeResetError {
    #[error("Failed to identify probe usb interface: {0}")]
    RusbError(#[from] rusb::Error),
    #[error("Could not find any connected usb device which matches the provided DebugProbeInfo")]
    ProbeNotFound,
}

/// Speed to which the debug probe is set
const DEBUG_PROBE_SPEED_HZ: u32 = 8000;

/// Tries to attach the probe and runs the provided closure if attaching was successful
///
/// This function first tries to attach the probe normally. If this fails an attach under reset is performed. It also sets the speed of the probe to a max of [`DEBUG_PROBE_SPEED_HZ`]
pub fn try_attach<F>(
    testchannel: &CombinedTestChannel,
    target_info: &TargetInfo,
    probe_info: &DebugProbeInfo,
    function: F,
) -> Result<(), Box<dyn Error>>
where
    F: Fn(Session) -> Result<(), Box<dyn Error>>,
{
    let mut probe = testchannel.take_probe_owned();
    let _ = probe.set_speed(DEBUG_PROBE_SPEED_HZ);
    match probe.attach(&target_info.name, Permissions::new()) {
        Ok(session) => return function(session),
        Err(err) => {
            log::warn!(
                "Failed to attach probe {} to target {}: {}\nRetrying with attach-under-reset",
                probe_info.identifier,
                target_info.name,
                err
            )
        }
    }

    // Retry with attach under reset
    let probe_lister = Lister::new();
    let mut probe = probe_info.open(&probe_lister)?;
    let _ = probe.set_speed(DEBUG_PROBE_SPEED_HZ);

    match probe.attach_under_reset(&target_info.name, Permissions::new()) {
        Ok(session) => function(session),
        Err(err) => {
            log::warn!(
                "Failed to attach probe {} to target {} under reset: {}",
                probe_info.identifier,
                target_info.name,
                err
            );

            Err(Box::new(err))
        }
    }
}

/// Resets the usb interface to which the probe is connected to
pub fn reset_probe_usb(probe_info: &DebugProbeInfo) -> Result<(), ProbeResetError> {
    log::info!("start reset {:?}", probe_info);

    let mut usb_device =
        rusb::open_device_with_vid_pid(probe_info.vendor_id, probe_info.product_id)
            .ok_or(ProbeResetError::ProbeNotFound)?;

    log::info!("found device {:?}", usb_device);

    usb_device.reset()?;
    log::info!("success resetting device");

    /*
    let bus_path = format!(
                    "/dev/bus/usb/{:03}/{:03}",
                    device.device().bus_number(),
                    device.device().port_number()
                );

    if let Ok(file) = OpenOptions::new().read(true).write(true).open(bus_path) {
        log::info!("resetting usb {:?}", probe_info);
        unsafe {
            reset_usb(file.as_raw_fd()).unwrap();
        }

        log::info!(
            "Successfully reset the debug probe {} S/N: {:?}",
            probe_info.identifier,
            probe_info.serial_number
        );
        return Ok(());
    }
    */

    Ok(())
}

nix::ioctl_none!(
    /// Reset usb by calling USBDEVFS_RESET in linux usbdevice_fs.h
    reset_usb,
    b'U',
    20
);
