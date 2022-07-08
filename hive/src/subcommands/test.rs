//! The test subcommand
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Result};
use cargo_toml::Manifest;
use colored::Colorize;
use comm_types::test::{TaskRunnerMessage, TestResult, TestResults, TestRunStatus, TestStatus};
use ignore::Walk;
use prettytable::{cell, format, row, Table};
use reqwest::blocking::multipart::{Form, Part};
use tar::Builder;
use tungstenite::Message;

use crate::client::{get_http_client, get_ws_client};
use crate::config::HiveConfig;
use crate::{CliArgs, Commands};

pub(crate) fn test(cli_args: CliArgs, mut config: HiveConfig) -> Result<()> {
    let subcommand_args = if let Commands::Test(args) = &cli_args.command {
        args
    } else {
        panic!("You may only call this function if the actual subcommand matches")
    };

    super::show_testserver_prompt_if_none(&mut config, &cli_args)?;

    let project_path =
        fs::canonicalize(subcommand_args.path.clone().unwrap_or_else(|| "./".into()))?;
    let cargofile_path = project_path.join("Cargo.toml");

    if !cargofile_path.is_file() {
        bail!("Could not find Cargo.toml file in provided directory");
    }

    let manifest = Manifest::from_path(cargofile_path)?;

    if let Some(workspace) = manifest.workspace {
        if !workspace.members.contains(&"probe-rs".to_owned()) {
            bail!("Could not find probe-rs crate in workspace definition");
        }
    } else {
        bail!("This cargo project is not a workspace");
    }

    let test_results = super::show_progress(&cli_args, |progress| {
        progress.set_message("collecting files...");
        let tar_buffer = vec![];
        let mut tar = Builder::new(tar_buffer);

        for result in Walk::new(&project_path) {
            match result {
                Ok(entry) => {
                    let relative_path = pathdiff::diff_paths(entry.path(), &project_path).unwrap();

                    if relative_path == Path::new("") {
                        // Ignore basepath as it does not add anything to the archive
                        continue;
                    }

                    tar.append_path_with_name(entry.path(), relative_path)?;
                }
                Err(err) => bail!("Failed to read project files: {}", err),
            }
        }

        let tarfile = tar.into_inner()?;

        progress.set_message("Uploading files...");
        let client = get_http_client(cli_args.accept_invalid_certs);

        let fileupload = Part::bytes(tarfile)
            .file_name("project.tar")
            .mime_str("application/octet-stream")
            .unwrap();

        let form = Form::new().part("project", fileupload);

        let response = client
            .post(format!(
                "{}/test/run",
                config.testserver.as_ref().unwrap().as_https_url()
            ))
            .multipart(form)
            .send()
            .map_err(|err| {
                anyhow!(
                    "Failed to send test request to testserver.\nCaused by: {}",
                    err
                )
            })?;

        progress.set_message("Connecting websocket...");
        let ws_ticket: String = response.json().map_err(|err| anyhow!("Failed to deserialize received testserver response. This Hive CLI version might not be compatible with the connected testserver.\n\nCaused by: {}", err))?;

        let ws_url = format!(
            "{}/test/socket?auth={}",
            config.testserver.as_ref().unwrap().as_wss_url(),
            ws_ticket
        );
        let mut ws = get_ws_client(
            cli_args.accept_invalid_certs,
            config.testserver.as_ref().unwrap(),
            ws_url,
        )?;

        let test_results;

        loop {
            match ws.read_message()? {
                Message::Binary(bytes) => {
                    let message: TaskRunnerMessage = serde_json::from_slice(&bytes).expect("Failed to parse json from testserver websocket. Does the client version match the testserver version?");
                    match message {
                        TaskRunnerMessage::Status(status) => {
                            progress.set_message(format!("{}...", status))
                        }
                        TaskRunnerMessage::Error(err) => bail!("Test run failed: {}", err),
                        TaskRunnerMessage::Results(results) => {
                            test_results = results;
                            break;
                        }
                    }
                }
                Message::Close(_) => {
                    bail!("Websocket connection closed by testserver before receiving results.")
                }
                _ => (),
            }
        }

        Ok(test_results)
    })?;

    print_test_results(test_results);

    Ok(())
}

