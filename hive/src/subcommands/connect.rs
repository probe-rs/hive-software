use anyhow::Result;

use crate::config::{HiveConfig, Host};
use crate::Connect;

/// Connect subcommand handler
pub(crate) fn connect(args: Connect, config: HiveConfig) -> Result<()> {
    let address: Host = args.address.into();

    Ok(())
}
