use std::sync::{Arc, Mutex};
use std::thread;

use comm_types::hardware::{ProbeInfo, TargetState};
use comm_types::ipc::{HiveProbeData, HiveTargetData};
use controller::common::{
    create_expanders, create_shareable_testchannels, create_shareable_tss, CombinedTestChannel,
    TargetStackShield,
};
use controller::HiveIoExpander;
use lazy_static::lazy_static;
use log::Level;
use probe_rs::Probe;
use rppal::i2c::I2c;
use shared_bus::BusManager;
use simple_clap_logger::Logger;
use tokio::runtime::Builder;

mod comm;
mod database;

use database::HiveDb;
use database::{keys, CborDb};

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
    Logger::init_with_level(Level::Trace);

    initialize_statics();
    let db = Arc::new(HiveDb::open());
    let comm_db = db.clone();

    dummy_init_config_data(db.clone());

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

fn initialize_statics() {
    lazy_static::initialize(&SHARED_I2C);
    lazy_static::initialize(&EXPANDERS);
    lazy_static::initialize(&TSS);
    lazy_static::initialize(&TESTCHANNELS);
}

/// Current dummy implementation of the configuration data initialization. Later this will be done by the user in the configuration backend UI
fn dummy_init_config_data(db: Arc<HiveDb>) {
    let target_data: HiveTargetData = [
        // atsamd daughterboard
        Some([
            TargetState::Known("ATSAMD10C13A-SS".to_owned()),
            TargetState::Known("ATSAMD09D14A-M".to_owned()),
            TargetState::Known("ATSAMD51J18A-A".to_owned()),
            TargetState::Known("ATSAMD21E16L-AFT".to_owned()),
        ]),
        None,
        // lpc daughterboard
        Some([
            TargetState::NotConnected,
            TargetState::Known("LPC1114FDH28_102_5".to_owned()),
            TargetState::NotConnected,
            TargetState::Known("LPC1313FBD48_01,15".to_owned()),
        ]),
        // nrf daughterboard
        Some([
            TargetState::Known("nRF5340".to_owned()),
            TargetState::Known("nRF52832-QFAB-T".to_owned()),
            TargetState::Known("nRF52840".to_owned()),
            TargetState::Known("NRF51822-QFAC-R7".to_owned()),
        ]),
        None,
        // stm32 daughterboard
        Some([
            TargetState::Known("STM32G031F4P6".to_owned()),
            TargetState::NotConnected,
            TargetState::Known("STM32L151C8TxA".to_owned()),
            TargetState::NotConnected,
        ]),
        None,
        None,
    ];

    db.config_tree
        .c_insert(keys::config::TARGETS, &target_data)
        .unwrap();

    let probes = Probe::list_all();

    let probe_data: HiveProbeData = [
        Some(ProbeInfo {
            identifier: probes[0].identifier.clone(),
            vendor_id: probes[0].vendor_id,
            product_id: probes[0].product_id,
            serial_number: probes[0].serial_number.clone(),
            hid_interface: probes[0].hid_interface,
        }),
        Some(ProbeInfo {
            identifier: probes[1].identifier.clone(),
            vendor_id: probes[1].vendor_id,
            product_id: probes[1].product_id,
            serial_number: probes[1].serial_number.clone(),
            hid_interface: probes[1].hid_interface,
        }),
        None,
        None,
    ];

    db.config_tree
        .c_insert(keys::config::PROBES, &probe_data)
        .unwrap();
}
