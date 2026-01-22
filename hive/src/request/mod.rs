//! Helper methods to handle requests to the Hive testserver and manage authorization

use anyhow::{Result, anyhow, bail};
use comm_types::token::API_TOKEN_HEADER;
use reqwest::{
    StatusCode,
    blocking::{RequestBuilder, Response},
};

use crate::{
    CliArgs,
    config::HiveConfig,
    request::token::{delete_api_token, get_api_token_or_prompt},
};

pub mod client;
pub mod token;

trait HiveRequestBuilder {
    /// Adds the provided API token to the request
    fn with_token(self, token: &str) -> RequestBuilder;
}

impl HiveRequestBuilder for RequestBuilder {
    fn with_token(self, token: &str) -> RequestBuilder {
        self.header(API_TOKEN_HEADER, token)
    }
}

/// Sends a HTTP request to the testserver.
///
/// # Response status
/// This function checks if the received response status is 200 (OK)
/// If this is not the case an error is returned. See the API token section below for exceptions.
///
/// # API token
/// The Hive testserver requires an API token for authorization.
/// This function automatically appends the token to the provided request builder.
///
/// ## Automatic retry
/// In case the request fails due to the API token being invalid or missing in the keyring, the user is prompted to enter a new API token.
/// Token addition/deletion from the keyring is automatically managed by this function as well. If the retry fails again this function returns an error.
///
/// In case the user has supplied the no-human flag the function does not attempt to retry and returns an error instead.
pub fn send_request(
    request: RequestBuilder,
    config: &HiveConfig,
    cli_args: &CliArgs,
) -> Result<Response> {
    let api_token = get_api_token_or_prompt(config, cli_args)?;

    let retryable_request = request.try_clone();

    let response = send_with_token(request, &api_token)?;

    if !response.status().is_success() {
        if response.status() == StatusCode::UNAUTHORIZED {
            // API token invalid or expired

            println!("The API token for the current testserver is invalid or expired.");
            delete_api_token(config)?;

            let api_token = get_api_token_or_prompt(config, cli_args)?;

            if let Some(request) = retryable_request {
                let response = send_with_token(request, &api_token)?;

                if response.status().is_success() {
                    return Ok(response);
                };
            }
        }

        // Bad status or retry failed
        bail!(
            "Recieved error status from server: {} {}",
            response.status(),
            String::from_utf8_lossy(&response.bytes()?)
        )
    }

    Ok(response)
}

fn send_with_token(request: RequestBuilder, token: &str) -> Result<Response> {
    let request = request.with_token(token);

    request.send().map_err(|err| {
        anyhow!(
            "Failed to send test request to testserver. Status: {:?}\nCaused by: {}",
            err.status(),
            err
        )
    })
}
