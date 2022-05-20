//! Webserver request handlers
use std::sync::Arc;

use async_graphql_axum::{GraphQLRequest, GraphQLResponse};
use axum::extract::Extension;
use comm_types::auth::JwtClaims;
use tower_cookies::Cookies;

use crate::database::HiveDb;

use super::backend::auth::BackendAuthSchema;
use super::backend::BackendSchema;

pub(super) async fn graphql_backend(
    Extension(db): Extension<Arc<HiveDb>>,
    Extension(cookies): Extension<Cookies>,
    schema: Extension<BackendSchema>,
    req: GraphQLRequest,
    Extension(claims): Extension<JwtClaims>,
) -> GraphQLResponse {
    schema
        .execute(req.into_inner().data(claims).data(db).data(cookies))
        .await
        .into()
}

pub(super) async fn graphql_backend_auth(
    Extension(db): Extension<Arc<HiveDb>>,
    Extension(cookies): Extension<Cookies>,
    schema: Extension<BackendAuthSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema
        .execute(req.into_inner().data(db).data(cookies))
        .await
        .into()
}
