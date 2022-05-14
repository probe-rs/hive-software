//! Backend graphql schemas
use async_graphql::{EmptySubscription, Schema};

pub(super) mod auth;
mod model;
mod mutation;
mod query;

const TOKEN_EXPIRE_TIME: u64 = 1800; // 30min

pub(super) type BackendSchema =
    Schema<query::BackendQuery, mutation::BackendMutation, EmptySubscription>;

pub(super) fn build_schema() -> BackendSchema {
    Schema::build(
        query::BackendQuery,
        mutation::BackendMutation,
        EmptySubscription,
    )
    .finish()
}
