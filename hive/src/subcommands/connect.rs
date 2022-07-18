//! The connect subcommand
use anyhow::Result;

use crate::config::HiveConfig;
use crate::models::Host;
use crate::{CliArgs, Commands};

/// Connect subcommand handler
pub fn connect(cli_args: CliArgs, mut config: HiveConfig) -> Result<()> {
    let subcommand_args = if let Commands::Connect(args) = cli_args.command {
        args
    } else {
        panic!("You may only call this function if the actual subcommand matches")
    };

    let address: Host = subcommand_args.address.into();

    // We check if the provided host sends a response and is a testserver
    super::get_testserver_capabilities(&address, cli_args.accept_invalid_certs)?;

    config.testserver = Some(address);

    config.save_config()?;

    println!("Successfully connected to testserver.\nYou can now start testing your project by using 'hive test'");

    Ok(())
}
