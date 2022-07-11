//! Handles the running and reporting of the tests

use std::error::Error;
use std::panic::{self, PanicInfo};

use antidote::Mutex as PoisonFreeMutex;
use comm_types::hardware::TargetInfo;
use comm_types::test::{TestResult, TestStatus};
use controller::common::hardware::CombinedTestChannel;
use hive_test::TestChannelHandle;
use lazy_static::lazy_static;
use tokio::sync::mpsc::Sender;

use probe_rs_test::Session;

lazy_static! {
    pub(crate) static ref TEST_FUNCTIONS: Vec<&'static HiveTestFunction<Session>> = {
        let mut tests: Vec<&HiveTestFunction<Session>> = vec![];
        // We order the tests according to the order field
        for test in inventory::iter::<HiveTestFunction<Session>> {
            tests.push(test);
        }

        tests.sort_unstable_by(|a, b| a.ordered.cmp(&b.ordered));

        tests
    };
}

use crate::comm::Message;
use crate::hive::tests::HiveTestFunction;

pub(crate) fn run_tests(
    testchannel: &mut CombinedTestChannel,
    target_info: &TargetInfo,
    tss_pos: u8,
    comm_sender: &Sender<Message>,
) {
    log::trace!(
        "Testing target {}, on tss {} with {}",
        target_info.name,
        tss_pos,
        testchannel.get_channel()
    );

    let probe_info = testchannel.get_probe_info().unwrap();
    let probe_name = probe_info.identifier.clone();
    let probe_sn = match probe_info.serial_number {
        Some(number) => number,
        None => "None".to_owned(),
    };

    // Check if Testchannel is ready, it might not be anymore in case probe reinitialization failed.
    if !testchannel.is_ready() {
        skip_tests(
            comm_sender.clone(),
            &target_info.name,
            &probe_name,
            &probe_sn,
            "Failed to reinitialize the debug probe for this testrun",
        );
        return;
    }

    // Check if the target status is not OK, which means that no tests can be performed
    if target_info.status.is_err() {
        skip_tests(
            comm_sender.clone(),
            &target_info.name,
            &probe_name,
            &probe_sn,
            target_info.status.as_ref().unwrap_err(),
        );
        return;
    }

    let probe = testchannel.take_probe_owned();
    match probe.attach(&target_info.name) {
        Ok(session) => {
            let session = PoisonFreeMutex::new(session);

            for test in TEST_FUNCTIONS.iter() {
                match panic::catch_unwind(|| {
                    (test.test_fn)(
                        &mut *testchannel.get_rpi().lock() as &mut dyn TestChannelHandle,
                        &mut *session.lock(),
                        &target_info.clone().into(),
                    );
                }) {
                    Ok(_) => {
                        let status = match test.should_panic {
                            true => TestStatus::Failed("Test function did not panic.".to_owned()),
                            false => TestStatus::Passed,
                        };

                        let result = TestResult {
                            status,
                            should_panic: test.should_panic,
                            test_name: test.name.to_owned(),
                            module_path: test.module_path.to_owned(),
                            target_name: target_info.name.to_owned(),
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
                            Err(_) => "Unknown".to_owned(),
                        };

                        let status = match test.should_panic {
                            true => TestStatus::Passed,
                            false => TestStatus::Failed(cause),
                        };

                        let result = TestResult {
                            status,
                            should_panic: test.should_panic,
                            test_name: test.name.to_owned(),
                            module_path: test.module_path.to_owned(),
                            target_name: target_info.name.to_owned(),
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
            let reason = match err {
                probe_rs_test::Error::ChipNotFound(err) => {
                    log::warn!("Could not find a valid chip specification for target {}. This target might not be supported by probe-rs.\nCaused by: {}\nskipping...", target_info.name, err);
                    format!("Unknown target {}", target_info.name)
                }
                probe_rs_test::Error::Probe(err) => {
                    log::warn!(
                        "Probe {} failed to connect to target {}: {}\nCaused by: {:?}\nskipping...",
                        probe_name,
                        target_info.name,
                        err,
                        err.source()
                    );
                    format!("Probe error: {:?}", err.source())
                }
                _ => {
                    log::error!(
                        "Error at testing target {} with probe {}: {} source: {:?}\nskipping...",
                        target_info.name,
                        probe_name,
                        err,
                        err.source()
                    );
                    format!("Failed to attach probe: {}", err)
                }
            };

            skip_tests(
                comm_sender.clone(),
                &target_info.name,
                &probe_name,
                &probe_sn,
                &reason,
            );
        }
    }

    // reinitialize probe, and transfer ownership back to test_channel
    testchannel.reinitialize_probe().unwrap_or_else(|err|{
        log::warn!(
            "Failed to reinitialize the debug probe connected to {}: {}. Skipping the remaining tests on this Testchannel.",
            testchannel.get_channel(),
            err
        )
    })
}

/// Disables the printing of panics in this program, returns the previously used panic hook
pub(crate) fn disable_panic_print() -> Box<dyn for<'r, 's> Fn(&'r PanicInfo<'s>) + Send + Sync> {
    let standard_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));

    standard_hook
}

/// Skips all tests in the current iteration and sends the results to the comm module
fn skip_tests(
    comm_sender: Sender<Message>,
    target_name: &str,
    probe_name: &str,
    probe_sn: &str,
    reason: &str,
) {
    for test in TEST_FUNCTIONS.iter() {
        let result = TestResult {
            status: TestStatus::Skipped(reason.to_owned()),
            should_panic: test.should_panic,
            test_name: test.name.to_owned(),
            module_path: test.module_path.to_owned(),
            target_name: target_name.to_owned(),
            probe_name: probe_name.to_owned(),
            probe_sn: probe_sn.to_owned(),
        };

        comm_sender
            .blocking_send(Message::TestResult(result))
            .unwrap()
    }
}
