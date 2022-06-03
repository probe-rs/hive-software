//! Data models used for graphql
use async_graphql::{Enum, SimpleObject};
use comm_types::results::{TestResult, TestResults, TestStatus};

/// Flattened version of [`TestResults`] to use it in graphql api
#[derive(SimpleObject)]
pub(super) struct FlatTestResults {
    results: Vec<FlatTestResult>,
}

impl From<TestResults> for FlatTestResults {
    fn from(results: TestResults) -> Self {
        Self {
            results: results
                .results
                .into_iter()
                .map(|result| result.into())
                .collect(),
        }
    }
}

/// Flattened version of [`TestResult`] to use it in graphql api
#[derive(SimpleObject)]
pub(super) struct FlatTestResult {
    pub status: FlatTestStatus,
    pub should_panic: bool,
    pub test_name: String,
    pub target_name: String,
    pub probe_name: String,
    pub probe_sn: String,
}

impl From<TestResult> for FlatTestResult {
    fn from(result: TestResult) -> Self {
        Self {
            status: result.status.into(),
            should_panic: result.should_panic,
            test_name: result.test_name,
            target_name: result.target_name,
            probe_name: result.probe_name,
            probe_sn: result.probe_sn,
        }
    }
}

/// Flattened version of [`TestStatus`] to use it in graphql api
#[derive(SimpleObject)]
pub(super) struct FlatTestStatus {
    status: Status,
    reason: Option<String>,
}

impl From<TestStatus> for FlatTestStatus {
    fn from(status: TestStatus) -> Self {
        match status {
            TestStatus::Passed => Self {
                status: Status::Passed,
                reason: None,
            },
            TestStatus::Failed(reason) => Self {
                status: Status::Failed,
                reason: Some(reason),
            },
            TestStatus::Skipped(reason) => Self {
                status: Status::Skipped,
                reason: Some(reason),
            },
        }
    }
}

#[derive(Enum, PartialEq, Eq, Copy, Clone)]
pub(super) enum Status {
    Passed,
    Failed,
    Skipped,
}
