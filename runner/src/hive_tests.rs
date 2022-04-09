use hive_test::{hive_test, inventory, TestChannelHandle};
use probe_rs_test::Probe;

#[hive_test]
fn my_first_test(test_channel: &mut dyn TestChannelHandle, probe: &mut Probe) {
    let _unused = test_channel;
    let _unused2 = probe;

    let a = 2;
    let b = 2;

    assert_eq!(a + b, 4);
}

#[hive_test]
fn only_pass_for_jlink(test_channel: &mut dyn TestChannelHandle, probe: &mut Probe) {
    let _unused = test_channel;
    assert_eq!(probe.get_name(), "J-Link".to_string());
}
