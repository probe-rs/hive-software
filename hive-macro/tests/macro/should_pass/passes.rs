use hive_test::TestChannelHandle;
use probe_rs_test::Session;

#[hive_macro::hive_test]
fn my_fancy_test(test_channel: &mut dyn TestChannelHandle, session: &mut Session) {
    let _channel = test_channel;
    let _session = session;

    // Doing important test
    let mut i = 0;
    i += 1;

    assert_eq!(i, 1);
}

fn main() {}
