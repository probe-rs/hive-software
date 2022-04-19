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

#[hive_test(should_panic = true, order = 30)]
fn check_order_30(test_channel: &mut dyn TestChannelHandle, session: &mut Session) {
    let _unused = test_channel;
    let _unused2 = session;

    let a = 4;
    let b = 2;

    assert_eq!(a + b, 4);
}

#[hive_test(order = 30)]
fn check_order_30_1(test_channel: &mut dyn TestChannelHandle, session: &mut Session) {
    let _unused = test_channel;
    let _unused2 = session;

    let a = 4;
    let b = 3;

    assert_eq!(a + b, 8);
}

#[hive_test(order = 30)]
fn check_order_30_2(test_channel: &mut dyn TestChannelHandle, session: &mut Session) {
    let _unused = test_channel;
    let _unused2 = session;

    let a = 4;
    let b = 4;

    assert_eq!(a + b, 8);
}

#[hive_test(order = 35)]
fn check_order_35(test_channel: &mut dyn TestChannelHandle, session: &mut Session) {
    let _unused = test_channel;
    let _unused2 = session;

    let a = 4;
    let b = 4;

    assert_eq!(a + b, 8);
}

#[hive_test(order = 35)]
fn check_order_35_1(test_channel: &mut dyn TestChannelHandle, session: &mut Session) {
    let _unused = test_channel;
    let _unused2 = session;

    let a = 4;
    let b = 4;

    assert_eq!(a + b, 8);
}
