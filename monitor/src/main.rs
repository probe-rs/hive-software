//! The monitor is the heart of the Hive testrack.
//!
//! It contains the following key functionalities:
//! - **Database** (Used to store testrack configuration data as well as user data)
//! - **Webserver** (Provides various API endpoints to communicate with the outside world)
//! - **Testprograms** (Manages, Builds and Links testprograms that can be flashed onto the targets)
//! - **Task manager** (Manages all received test requests and hardware reinitialization requests)
//! - **Flash** (Flashes the testprogram onto every target)
//!
//! # Overview
//! In order to understand how Hive works we first need to have a look at how the testing procedure is supposed to work.
//!
//! Generally the goal of Hive is to provide automated testing of all kinds of possible probe to target combinations. At a very basic level we want to be able to test the following:
//! - RAM Read/Write
//! - Flash Read/Write
//! - Core Halt/Breakpoint/Reset
//!
//! To properly test those things it is necessary to have some kind of program flashed on the target before the actual testing can begin. This program is called a "testprogram" in Hive.
//! So it is not necessarily testing anything but just enabling that tests can be performed on the target by having some kind of defined program flow on the target.
//!
//! Now what is left is logic to flash those testprograms onto each connected target. What is even more important is to have a way to actually test a specific version of probe-rs against the Hive testrack hardware.
//! This version of probe-rs is different for each test run and will be referred to as probe-rs testcandidate to avoid any confusion with the probe-rs stable version.
//!
//! Ultimately there is a need to build the probe-rs testcandidate dynamically with all the Hive testfunctions defined by the user.
//!
//! ## Monitor vs Runner
//! It would make little to no sense (and is probably impossible) to rebuild the monitor with the provided probe-rs testcandidate on every test request while still keeping the webserver and other functionality of the monitor operational.
//!
//! This is where the [runner](https://github.com/probe-rs/hive-software/tree/master/runner) binary crate comes into play.
//! By using the runner binary we're able to split the need to have a binary which cannot afford any downtime as it exposes services with such requirements (monitor) and a binary which needs to be able to be built dynamically according to frequent changes (runner).
//!
//! Compared to the monitor the runner is quite trivial as all it does is to test the probe-rs testcandidate on the Hive testrack. Its responsibilities can be broken down into the following steps:
//! - Initialize testserver hardware
//! - Run the provided testfunctions on the hardware and collect results
//! - Return results and terminate
//!
//! Therefore the monitor is now able to manage the Hive testrack and provide communication interfaces without facing interruption. The monitor now roughly needs to do the following on a test request:
//! - Receive test request with a prebuilt runner binary via communication interface
//! - Rebuild/link testprogram binaries in case the testprogram was modified
//! - Reinitialize hardware in case configuration data changed
//! - Flash testprograms onto targets
//! - Run the received runner binary
//! - Receive results from runner and send back to request source
//! - Reinitialize hardware and prepare for next test request
//!
//! Another advantage of having the runner as a separate binary is to be able to run the runner binary in some kind of isolated environment. This is important to the server as executing the runner is basically intended remote code execution.
//! The monitor has no control over the contents of the runner and is therefore also not able to determine if the runner binary contains any malicious code.
//!
//! # Operation modes
//! The monitor is intended to support multiple operation modes. Currently only standalone mode and init mode is implemented.
//!
//! ## Init mode
//! Init mode is the first mode that the binary needs to be run in after installation as it is the only way to create the initial admin user who is able to access the backend user interface to further configure the testrack.
//!
//! ## Standalone mode
//! This is the default operation mode of the monitor which uses one Hive testrack and exposes its functions to the public via a webserver.
use std::path::Path;
use std::sync::{Arc, Mutex};

use clap::{Parser, ValueEnum};
use controller::hardware::{self, HiveHardware, HiveIoExpander, MAX_TSS};
use controller::logger;
use lazy_static::lazy_static;
use log::Level;
use rppal::i2c::I2c;
use shared_bus::BusManager;
use tokio::sync::broadcast::{self, Sender};
use tokio::sync::Mutex as AsyncMutex;

