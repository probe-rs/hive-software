use anyhow::{anyhow, Result};
use comm_types::hardware::Capabilities;

use crate::client;
use crate::models::Host;

pub(crate) mod connect;
pub(crate) mod test;

/// Try to get the [`Capabilities`] of the provided testserver.
///
/// # Error
/// Returns an error in case the connection cannot be established or fails. Or the response contains unexpected data.
fn get_testserver_capabilities(address: &Host, accept_invalid_certs: bool) -> Result<Capabilities> {
    let client = client::get_client(accept_invalid_certs);

    client.get(format!("{}/test/capabilities", address.as_https_url())).send()
        .map_err(|err| {
            anyhow!(
            "Failed to connect to provided testserver. Is the address '{}' correct?\nCaused by: {}",
            address.as_https_url(),
            err
        )
        })?
        .json()
        .map_err(|err| anyhow!("Testserver response contained unexpected data. Is it up to date and really a testerver?\n Caused by: {}", err))
}
