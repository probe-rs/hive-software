//! Simplified cbor encoder which uses the formatting in [`super::console`] but adds a log-level entry for easy level threshhold implementations in the user interface
use chrono::Local;
use ciborium::ser::into_writer;
use colored::Colorize;
use log::Level;
use log4rs::encode::Encode;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub(in crate::logger) struct CborEncoder;

impl CborEncoder {
    pub fn new() -> Self {
        Self
    }
}

impl Encode for CborEncoder {
    fn encode(
        &self,
        w: &mut dyn log4rs::encode::Write,
        record: &log::Record,
    ) -> anyhow::Result<()> {
        let time = Local::now();

        let module_path = record.module_path().unwrap_or("unknown");

        let level = match record.level() {
            log::Level::Error => "error:".red().bold(),
            log::Level::Warn => "warn:".yellow().bold(),
            log::Level::Info => "info:".green().bold(),
            log::Level::Debug => "debug:".blue().bold(),
            log::Level::Trace => "trace:".magenta().bold(),
        };

        let entry = LogEntry {
            level: record.level(),
            message: format!(
                "{} {:6} {} {}\n",
                time.format("%d.%m.%Y %H:%M:%S"),
                level,
                module_path.italic(),
                record.args()
            ),
        };

        into_writer(&entry, &mut *w)?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: Level,
    pub message: String,
}
