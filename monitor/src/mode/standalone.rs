//! Handle standalone mode
use std::sync::Arc;
use std::thread;

use tokio::runtime::Builder;

use crate::database::HiveDb;
use crate::{comm, dummy_unlock_probes, flash, init};

pub(crate) fn run_standalone_mode(db: Arc<HiveDb>) {
    init::check_uninit(db.clone());

    init::initialize_statics();

    init::init_hardware(db.clone());

    init::init_testprograms(db.clone());

    flash::flash_testbinaries(db.clone());

    let rt = Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();
    let comm_tread = thread::spawn(move || {
        rt.block_on(async {
            comm::serve(db.clone()).await;
        });
    });

    dummy_unlock_probes();
    log::info!("Dropped the debug probes... runner can now be started.");

    // Wait for comm thread to shutdown
    comm_tread.join().unwrap();
}
