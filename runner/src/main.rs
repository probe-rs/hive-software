use comm_types::hardware::TargetState;
use controller::common::{
    create_expanders, create_shareable_testchannels, create_shareable_tss, CombinedTestChannel,
    TargetStackShield,
};
use controller::HiveIoExpander;
use hurdles::Barrier;
use lazy_static::lazy_static;
use ll_api::TestChannel;
use log::Level;
use rppal::i2c::I2c;
use shared_bus::BusManager;
use simple_clap_logger::Logger;
use tokio::runtime::Builder;
use tokio::sync::mpsc::Receiver;

use probe_rs_test::Probe;

use std::sync::Mutex;
use std::thread;

use crate::comm::Message;

mod comm;
mod hive_tests;
mod test;

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

    initialize_statics();
    initialize_probe_data();
    initialize_target_data();

    let mut testing_threads = vec![];

    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let (comm_sender, comm_receive): (_, Receiver<Message>) = tokio::sync::mpsc::channel(30);
    let comm_tread = thread::spawn(move || {
        rt.block_on(async {
            comm::ipc(comm_receive).await;
        });
    });

    let mut panic_hook_sync = Barrier::new(get_available_channel_count() + 1);

    for test_channel in TESTCHANNELS.iter() {
        let channel = test_channel.lock().unwrap();

        let mut panic_hook_sync = panic_hook_sync.clone();

        if channel.is_ready() {
            drop(channel);
            let comm_sender = comm_sender.clone();

            testing_threads.push(thread::spawn(move || {
                let mut channel = test_channel.lock().unwrap();
                let sender = comm_sender;

                // wait for all threads to be ready for running the testfunctions, once the panic hook has been set by the main thread
                panic_hook_sync.wait();
                panic_hook_sync.wait();

                channel.connect_all_available_and_execute(
                    &TSS,
                    |test_channel, target_name, tss_pos| {
                        test::run_tests(test_channel, target_name, tss_pos, &sender);
                    },
                );
            }));
        }
    }

    // Disable panic printing, once all testchannels are ready to run the testfunctions
    panic_hook_sync.wait();
    //let standard_hook = test::disable_panic_print();
    panic_hook_sync.wait();

    // drop mpsc sender instance owned by main thread to quit the communications loop once all testfunctions have finished
    drop(comm_sender);

    // Wait for all tests to finish
    for thread in testing_threads {
        thread.join().unwrap();
    }
    log::debug!("Joined all testing threads");

    // Reenable panic printing
    //panic::set_hook(standard_hook);

    // Wait for communications to finish
    comm_tread.join().unwrap();
    log::debug!("Joined comm thread");
}

fn initialize_statics() {
    lazy_static::initialize(&SHARED_I2C);
    lazy_static::initialize(&EXPANDERS);
    lazy_static::initialize(&TSS);
    lazy_static::initialize(&TESTCHANNELS);
}

fn initialize_target_data() {
    let target_pos_0 = [
        TargetState::Known("Neat Target 0".to_string()),
        TargetState::NotConnected,
        TargetState::Known("Neat Target 2".to_string()),
        TargetState::NotConnected,
    ];

    let target_pos_2 = [
        TargetState::NotConnected,
        TargetState::Known("Other Target 1".to_string()),
        TargetState::Known("Other Target 2".to_string()),
        TargetState::NotConnected,
    ];

    let mut tss_0 = TSS[0].lock().unwrap();
    tss_0.set_targets(target_pos_0);

    let mut tss_2 = TSS[2].lock().unwrap();
    tss_2.set_targets(target_pos_2);
}

fn initialize_probe_data() {
    let found_probes = Probe::list_all();

    log::debug!(
        "Found {} attached probes:\n{:#?}",
        found_probes.len(),
        found_probes
    );

    let testchannel_0 = TESTCHANNELS[0].lock().unwrap();
    testchannel_0.bind_probe(found_probes[0].open().unwrap());

    let testchannel_1 = TESTCHANNELS[1].lock().unwrap();
    testchannel_1.bind_probe(found_probes[1].open().unwrap());
}

/// Handles the reinitialization of a probe on the provided testchannel.
fn reinitialize_probe(channel: TestChannel) -> Probe {
    let found_probes = Probe::list_all();

    // TODO open and return correct probe based on data received by monitor
    todo!()
}

/// returns the amount of testchannels which are ready for testing (A testchannel is considered ready once a probe has been bound to it)
fn get_available_channel_count() -> usize {
    let mut available_channels = 0;

    for test_channel in TESTCHANNELS.iter() {
        let channel = test_channel.lock().unwrap();
        if channel.is_ready() {
            available_channels += 1;
        }
    }

    available_channels
}
