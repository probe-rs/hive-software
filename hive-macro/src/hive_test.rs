//! Contains the logic for the hive_test macro
use darling::FromMeta;
use proc_macro::TokenStream;
use proc_macro_error::abort;
use proc_macro2::Span;
use quote::quote;
use syn::{
    AttributeArgs, FnArg, ItemFn, Pat, PatType, ReturnType, Type, Visibility, spanned::Spanned,
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
const FUNCTION_EXAMPLE: &str = "fn my_fancy_test(test_channel: &mut dyn TestChannelHandle, session: &mut Session, target_info: &HiveTargetInfo, defines: &DefineRegistry);";

const TESTFUNCTION_ARGUMENT_LENGTH: usize = 4;
const TESTFUNCTION_ARGUMENT_IDENT: [&str; TESTFUNCTION_ARGUMENT_LENGTH] =
    ["test_channel", "session", "target_info", "defines"];
const TESTFUNCTION_ARGUMENT_TYPE: [&str; TESTFUNCTION_ARGUMENT_LENGTH] = [
    "TestChannelHandle",
    "Session",
    "HiveTargetInfo",
    "DefineRegistry",
];
/// Whether the argument at the respective position should be mutable or not
const TESTFUNCTION_ARGUMENT_REFERENCE_MUTABILITY: [bool; TESTFUNCTION_ARGUMENT_LENGTH] =
    [true, true, false, false];

pub fn run(attr: TokenStream, item: TokenStream) -> TokenStream {
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
            HiveTestFunction{
                name: #test_fn_name,
                module_path: MODULE_PATH,
                ordered: #order,
                should_panic: #panics,
                test_fn: #test_fn_ident,
            }
        }
    };

    new_code.into()
}

/// Checks the provided function argument identifiers against the given identifiers in [`TESTFUNCTION_ARGUMENT_IDENT`], also allows _ for unused
fn check_fn_arg_ident(pos: usize, input: &PatType) {
    if let Pat::Ident(ref path) = *input.pat {
        if path.ident == TESTFUNCTION_ARGUMENT_IDENT[pos] || path.ident.to_string().starts_with('_')
        {
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

        if reference.mutability.is_none() && TESTFUNCTION_ARGUMENT_REFERENCE_MUTABILITY[pos] {
            abort!(
                reference.span(), "Function argument should be mutable";
                help = "This argument in a Hive testfunction is a mutable reference.";
                example = "example: {}", FUNCTION_EXAMPLE
            );
        } else if reference.mutability.is_some() && !TESTFUNCTION_ARGUMENT_REFERENCE_MUTABILITY[pos]
        {
            abort!(
                reference.span(), "Function argument should not be mutable";
                help = "This argument in a Hive testfunction is an immutable reference.";
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
            help = "All arguments in a Hive testfunction are accessed by reference and not by value.";
            example = "example: {}", FUNCTION_EXAMPLE
        );
    }
}
