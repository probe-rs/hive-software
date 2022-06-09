//! Hive webserver
use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::extractor_middleware;
use axum::routing::{self, post};
use axum::{middleware, Extension, Router};
use axum_server::tls_rustls::RustlsConfig;
use hyper::StatusCode;
use tokio::sync::mpsc::Sender;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;

use crate::database::HiveDb;
use crate::testmanager::{ReinitializationTask, TestTask};
use crate::SHUTDOWN_SIGNAL;

mod auth;
mod backend;
mod csrf;
mod handlers;
mod test;

const STATIC_FILES: &str = "data/webserver/static/";
const PEM_CERT: &str = "data/webserver/cert/cert.pem";
const PEM_KEY: &str = "data/webserver/cert/key.pem";

pub(crate) async fn web_server(
    db: Arc<HiveDb>,
    test_task_sender: Sender<TestTask>,
    reinit_task_sender: Sender<ReinitializationTask>,
) {
    let app = app(db, test_task_sender, reinit_task_sender);
    let addr = SocketAddr::from(([0, 0, 0, 0], 4356));
    let tls_config = RustlsConfig::from_pem_file(PEM_CERT, PEM_KEY).await.unwrap_or_else(|_| panic!("Failed to find the PEM certificate file. It should be stored in the application data folder: Cert: {} Key: {}", PEM_CERT, PEM_KEY));

    let server = axum_server::bind_rustls(addr, tls_config).serve(app.into_make_service());
    let mut shutdown_signal = SHUTDOWN_SIGNAL.subscribe();

    tokio::select! {
        result = server => {result.expect("Unhandled webserver error encountered")}
        result = shutdown_signal.recv() => {result.expect("Failed to receive global shutdown signal")}
    }
}

/// Builds the webserver with all endpoints
fn app(
    db: Arc<HiveDb>,
    test_task_sender: Sender<TestTask>,
    reinit_task_sender: Sender<ReinitializationTask>,
) -> Router {
    let graphql_routes = Router::new()
        .route("/backend", post(handlers::graphql_backend))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(csrf::require_csrf_token))
                .layer(extractor_middleware::<auth::HiveAuth>())
                .layer(Extension(db.clone()))
                .layer(Extension(reinit_task_sender))
                .layer(Extension(backend::build_schema())),
        );

    println!("{}", backend::build_schema().sdl());

    let auth_routes = Router::new()
        .route("/backend", post(handlers::graphql_backend_auth))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(csrf::require_csrf_token))
                .layer(Extension(db.clone()))
                .layer(Extension(backend::auth::build_schema())),
        );

    Router::new()
    // Auth handlers
    .nest("/auth", auth_routes)
    // Graphql handlers
    .nest("/graphql", graphql_routes)
    // REST test request endpoint
    .nest("/test", test::test_routes(db, test_task_sender))
    // Static fileserver used to host the hive-backend-ui Vue app
    .fallback(routing::get_service(ServeDir::new(STATIC_FILES)).handle_error(
        |error: std::io::Error| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to fetch static files, this is likely due to a bug in the software or wrong software setup: {}", error),
            )
        },
    ))
    // Global layers
    .layer(
        ServiceBuilder::new()
        .layer(CookieManagerLayer::new())
        .layer(middleware::from_fn(csrf::provide_csrf_token))
    )
}
