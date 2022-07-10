pub use comm_types::hardware::{Architecture, HiveTargetInfo, Memory};
pub use controller::runner::TestChannelHandle;
pub use hive_macro::hive_test;
pub use inventory;
use probe_rs_test::Session;

#[allow(dead_code)]
pub struct HiveTestFunction {
    pub name: &'static str,
    pub ordered: usize,
    pub should_panic: bool,
    pub test_fn: fn(
        test_channel: &mut dyn TestChannelHandle,
        session: &mut Session,
        target_info: &HiveTargetInfo,
    ),
}

inventory::collect!(HiveTestFunction);
