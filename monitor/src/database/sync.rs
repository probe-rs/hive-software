//! Provides functions to synchronize the db data with the runtime data
use std::sync::Arc;

use comm_types::{
    hardware::{ProbeInfo, ProbeState},
    ipc::HiveTargetData,
};
use controller::hardware::HiveHardware;
use hive_db::{CborDb, CborTransactional};
use sled::transaction::UnabortableTransactionError;

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

/// Synchronize the DB probe data with the provided [`HiveHardware`] data.
pub(crate) fn sync_testchannel_probe_data(db: Arc<MonitorDb>, hardware: &HiveHardware) {
    db.config_tree
        .transaction::<_, _, UnabortableTransactionError>(|tree| {
            let mut probe_data = tree
                .c_get(&keys::config::ASSIGNED_PROBES)?
                .unwrap_or_default();

            for (idx, testchannel) in hardware.testchannels.iter().enumerate() {
                let testchannel = testchannel.lock().unwrap();

                let probe_info = testchannel.get_probe_info();
                if probe_info.is_none() {
                    continue;
                }

                let probe_info = probe_info.unwrap();

                probe_data[idx] = ProbeState::Known(ProbeInfo {
                    identifier: probe_info.identifier,
                    vendor_id: probe_info.vendor_id,
                    product_id: probe_info.product_id,
                    serial_number: probe_info.serial_number,
                    hid_interface: probe_info.hid_interface,
                });
            }

            tree.c_insert(&keys::config::ASSIGNED_PROBES, &probe_data)?;

            Ok(())
        })
        .unwrap();
}
