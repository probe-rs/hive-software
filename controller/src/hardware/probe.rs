//! Functions used to manage debug probe related functionality
use std::error::Error;

use comm_types::hardware::TargetInfo;

// Depending on the usecase, the probe-rs dependency is either stable, or the one being tested by Hive
#[cfg(not(feature = "runner"))]
use probe_rs::{DebugProbeInfo, Session};
#[cfg(feature = "runner")]
use probe_rs_test::{DebugProbeInfo, Session};

use super::CombinedTestChannel;

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
    match probe.attach(&target_info.name) {
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
    let mut probe = probe_info.open()?;
    let _ = probe.set_speed(DEBUG_PROBE_SPEED_HZ);

    match probe.attach_under_reset(&target_info.name) {
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
