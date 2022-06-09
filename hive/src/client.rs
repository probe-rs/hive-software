//! The http client used to make request to the testserver
use reqwest::blocking::Client;

/// Get the http client to issue requests.
///
/// # Panics
/// If the client fails to build due to being unable to init the TLS backend or the system configuration cannot be loaded.
pub(crate) fn get_client(accept_invalid_certs: bool) -> Client {
    if accept_invalid_certs {
        log::warn!(
            "Option 'accept_invalid_certs' is set to true. This might be dangerous, use with care."
        )
    }

    Client::builder()
        .danger_accept_invalid_certs(accept_invalid_certs)
        .build()
        .unwrap_or_else(|err| panic!("Failed to build http client: {}", err))
}
