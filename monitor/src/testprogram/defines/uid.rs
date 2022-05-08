//! A UID which is unique for each testrun
use uid::IdU32;

use super::HiveDefine;

/// Defines a `HIVE_UID` symbol which holds a 32 bit UID which is unique on each test-run
pub(super) struct HiveUid {
    uid: IdU32<Self>,
}

impl HiveUid {
    pub fn new() -> Self {
        HiveUid { uid: IdU32::new() }
    }
}

impl HiveDefine for HiveUid {
    fn generate(&mut self) {
        self.uid = IdU32::new();
    }

    fn to_file_line(&self) -> String {
        format!(".set HIVE_UID, {}", self.uid)
    }
}
