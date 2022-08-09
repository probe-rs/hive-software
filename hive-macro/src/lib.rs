//! This crate provides all the macros used for Hive.
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;

mod hive;
mod hive_test;

/// The macro to annotate a Hive testfunction
///
/// Each function annotated with this macro is collected by the test runner and ran on the Hive testrack.
///
/// # Attributes
/// The macro accepts optional attributes to give control over how the tests are run. The following attributes are available:
/// - order: usize (Default: 0)
/// - should_panic: bool (Default: false)
///
/// The order attribute controls the order in which all collected testfunctions are run. Where the smallest order number gets run first. It is also allowed to have multiple functions with the same order. In that case all functions with the same order will get run in a random order. This system allows for complex ordering, while still maintianing the options to run tests in a random order where possible.
///
/// The should_panic attribute specifies if the testfunction succeeds when it panics or not.
///
/// # Examples
/// Basic usage:
/// ```rust
/// #[hive_test]
///fn my_fancy_test(_test_channel: &mut dyn TestChannelHandle, _session: &mut Session, _target_info: &HiveTargetInfo, _defines: &DefineRegistry) {
///    // Doing important test
///    let mut i = 0;
///    i += 1;
///
///    assert_eq!(i, 1);
///}
/// ```
/// Advanced usage with attributes:
///```rust
/// #[hive_test(order = 100, should_panic = true)]
///fn my_fancy_test(_test_channel: &mut dyn TestChannelHandle, _session: &mut Session, _target_info: &HiveTargetInfo, _defines: &DefineRegistry) {
///    // Intentional panic
///    panic!();
///}
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn hive_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    hive_test::run(attr, item)
}

/// The macro to annotate the top-level Hive test module.
///
/// The top level hive test module must be named `tests` and is annotated with this macro to setup all global requirements to make the `#[hive_test]` macros work properly.
///
/// # Example
/// ```rust
/// #[hive]
/// mod tests {
///     // Add test functions and any child modules here...
/// }
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn hive(attr: TokenStream, item: TokenStream) -> TokenStream {
    hive::run(attr, item)
}
