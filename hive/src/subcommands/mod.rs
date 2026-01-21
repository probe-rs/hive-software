//! Subcommand logic
use std::time::Duration;

use anyhow::{Result, anyhow, bail};
use comm_types::hardware::Capabilities;
use dialoguer::Input;
use dialoguer::theme::ColorfulTheme;
use indicatif::{ProgressBar, ProgressStyle};

use crate::config::HiveConfig;
use crate::models::Host;
use crate::request::{client, send_request};
use crate::{CliArgs, validate};

pub mod connect;
pub mod list;
pub mod test;

/// Try to get the [`Capabilities`] of the provided testserver.
///
/// # Error
/// Returns an error in case the connection cannot be established or fails. Or the response contains unexpected data.
fn get_testserver_capabilities(
    accept_invalid_certs: bool,
    hive_config: &HiveConfig,
    cli_args: &CliArgs,
) -> Result<Capabilities> {
    let client = client::get_http_client(accept_invalid_certs);

    let response = send_request(
        client.get(format!(
            "{}/test/capabilities",
            hive_config
                .testserver
                .as_ref()
                .expect("Testserver not defined in config. This is a bug, please file an issue.")
                .as_https_url()
        )),
        hive_config,
        cli_args,
    )?;

    if !response.status().is_success() {
        bail!(
            "Recieved error status from server: {} {}",
            response.status(),
            String::from_utf8_lossy(&response.bytes()?)
        )
    }

    response.json()
        .map_err(|err| anyhow!("Testserver response contained unexpected data. Is it up to date and really a testerver?\n Caused by: {}", err))
}

/// Prompts the user to enter the testserver address, if the testserver address is not defined in the config (aka [`None`]).
///
/// This function checks the entered address and verifies the connection to the testserver. This function fails in case the no-human flag is set.
///
/// In case the function succeeds the testserver host value in the config is guranteed to be [`Some`]
fn show_testserver_prompt_if_none(config: &mut HiveConfig, cli_args: &CliArgs) -> Result<()> {
    if config.testserver.is_none() {
        if cli_args.no_human {
            bail!(
                "No testserver address found in config. Add a testserver by using the connect subcommand"
            );
        }

        let testserver_address_input = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Testserver address")
            .validate_with(|input: &String| -> Result<(), &str> {
                match validate::ip_or_url(input) {
                    Ok(_) => Ok(()),
                    Err(_) => Err("Invalid testserver address"),
                }
            })
            .interact_text()
            .unwrap();

        let host: Host = validate::ip_or_url(&testserver_address_input)
            .unwrap()
            .into();

        config.testserver = Some(host);

        // We check if the provided host sends a response and is a testserver
        get_testserver_capabilities(cli_args.accept_invalid_certs, config, cli_args)?;

        config.save_config()?;
    }

    Ok(())
}

/// Sets up an indefinite progress spinner for the duration of the provided closure. The closure receives a progress handle which makes it possible to alter the progress state inside the closure.
/// On completion of the provided closure the progress is cleaned up from the terminal.
///
/// # No human
/// In case the no human flag is supplied the provided progress instance is hidden and any manipulation to it does not have any visual effect.
fn show_progress<F, T>(cli_args: &CliArgs, f: F) -> Result<T>
where
    F: FnOnce(&ProgressBar) -> Result<T>,
{
    if cli_args.no_human {
        return f(&ProgressBar::hidden());
    }

    let progress = ProgressBar::new_spinner();

    progress.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.blue} {msg}")
            .expect("Spinner template invalid. This is a bug.")
            .tick_strings(&["⣾", "⣽", "⣻", "⢿", "⡿", "⣟", "⣯", "⣷"]),
    );

    progress.enable_steady_tick(Duration::from_millis(120));

    let result = f(&progress);

    progress.finish_and_clear();

    result
}
