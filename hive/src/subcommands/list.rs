//! The test subcommand
use anyhow::Result;
use colored::Colorize;

use crate::config::HiveConfig;
use crate::CliArgs;

pub fn list(cli_args: CliArgs, mut config: HiveConfig) -> Result<()> {
    super::show_testserver_prompt_if_none(&mut config, &cli_args)?;

    let capabilities =
        super::get_testserver_capabilities(cli_args.accept_invalid_certs, &config, &cli_args)?;

    let available_probes = if capabilities.available_probes.is_empty() {
        "No available probes found on this testserver. It is not possible to run any tests on it.\n\n Please contact the Hive testserver administrator.".yellow().to_string()
    } else {
        capabilities.available_probes.join(", ")
    };

    let available_targets = if capabilities.available_targets.is_empty() {
        "No available targets found on this testserver. It is not possible to run any tests on it.\n\n Please contact the Hive testserver administrator.".yellow().to_string()
    } else {
        capabilities.available_targets.join(", ")
    };

    println!(
        "{}\n {}\n\n{}\n {}",
        "Available Probes:".bold().blue(),
        available_probes,
        "Available Targets:".bold().blue(),
        available_targets
    );

    Ok(())
}
