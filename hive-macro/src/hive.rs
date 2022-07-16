//! Logic for the hive macro
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error::abort;
use quote::quote;
use syn::{
    spanned::Spanned, AttributeArgs, Item, ItemConst, ItemMacro, ItemMod, ItemStruct, ItemUse,
    UseTree, Visibility,
};

const MODULE_EXAMPLE: &str = "\n\n#[hive]\npub mod tests {\n\t// Testfunctions etc...\n}";
/// The mandatory name of the Hive test top level module which is annotated with the `#[hive]` macro
const TOP_LEVEL_MODULE_NAME: &str = "tests";
/// List of allowed crates as dependencies in Hive test modules
const ALLOWED_DEPENDENCIES: [&str; 3] = ["std", "hive_test", "probe_rs_test"];

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

    // Check if a module body exists
    if input.content.is_none() {
        abort!(
            input.span(), "Top level module for Hive must have a body";
            help = "Add a module body";
            example = "example: {}", MODULE_EXAMPLE
        );
    }

    let mut path_tree = vec![];
    insert_module_path_and_check_dependencies(&mut input, 0, &mut path_tree);

    check_test_module_dependencies(&input.content.as_ref().unwrap().1);

    let test_fn_declaration = syn::parse_macro_input::parse::<ItemStruct>(
        quote! {
                pub struct HiveTestFunction<Session> {
                    pub name: &'static str,
                    pub module_path: &'static str,
                    pub ordered: usize,
                    pub should_panic: bool,
                    pub test_fn: fn(
                        test_channel: &mut dyn TestChannelHandle,
                        session: &mut Session,
                        target_info: &HiveTargetInfo,
                        defines: &DefineRegistry,
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

/// Recursively insert module paths and HiveTestFunction use declarations as well as check the dependencies of each encountered module
fn insert_module_path_and_check_dependencies(
    module: &mut ItemMod,
    depth: usize,
    path_tree: &mut Vec<String>,
) {
    if let Some((_, ref mut content)) = module.content {
        check_test_module_dependencies(content);

        if depth != 0 {
            content.push(get_hive_test_fn_use().into());
        }

        content.push(get_module_path(depth, module.ident.to_string(), path_tree).into());

        for item in content.iter_mut() {
            if let Item::Mod(module) = item {
                let new_depth = depth + 1;
                insert_module_path_and_check_dependencies(module, new_depth, path_tree);
            }
        }
    }
}

/// Returns a constant &str which contains the current module path up to the top level test module which is used by the hive_test macro to locate the individual test functions in a module context
fn get_module_path(depth: usize, module_name: String, path_tree: &mut Vec<String>) -> ItemConst {
    if path_tree.len() <= depth {
        path_tree.push(module_name);
    } else {
        path_tree[depth] = module_name;
    }

    let mut path = String::new();
    for idx in 0..=depth {
        if idx != 0 {
            path.push_str("::");
        }

        path.push_str(path_tree[idx].as_str());
    }

    syn::parse_macro_input::parse::<ItemConst>(
        quote! {
            const MODULE_PATH: &str = #path;
        }
        .into(),
    )
    .unwrap()
}

/// Get the use declaration for the HiveTestFn struct
fn get_hive_test_fn_use() -> ItemUse {
    syn::parse_macro_input::parse::<ItemUse>(
        quote! {
            use crate::hive::tests::HiveTestFunction;
        }
        .into(),
    )
    .unwrap()
}

/// Checks the dependencies of the provided test module and aborts in case the user is using dependencies which are not available on the Hive test runner.
///
/// This prevents any build errors on the testserver which would be caused in such a situation
fn check_test_module_dependencies(items: &[Item]) {
    for item in items.iter() {
        if let Item::Use(dependency) = item {
            let (is_allowed, name, span) = is_allowed_dependency(&dependency.tree);

            if !is_allowed {
                let name = name.unwrap();

                if name == "probe_rs" {
                    abort!(
                        span.unwrap(), "The dependency '{}' is not allowed in Hive test modules", name;
                        help = "Use the alias 'probe_rs_test' instead";
                        example = "example: use probe_rs_test::Session;"
                    )
                }

                abort!(
                    span.unwrap(), "The dependency '{}' is not allowed in Hive test modules", name;
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
