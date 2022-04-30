use std::sync::Mutex;
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
    static ref DB: HiveDb = HiveDb::open();
}

fn main() {
    Logger::init_with_level(Level::Info);

    init::initialize_statics();

    init::dummy_init_config_data();
    init::init_hardware_from_db_data().expect("TODO, stop initialization and enter 'NOT READY' state which should tell the user to provide the initialization in the backend UI");
    init::init_target_info_from_registry();
    init::init_testprograms();

    binaries::flash_testbinaries();

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

// Current dummy implementation of unlocking the debug probes, so the runner can take over control. Only used for testing/demo purposes
fn dummy_unlock_probes() {
    for testchannel in TESTCHANNELS.iter() {
        let testchannel = testchannel.lock().unwrap();

        if testchannel.is_ready() {
            testchannel.take_probe_owned(); // We just drop the probe to make it available to the runner
        }
    }
}
