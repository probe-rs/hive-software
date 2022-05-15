//! IPC request handlers
use std::sync::Arc;

use axum::Extension;
use colored::Colorize;
use comm_types::cbor::{Cbor, ServerParseError};
use comm_types::ipc::{HiveProbeData, HiveTargetData, IpcMessage};
use comm_types::results::TestResult;

use crate::database::{keys, CborDb, HiveDb};

pub(crate) async fn probe_handler(Extension(db): Extension<Arc<HiveDb>>) -> Cbor<IpcMessage> {
    log::debug!("Received an IPC request on probe handler");

    let data: HiveProbeData = db
        .config_tree
        .c_get(keys::config::ASSIGNED_PROBES)
        .unwrap()
        .expect("Probe data was not found in the database. The data should be initialized before the runner is started.");

    Cbor(IpcMessage::ProbeInitData(data))
}

pub(crate) async fn target_handler(Extension(db): Extension<Arc<HiveDb>>) -> Cbor<IpcMessage> {
    log::info!("Received an IPC request on target handler");

    let data: HiveTargetData = db
        .config_tree
        .c_get(keys::config::ASSIGNED_TARGETS)
        .unwrap()
        .expect("Target data was not found in the database. The data should be initialized before the runner is started.");

    Cbor(IpcMessage::TargetInitData(data))
}

pub(crate) async fn runner_log_handler(Cbor(message): Cbor<IpcMessage>) -> Cbor<IpcMessage> {
    log::info!("Received {:#?} on runner log handler.", message);
    todo!();

    Cbor(IpcMessage::Empty)
}

pub(crate) async fn test_result_handler(
    Cbor(message): Cbor<IpcMessage>,
) -> Result<Cbor<IpcMessage>, ServerParseError> {
    if let IpcMessage::TestResults(mut results) = message {
        /*log::info!("Received test results on result handler: {:#?}", results);
        todo!();*/

        // dummy implementation which pretty prints the test results
        let mut ordered: Vec<Vec<TestResult>> = vec![];

        while !results.is_empty() {
            let mut new_group = true;

            for group in ordered.iter_mut() {
                if group[0].probe_name == results[0].probe_name
                    && group[0].probe_sn == results[0].probe_sn
                    && group[0].target_name == results[0].target_name
                {
                    group.push(results.remove(0));
                    new_group = false;
                    break;
                }
            }

            if new_group {
                ordered.push(vec![results.remove(0)]);
            }
        }

        println!("{}", "Test results:".bold());
        for group in ordered {
            println!(
                "{} {} {} ({})",
                group[0].target_name,
                "<-->".blue().bold(),
                group[0].probe_name,
                group[0].probe_sn
            );

            for result in group.iter() {
                let result_text = match &result.status {
                    comm_types::results::TestStatus::PASSED => {
                        format!("{}", "passed".green().bold())
                    }
                    comm_types::results::TestStatus::FAILED(cause) => {
                        format!("{}\n\n\tCaused by: {}\n", "failed".red().bold(), cause)
                    }
                    comm_types::results::TestStatus::SKIPPED(cause) => {
                        println!("\t all tests -> {} ({})", "skipped".yellow().bold(), cause);
                        break;
                    }
                };

                let should_panic_text = match result.should_panic {
                    true => "(Should Panic)",
                    false => "",
                };

                println!(
                    "\t{} {} -> {}",
                    result.test_name,
                    should_panic_text.italic(),
                    result_text
                );
            }
        }
    } else {
        return Err(ServerParseError::InvalidCbor);
    }

    Ok(Cbor(IpcMessage::Empty))
}
