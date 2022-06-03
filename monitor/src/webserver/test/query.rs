//! The graphql query
use std::sync::Arc;

use async_graphql::{Context, Object};
use comm_types::{hardware::TargetState, ipc::HiveTargetData};

use crate::database::{keys, CborDb, HiveDb};

pub(in crate::webserver) struct TestQuery;

#[Object]
impl TestQuery {
    /// All targets which are available for testing
    async fn available_targets<'ctx>(&self, ctx: &Context<'ctx>) -> Vec<String> {
        let db = ctx.data::<Arc<HiveDb>>().unwrap();

        let assigned_targets: HiveTargetData = db
            .config_tree
            .c_get(keys::config::ASSIGNED_TARGETS)
            .unwrap()
            .expect("DB not initialized");

        let mut available_targets = vec![];

        for daughterboard in assigned_targets
            .into_iter()
            .filter_map(|daughterboard| daughterboard)
        {
            for target in daughterboard {
                if let TargetState::Known(target_info) = target {
                    available_targets.push(target_info.name);
                }
            }
        }

        available_targets
    }
}
