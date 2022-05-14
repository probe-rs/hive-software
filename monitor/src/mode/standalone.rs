//! Handle standalone mode
use std::thread;

use tokio::runtime::Builder;

use crate::{comm, database, dummy_unlock_probes, flash, init};

pub(crate) fn run_standalone_mode() {
    init::check_uninit();

    init::initialize_statics();

    init::init_tss();
    init::init_hardware_from_db_data().expect("TODO, stop initialization and enter 'NOT READY' state which should tell the user to provide the initialization in the backend UI");
    init::init_target_info_from_registry();
    init::init_testprograms();

    flash::flash_testbinaries();

    // Synchronize the target data in the DB with the runtime data so that the runner receives valid data.
    database::sync::sync_tss_target_data();

    let rt = Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();
    let comm_tread = thread::spawn(move || {
        rt.block_on(async {
            comm::serve().await;
        });
    });

    dummy_unlock_probes();
    log::info!("Dropped the debug probes... runner can now be started.");

    comm_tread.join().unwrap();
}
