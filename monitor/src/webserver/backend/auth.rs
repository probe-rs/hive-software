//! Backend graphql auth provider
use std::sync::Arc;

use anyhow::anyhow;
use async_graphql::{
    Context, EmptySubscription, ErrorExtensions, Object, Result as GraphQlResult, Schema,
};
use tower_cookies::Cookies;

use crate::database::MonitorDb;

use crate::webserver::auth;

use super::model::UserResponse;

pub(in crate::webserver) type BackendAuthSchema =
    Schema<BackendAuthQuery, BackendAuthMutation, EmptySubscription>;

pub(in crate::webserver) fn build_schema() -> BackendAuthSchema {
    Schema::build(BackendAuthQuery, BackendAuthMutation, EmptySubscription).finish()
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
        let db = ctx.data::<Arc<MonitorDb>>().unwrap();
        let cookies = ctx.data::<Cookies>().unwrap();

        let user = auth::authenticate_user(db.clone(), &username, &password, cookies)
            .await
            .map_err(|_| anyhow!("Not authorized").extend_with(|_, e| e.set("code", 403)))?;

        Ok(user.into())
    }
}

pub(in crate::webserver) struct BackendAuthMutation;

#[Object]
impl BackendAuthMutation {
    /// Log the currently authenticated user out by deleting the auth jwt cookie
    async fn logout<'ctx>(&self, ctx: &Context<'ctx>) -> GraphQlResult<bool> {
        let cookies = ctx.data::<Cookies>().unwrap();

        auth::logout(cookies);

        Ok(true)
    }
}
