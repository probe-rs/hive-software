//! Runner initialization functions
use comm_types::ipc::{HiveProbeData, HiveTargetData};
use controller::hardware::InitError;

use crate::{EXPANDERS, HARDWARE, SHARED_I2C, TEST_FUNCTIONS};

pub fn init_hardware_from_monitor_data(
    target_data: HiveTargetData,
    probe_data: HiveProbeData,
) -> Result<(), InitError> {
    HARDWARE.initialize_target_data(target_data)?;
    HARDWARE.initialize_probe_data(probe_data)
}

pub fn initialize_statics() {
    lazy_static::initialize(&SHARED_I2C);
    lazy_static::initialize(&EXPANDERS);
    lazy_static::initialize(&HARDWARE);
    lazy_static::initialize(&TEST_FUNCTIONS);
}
