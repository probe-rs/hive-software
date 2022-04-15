//! Handles the running and reporting of the tests

use std::error::Error;
use std::panic::{self, PanicInfo};

use antidote::Mutex as PoisonFreeMutex;
use comm_types::results::{TestResult, TestStatus};
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
    log::trace!(
        "Testing target {}, on tss {} with {}",
        target_name,
        tss_pos,
        test_channel.get_channel()
    );

    let probe_info_lock = test_channel.get_probe_info().lock();
    let probe_info = probe_info_lock.as_ref().unwrap();
    let probe_name = probe_info.identifier.clone();
    let probe_sn = match probe_info.serial_number.clone() {
        Some(number) => number,
        None => "None".to_string(),
    };

    // Check if Testchannel is ready, it might not be anymore in case probe reinitialization failed.
    if !test_channel.is_ready() {
        for test in inventory::iter::<HiveTestFunction> {
            let result = TestResult {
                status: TestStatus::SKIPPED(
                    "Failed to reinitialize the debug probe for this testrun".to_string(),
                ),
                should_panic: test.should_panic,
                test_name: test.name.to_string(),
                target_name: target_name.to_string(),
                probe_name: probe_name.clone(),
                probe_sn: probe_sn.clone(),
            };

            comm_sender
                .blocking_send(Message::TestResult(result))
                .unwrap()
        }
        return;
    }

    let probe = test_channel.take_probe_owned();
    match probe.attach(target_name) {
        Ok(session) => {
            let session = PoisonFreeMutex::new(session);

            for test in inventory::iter::<HiveTestFunction> {
                match panic::catch_unwind(|| {
                    (test.test_fn)(
                        &mut *test_channel.get_rpi().lock() as &mut dyn TestChannelHandle,
                        &mut session.lock(),
                    );
                }) {
                    Ok(_) => {
                        let status = match test.should_panic {
                            true => TestStatus::FAILED("Test function did not panic.".to_string()),
                            false => TestStatus::PASSED,
                        };

                        let result = TestResult {
                            status,
                            should_panic: test.should_panic,
                            test_name: test.name.to_string(),
                            target_name: target_name.to_string(),
                            probe_name: probe_name.clone(),
                            probe_sn: probe_sn.clone(),
                        };

                        comm_sender
                            .blocking_send(Message::TestResult(result))
                            .unwrap()
                    }
                    Err(err) => {
                        let cause = match err.downcast::<String>() {
                            Ok(err) => *err,
                            Err(_) => "Unknown".to_string(),
                        };

                        let status = match test.should_panic {
                            true => TestStatus::PASSED,
                            false => TestStatus::FAILED(cause),
                        };

                        let result = TestResult {
                            status,
                            should_panic: test.should_panic,
                            test_name: test.name.to_string(),
                            target_name: target_name.to_string(),
                            probe_name: probe_name.clone(),
                            probe_sn: probe_sn.clone(),
                        };

                        comm_sender
                            .blocking_send(Message::TestResult(result))
                            .unwrap();
                    }
                };
            }
        }
        Err(err) => {
            log::debug!(
                "Failed to attach {} with probe {} to target {}, skipping...",
                test_channel.get_channel(),
                probe_name,
                target_name
            );

            log::error!("{} source: {:?}", err, err.source());
        }
    }

    // reinitialize probe, and transfer ownership back to test_channel
    match test_channel
        .get_probe_info()
        .lock()
        .as_ref()
        .unwrap()
        .open()
    {
        Ok(probe) => test_channel.return_probe(probe),
        Err(err) => {
            log::warn!(
                "Failed to reinitialize the debug probe connected to {}: {}. Skipping the remaining tests on this Testchannel.",
                test_channel.get_channel(),
                err
            )
        }
    }
}

/// Disables the printing of panics in this program, returns the previously used panic hook
pub(crate) fn disable_panic_print() -> Box<dyn for<'r, 's> Fn(&'r PanicInfo<'s>) + Send + Sync> {
    let standard_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    standard_hook
}
