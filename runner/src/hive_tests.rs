use hive_test::{hive_test, inventory, TestChannelHandle};
use probe_rs_test::Session;

#[hive_test]
fn my_first_test(test_channel: &mut dyn TestChannelHandle, session: &mut Session) {
    let _unused = test_channel;
    let _unused2 = session;

    let a = 2;
    let b = 2;

    assert_eq!(a + b, 4);
}

#[hive_test]
fn only_pass_for_stm(test_channel: &mut dyn TestChannelHandle, session: &mut Session) {
    let _unused = test_channel;
    assert_eq!(session.target().name, "STM32F303x".to_string());
}
