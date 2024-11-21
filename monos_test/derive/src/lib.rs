use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn kernel_test(_attribute: TokenStream, item: TokenStream) -> TokenStream {
    let expanded = expand(parse_macro_input!(item));

    TokenStream::from(expanded)
}

fn expand(test_fn: ItemFn) -> proc_macro2::TokenStream {
    let fn_name_ident = &test_fn.sig.ident;
    let fn_name = fn_name_ident.to_string();

    let description_name = format_ident!("__KERNEL_TEST_{}", fn_name);

    let name = quote! { #fn_name };

    let test_location = quote! {
        monos_test::Location {
            module: module_path!(),
            file: file!(),
            line: line!(),
            column: column!(),
        }
    };

    quote! {
        #test_fn

        #[linkme::distributed_slice(monos_test::KERNEL_TESTS)]
        static #description_name: monos_test::TestDescription = monos_test::TestDescription {
            name: #name,
            test_fn: #fn_name_ident,
            location: #test_location,
        };
    }
}
