//! Logic for the hive macro
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::quote;
use syn::{
    spanned::Spanned, AttributeArgs, Item, ItemMacro, ItemMod, ItemStruct, UseTree, Visibility,
};

const MODULE_EXAMPLE: &str = "\n\n#[hive]\npub mod tests {\n\t// Testfunctions etc...\n}";
/// The mandatory name of the Hive test top level module which is annotated with the `#[hive]` macro
const TOP_LEVEL_MODULE_NAME: &str = "tests";
/// List of allowed crates as dependencies in Hive test modules
const ALLOWED_DEPENDENCIES: [&str; 4] = ["std", "probe_rs", "hive_test", "probe_rs_test"];

pub fn run(attr: TokenStream, item: TokenStream) -> TokenStream {
    // parse macro attributes
    let attributes = match syn::parse_macro_input::parse::<AttributeArgs>(attr) {
        Ok(r) => r,
        Err(err) => abort!(
            err.to_compile_error(), "Failed to parse macro attributes";
            help = "Attributes should be written in 'key = value' pairs, separated by comma.";
        ),
    };

    if !attributes.is_empty() {
        abort!(
            attributes[0].span(), "This macro does not support attributes";
            help = "Remove any macro attributes"
        )
    }

    // parse module
    let mut input = match syn::parse_macro_input::parse::<ItemMod>(item) {
        Ok(r) => r,
        Err(err) => abort!(
            err.to_compile_error(), "Failed to parse module";
            help = "This macro can only be applied to a module, see the example below.";
            example = "example: {}", MODULE_EXAMPLE
        ),
    };

    // Check top-level module name
    if input.ident.to_string() != TOP_LEVEL_MODULE_NAME {
        abort!(
            input.ident.span(), "Top level module for Hive tests must be called {}", TOP_LEVEL_MODULE_NAME;
            help = "Rename this module to {}", TOP_LEVEL_MODULE_NAME;
            example = "example: {}", MODULE_EXAMPLE
        );
    }

    // Check top-level module visibility
    if let Visibility::Public(_) = input.vis {
    } else {
        abort!(
            input.vis.span(), "Top level module for Hive tests must be public";
            help = "Set the module visibility to public";
            example = "example: {}", MODULE_EXAMPLE
        );
    }

    check_test_module_dependencies(&input.content.as_ref().unwrap().1);

    let test_fn_declaration = syn::parse_macro_input::parse::<ItemStruct>(
        quote! {
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
        }
        .into(),
    )
    .unwrap();

    let test_fn_registration = syn::parse_macro_input::parse::<ItemMacro>(
        quote! {
            inventory::collect!(HiveTestFunction<Session>);
        }
        .into(),
    )
    .unwrap();

    if let Some(ref mut content) = input.content {
        content.1.push(test_fn_declaration.into());
        content.1.push(test_fn_registration.into());
    } else {
        abort!(
            input.span(), "Cannot apply macro to modules which are declared without body";
            help = "This macro can only be applied to a module which has a body, see the example below.";
            example = "example: {}", MODULE_EXAMPLE
        )
    }

    quote!(#input).into()
}

/// Checks the dependencies of the provided test module and aborts in case the user is using dependencies which are not available on the Hive test runner.
///
/// This prevents any build errors on the testserver which would be caused in such a situation
fn check_test_module_dependencies(items: &[Item]) {
    for item in items.iter() {
        if let Item::Use(dependency) = item {
            let (is_allowed, name, span) = is_allowed_dependency(&dependency.tree);

            if !is_allowed {
                abort!(
                    span.unwrap(), "The dependency '{}' is not allowed in Hive test modules", name.unwrap();
                    help = "The Hive testserver does not support this dependency. If it is required nonetheless, please file an issue"
                )
            }
        }
    }
}

/// Check if provided dependency tree only contains allowed dependencies recursively.
///
/// The return value is a tuple with a bool indicating if all encountered dependencies were allowed or not.
/// In case any unallowed dependencies were found the option contains the name of the unallowed dependency and the span of it.
fn is_allowed_dependency(tree: &UseTree) -> (bool, Option<String>, Option<Span>) {
    match tree {
        syn::UseTree::Path(ref path) => (
            ALLOWED_DEPENDENCIES.contains(&path.ident.to_string().as_str()),
            Some(path.ident.to_string()),
            Some(path.ident.span()),
        ),
        syn::UseTree::Name(ref name) => (
            ALLOWED_DEPENDENCIES.contains(&name.ident.to_string().as_str()),
            Some(name.ident.to_string()),
            Some(name.ident.span()),
        ),
        syn::UseTree::Rename(ref rename) => (
            ALLOWED_DEPENDENCIES.contains(&rename.ident.to_string().as_str()),
            Some(rename.ident.to_string()),
            Some(rename.ident.span()),
        ),
        syn::UseTree::Group(ref group) => {
            for tree in group.items.iter() {
                let result = is_allowed_dependency(tree);

                if !result.0 {
                    return result;
                }
            }

            (true, None, None)
        }
        _ => unreachable!(),
    }
}
