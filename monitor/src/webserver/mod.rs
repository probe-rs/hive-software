//! Hive webserver
//!
//! The webserver is the main interface used for communication with the outside world. It offers various endpoints and APIs:
//! - static fileserver (Used to host the Hive backend UI)
//! - /auth endpoint (Used to authenticate Hive users)
//! - /graphql endpoint (Used to serve and receive data from the Hive backend UI)
//! - /test endpoint (Used to receive and handle test requests)
//!
//! The server automatically hands out CSRF tokens in a cookie so the user can protect forms against CSRF attacks. CSRF Tokens are handed out at every endpoint though only enforced by endpoints which actually receive data from the user.
//!
//! # TLS
//! The webserver is using rustls to encrypt every connection. The certificate needs to be stored at the following locations: [`PEM_CERT`] and [`PEM_KEY`]. For the webserver to work properly.
//!
//! # Rate limit
//! The webserver has a built in rate limiting webserver with load shedding. The currently chosen values have not been tested extensively and are very conservative.
use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::error_handling::HandleErrorLayer;
use axum::handler::HandlerWithoutStateExt;
use axum::middleware::from_extractor;
use axum::response::Redirect;
use axum::routing::{self, post};
use axum::{BoxError, Extension, Router, middleware};
use axum_server::tls_rustls::RustlsConfig;
use hyper::{StatusCode, Uri};
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::services::{ServeDir, ServeFile};

use crate::database::MonitorDb;
use crate::tasks::TaskManager;
use crate::{Args, SHUTDOWN_SIGNAL};

mod api_token;
mod auth;
mod backend;
mod csrf;
mod handlers;
mod test;

pub use backend::get_schema_sdl;

const STATIC_FILES: &str = "data/webserver/static/";
const PEM_CERT: &str = "data/webserver/cert/cert.pem";
const PEM_KEY: &str = "data/webserver/cert/key.pem";

const GLOBAL_RATE_LIMIT_REQUEST_AMOUNT: u64 = 20;
const GLOBAL_RATE_LIMIT_DURATION_SECS: u64 = 1;
const GLOBAL_REQUEST_BUFFER_SIZE: usize = 40;

pub async fn web_server(db: Arc<MonitorDb>, task_manager: Arc<TaskManager>, cli_args: Arc<Args>) {
    let app = app(db, task_manager);
    let http_addr = SocketAddr::from(([0, 0, 0, 0], cli_args.http_port));
    let https_addr = SocketAddr::from(([0, 0, 0, 0], cli_args.https_port));
    let tls_config = RustlsConfig::from_pem_file(PEM_CERT, PEM_KEY).await.unwrap_or_else(|_| panic!("Failed to find the PEM certificate file. It should be stored in the application data folder: Cert: {} Key: {}", PEM_CERT, PEM_KEY));

    let server = axum_server::bind_rustls(https_addr, tls_config).serve(app.into_make_service());
    let http_server = redirect_http_to_https_server(cli_args, http_addr);
    let mut shutdown_signal = SHUTDOWN_SIGNAL.subscribe();

    tokio::select! {
        result = server => {result.expect("Unhandled webserver error encountered")}
        result = http_server => {result.expect("Unhandled http server error encountered")}
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
                .layer(from_extractor::<auth::HiveAuth>())
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

async fn handle_serve_dir_error<E: Display>(error: E) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!(
            "Failed to fetch static files, this is likely due to a bug in the software or wrong software setup: {}",
            error
        ),
    )
}

async fn handle_loadshed_error(err: BoxError) -> (StatusCode, String) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        format!("Too many requests: {}", err),
    )
}

/// Creates a http server which redirects all http traffic to https
async fn redirect_http_to_https_server(
    cli_args: Arc<Args>,
    addr: SocketAddr,
) -> Result<(), std::io::Error> {
    let http_port: u16 = cli_args.http_port;
    let https_port: u16 = cli_args.https_port;

    fn make_https(uri: Uri, http_port: u16, https_port: u16) -> Result<Uri, BoxError> {
        let mut parts = uri.into_parts();

        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);

        if let Some(authority) = parts.authority {
            let https_authority = authority
                .as_str()
                .replace(&http_port.to_string(), &https_port.to_string());

            parts.authority = Some(https_authority.parse()?)
        } else {
            return Err("URI authority is missing".into());
        }

        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }

        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |uri: Uri| async move {
        match make_https(uri, http_port, https_port) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(err) => Err((StatusCode::BAD_REQUEST, err.to_string())),
        }
    };

    let listener = TcpListener::bind(&addr).await.unwrap_or_else(|_| {
        panic!(
            "Failed to bind {}. This is likely caused by a system configuration issue.",
            addr
        )
    });

    axum::serve(listener, redirect.into_make_service()).await
}
