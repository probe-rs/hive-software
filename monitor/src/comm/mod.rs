//! Communications handler

mod ipc;
mod webserver;

pub(crate) async fn serve() {
    log::info!("starting server");
    let ipc_handle = tokio::spawn(ipc::ipc_server());

    ipc_handle.await.unwrap();
}
