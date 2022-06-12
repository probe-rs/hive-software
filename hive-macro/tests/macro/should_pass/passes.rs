use hive_test::TestChannelHandle;
use probe_rs_test::Session;

#[hive_macro::hive_test]
fn my_fancy_test(_test_channel: &mut dyn TestChannelHandle, session: &mut Session) {
    let _session = session;

    // Doing important test
    let mut i = 0;
    i += 1;

    assert_eq!(i, 1);
}

fn main() {}
