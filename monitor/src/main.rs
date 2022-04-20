use std::sync::{Arc, Mutex};
use std::thread;

use controller::common::{
    create_expanders, create_shareable_testchannels, create_shareable_tss, CombinedTestChannel,
    TargetStackShield,
};
use controller::HiveIoExpander;
use lazy_static::lazy_static;
use log::Level;
use rppal::i2c::I2c;
use shared_bus::BusManager;
use simple_clap_logger::Logger;
use tokio::runtime::Builder;

mod binaries;
mod comm;
mod database;
mod init;

use database::HiveDb;

lazy_static! {
    static ref SHARED_I2C: &'static BusManager<Mutex<I2c>> = {
        let i2c = I2c::new()
            .expect("Failed to acquire I2C bus. It might still be blocked by another process");
        shared_bus::new_std!(I2c = i2c).unwrap()
    };
    static ref EXPANDERS: [HiveIoExpander; 8] = create_expanders(&SHARED_I2C);
    static ref TSS: Vec<Mutex<TargetStackShield>> = create_shareable_tss(&SHARED_I2C, &EXPANDERS);
    static ref TESTCHANNELS: [Mutex<CombinedTestChannel>; 4] = create_shareable_testchannels();
}

fn main() {
    Logger::init_with_level(Level::Info);

    init::initialize_statics();
    let db = Arc::new(HiveDb::open());
    let comm_db = db.clone();

    init::dummy_init_config_data(db.clone());
    init::init_testprograms(db.clone());

    let rt = Builder::new_current_thread().enable_io().build().unwrap();
    let comm_tread = thread::spawn(move || {
        rt.block_on(async {
            comm::serve(comm_db).await;
        });
    });

    // Drop DB so all buffered changes are written to memory once all Arc instances of the db have been dropped
    drop(db);

    comm_tread.join().unwrap();
}
