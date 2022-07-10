//! Dummy hive testdirectory
//!
//! This entire module is replaced by the corresponding hive module in the probe-rs test folder of the specific probe-rs testcandidate.

pub mod tests {
    use hive_test::{HiveTargetInfo, TestChannelHandle};
    use probe_rs_test::Session;

    /// This is only a dummy testfunction struct as the real testfunction is inserted by using the #[hive] macro on a module.
    /// It is implemented to allow checks to pass during development
    pub struct HiveTestFunction<Session> {
        pub name: &'static str,
        pub ordered: usize,
        pub should_panic: bool,
        pub test_fn: fn(
            test_channel: &mut dyn TestChannelHandle,
            session: &mut Session,
            target_info: &HiveTargetInfo,
        ),
    }

    inventory::collect!(HiveTestFunction<Session>);
}
