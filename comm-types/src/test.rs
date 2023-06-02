//! Hive test request and result types
use serde::{Deserialize, Serialize};

/// Status and result/error messages which are sent from the task runner to the corresponding websocket of the test task
#[derive(Debug, Serialize, Deserialize)]
pub enum TaskRunnerMessage {
    Status(String),
    Error(String),
    Results(TestResults),
}

/// Test options which are passed on a test request
#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct TestOptions {
    pub filter: Option<TestFilter>,
}

/// Filter used in test requests to filter probes and targets on which the tests should be executed
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub struct TestFilter {
    pub probes: Option<Filter>,
    pub targets: Option<Filter>,
}

/// String values to include or exclude in a filter
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum Filter {
    Include(Vec<String>),
    Exclude(Vec<String>),
}

/// A batch of [`TestResult`]s from an entire testrun
#[derive(Debug, Serialize, Deserialize)]
pub struct TestResults {
    pub status: TestRunStatus,
    pub results: Option<Vec<TestResult>>,
    pub error: Option<TestRunError>,
}

/// A single test result
#[derive(Debug, Serialize, Deserialize)]
pub struct TestResult {
    pub status: TestStatus,
    pub backtrace: Option<String>,
    pub should_panic: bool,
    pub test_name: String,
    pub module_path: String,
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

/// Status of an entire test run
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestRunStatus {
    Ok,
    Error,
}

/// The error type which is returned in case something goes wrong during a test run
#[derive(Debug, Serialize, Deserialize)]
pub struct TestRunError {
    pub err: String,
    pub source: Option<String>,
}
