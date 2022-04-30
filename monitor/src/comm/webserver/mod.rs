//! Hive webserver
use std::net::SocketAddr;

use axum::routing;
use axum::routing::{get, post};
use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use hyper::StatusCode;
use tower_http::services::ServeDir;

mod handlers;

const STATIC_FILES: &str = "data/webserver/static/";
const PEM_CERT: &str = "data/webserver/cert/cert.pem";
const PEM_KEY: &str = "data/webserver/cert/key.pem";

pub(super) async fn web_server() {
    let app = app();
    let addr = SocketAddr::from(([0, 0, 0, 0], 4356));
    let tls_config = RustlsConfig::from_pem_file(PEM_CERT, PEM_KEY).await.expect(&format!("Failed to find the PEM certificate file. It should be stored in the application data folder: Cert: {} Key: {}", PEM_CERT, PEM_KEY));
    axum_server::bind_rustls(addr, tls_config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

/// Builds the webserver with all endpoints
fn app() -> Router {
    Router::new()
    // Static fileserver used to host the hive-backend-ui Vue app
    .route(
        "/",
        routing::get_service(ServeDir::new(STATIC_FILES)).handle_error(
            |error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to fetch static files, this is likely due to a bug in the software or wrong software setup: {}", error),
                )
            },
        ),
    )
    // Auth handler to get tokens for websocket connection establishment
    .route("/auth/", post(handlers::auth_handler))
    // Websocket handler for backend UI
    .route("/ws/backend/", get(handlers::backend_ws_handler))
}
