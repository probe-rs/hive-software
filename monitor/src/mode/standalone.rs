//! Handle standalone mode
use std::sync::Arc;
use std::thread;

use tokio::runtime::Builder;

use crate::database::MonitorDb;
use crate::testmanager::TestManager;
use crate::{flash, init, webserver, HARDWARE};

pub(crate) fn run_standalone_mode(db: Arc<MonitorDb>, mut test_manager: TestManager) {
    init::check_uninit(db.clone());

    init::initialize_statics();

    let mut hardware = HARDWARE.lock().unwrap();
    init::init_hardware(db.clone(), &mut hardware);

    init::init_testprograms(db.clone(), &hardware);
    drop(hardware);

    flash::flash_testbinaries(db.clone());

    let rt = Arc::new(
        Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap(),
    );

    let rt_async = rt.clone();
    let db_async = db.clone();
    let test_task_sender = test_manager.get_test_task_sender();
    let reinit_task_sender = test_manager.get_reinit_task_sender();
    let async_tread = thread::spawn(move || {
        rt_async.block_on(async {
            tokio::spawn(async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to receive shutdown event");
                crate::shutdown_application();
            });

            webserver::web_server(db_async, test_task_sender, reinit_task_sender).await;
        });
    });

    test_manager.run(db, rt);

    // Wait for async thread to shutdown
    async_tread.join().unwrap();
}
