//! The test subcommand
use anyhow::Result;
use colored::Colorize;

use crate::config::HiveConfig;
use crate::CliArgs;

pub fn list(cli_args: CliArgs, mut config: HiveConfig) -> Result<()> {
    super::show_testserver_prompt_if_none(&mut config, &cli_args)?;

    let capabilities =
        super::get_testserver_capabilities(cli_args.accept_invalid_certs, &config, &cli_args)?;

    println!(
        "{}\n {}\n\n{}\n {}",
        "Available Probes:".bold().blue(),
        capabilities.available_probes.join(", "),
        "Available Targets:".bold().blue(),
        capabilities.available_targets.join(", ")
    );

    Ok(())
}
