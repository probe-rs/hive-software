#![allow(unused)]

use hive_test::HiveTargetInfo;
use hive_test::TestChannelHandle;
use probe_rs_test::Session;

#[hive_macro::hive_test]
fn test_function(
    test_channel: &mut dyn TestChannelHandle,
    session: &mut Session,
    target_info: &mut String,
) {
}

fn main() {}
