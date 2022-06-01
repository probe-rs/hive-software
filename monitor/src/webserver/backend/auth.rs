//! Backend graphql auth provider
use std::sync::Arc;

use anyhow::anyhow;
use async_graphql::{
    Context, EmptyMutation, EmptySubscription, ErrorExtensions, Object, Result as GraphQlResult,
    Schema,
};
use tower_cookies::Cookies;

use crate::database::HiveDb;

use crate::webserver::auth;

use super::model::UserResponse;

pub(in crate::webserver) type BackendAuthSchema =
    Schema<BackendAuthQuery, EmptyMutation, EmptySubscription>;

pub(in crate::webserver) fn build_schema() -> BackendAuthSchema {
    Schema::build(BackendAuthQuery, EmptyMutation, EmptySubscription).finish()
}

pub(in crate::webserver) struct BackendAuthQuery;

#[Object]
impl BackendAuthQuery {
    /// Authenticates the provided user and sends back a jwt
    async fn authenticate_user<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        username: String,
        password: String,
    ) -> GraphQlResult<UserResponse> {
        let db = ctx.data::<Arc<HiveDb>>().unwrap();
        let cookies = ctx.data::<Cookies>().unwrap();

        let user = auth::authenticate_user(db.clone(), &username, &password, cookies)
            .await
            .map_err(|_| anyhow!("Not authorized").extend_with(|_, e| e.set("code", 403)))?;

        Ok(user.into())
    }
}
