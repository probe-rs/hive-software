//! Provides functions to synchronize the db data with the runtime data
use std::sync::Arc;

use comm_types::ipc::HiveTargetData;
use controller::common::hardware::HiveHardware;
use hive_db::CborDb;

use super::{keys, MonitorDb};

/// Synchronize the DB target data with the provided [`HiveHardware`] data.
pub(crate) fn sync_tss_target_data(db: Arc<MonitorDb>, hardware: &HiveHardware) {
    let mut target_data: HiveTargetData = Default::default();

    for tss in hardware.tss.iter().filter_map(|tss| tss.as_ref()) {
        let tss = tss.lock().unwrap();

        if tss.get_targets().is_none() {
            continue;
        }

        target_data[tss.get_position() as usize] = tss.get_targets().clone();
    }

    db.config_tree
        .c_insert(&keys::config::ASSIGNED_TARGETS, &target_data)
        .unwrap();
}
