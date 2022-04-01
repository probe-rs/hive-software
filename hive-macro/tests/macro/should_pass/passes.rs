use hive_test::TestChannelHandle;
use probe_rs::Probe;

#[hive_macro::hive_test]
fn my_fancy_test(test_channel: &mut dyn TestChannelHandle, probe: &mut Probe) {
    let _channel = test_channel;
    let _probe = probe;

    // Doing important test
    let mut i = 0;
    i += 1;

    assert_eq!(i, 1);
}

fn main() {}
