use std::sync::Arc;

use anyhow::anyhow;
use axum::extract::multipart::MultipartError;
use axum::extract::{ContentLengthLimit, Multipart};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use axum_macros::debug_handler;
use comm_types::hardware::{Capabilities, ProbeState};
use comm_types::ipc::HiveProbeData;
use comm_types::test::{TestOptions, TestResults};
use comm_types::{hardware::TargetState, ipc::HiveTargetData};
use thiserror::Error as ThisError;
use tokio::sync::mpsc::Sender;

use crate::{
    database::{keys, CborDb, HiveDb},
    testmanager::TestTask,
};

#[derive(Debug, ThisError)]
pub(super) enum TestRequestError {
    #[error("Failed to parse multipart request: {0}")]
    MultipartParse(#[from] MultipartError),
    #[error("Failed to parse json: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl IntoResponse for TestRequestError {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}

/// Get the capabilities of the testrack
pub(super) async fn capabilities(Extension(db): Extension<Arc<HiveDb>>) -> Json<Capabilities> {
    let assigned_targets: HiveTargetData = db
        .config_tree
        .c_get(keys::config::ASSIGNED_TARGETS)
        .unwrap()
        .expect("DB not initialized");

    let assigned_probes: HiveProbeData = db
        .config_tree
        .c_get(keys::config::ASSIGNED_PROBES)
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

/// Test the provided probe-rs project
#[debug_handler]
pub(super) async fn test(
    Extension(test_task_sender): Extension<Sender<TestTask>>,
    content: ContentLengthLimit<Multipart, 50_000_000>,
) -> Result<Json<TestResults>, TestRequestError> {
    let mut multipart = content.0;

    let mut options: Option<TestOptions> = None;
    let mut project = None;

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
            "project" => {
                let field_data_type = field.content_type();
                let field_file_name = field.file_name();

                if field_data_type != Some("application/octet-stream")
                    || field_file_name.unwrap_or_default().split('.').last() != Some("tar")
                {
                    return Err(anyhow!(
                        "Invalid file format provided for field 'project'. Expecting tar archive."
                    )
                    .into());
                }

                project = Some(field.bytes().await?)
            }
            name => return Err(anyhow!("Found unexpected field name: '{}'", name).into()),
        }
    }

    if project.is_none() {
        return Err(anyhow!("No project tar archive provided to perform the tests on. The field 'project' is missing.").into());
    }

    let project = project.unwrap();

    let (test_task, test_result_receiver) = TestTask::new(project, options.unwrap_or_default());

    test_task_sender
        .send(test_task)
        .await
        .expect("Test task receiver closed but axum server was still active");

    let results = test_result_receiver
        .await
        .expect("Oneshot sender was unexpectedly dropped by testmanager.");

    Ok(Json(results))
}
