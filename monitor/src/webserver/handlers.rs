//! Webserver request handlers
use std::sync::Arc;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::Extension;
use comm_types::auth::JwtClaims;
use tower_cookies::Cookies;

use crate::database::MonitorDb;
use crate::tasks::TaskManager;

use super::backend::auth::BackendAuthSchema;
use super::backend::BackendSchema;

#[axum::debug_handler]
pub(super) async fn graphql_backend(
    Extension(db): Extension<Arc<MonitorDb>>,
    Extension(task_manager): Extension<Arc<TaskManager>>,
    Extension(cookies): Extension<Cookies>,
    Extension(claims): Extension<JwtClaims>,
    schema: Extension<BackendSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema
        .execute(
            req.into_inner()
                .data(claims)
                .data(db)
                .data(task_manager)
                .data(cookies),
        )
        .await
        .into()
}

pub(super) async fn graphql_backend_auth(
    Extension(db): Extension<Arc<MonitorDb>>,
    Extension(cookies): Extension<Cookies>,
    schema: Extension<BackendAuthSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema
        .execute(req.into_inner().data(db).data(cookies))
        .await
        .into()
}
