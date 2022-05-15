//! Backend graphql auth provider
use std::sync::Arc;

use anyhow::anyhow;
use async_graphql::{
    Context, EmptyMutation, EmptySubscription, ErrorExtensions, Object, Result as GraphQlResult,
    Schema,
};

use crate::{
    comm::webserver::auth::{check_password, generate_jwt},
    database::HiveDb,
};

use super::TOKEN_EXPIRE_TIME;

pub(in crate::comm::webserver) type BackendAuthSchema =
    Schema<BackendAuthQuery, EmptyMutation, EmptySubscription>;

pub(in crate::comm::webserver) fn build_schema() -> BackendAuthSchema {
    Schema::build(BackendAuthQuery, EmptyMutation, EmptySubscription).finish()
}

pub(in crate::comm::webserver) struct BackendAuthQuery;

#[Object]
impl BackendAuthQuery {
    /// Authenticates the provided user and sends back a jwt
    async fn authenticate_user<'ctx>(
        &self,
        ctx: &Context<'ctx>,
        username: String,
        password: String,
    ) -> GraphQlResult<String> {
        let db = ctx.data::<Arc<HiveDb>>().unwrap();

        let user = check_password(db.clone(), username, password)
            .map_err(|_| anyhow!("Not authorized").extend_with(|_, e| e.set("code", 403)))?;

        Ok(generate_jwt(user, TOKEN_EXPIRE_TIME))
    }
}
