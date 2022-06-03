use std::env;
use std::path::PathBuf;
use std::process;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use log::Level;

mod config;
mod subcommands;
mod validate;

use validate::ValidHost;

const HIVE_LOG_ENV: &str = "HIVE_LOG";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct CliArgs {
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Test your probe-rs code
    Test(Test),
    /// Connect to a Hive-Testserver
    Connect(Connect),
}

#[derive(Debug, Args)]
struct Connect {
    /// The testserver address (either IP or domain)
    #[clap(parse(try_from_str=validate::ip_or_url))]
    address: ValidHost,
}

#[derive(Debug, Args)]
struct Test {
    /// Path to the probe-rs project root (equals to current directory if omitted)
    path: Option<PathBuf>,
    #[clap(short, long)]
    target: Option<Vec<String>>,
}

fn main() {
    let args = CliArgs::parse();
    set_log_level(&args.verbose.log_level());
    pretty_env_logger::init_custom_env(HIVE_LOG_ENV);

    let res = app(args);

    if let Err(err) = res {
        if err.source().is_some() {
            log::error!("{}\n\nCaused by:\n{}", err, err.source().unwrap());
        } else {
            log::error!("{}", err);
        }

        process::exit(1);
    }
}

fn app(args: CliArgs) -> Result<()> {
    let config = config::load_config()?;

    match args.command {
        Commands::Test(args) => subcommands::test::test(args, config),
        Commands::Connect(args) => subcommands::connect::connect(args, config),
    }
}

/// set the log level of the cli
fn set_log_level(verbosity: &Option<log::Level>) {
    match verbosity {
        Some(level) => env::set_var(HIVE_LOG_ENV, level.as_str()),
        None => env::set_var(HIVE_LOG_ENV, Level::Error.as_str()),
    }
}
