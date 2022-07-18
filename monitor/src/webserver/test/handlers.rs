//! Handler functions for each endpoint
use std::sync::Arc;

use anyhow::anyhow;
use axum::extract::multipart::MultipartError;
use axum::extract::Query;
use axum::extract::{ContentLengthLimit, Multipart, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use comm_types::hardware::TargetState;
use comm_types::hardware::{Capabilities, ProbeState};
use comm_types::test::TestOptions;
use hive_db::CborDb;
use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

use crate::tasks::ws::WsTicket;
use crate::tasks::{ws, TaskManager, TaskManagerError};
use crate::{
    database::{keys, MonitorDb},
    tasks::TestTask,
};

#[derive(Debug, ThisError)]
pub(super) enum TestRequestError {
    #[error("Failed to parse multipart request: {0}")]
    MultipartParse(#[from] MultipartError),
    #[error("Failed to parse json: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error(transparent)]
    TaskManager(#[from] TaskManagerError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for TestRequestError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}

/// Get the capabilities of the testrack
pub(super) async fn capabilities(Extension(db): Extension<Arc<MonitorDb>>) -> Json<Capabilities> {
    let assigned_targets = db
        .config_tree
        .c_get(&keys::config::ASSIGNED_TARGETS)
        .unwrap()
        .expect("DB not initialized");

    let assigned_probes = db
        .config_tree
        .c_get(&keys::config::ASSIGNED_PROBES)
        .unwrap()
        .expect("DB not initialized");

    let mut available_targets = vec![];

    for daughterboard in assigned_targets.into_iter().flatten() {
        for target in daughterboard {
            if let TargetState::Known(target_info) = target {
                available_targets.push(target_info.name);
            }
        }
    }

    let available_probes = assigned_probes
        .into_iter()
        .filter_map(|probe_state| {
            if let ProbeState::Known(probe_info) = probe_state {
                return Some(probe_info.identifier);
            }

            None
        })
        .collect();

    Json(Capabilities {
        available_probes,
        available_targets,
    })
}

/// Endpoint to initiate a test request
pub(super) async fn test(
    Extension(task_manager): Extension<Arc<TaskManager>>,
    content: ContentLengthLimit<Multipart, 400_000_000>,
) -> Result<Json<WsTicket>, TestRequestError> {
    let mut multipart = content.0;

    let mut options: Option<TestOptions> = None;
    let mut runner = None;

    while let Some(field) = multipart.next_field().await? {
        let field_name = field.name();

        if field_name.is_none() {
            return Err(anyhow!(
                "Invalid multipart request. Expecting the field name to be present"
            )
            .into());
        }

        let field_name = field_name.unwrap();

        match field_name {
            "options" => {
                options = Some(serde_json::from_slice(&field.bytes().await?)?);
            }
            "runner" => {
                let field_data_type = field.content_type();

                if field_data_type != Some("application/octet-stream") {
                    return Err(anyhow!(
                        "Invalid file format provided for field 'runner'. Expecting binary executable."
                    )
                    .into());
                }

                runner = Some(field.bytes().await?)
            }
            name => return Err(anyhow!("Found unexpected field name: '{}'", name).into()),
        }
    }

    if runner.is_none() {
        return Err(anyhow!(
            "No runner binary provided to perform the tests on. The field 'runner' is missing."
        )
        .into());
    }

    let runner = runner.unwrap();

    let test_task = TestTask::new(runner, options.unwrap_or_default());

    let ws_ticket = task_manager.register_test_task(test_task).await?;

    Ok(Json(ws_ticket))
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct WsQueryParams {
    pub auth: String,
}

/// Test websocket handler which reports status on test progress and finally sends back the result
pub(super) async fn ws_handler(
    Query(query): Query<WsQueryParams>,
    ws: WebSocketUpgrade,
    Extension(task_manager): Extension<Arc<TaskManager>>,
) -> Result<impl IntoResponse, TaskManagerError> {
    let receiver = task_manager
        .validate_test_task_ticket(query.auth.into())
        .await?;

    Ok(ws
        .protocols(["application/json"])
        .on_upgrade(|socket| ws::socket_handler(socket, receiver)))
}
