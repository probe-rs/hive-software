use comm_types::ipc::{HiveProbeData, HiveTargetData};
use controller::common::{
    create_expanders, create_shareable_testchannels, create_shareable_tss, CombinedTestChannel,
    TargetStackShield,
};
use controller::HiveIoExpander;
use hurdles::Barrier;
use lazy_static::lazy_static;
use log::Level;
use rppal::i2c::I2c;
use shared_bus::BusManager;
use simple_clap_logger::Logger;
use tokio::runtime::Builder;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use tokio::sync::Notify;

use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::comm::Message;

mod comm;
mod hive_tests;
mod init;
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
    log::info!("starting the runner");

    init::initialize_statics();

    let mut testing_threads = vec![];

    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let (comm_sender, comm_receiver): (_, Receiver<Message>) = tokio::sync::mpsc::channel(30);
    let (init_data_sender, init_data_receiver) = oneshot::channel();
    let notify_results_ready = Arc::new(Notify::new());
    let notify_results_ready_copy = notify_results_ready.clone();
    let comm_tread = thread::Builder::new()
        .name("comm thread".to_owned())
        .spawn(move || {
            rt.block_on(async {
                comm::ipc(comm_receiver, init_data_sender, notify_results_ready_copy).await;
            });
        })
        .unwrap();

    // Wait until the init data was received from monitor
    let (probe_data, target_data) = match init_data_receiver.blocking_recv() {
        Ok(data) => data,
        Err(_) => {
            log::error!(
                "The oneshot sender in the async comm-thread has been dropped, shutting down. This is either caused by a panic in the comm-thread or an error in the code.",
            );

            shutdown_on_init(comm_sender);
            unreachable!();
        }
    };

    match init_hardware_from_monitor_data(target_data, probe_data) {
        Ok(_) => log::debug!("Successfully initialized hardware from monitor data."),
        Err(err) => {
            log::error!(
                "Failed to initialize the hardware data: {} Shutting down...",
                err
            );

            comm_sender
                .clone()
                .blocking_send(comm::Message::InitError(err))
                .unwrap();

            shutdown_on_init(comm_sender);
            unreachable!();
        }
    }

    let mut panic_hook_sync = Barrier::new(get_available_channel_count() + 1);

    for (idx, test_channel) in TESTCHANNELS.iter().enumerate() {
        let channel = test_channel.lock().unwrap();

        let mut panic_hook_sync = panic_hook_sync.clone();

        if channel.is_ready() {
            drop(channel);
            let comm_sender = comm_sender.clone();

            testing_threads.push(
                thread::Builder::new()
                    .name(format!("testing thread {}", idx).to_owned())
                    .spawn(move || {
                        log::trace!("Created testing thread {}", idx);

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
                    })
                    .unwrap(),
            );
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

    notify_results_ready.notify_one();

    // Wait for communications to finish
    comm_tread.join().unwrap();
    log::debug!("Joined comm thread");
}

fn init_hardware_from_monitor_data(
    target_data: HiveTargetData,
    probe_data: HiveProbeData,
) -> Result<(), init::InitError> {
    init::initialize_target_data(target_data)?;
    init::initialize_probe_data(probe_data)
}

/// Returns the amount of testchannels which are ready for testing (A testchannel is considered ready once a probe has been bound to it)
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

/// Detects if a Daugtherboard is present on each connected TSS, is true if present.
///
/// # Failure
/// In case the function fails to determine if a daughterboard is present on a TSS or not, it assumes that none is present.
/// Detect failures usually occur if the received TSS data from the monitor is desynced from the actual hardware configuration, or due to hardware errors.
/// If the false value is wrongly assumed by this function it will later cause a desync error which in turn forces the monitor to resync the hardware configuration and rerun the tests on the runner.
fn detect_connected_daughterboards() -> [bool; 8] {
    let mut detected = [false; 8];
    for tss in TSS.iter() {
        let mut tss = tss.lock().unwrap();

        match tss.inner.get_mut().daughterboard_is_connected() {
            Ok(is_connected) => {
                detected[tss.get_position() as usize] = is_connected;
            }
            Err(err) => {
                log::warn!(
                    "Failed to detect daughterboard on TSS {}, assuming none is connected. \n\nCaused by: {}",
                    tss.get_position(),
                    err
                );
            }
        }
    }
    detected
}

/// Handles the shutdown procedure, if the runner needs to shutdown during the init phase (before any tests were ran)
fn shutdown_on_init(comm_sender: Sender<Message>) {
    drop(comm_sender);
    // Timeout before shutting down comm thread
    thread::sleep(Duration::from_secs(2));
    std::process::exit(1);
}