/// Pretty print the provided [`TestResults`]
fn print_test_results(results: TestResults) {
    let table_format = format::FormatBuilder::new()
        .borders(' ')
        .column_separator(' ')
        .build();

    let mut table = Table::new();
    table.set_format(table_format);

    match results.status {
        TestRunStatus::Ok => {
            let results = results.results.unwrap();

            let mut target_map: HashMap<String, HashMap<String, Vec<TestResult>>> = HashMap::new();

            for result in results.into_iter() {
                // Use both S/N and name as key in case the same probe model is used multiple times (for whatever strange reason)
                let probe_key = format!("{} (S/N: {})", result.probe_name, result.probe_sn);

                if !target_map.contains_key(&result.target_name) {
                    target_map.insert(result.target_name.clone(), HashMap::new());
                }

                let probe_map = target_map.get_mut(&result.target_name).unwrap();
                if !probe_map.contains_key(&probe_key) {
                    probe_map.insert(probe_key, vec![result]);
                    continue;
                }

                probe_map.get_mut(&probe_key).unwrap().push(result);
            }

            // TODO preliminary implementation, it needs to be determined which sortings make sense (Sort by name, test order, ...)
            table.set_titles(row![buH3c-> "Hive Test Results:"]);

            for (target_name, probe_map) in target_map.into_iter() {
                table.add_row(row![bH3-> target_name]);

                for (probe_name, results) in probe_map.into_iter() {
                    table.add_row(row![FmH3-> format!(
                        "  > {}",
                        probe_name
                    )]);

                    let mut skip_reason = None;
                    if results.iter().all(|result| {
                        if let TestStatus::Skipped(ref reason) = result.status {
                            if skip_reason.is_none() {
                                skip_reason = Some(reason);
                                return true;
                            } else {
                                return skip_reason == Some(reason);
                            }
                        }
                        false
                    }) {
                        if let Some(reason) = skip_reason {
                            table.add_row(row!["", b->"all tests", bFyr-> "skipped"]);
                            table.add_row(row!["", iH2->pad_string(2, reason)]);
                        }
                        continue;
                    }

                    for result in results.into_iter() {
                        let test_fn_name = match result.should_panic {
                            true => format!("{} {}", result.test_name, "(should panic)".italic()),
                            false => result.test_name,
                        };

                        match result.status {
                            TestStatus::Passed => {
                                table.add_row(row!["", test_fn_name, bFgr-> "passed"]);
                            }
                            TestStatus::Failed(reason) => {
                                table.add_row(row!["", test_fn_name, bFrr-> "failed"]);
                                table.add_row(row!["", iH2->pad_string(2, &reason)]);
                            }
                            TestStatus::Skipped(reason) => {
                                table.add_row(row!["", test_fn_name, bFyr-> "skipped"]);
                                table.add_row(row!["", iH2->pad_string(2, &reason)]);
                            }
                        }
                    }
                }

                table.add_empty_row();
                table.add_empty_row();
            }
        }
        TestRunStatus::Error => {
            let err = results.error.unwrap();

            table.set_titles(row![buH2c-> "Hive Test Results:"]);

            table.add_row(row![bFr->"Error:", err.err]);

            if let Some(source) = err.source {
                table.add_empty_row();
                table.add_row(row![i->"Caused by:", source]);
            }
        }
    }

    println!("");
    table.printstd();
}

/// Add a whitespace padding on multiline strings
fn pad_string(padding: usize, string: &str) -> String {
    let pad = vec![' ' as u8; padding];
    let pad_string = String::from_utf8(pad).unwrap();
    let mut new_string = String::from(&pad_string);

    let last_line_idx = string.lines().count() - 1;

    for (idx, line) in string.lines().enumerate() {
        new_string.push_str(line);
        if idx != last_line_idx {
            new_string.push('\n');
            new_string.push_str(&pad_string);
        }
    }

    new_string
}
