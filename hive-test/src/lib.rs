pub use controller::runner::TestChannelHandle;
pub use hive_macro::hive_test;
pub use probe_rs::Probe;

#[allow(dead_code)]
pub struct HiveTestFunction {
    pub name: &'static str,
    pub ordered: usize,
    pub should_panic: bool,
    pub test_fn: fn(test_channel: &mut dyn TestChannelHandle, probe: &mut Probe),
}

inventory::collect!(HiveTestFunction);
