//! Communications handler
use std::sync::Arc;

use crate::database::HiveDb;

mod ipc;
mod webserver;

pub(crate) async fn serve(db: Arc<HiveDb>) {
    log::info!("starting server");
    let ipc_handle = tokio::spawn(ipc::ipc_server(db.clone()));

    ipc_handle.await.unwrap();
}
