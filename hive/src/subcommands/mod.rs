use anyhow::{anyhow, bail, Result};
use comm_types::hardware::Capabilities;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;

use crate::config::HiveConfig;
use crate::models::Host;
use crate::{client, validate, CliArgs};

pub(crate) mod connect;
pub(crate) mod list;
pub(crate) mod test;

/// Try to get the [`Capabilities`] of the provided testserver.
///
/// # Error
/// Returns an error in case the connection cannot be established or fails. Or the response contains unexpected data.
fn get_testserver_capabilities(address: &Host, accept_invalid_certs: bool) -> Result<Capabilities> {
    let client = client::get_client(accept_invalid_certs);

    client.get(format!("{}/test/capabilities", address.as_https_url())).send()
        .map_err(|err| {
            anyhow!(
            "Failed to connect to provided testserver. Is the address '{}' correct?\nCaused by: {}",
            address.as_https_url(),
            err
        )
        })?
        .json()
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
            bail!("No testserver address found in config. Add a testserver by using the connect subcommand");
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

        // We check if the provided host sends a response and is a testserver
        get_testserver_capabilities(&host, cli_args.accept_invalid_certs)?;

        config.testserver = Some(host);

        config.save_config()?;
    }

    Ok(())
}
