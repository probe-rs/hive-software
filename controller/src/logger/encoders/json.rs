//! JSON encoder to store the logs as formatted plaintext for file storage. JSON is used to store the log entries as it is human-readable and allows easy processing in software.
use chrono::Local;
use log4rs::encode::Encode;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub(in crate::logger) struct JsonEncoder;

impl JsonEncoder {
    pub fn new() -> Self {
        Self
    }
}

impl Encode for JsonEncoder {
    fn encode(
        &self,
        w: &mut dyn log4rs::encode::Write,
        record: &log::Record,
    ) -> anyhow::Result<()> {
        let time = Local::now();

        let module_path = record.module_path().unwrap_or("unknown");

        let mut json = serde_json::to_vec(&LogEntry {
            timestamp: time.format("%d.%m.%Y %H:%M:%S").to_string(),
            level: record.level().to_string(),
            module: module_path.to_owned(),
            message: record.args().to_string(),
        })?;

        // Add newline
        json.push(0xA);

        w.write(&json)?;

        Ok(())
    }
}

/// A single Hive log entry
#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub module: String,
    pub message: String,
}
