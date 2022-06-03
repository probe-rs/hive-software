//! Test endpoint graphql schemas
use async_graphql::{EmptySubscription, Schema};

mod model;
mod mutation;
mod query;

pub(super) type TestSchema = Schema<query::TestQuery, mutation::TestMutation, EmptySubscription>;

pub(super) fn build_schema() -> TestSchema {
    Schema::build(query::TestQuery, mutation::TestMutation, EmptySubscription).finish()
}
