//! Types used in IPC between runner and monitor
use ciborium::de::from_reader;
use hyper::body::Buf;
use hyper::{header, Body, Response};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::cbor::CBOR_MIME;
use super::hardware::{ProbeInfo, TargetState};
use super::results::TestResult;

pub type HiveProbeData = [Option<ProbeInfo>; 4];
pub type HiveTargetData = [Option<[TargetState; 4]>; 8];

/// All possible message types that can be sent via Hive IPC
#[derive(Debug, Serialize, Deserialize)]
pub enum IpcMessage {
    /// Information on the connected probes required by the runner
    /// The position in the array corresponds to the Testchannel the probe is connected to. The option is [`None`] if no probe is connected to the specific Testchannel.
    ProbeInitData(HiveProbeData),
    /// Information on the connected targets required by the runner
    /// The position in the array corresponds to the TSS position. The Option is [`None`] if there is no connected Daugtherboard or contains the Targetstate array in which the position corresponds to the target position on the Daughterboard.
    TargetInitData(HiveTargetData),
    /// Log generated by the runner on a test run
    RunnerLog,
    /// Test results generated by the runner
    TestResults(Vec<TestResult>),
    /// Desync error returned by runner, in case the received probe and target data does not seem to match the hardware detected by the runner
    DesyncError,
    /// Empty value
    Empty,
}

impl IpcMessage {
    /// Tries to parse an [`IpcMessage`] from the provided HTTP response
    pub async fn from_response(res: Response<Body>) -> Result<Self, ClientParseError> {
        if res.headers().get(header::CONTENT_TYPE).is_some() {
            if res.headers().get(header::CONTENT_TYPE).unwrap() != CBOR_MIME {
                return Err(ClientParseError::InvalidHeader);
            }
        } else {
            return Err(ClientParseError::InvalidHeader);
        }

        let body = hyper::body::aggregate(res)
            .await
            .map_err(|_| ClientParseError::InvalidBody)?;
        let msg = from_reader(body.reader()).map_err(|_| ClientParseError::InvalidCbor)?;

        Ok(msg)
    }
}

#[derive(Debug, Error)]
pub enum ClientParseError {
    #[error(
        "Response had an invalid header configuration, check that content-type is application/cbor"
    )]
    InvalidHeader,
    #[error(
        "Failed to deserialize CBOR body as IpcMessage, check that the server sends the correct types"
    )]
    InvalidCbor,
    #[error("Response contained invalid body format")]
    InvalidBody,
}
