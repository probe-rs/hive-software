//! Hive webserver
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::extract::extractor_middleware;
use axum::routing::{self, post};
use axum::{middleware, BoxError, Extension, Router};
use axum_server::tls_rustls::RustlsConfig;
use hyper::StatusCode;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::database::MonitorDb;
use crate::tasks::TaskManager;
use crate::SHUTDOWN_SIGNAL;

mod auth;
mod backend;
mod csrf;
mod handlers;
mod test;

const STATIC_FILES: &str = "data/webserver/static/";
const PEM_CERT: &str = "data/webserver/cert/cert.pem";
const PEM_KEY: &str = "data/webserver/cert/key.pem";

const GLOBAL_RATE_LIMIT_REQUEST_AMOUNT: u64 = 20;
const GLOBAL_RATE_LIMIT_DURATION_SECS: u64 = 1;
const GLOBAL_REQUEST_BUFFER_SIZE: usize = 40;

pub(crate) async fn web_server(db: Arc<MonitorDb>, task_manager: Arc<TaskManager>) {
    let app = app(db, task_manager);
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
fn app(db: Arc<MonitorDb>, task_manager: Arc<TaskManager>) -> Router {
    let graphql_routes = Router::new()
        .route("/backend", post(handlers::graphql_backend))
        .layer(
            ServiceBuilder::new()
                .layer(middleware::from_fn(csrf::require_csrf_token))
                .layer(extractor_middleware::<auth::HiveAuth>())
                .layer(Extension(db.clone()))
                .layer(Extension(task_manager.clone()))
                .layer(Extension(backend::build_schema())),
        );

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
        .nest("/test", test::test_routes(db, task_manager))
        // Static fileserver used to host the hive-backend-ui Vue app
        .fallback(
            routing::get_service(
                ServeDir::new(STATIC_FILES)
                    .fallback(ServeFile::new(format!("{}index.html", STATIC_FILES))),
            )
            .handle_error(handle_serve_dir_error),
        )
        // Global layers
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_loadshed_error))
                .layer(
                    ServiceBuilder::new()
                        .load_shed()
                        .buffer(GLOBAL_REQUEST_BUFFER_SIZE)
                        .rate_limit(
                            GLOBAL_RATE_LIMIT_REQUEST_AMOUNT,
                            Duration::from_secs(GLOBAL_RATE_LIMIT_DURATION_SECS),
                        )
                        .into_inner(),
                )
                .layer(CookieManagerLayer::new())
                .layer(middleware::from_fn(csrf::provide_csrf_token)),
        )
}

async fn handle_serve_dir_error(error: std::io::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Failed to fetch static files, this is likely due to a bug in the software or wrong software setup: {}", error),
    )
}

async fn handle_loadshed_error(err: BoxError) -> (StatusCode, String) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        format!("Too many requests: {}", err),
    )
}
