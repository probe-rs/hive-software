use std::path::Path;
use std::sync::{Arc, Mutex};

use clap::{ArgEnum, Parser};
use controller::common::hardware::{self, HiveHardware, HiveIoExpander, MAX_TSS};
use controller::common::logger;
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

const LOGFILE_PATH: &str = "./data/logs/monitor.log";
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
    static ref HARDWARE_DB_DATA_CHANGED: AsyncMutex<bool> = AsyncMutex::new(false);
    static ref ACTIVE_TESTPROGRAM_CHANGED: AsyncMutex<bool> = AsyncMutex::new(false);
    static ref SHUTDOWN_SIGNAL: Sender<()> = {
        let (sender, _) = broadcast::channel::<()>(1);
        sender
    };
}

/// The different modes the monitor can be run in. For more information please take a look at the [`mode`] module.
#[derive(Clone, Debug, ArgEnum)]
enum ApplicationMode {
    Init,
    Standalone,
    ClusterSlave,
    ClusterMaster,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Which mode the monitor runs in
    #[clap(arg_enum, default_value_t = ApplicationMode::Standalone)]
    mode: ApplicationMode,
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() {
    let cli_args = Args::parse();
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
            mode::standalone::run_standalone_mode(db, task_manager, task_runner)
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
