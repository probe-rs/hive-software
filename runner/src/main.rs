use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{panic, thread};

use anyhow::Result;
use comm_types::ipc::{HiveProbeData, HiveTargetData};
use controller::common::hardware::{self, HiveHardware, HiveIoExpander, MAX_TSS};
use controller::common::logger;
use hurdles::Barrier;
use lazy_static::lazy_static;
use log::Level;
use rppal::i2c::I2c;
use shared_bus::BusManager;
use test::TEST_FUNCTIONS;
use tokio::runtime::Builder;
use tokio::sync::broadcast::{self, Sender as BroadcastSender};
use tokio::sync::mpsc::{Receiver as MpscReceiver, Sender as MpscSender};
use tokio::sync::oneshot::{self, Receiver as OneshotReceiver};
use tokio::sync::Notify;

use crate::comm::Message;

mod comm;
mod hive;
mod init;
mod test;

const LOGFILE_PATH: &str = "./data/logs/runner.log";
const MAX_LOGFILE_SIZE: u64 = 50_000_000; // 50MB

lazy_static! {
    static ref SHARED_I2C: &'static BusManager<Mutex<I2c>> = {
        let i2c = I2c::new()
            .expect("Failed to acquire I2C bus. It might still be blocked by another process");
        shared_bus::new_std!(I2c = i2c).unwrap()
    };
    static ref EXPANDERS: [HiveIoExpander; MAX_TSS] = hardware::create_expanders(&SHARED_I2C);
    static ref HARDWARE: HiveHardware = HiveHardware::new(&SHARED_I2C, &EXPANDERS);
    static ref SHUTDOWN_SIGNAL: BroadcastSender<()> = {
        let (sender, _) = broadcast::channel::<()>(1);
        sender
    };
}

fn main() {
    logger::init_logging(
        Path::new(LOGFILE_PATH),
        MAX_LOGFILE_SIZE,
        Level::Info.to_level_filter(),
    );
    log::info!("starting the runner");

    init::initialize_statics();

    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let (comm_sender, comm_receiver): (_, MpscReceiver<Message>) = tokio::sync::mpsc::channel(30);
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

    let run_is_err = run(comm_sender, init_data_receiver, notify_results_ready).is_err();

    if run_is_err {
        // Manually stop async comm tasks from executing
        shutdown_err();
    }

    // Wait for communications to finish
    comm_tread.join().unwrap();

    let exit_code = match run_is_err {
        true => 1,
        false => 0,
    };

    std::process::exit(exit_code);
}

/// Run the main thread
fn run(
    comm_sender: MpscSender<Message>,
    init_data_receiver: OneshotReceiver<(HiveProbeData, HiveTargetData)>,
    notify_results_ready: Arc<Notify>,
) -> Result<()> {
    // Wait until the init data was received from monitor
    let (probe_data, target_data) = init_data_receiver.blocking_recv().map_err(|err| {
        log::error!(
            "The oneshot sender in the async comm-thread has been dropped, shutting down. This is either caused by a panic in the comm-thread or an error in the code.",
        );
        err
    })?;

    match init::init_hardware_from_monitor_data(target_data, probe_data) {
        Ok(_) => log::debug!("Successfully initialized hardware from monitor data."),
        Err(err) => {
            log::error!(
                "Failed to initialize the hardware data: {} Shutting down...",
                err
            );

            return Err(err.into());
        }
    }

    let mut panic_hook_sync = Barrier::new(get_available_channel_count() + 1);

    let mut testing_threads = vec![];

    for (idx, test_channel) in HARDWARE.testchannels.iter().enumerate() {
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
                            &HARDWARE.tss,
                            |test_channel, target_info, tss_pos| {
                                test::run_tests(test_channel, target_info, tss_pos, &sender);
                            },
                        );
                    })
                    .unwrap(),
            );
        }
    }

    // Disable panic printing, once all testchannels are ready to run the testfunctions
    panic_hook_sync.wait();
    let standard_hook = test::disable_panic_print();
    panic_hook_sync.wait();

    // drop mpsc sender instance owned by main thread to quit the communications loop once all testfunctions have finished
    drop(comm_sender);

    // Wait for all tests to finish
    for thread in testing_threads {
        thread.join().unwrap();
    }

    // Reenable panic printing
    panic::set_hook(standard_hook);

    notify_results_ready.notify_one();

    log::info!("Finished testing task, shutting down...");

    Ok(())
}

/// Returns the amount of testchannels which are ready for testing (A testchannel is considered ready once a probe has been bound to it)
fn get_available_channel_count() -> usize {
    let mut available_channels = 0;

    for test_channel in HARDWARE.testchannels.iter() {
        let channel = test_channel.lock().unwrap();
        if channel.is_ready() {
            available_channels += 1;
        }
    }

    available_channels
}

fn shutdown_err() {
    SHUTDOWN_SIGNAL
        .send(())
        .expect("Failed to send shutdown signal as there are no receivers active");
}
