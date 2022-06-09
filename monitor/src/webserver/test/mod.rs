//! Test endpoint
use std::sync::Arc;

use axum::routing::{get, post};
use axum::{Extension, Router};
use tokio::sync::mpsc::Sender;
use tower::ServiceBuilder;

use crate::database::HiveDb;
use crate::testmanager::TestTask;

mod handlers;

pub(super) fn test_routes(db: Arc<HiveDb>, test_task_sender: Sender<TestTask>) -> Router {
    Router::new()
        .route("/capabilities", get(handlers::capabilities))
        .route("/run", post(handlers::test))
        .layer(
            ServiceBuilder::new()
                .layer(Extension(db.clone()))
                .layer(Extension(test_task_sender)),
        )
}
