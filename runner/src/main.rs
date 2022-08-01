//! The Hive runner is a binary crate which is used to run the Hive tests which are defined in the probe-rs crate.
//!
//! The runner is not meant as a standalone binary and can only be used meaningfully in combination with the [monitor](https://github.com/probe-rs/hive-software/tree/master/monitor)
//! binary which is executing the runner binary to perform tests.
//!
//! In order to test the provided probe-rs crate using the Hive testfunctions defined in the probe-rs crate it uses the probe-rs testcandidate directly as dependency.
//! This means that the runner binary has to be built on each test request separately with the new probe-rs code and tests. The building process is usually handled by the [Hive CLI](https://github.com/probe-rs/hive-software/tree/master/hive)
//! which takes care of the runner building process.
//!
//! # Overview
//! The general logic of the runner is quite simple as it is only meant to execute the tests it was built with on the hardware and reporting the results to the monitor via IPC.
//!
//! ## Startup
//! On startup the runner binary starts an async thread which is used for IPC communication with the monitor. Once the IPC thread is ready it will issue data requests via IPC to the monitor. Specifically the following data is requested:
//! - Hive Target data (Which shows where specific targets are connected on the testrack)
//! - Hive Probe data (Which shows where specific probes are connected on the testrack)
//! - Hive Define data (Contains the state of the Hive Define variables which are used on the currently flashed testprogram)
//!
//! In case this initial data transfer fails the runner shuts down with a non-zero exit code, as it cannot perform tests without knowing the current testrack state.
//!
//! With the initialization data received it tries to initialize the testrack hardware using the received data.
//! If the received data does not match with the detected testrack hardware it is considered a data desync which leads to a runner shutdown with a non-zero exit code.
//! Such data desyncs are only expected to happen in case a daughterboard is removed in between the monitor hardware reinitialization and runner execution or in case of a hardware failure.
//!
//! ## Testing
//! After startup the main thread spawns n new threads where n equals to the currently active testchannels which do have a debug probe connected.
//! Each of those threads now connects to every available target using the debug probe assigned to its testchannel. On each probe to target connection every Hive-test which was built into this runner binary is executed on the target.
//!
//! The result of each test is cached into a buffer and ultimately, once all tests on all threads have been executed, sent via IPC to the monitor.
//! Once the results have been sent the runner terminates.
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::{panic, thread};

use anyhow::Result;
use comm_types::defines::DefineRegistry;
use comm_types::ipc::{HiveProbeData, HiveTargetData};
use comm_types::test::{Filter, TestOptions};
use controller::hardware::{self, CombinedTestChannel, HiveHardware, HiveIoExpander, MAX_TSS};
use controller::logger;
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
use wildmatch::WildMatch;

use crate::comm::Message;

mod comm;
mod hive;
mod init;
mod test;

/// Path to where the runner log file is stored
const LOGFILE_PATH: &str = "./data/logs/runner.log";
/// Max size of the runner logfile until it gets deleted and replaced by a new one
const MAX_LOGFILE_SIZE: u64 = 50_000_000; // 50MB

lazy_static! {
    static ref SHARED_I2C: &'static BusManager<Mutex<I2c>> = {
        let i2c = I2c::new()
            .expect("Failed to acquire I2C bus. It might still be blocked by another process");
        shared_bus::new_std!(I2c = i2c).unwrap()
    };
    static ref EXPANDERS: [HiveIoExpander; MAX_TSS] = hardware::create_expanders(&SHARED_I2C);
    static ref HARDWARE: HiveHardware = HiveHardware::new(&SHARED_I2C, &EXPANDERS);
    /// Crate global shutdown signal to signal async threads to shutdown
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
    init_data_receiver: OneshotReceiver<(
        HiveProbeData,
        HiveTargetData,
        DefineRegistry,
        TestOptions,
    )>,
    notify_results_ready: Arc<Notify>,
) -> Result<()> {
    // Wait until the init data was received from monitor
    let (probe_data, target_data, define_data, test_options) = init_data_receiver.blocking_recv().map_err(|err| {
        log::error!(
            "The oneshot sender in the async comm-thread has been dropped, shutting down. This is either caused by a panic in the comm-thread or an error in the code.",
        );
        err
    })?;

    let define_registry = Arc::new(define_data);
    let test_options = Arc::new(test_options);

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

    let mut panic_hook_sync = Barrier::new(get_available_channel_count(test_options.clone()) + 1);

    let mut testing_threads = vec![];

    for (idx, test_channel) in HARDWARE.testchannels.iter().enumerate() {
        let channel = test_channel.lock().unwrap();

        let mut panic_hook_sync = panic_hook_sync.clone();

        if channel_is_ready_and_not_filtered(&channel, test_options.clone()) {
            drop(channel);
            let comm_sender = comm_sender.clone();
            let define_registry = define_registry.clone();
            let test_options = test_options.clone();

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
                                test::run_tests(
                                    test_channel,
                                    target_info,
                                    tss_pos,
                                    &sender,
                                    define_registry.clone(),
                                    test_options.clone(),
                                );
                            },
                        );
                    })
                    .unwrap(),
            );
        }
    }

    // Set the custom test thread panic hook, once all testchannels are ready to run the testfunctions
    panic_hook_sync.wait();
    let standard_hook = test::set_test_fn_panic_hook();
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
fn get_available_channel_count(test_options: Arc<TestOptions>) -> usize {
    let mut available_channels = 0;

    for test_channel in HARDWARE.testchannels.iter() {
        let channel = test_channel.lock().unwrap();
        if channel_is_ready_and_not_filtered(&channel, test_options.clone()) {
            available_channels += 1;
        }
    }

    available_channels
}

/// Check if the provided testchannel is ready for testing and not filtered out by the provided test options
fn channel_is_ready_and_not_filtered(
    channel: &CombinedTestChannel,
    test_options: Arc<TestOptions>,
) -> bool {
    if !channel.is_ready() {
        return false;
    }

    if let Some(options) = test_options.filter.as_ref() {
        if let Some(probe_filter) = options.probes.as_ref() {
            let probe_info = channel.get_probe_info().unwrap();
            let mut filter_is_include = true;

            let probes = match probe_filter {
                Filter::Include(probes) => probes,
                Filter::Exclude(probes) => {
                    filter_is_include = false;
                    probes
                }
            };

            let probe_identifier = probe_info.identifier.to_lowercase();
            let probe_serial = probe_info.serial_number.unwrap_or_default().to_lowercase();

            for probe in probes {
                let probe_lowercase = probe.to_lowercase();
                if WildMatch::new(&probe_lowercase).matches(&probe_identifier)
                    || WildMatch::new(&probe_lowercase).matches(&probe_serial)
                {
                    return filter_is_include;
                }
            }

            // Nothing matched, so we either put as filtered in case of an include filter or put as not filtered in case of an exclude filter
            return !filter_is_include;
        }
    }

    true
}

/// Triggers the shutdown signal and causes the application to terminate early
fn shutdown_err() {
    SHUTDOWN_SIGNAL
        .send(())
        .expect("Failed to send shutdown signal as there are no receivers active");
}
