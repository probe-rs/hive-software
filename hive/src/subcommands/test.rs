//! The test subcommand
use std::fs;
use std::path::Path;

use anyhow::{anyhow, bail, Result};
use cargo_toml::Manifest;
use colored::Colorize;
use comm_types::test::{TaskRunnerMessage, TestResults, TestRunStatus};
use ignore::Walk;
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
    println!("{}\n", "Hive Test Results:".bold());

    match results.status {
        TestRunStatus::Ok => todo!("pretty print ok"),
        TestRunStatus::Error => {
            let err = results.error.unwrap();

            println!("\t{} {}", "Error: ".bold().red(), err.err);
            if let Some(source) = err.source {
                println!("\n\n{} {}", "Caused by:".bold(), source);
            }
        }
    }
}
