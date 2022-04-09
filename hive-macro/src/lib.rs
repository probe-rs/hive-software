//! # Hive-Macro
//! This crate provides all the macros used for Hive.

use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::{
    spanned::Spanned, AttributeArgs, FnArg, ItemFn, Pat, PatType, ReturnType, Type, Visibility,
};

/// Arguments for hive_test macro
#[derive(Debug, FromMeta)]
struct TestOptions {
    #[darling(default)]
    order: Option<usize>,
    #[darling(default)]
    should_panic: Option<bool>,
}

const ATTRIBUTE_EXAMPLE: &str = "#[hive_test(order = 1, should_panic = true)]";
const ATTRIBUTE_KEYS: &str = "'order = usize', 'should_panic = bool'";
const FUNCTION_EXAMPLE: &str =
    "fn my_fancy_test(test_channel: &mut dyn TestChannelHandle, probe: &mut Probe);";

const TESTFUNCTION_ARGUMENT_LENGTH: usize = 2;
const TESTFUNCTION_ARGUMENT_IDENT: [&str; 2] = ["test_channel", "probe"];
const TESTFUNCTION_ARGUMENT_TYPE: [&str; 2] = ["TestChannelHandle", "Probe"];

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
///fn my_fancy_test(test_channel: &mut dyn TestChannelHandle, probe: &mut Probe) {
///    let _channel = test_channel;
///    let _probe = probe;
///
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
///fn my_fancy_test(test_channel: &mut dyn TestChannelHandle, probe: &mut Probe) {
///    let _channel = test_channel;
///    let _probe = probe;
///
///    // Intentional panic
///    panic!();
///}
/// ```
#[proc_macro_attribute]
#[proc_macro_error]
pub fn hive_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    // parse macro attributes
    let attributes = match syn::parse_macro_input::parse::<AttributeArgs>(attr) {
        Ok(r) => r,
        Err(err) => abort!(
            err.to_compile_error(), "Failed to parse macro attributes";
            help = "Attributes should be written in 'key = value' pairs, separated by comma.";
            example = "example: {}", ATTRIBUTE_EXAMPLE
        ),
    };

    let args = match TestOptions::from_list(&attributes) {
        Ok(v) => v,
        Err(err) => abort!(
            err.write_errors(), "Failed to parse supplied macro attribute keys.";
            help = "The following attributes and datatypes are allowed in this context: {}", ATTRIBUTE_KEYS;
            example = "example: {}", ATTRIBUTE_EXAMPLE
        ),
    };

    let order = args.order.unwrap_or(0);
    let panics = args.should_panic.unwrap_or(false);

    // parse testfunction
    let input = match syn::parse_macro_input::parse::<ItemFn>(item) {
        Ok(r) => r,
        Err(err) => abort!(
            err.to_compile_error(), "Failed to parse testfunction";
            help = "This macro can only be applied to a Hive testfunction, see the example below.";
            example = "example: {}", FUNCTION_EXAMPLE
        ),
    };

    // Check fn visibility
    if input.vis != Visibility::Inherited {
        abort!(
            input.span(), "Wrong testfunction visibility";
            help = "Hive testfunctions should be private and not have any special visibility parameters.";
            example = "example: {}", FUNCTION_EXAMPLE
        );
    }

    // Check fn return value
    if input.sig.output != ReturnType::Default {
        abort!(
            input.sig.output.span(), "Non () return type found";
            help = "Hive testfunctions should not have any return value.";
            example = "example: {}", FUNCTION_EXAMPLE
        );
    }

    // Check fn argument amount
    if input.sig.inputs.len() != TESTFUNCTION_ARGUMENT_LENGTH {
        abort!(
            input.sig.inputs.span(), "Incorrect amount of function arguments";
            help = "Hive testfunctions should have {} arguments but {} were supplied.", TESTFUNCTION_ARGUMENT_LENGTH, input.sig.inputs.len();
            example = "example: {}", FUNCTION_EXAMPLE
        );
    }

    // Check argument identifier and type
    for i in 0..TESTFUNCTION_ARGUMENT_LENGTH {
        // Check that no self is used
        if let FnArg::Typed(ref input) = input.sig.inputs[i] {
            check_fn_arg_ident(i, input);
            check_fn_arg_type(i, input);
        } else {
            abort!(
                input.sig.inputs[i].span(), "Function should not take self as argument.";
                help = "Hive testfunctions do not use self, see the example below.";
                example = "example: {}", FUNCTION_EXAMPLE
            );
        }
    }

    let test_fn_name = input.sig.ident.to_string();
    let test_fn_ident = syn::Ident::new(&test_fn_name, Span::call_site());

    let new_code = quote! {
        #input
        inventory::submit! {
            hive_test::HiveTestFunction{
                name: #test_fn_name,
                ordered: #order,
                should_panic: #panics,
                test_fn: #test_fn_ident,
            }
        }
    };

    new_code.into()
}

/// Checks the provided function argument identifiers against the given identifiers in [`TESTFUNCTION_ARGUMENT_IDENT`]
fn check_fn_arg_ident(pos: usize, input: &PatType) {
    if let Pat::Ident(ref path) = *input.pat {
        if path.ident == TESTFUNCTION_ARGUMENT_IDENT[pos] {
            return;
        }
    }
    abort!(
        input.span(), "Wrong function argument name";
        help = "For better readability Hive testfunctions should have the same argument names as shown in the example below.";
        example = "example: {}", FUNCTION_EXAMPLE
    );
}

/// Checks the provided function argument types against the given types in [`TESTFUNCTION_ARGUMENT_TYPE`]
fn check_fn_arg_type(pos: usize, input: &PatType) {
    if let Type::Reference(ref reference) = *input.ty {
        if let Some(ref lifetime) = reference.lifetime {
            abort!(
                lifetime.span(), "No reference lifetime allowed";
                help = "References inside Hive testfunction arguments do not need any lifetime specification.";
                example = "example: {}", FUNCTION_EXAMPLE
            );
        }

        if reference.mutability.is_none() {
            abort!(
                reference.span(), "Function arguments should be mutable";
                help = "All arguments in a Hive testfunction are mutable references.";
                example = "example: {}", FUNCTION_EXAMPLE
            );
        }

        if let Type::Path(ref path) = *reference.elem {
            if !path.path.is_ident(TESTFUNCTION_ARGUMENT_TYPE[pos]) {
                abort!(
                    path.path.span(), "Wrong function argument type";
                    help = "Only the argument types listed in the example below are valid types to use in a Hive testfunction.";
                    example = "example: {}", FUNCTION_EXAMPLE
                );
            }
        }
    } else {
        abort!(
            input.ty.span(), "Function arguments should be accessed by reference";
            help = "All arguments in a Hive testfunction are accessed by reference and not by type.";
            example = "example: {}", FUNCTION_EXAMPLE
        );
    }
}
