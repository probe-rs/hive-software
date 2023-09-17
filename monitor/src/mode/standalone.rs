//! Run in standalone mode
use std::sync::Arc;
use std::thread;

use tokio::runtime::Builder;

use crate::database::MonitorDb;
use crate::tasks::TaskManager;
use crate::{flash, init, webserver, Args, HARDWARE};

pub fn run_standalone_mode(db: Arc<MonitorDb>, cli_args: Arc<Args>) {
    init::check_uninit(db.clone());

    init::initialize_statics();

    let mut hardware = HARDWARE.lock().unwrap();
    init::init_hardware(db.clone(), &mut hardware);

    init::init_testprograms(db.clone(), &hardware);

    flash::flash_testbinaries(db.clone(), &hardware);
    drop(hardware);

    // Create async runtime
    let rt = Arc::new(
        Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap(),
    );

    let (task_manager, task_scheduler) = TaskManager::new(db.clone(), rt.clone());

    let async_tread = thread::spawn(move || {
        rt.block_on(async {
            tokio::spawn(async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to receive shutdown event");
                crate::shutdown_application();
            });

            webserver::web_server(db, task_manager, cli_args).await;
        });
    });

    task_scheduler.run();

    // Wait for async thread to shutdown
    async_tread.join().unwrap();
}
