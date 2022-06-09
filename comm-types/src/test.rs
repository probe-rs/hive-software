//! Hive test request and result types
use serde::{Deserialize, Serialize};

/// Test options which are passed on a test request
#[derive(Debug, Serialize, Deserialize)]
pub struct TestOptions {
    // TODO Add options like probe/target filters
}

impl Default for TestOptions {
    fn default() -> Self {
        Self {}
    }
}

/// A batch of [`TestResult`]s from an entire testrun
#[derive(Debug, Serialize, Deserialize)]
pub struct TestResults {
    pub results: Vec<TestResult>,
}

/// A single test result
#[derive(Debug, Serialize, Deserialize)]
pub struct TestResult {
    pub status: TestStatus,
    pub should_panic: bool,
    pub test_name: String,
    pub target_name: String,
    pub probe_name: String,
    pub probe_sn: String,
}

/// Status of a test, failed and skipped contain the reason for the skipping/failure
#[derive(Debug, Serialize, Deserialize)]
pub enum TestStatus {
    Passed,
    Failed(String),
    Skipped(String),
}
