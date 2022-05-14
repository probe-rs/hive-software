//! Backend graphql schemas
use async_graphql::{EmptySubscription, Schema};

mod model;
mod mutation;
mod query;

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
