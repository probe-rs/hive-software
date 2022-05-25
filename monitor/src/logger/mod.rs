//! The monitor logger which manages all logs
use std::path::Path;

use log::{Level, LevelFilter};
use log4rs::append::console::{ConsoleAppender, Target};
use log4rs::append::rolling_file::policy::compound::roll::delete::DeleteRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::filter::threshold::ThresholdFilter;

mod encoders;

use encoders::cbor::CborEncoder;
use encoders::console::ConsoleEncoder;

pub(crate) const LOGGER_RAMDISK_PATH: &str = "/mnt/hivetmp/";
const MAX_LOGFILE_SIZE: u64 = 50_000_000; // 50MB

pub(super) fn init_logging(console_log_level: Level) {
    let console_appender = ConsoleAppender::builder()
        .target(Target::Stdout)
        .encoder(Box::new(ConsoleEncoder::new()))
        .build();
    let file_appender = RollingFileAppender::builder()
        .encoder(Box::new(CborEncoder::new()))
        .build(
            Path::new(LOGGER_RAMDISK_PATH).join("monitor.log"),
            Box::new(CompoundPolicy::new(
                Box::new(SizeTrigger::new(MAX_LOGFILE_SIZE)),
                Box::new(DeleteRoller::new()),
            )),
        )
        .expect("Failed to setup file logger. Is the RAM-Disk for the log-storage setup properly?");

    let log_config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(
                    console_log_level.to_level_filter(),
                )))
                .build("console", Box::new(console_appender)),
        )
        .appender(Appender::builder().build("monitor file", Box::new(file_appender)))
        .build(
            Root::builder()
                .appender("monitor file")
                .appender("console")
                .build(LevelFilter::Trace),
        )
        .unwrap();

    log4rs::init_config(log_config).unwrap();
}
