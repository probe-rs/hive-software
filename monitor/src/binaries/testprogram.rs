//! Handles the management of existing and new testprograms
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct TestProgram {
    pub name: String,
    pub path: PathBuf,
}

pub(crate) const TESTPROGRAM_PATH: &str = "data/testprograms/";
