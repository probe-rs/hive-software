//! The Hive CLI for client side usage of the Hive test functionalities.
//!
//! The CLI is built using clap and can be used by human and non human users. If the flag --no-human is passed the binary can be used inside other scripts etc as in this case any graphical output like progress indicators, etc. are disabled.
//! Input promts are disabled as well and lead to errors instead which point at what piece of information was missing.
//!
//! The CLI application is capable of storing cofiguration data persistently on the OS. Thus any configuration is usually a one-off task which does not need to be repeated frequently by the user.
//!
//! # External dependencies
//! The CLI application requires to have access to the [Cross](https://github.com/cross-rs/cross) binary in order to be able to cross compile the runner for the aarch64 architecture of the Hive testrack.
//!
//! [Libgit2](https://libgit2.org/) is also required to allow the application to manage the runner source code.
use std::env;
use std::path::PathBuf;
use std::process;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use log::Level;

mod client;
mod config;
mod models;
mod subcommands;
mod validate;
mod workspace;

use validate::ValidHost;

const HIVE_LOG_ENV: &str = "HIVE_LOG";

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct CliArgs {
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
    #[clap(subcommand)]
    command: Commands,
    /// Deactivates all user input prompts and progress indicators
    #[clap(short, long)]
    no_human: bool,
    /// Accept invalid tls certificates
    #[clap(short = 'i', long)]
    accept_invalid_certs: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Test your probe-rs code
    Test(Test),
    /// Connect to a Hive-Testserver
    Connect(Connect),
    /// List available targets and probes on connected testserver
    List,
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

    process::exit(0);
}

fn app(cli_args: CliArgs) -> Result<()> {
    let config = config::HiveConfig::load_config()?;

    match cli_args.command {
        Commands::Test(_) => subcommands::test::test(cli_args, config),
        Commands::Connect(_) => subcommands::connect::connect(cli_args, config),
        Commands::List => subcommands::list::list(cli_args, config),
    }
}

/// set the log level of the cli
fn set_log_level(verbosity: &Option<log::Level>) {
    match verbosity {
        Some(level) => env::set_var(HIVE_LOG_ENV, level.as_str()),
        None => env::set_var(HIVE_LOG_ENV, Level::Error.as_str()),
    }
}
