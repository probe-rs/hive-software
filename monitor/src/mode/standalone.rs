//! Handle standalone mode
use std::sync::Arc;
use std::thread;

use tokio::runtime::Builder;

use crate::database::MonitorDb;
use crate::tasks::runner::TaskRunner;
use crate::tasks::TaskManager;
use crate::{flash, init, webserver, HARDWARE};

pub(crate) fn run_standalone_mode(
    db: Arc<MonitorDb>,
    task_manager: Arc<TaskManager>,
    task_runner: TaskRunner,
) {
    init::check_uninit(db.clone());

    init::initialize_statics();

    let mut hardware = HARDWARE.lock().unwrap();
    init::init_hardware(db.clone(), &mut hardware);

    init::init_testprograms(db.clone(), &hardware);

    flash::flash_testbinaries(db.clone(), &hardware);
    drop(hardware);

    let rt = Arc::new(
        Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap(),
    );

    let rt_async = rt.clone();
    let task_manager_async = task_manager.clone();
    let async_tread = thread::spawn(move || {
        rt_async.block_on(async {
            tokio::spawn(async {
                tokio::signal::ctrl_c()
                    .await
                    .expect("Failed to receive shutdown event");
                crate::shutdown_application();
            });

            webserver::web_server(db, task_manager_async).await;
        });
    });

    task_runner.run(rt, &task_manager);

    // Wait for async thread to shutdown
    async_tread.join().unwrap();
}
