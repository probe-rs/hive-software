use std::time::Duration;

use hive_test::{hive_test, inventory, TestChannelHandle};
use probe_rs_test::Session;

#[hive_test]
fn my_first_test(_test_channel: &mut dyn TestChannelHandle, _session: &mut Session) {
    let a = 2;
    let b = 2;

    assert_eq!(a + b, 4);
}

#[hive_test]
fn check_uid(_test_channel: &mut dyn TestChannelHandle, session: &mut Session) {
    let cores = session.list_cores();
    println!("found cores: {:?}", cores);
    session
        .core(cores[0].0)
        .unwrap()
        .reset_and_halt(Duration::from_millis(200))
        .unwrap();

    session
        .core(cores[0].0)
        .unwrap()
        .set_hw_breakpoint(0xDA)
        .unwrap();
    session.core(cores[0].0).unwrap().run().unwrap();

    session
        .core(cores[0].0)
        .unwrap()
        .wait_for_core_halted(Duration::from_millis(500))
        .unwrap();

    let uid = session.core(cores[0].0).unwrap().read_core_reg(0).unwrap();
    assert_eq!(uid, 2);
}

#[hive_test]
fn only_pass_for_stm(_test_channel: &mut dyn TestChannelHandle, session: &mut Session) {
    assert_eq!(session.target().name, "STM32F303x".to_string());
}

#[hive_test(should_panic = true, order = 30)]
fn check_order_30(_test_channel: &mut dyn TestChannelHandle, _session: &mut Session) {
    let a = 4;
    let b = 2;

    assert_eq!(a + b, 4);
}

#[hive_test(order = 30)]
fn check_order_30_1(_test_channel: &mut dyn TestChannelHandle, _session: &mut Session) {
    let a = 4;
    let b = 3;

    assert_eq!(a + b, 8);
}

#[hive_test(order = 30)]
fn check_order_30_2(_test_channel: &mut dyn TestChannelHandle, _session: &mut Session) {
    let a = 4;
    let b = 4;

    assert_eq!(a + b, 8);
}

#[hive_test(order = 35)]
fn check_order_35(_test_channel: &mut dyn TestChannelHandle, _session: &mut Session) {
    let a = 4;
    let b = 4;

    assert_eq!(a + b, 8);
}

#[hive_test(order = 35)]
fn check_order_35_1(_test_channel: &mut dyn TestChannelHandle, _session: &mut Session) {
    let a = 4;
    let b = 4;

    assert_eq!(a + b, 8);
}
