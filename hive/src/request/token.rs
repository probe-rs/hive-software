//! Manages the API tokens used to access the endpoints of the testserver
use anyhow::{bail, Result};
use dialoguer::{theme::ColorfulTheme, Input};
use keyring::{Entry, Error as KeyringError};

use crate::{config::HiveConfig, CliArgs};

/// Prompts the user to enter the API token to access the test endpoints if no token has been found in the keyring for the currently connected testserver
///
/// This function fails in case the no-human flag is set.
///
/// Returns the current API token retrieved in the keyring or by prompt for the current testserver
pub(super) fn get_api_token_or_prompt(config: &HiveConfig, cli_args: &CliArgs) -> Result<String> {
    let keychain_entry = get_entry(config)?;

    if let Some(token) = cli_args.token.as_ref() {
        // User provided a token as CLI arg
        return Ok(token.to_owned());
    }

    let mut token = match keychain_entry.get_password() {
        Ok(token) => Some(token),
        Err(err) => match err {
            keyring::Error::NoEntry => None,
            err => return Err(err.into()),
        },
    };

    if token.is_none() {
        // Prompt user for token
        if cli_args.no_human {
            bail!("No API token provided for accessing the test API. Please provide a token using the --token flag.")
        }

        let token_input = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Please enter your Hive API token\n(The token will be securely stored for further use)")
            .validate_with(|input: &String| -> Result<(), &str> {
                if input.chars().all(|c| c.is_ascii_alphanumeric()) {
                    Ok(())
                } else {
                    Err("API token may only contain alphanumeric ASCII characters")
                }
            })
            .interact_text()
            .unwrap();

        save_api_token(config, &token_input)?;

        token = Some(token_input);
    }

    Ok(token.unwrap())
}

/// Deletes the API token of the currently selected testserver from the keyring if it exists
pub fn delete_api_token(config: &HiveConfig) -> Result<()> {
    let keyring_entry = get_entry(config)?;

    if let Err(err) = keyring_entry.delete_password() {
        match err {
            KeyringError::NoEntry => (), // Entry does not exist, ignore
            err => Err(err)?,
        }
    }

    Ok(())
}

/// Saves the API token of the currently selected testserver in the keyring
fn save_api_token(config: &HiveConfig, token: &str) -> Result<()> {
    let keyring_entry = get_entry(config)?;

    keyring_entry.set_password(token)?;

    Ok(())
}

/// Returns the keychain entry for the currently active testserver configuration
fn get_entry(config: &HiveConfig) -> Result<Entry> {
    let entry = Entry::new("Hive", &config.testserver.as_ref().expect("Called get_entry before the testserver was defined in config. This is a bug, please file an issue.").as_https_url())?;

    Ok(entry)
}
