//! Hive webserver
use std::net::SocketAddr;

use axum::routing::{self, get, post};
use axum::{Extension, Router};
use axum_server::tls_rustls::RustlsConfig;
use hyper::StatusCode;
use tower_http::auth::RequireAuthorizationLayer;
use tower_http::services::ServeDir;

mod auth;
mod backend;
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
    let ws_routes = Router::new().route(
        "/backend",
        get(handlers::backend_ws_handler).layer(RequireAuthorizationLayer::custom(auth::HiveAuth)),
    );

    let graphql_routes = Router::new()
        .route("/backend", post(handlers::graphql_backend))
        .layer(Extension(backend::build_schema()));
    println!("{}", backend::build_schema().sdl());

    Router::new()
    // Static fileserver used to host the hive-backend-ui Vue app
    .fallback(routing::get_service(ServeDir::new(STATIC_FILES)).handle_error(
        |error: std::io::Error| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch static files, this is likely due to a bug in the software or wrong software setup: {}", error),
            )
        },
    ))
    // Auth handler to get tokens for websocket connection establishment
    .route("/auth", post(auth::ws_auth_handler))
    // Websocket handler for backend UI
    .nest("/ws", ws_routes)
    .nest("/graphql", graphql_routes)
}
