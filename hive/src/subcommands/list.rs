//! The test subcommand
use anyhow::Result;

use crate::config::HiveConfig;
use crate::CliArgs;

pub(crate) fn list(cli_args: CliArgs, mut config: HiveConfig) -> Result<()> {
    super::show_testserver_prompt_if_none(&mut config, &cli_args)?;

    let capabilities = super::get_testserver_capabilities(
        &config.testserver.unwrap(),
        cli_args.accept_invalid_certs,
    )?;

    println!(
        "Available Probes: {}\n\nAvailable Targets: {}",
        capabilities.available_probes.join(", "),
        capabilities.available_targets.join(", ")
    );

    Ok(())
}
