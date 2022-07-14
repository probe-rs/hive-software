//! The standard console log encoder which pretty prints the log entries
use std::io::Write;

use chrono::Local;
use colored::Colorize;
use log4rs::encode::Encode;

#[derive(Debug)]
pub(in crate::logger) struct ConsoleEncoder;

impl ConsoleEncoder {
    pub fn new() -> Self {
        Self
    }
}

impl Encode for ConsoleEncoder {
    fn encode(
        &self,
        w: &mut dyn log4rs::encode::Write,
        record: &log::Record,
    ) -> anyhow::Result<()> {
        let time = Local::now();
        let mut buffer = vec![];

        let module_path = record.module_path().unwrap_or("unknown");

        let level = match record.level() {
            log::Level::Error => "error:".red().bold(),
            log::Level::Warn => "warn:".yellow().bold(),
            log::Level::Info => "info:".green().bold(),
            log::Level::Debug => "debug:".blue().bold(),
            log::Level::Trace => "trace:".magenta().bold(),
        };

        writeln!(
            &mut buffer,
            "{} {:6} {} {}",
            time.format("%d.%m.%Y %H:%M:%S"),
            level,
            module_path.italic(),
            record.args()
        )?;

        w.write_all(&buffer)?;

        Ok(())
    }
}
