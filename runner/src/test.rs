//! Handles the running and reporting of the tests

use std::panic::{self, PanicInfo};

use controller::common::CombinedTestChannel;
use controller::runner::TestChannelHandle;
use hive_test::HiveTestFunction;
use tokio::sync::mpsc::Sender;

use crate::comm::Message;

pub(crate) fn run_tests(
    test_channel: &mut CombinedTestChannel,
    target_name: &str,
    tss_pos: u8,
    comm_sender: &Sender<Message>,
) {
    comm_sender
        .blocking_send(Message::Message(format!(
            "Testing target {}, on tss {} with testchannel {:?}",
            target_name,
            tss_pos,
            test_channel.get_channel()
        )))
        .unwrap();

    let standard_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {
        // Do not print panics caused by tests
    }));

    for test in inventory::iter::<HiveTestFunction> {
        match panic::catch_unwind(|| {
            (test.test_fn)(
                &mut *test_channel.get_rpi().lock() as &mut dyn TestChannelHandle,
                test_channel.get_probe().lock().as_mut().unwrap(),
            );
        }) {
            Ok(_) => comm_sender
                .blocking_send(Message::TestResult(
                    "finished test successfully".to_string(),
                ))
                .unwrap(),
            Err(err) => {
                let cause = match err.downcast::<String>() {
                    Ok(err) => *err,
                    Err(_) => "unknown".to_string(),
                };

                comm_sender
                    .blocking_send(Message::TestResult(format!(
                        "test failed. Caused by: {:?}",
                        cause,
                    )))
                    .unwrap();
            }
        };
    }

    panic::set_hook(standard_hook);
}

/// Disables the printing of panics in this program, returns the previously used panic hook
pub(crate) fn disable_panic_print() -> Box<dyn for<'r, 's> Fn(&'r PanicInfo<'s>) + Send + Sync> {
    let standard_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    standard_hook
}
