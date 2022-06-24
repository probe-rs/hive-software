//! The http client used to make request to the testserver
use std::net::{SocketAddr, TcpStream};

use anyhow::Result;
use native_tls::TlsConnector;
use reqwest::blocking::Client;
use tungstenite::protocol::WebSocketConfig;
use tungstenite::stream::MaybeTlsStream;
use tungstenite::{Connector, WebSocket};

/// Get the http client to issue requests.
///
/// # Panics
/// If the client fails to build due to being unable to init the TLS backend or the system configuration cannot be loaded.
pub(crate) fn get_http_client(accept_invalid_certs: bool) -> Client {
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

pub(crate) fn get_ws_client(
    accept_invalid_certs: bool,
    url: String,
) -> Result<WebSocket<MaybeTlsStream<TcpStream>>> {
    let tcp_stream = TcpStream::connect(url.parse::<SocketAddr>()?)?;

    let tls_connector = TlsConnector::builder()
        .danger_accept_invalid_certs(accept_invalid_certs)
        .build()
        .unwrap();

    let (ws, res) = tungstenite::client_tls_with_config(
        url,
        tcp_stream,
        Some(WebSocketConfig::default()),
        Some(Connector::NativeTls(tls_connector)),
    )?;

    Ok(ws)
}
