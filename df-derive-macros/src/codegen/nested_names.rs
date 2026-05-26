use crate::ir::NestedNamePolicy;
use proc_macro2::TokenStream;
use quote::quote;

pub(in crate::codegen) fn compose_nested_name(
    policy: &NestedNamePolicy,
    parent_name: &str,
    inner_name: &TokenStream,
) -> TokenStream {
    match policy {
        NestedNamePolicy::Field => quote! {
            ::std::format!("{}.{}", #parent_name, #inner_name)
        },
        NestedNamePolicy::Flatten => quote! {
            ::std::string::ToString::to_string(&(#inner_name))
        },
        NestedNamePolicy::Prefix(prefix) => quote! {
            ::std::format!("{}.{}", #prefix, #inner_name)
        },
    }
}
