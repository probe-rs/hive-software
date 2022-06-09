//! The test subcommand
use anyhow::{anyhow, bail, Result};
use cargo_toml::Manifest;
use dialoguer::{theme::ColorfulTheme, Input};
use ignore::Walk;
use indicatif::ProgressBar;
use reqwest::blocking::{
    multipart::{Form, Part},
    Client,
};
use tar::Builder;

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

    let progress = ProgressBar::new_spinner();

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

    let client = Client::new();

    let fileupload = Part::bytes(tarfile)
        .file_name("project.tar")
        .mime_str("application/octet-stream")
        .unwrap();

    let form = Form::new().part("project", fileupload);

    client
        .post(format!(
            "{}/test/run",
            config.testserver.unwrap().as_https_url()
        ))
        .multipart(form)
        .send()
        .map_err(|err| {
            anyhow!(
                "Failed to send test request to testserver.\nCaused by: {}",
                err
            )
        })?;

    todo!(
        "Implement test result formatting, add test option to form, add loader or upload progress"
    );

    Ok(())
}
