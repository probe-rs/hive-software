use std::thread;

use log::Level;
use simple_clap_logger::Logger;
use tokio::runtime::Builder;

mod comm;

fn main() {
    Logger::init_with_level(Level::Trace);

    let rt = Builder::new_current_thread().enable_io().build().unwrap();
    let comm_tread = thread::spawn(move || {
        rt.block_on(async {
            comm::serve().await;
        });
    });

    comm_tread.join().unwrap();
}
