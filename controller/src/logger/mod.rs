//! The standard hive logger which manages all logs
use std::path::Path;

use log::LevelFilter;
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

pub fn init_logging(
    logfile_path: &Path,
    logfile_max_size_bytes: u64,
    console_log_level_filter: LevelFilter,
) {
    let console_appender = ConsoleAppender::builder()
        .target(Target::Stdout)
        .encoder(Box::new(ConsoleEncoder::new()))
        .build();
    let file_appender = RollingFileAppender::builder()
        .encoder(Box::new(CborEncoder::new()))
        .build(
            logfile_path,
            Box::new(CompoundPolicy::new(
                Box::new(SizeTrigger::new(logfile_max_size_bytes)),
                Box::new(DeleteRoller::new()),
            )),
        )
        .expect("Failed to setup file logger. Is the RAM-Disk for the log-storage setup properly?");

    let log_config = Config::builder()
        .appender(
            Appender::builder()
                .filter(Box::new(ThresholdFilter::new(console_log_level_filter)))
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