mod database;
mod flash;
mod init;
mod mode;
mod tasks;
mod testprogram;
mod webserver;

use database::MonitorDb;

/// Path to the monitor log file
const LOGFILE_PATH: &str = "./data/logs/monitor.log";
/// Max monitor logfile size until it is deleted and recreated
const MAX_LOGFILE_SIZE: u64 = 50_000_000; // 50MB

lazy_static! {
    static ref SHARED_I2C: &'static BusManager<Mutex<I2c>> = {
        let i2c = I2c::new()
            .expect("Failed to acquire I2C bus. It might still be blocked by another process");
        shared_bus::new_std!(I2c = i2c).unwrap()
    };
    static ref EXPANDERS: [HiveIoExpander; MAX_TSS] = hardware::create_expanders(&SHARED_I2C);
    // HiveHardware is wrapped in a mutex in order to ensure that no hardware modifications are done concurrently (For example to avoid that the hardware is reinitialized in the monitor while the runner is running tests)
    static ref HARDWARE: Mutex<HiveHardware> =
        Mutex::new(HiveHardware::new(&SHARED_I2C, &EXPANDERS));
    /// Flag which is set to true if any data related to the testrack hardware has changed in the database
    static ref HARDWARE_DB_DATA_CHANGED: AsyncMutex<bool> = AsyncMutex::new(false);
    /// Flag which is set to true if the active testprogram has been changed
    static ref ACTIVE_TESTPROGRAM_CHANGED: AsyncMutex<bool> = AsyncMutex::new(false);
    static ref SHUTDOWN_SIGNAL: Sender<()> = {
        let (sender, _) = broadcast::channel::<()>(1);
        sender
    };
}

/// The different modes the monitor can be run in. For more information please take a look at the [`mode`] module.
#[derive(Clone, Debug, ValueEnum)]
pub enum ApplicationMode {
    Init,
    Standalone,
    ClusterSlave,
    ClusterMaster,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// Which mode the monitor runs in
    #[clap(value_enum, default_value_t = ApplicationMode::Standalone)]
    pub mode: ApplicationMode,
    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
    /// The port used for http connections
    #[arg(long, default_value_t = 80)]
    pub http_port: u16,
    /// The port used for https connections
    #[arg(long, default_value_t = 443)]
    pub https_port: u16,
}

fn main() {
    let cli_args = Arc::new(Args::parse());

    logger::init_logging(
        Path::new(LOGFILE_PATH),
        MAX_LOGFILE_SIZE,
        get_log_level(&cli_args.verbose.log_level()).to_level_filter(),
    );

    let db = Arc::new(MonitorDb::open());

    let task_manager = Arc::new(tasks::TaskManager::new());

    let task_runner = tasks::runner::TaskRunner::new(db.clone());

    match cli_args.mode {
        ApplicationMode::Init => mode::init::run_init_mode(db),
        ApplicationMode::Standalone => {
            mode::standalone::run_standalone_mode(db, task_manager, task_runner, cli_args)
        }
        ApplicationMode::ClusterSlave => todo!(),
        ApplicationMode::ClusterMaster => todo!(),
    }
    // Only global shutdown procedures should be called here, application logic should be put inside the appropriate mode handler
}

/// Gets the log level of the application to the provided verbosity flag. If no flag was provided the default [`Level::Error`] is used.
fn get_log_level(verbosity: &Option<log::Level>) -> Level {
    match verbosity {
        Some(level) => *level,
        None => Level::Error,
    }
}

/// Sends the shutdown signal to tokio tasks which then stop and return to terminate the program.
fn shutdown_application() {
    SHUTDOWN_SIGNAL
        .send(())
        .expect("No receivers available to send the shutdown signal to.");

    println!("Shutting down application...");
}
