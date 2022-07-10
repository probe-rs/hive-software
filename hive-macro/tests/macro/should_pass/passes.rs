use hive_test::HiveTargetInfo;
use hive_test::TestChannelHandle;
use probe_rs_test::Session;

#[hive_macro::hive_test]
fn my_fancy_test(
    _test_channel: &mut dyn TestChannelHandle,
    session: &mut Session,
    target_info: &HiveTargetInfo,
) {
    let _session = session;
    let _target_info = target_info;

    // Doing important test
    let mut i = 0;
    i += 1;

    assert_eq!(i, 1);
}

fn main() {}
