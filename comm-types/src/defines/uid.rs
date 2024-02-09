//! A UID which is unique for each testrun
use serde::{Deserialize, Serialize};
use uid::IdU32;

use super::HiveDefine;

/// Defines a `HIVE_UID` symbol which holds a 32 bit UID which is unique on each test-run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiveUid {
    uid: u32,
}

impl Default for HiveUid {
    fn default() -> Self {
        Self::new()
    }
}

impl HiveUid {
    pub fn new() -> Self {
        HiveUid {
            uid: IdU32::<Self>::new().get(),
        }
    }

    pub fn get_id(&self) -> u32 {
        self.uid
    }
}

#[typetag::serde]
impl HiveDefine for HiveUid {
    fn generate(&mut self) {
        self.uid = IdU32::<Self>::new().get();
    }

    fn to_file_line(&self) -> String {
        format!(".set HIVE_UID, {}", self.uid)
    }
}
