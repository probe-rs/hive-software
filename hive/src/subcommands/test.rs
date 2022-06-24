//! The test subcommand
use anyhow::{anyhow, bail, Result};
use cargo_toml::Manifest;
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

    let project_path = subcommand_args.path.clone().unwrap_or_else(|| "./".into());
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

    super::show_progress(&cli_args, |progress| {
        progress.set_message("collecting files...");
        let tar_buffer = vec![];
        let mut tar = Builder::new(tar_buffer);

        for result in Walk::new(project_path) {
            match result {
                Ok(entry) => {
                    tar.append_path(entry.path())?;
                    progress.inc(1);
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
            config.testserver.unwrap().as_wss_url(),
            ws_ticket
        );
        let mut ws = get_ws_client(cli_args.accept_invalid_certs, ws_url)?;

        loop {
            match ws.read_message()? {
                Message::Binary(bytes) => {
                    todo!("parse json")
                }
                Message::Close(_) => {
                    bail!("Websocket connection closed by testserver before receiving results.")
                }
                _ => (),
            }
        }

        Ok(())
    })?;

    /*let test_results: TestResults = response.json().map_err(|err| anyhow!("Failed to deserialize received testserver response. This Hive CLI version might not be compatible with the connected testserver.\n\nCaused by: {}", err))?;

    match test_results.status {
        TestRunStatus::Ok => {
            println!("{:#?}", test_results.results.unwrap());
            todo!("Implement test result formatting");
        }
        TestRunStatus::Error => {
            let test_run_error = test_results.error.unwrap();

            match test_run_error.source {
                Some(source) => bail!(
                    "Testserver failed to run tests: {}\n\nCaused by: {}",
                    test_run_error.err,
                    source
                ),
                None => bail!("Testserver failed to run tests: {}", test_run_error.err),
            }
        }
    }*/

    Ok(())
}
