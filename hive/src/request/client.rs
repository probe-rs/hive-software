//! The http and ws clients used to send requests to the testserver
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};

use anyhow::{bail, Result};
use comm_types::token::API_TOKEN_HEADER;
use http::HeaderValue;
use native_tls::TlsConnector;
use reqwest::blocking::Client;
use tungstenite::client::IntoClientRequest;
use tungstenite::protocol::WebSocketConfig;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Connector, WebSocket};

use crate::config::HiveConfig;
use crate::models::Host;
use crate::request::get_api_token_or_prompt;
use crate::CliArgs;

/// Get the http client to issue requests.
///
/// # Panics
/// If the client fails to build due to being unable to init the TLS backend or the system configuration cannot be loaded.
pub fn get_http_client(accept_invalid_certs: bool) -> Client {
    if accept_invalid_certs {
        log::warn!(
            "Option 'accept_invalid_certs' is set to true. This might be dangerous, use with care."
        )
    }

    // We need to create a custom tls to allow requests done by IP address instead of url by disabling TLS server name identification (SNI). Reqwest or its dependencies sends the SNI, even if the host address is IP,
    // which is against the spec (https://datatracker.ietf.org/doc/html/rfc6066#section-3) and thus getting rejected by the Hive server Rustls layer.
    // SNI verification is currently only omitted if accept-invalid-certs is true
    // Also see https://github.com/seanmonstar/reqwest/issues/1328
    let tls = TlsConnector::builder()
        .danger_accept_invalid_certs(accept_invalid_certs)
        .use_sni(!accept_invalid_certs)
        .build()
        .unwrap();

    Client::builder()
        .use_preconfigured_tls(tls)
        .build()
        .unwrap_or_else(|err| panic!("Failed to build http client: {}", err))
}

/// Open a websocket on the provided url
pub fn get_ws_client(
    accept_invalid_certs: bool,
    host: &Host,
    url: &str,
    config: &HiveConfig,
    cli_args: &CliArgs,
) -> Result<WebSocket<MaybeTlsStream<TcpStream>>> {
    let mut tcp_stream = None;

    let api_token = get_api_token_or_prompt(config, cli_args)?;

    for addr in host.as_secure_parts().to_socket_addrs().unwrap() {
        if let Ok(stream) = TcpStream::connect::<SocketAddr>(addr) {
            tcp_stream = Some(stream);
            break;
        }
    }

    if tcp_stream.is_none() {
        bail!("Failed to connect tcp socket to testserver");
    }

    // We need to create a custom tls to allow requests done by IP address instead of url by disabling TLS server name identification (SNI). Reqwest or its dependencies sends the SNI, even if the host address is IP,
    // which is against the spec (https://datatracker.ietf.org/doc/html/rfc6066#section-3) and thus getting rejected by the Hive server Rustls layer.
    // SNI verification is currently only omitted if accept-invalid-certs is true
    // Also see https://github.com/sfackler/rust-native-tls/issues/215
    let tls_connector = TlsConnector::builder()
        .danger_accept_invalid_certs(accept_invalid_certs)
        .use_sni(!accept_invalid_certs)
        .build()
        .unwrap();

    let mut client_request = IntoClientRequest::into_client_request(url).expect("Failed to create a tungstenite client request out of the provided WS URL. This is a bug, please open an issue.");
    client_request
        .headers_mut()
        .append(API_TOKEN_HEADER, HeaderValue::from_str(&api_token).expect("Failed to put API token value into header. This should never happen as users should not be allowed to provide non ASCII characters as token. Please open an issue."));

    let (ws, res) = tungstenite::client_tls_with_config(
        client_request,
        tcp_stream.unwrap(),
        Some(WebSocketConfig::default()),
        Some(Connector::NativeTls(tls_connector)),
    )?;

    log::debug!("Received ws response: {:#?}", res);

    Ok(ws)
}
