//! Provides functions to synchronize the db data with the runtime data
use std::sync::Arc;

use comm_types::ipc::HiveTargetData;

use super::{keys, CborDb, HiveDb};
use crate::TSS;

/// Synchronize the DB target data with the current target data in the runtime [`TSS`].
pub(crate) fn sync_tss_target_data(db: Arc<HiveDb>) {
    let mut target_data: HiveTargetData = Default::default();

    for tss in TSS.iter() {
        let tss = tss.lock().unwrap();

        if tss.get_targets().is_none() {
            continue;
        }

        target_data[tss.get_position() as usize] = tss.get_targets().clone();
    }

    db.config_tree
        .c_insert(keys::config::ASSIGNED_TARGETS, &target_data)
        .unwrap();
}
