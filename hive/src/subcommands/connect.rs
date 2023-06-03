//! The connect subcommand
use anyhow::{bail, Result};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;

use crate::config::HiveConfig;
use crate::models::Host;
use crate::validate;
use crate::{CliArgs, Commands};

/// Connect subcommand handler
pub fn connect(cli_args: CliArgs, mut config: HiveConfig) -> Result<()> {
    let subcommand_args = if let Commands::Connect(args) = cli_args.command {
        args
    } else {
        panic!("You may only call this function if the actual subcommand matches")
    };

    let address: Host = match subcommand_args.address {
        Some(address) => address.into(),
        None => {
            if cli_args.no_human {
                bail!("No testserver address specified as argument");
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

            validate::ip_or_url(&testserver_address_input)
                .unwrap()
                .into()
        }
    };

    // We check if the provided host sends a response and is a testserver
    super::get_testserver_capabilities(&address, cli_args.accept_invalid_certs)?;

    config.testserver = Some(address);

    config.save_config()?;

    println!("Successfully connected to testserver.\nYou can now start testing your project by using 'hive test'");

    Ok(())
}
