// Schema/empty-frame helpers used by `to_dataframe::schema` and
// `empty_dataframe`.

use proc_macro2::TokenStream;
use quote::quote;

use crate::ir::NestedNamePolicy;

use super::encoder::idents;
use super::external_paths::ExternalPaths;

#[derive(Clone, Copy)]
enum EmitMode {
    SchemaEntries,
    EmptyRows,
}

/// Emit a runtime loop that wraps the per-iteration `DataType` accumulator
/// (named via [`idents::schema_wrapped_dtype`]) in `layers` `List<>` envelopes.
/// Thin wrapper over [`super::external_paths::wrap_list_layers_runtime`] that
/// pins the wrapped-variable ident to the schema-helpers' shared local.
fn gen_wrap_dtype_layers(layers: usize, paths: &ExternalPaths) -> TokenStream {
    let pp = paths.prelude();
    let wrapped = idents::schema_wrapped_dtype();
    super::external_paths::wrap_list_layers_runtime(pp, &wrapped, layers)
}

pub fn nested_empty_series_row(
    type_path: &TokenStream,
    to_df_trait: &syn::Path,
    name: &str,
    name_policy: &NestedNamePolicy,
    list_layers: usize,
    paths: &ExternalPaths,
) -> TokenStream {
    generate_for_struct(
        type_path,
        to_df_trait,
        name,
        name_policy,
        list_layers,
        EmitMode::EmptyRows,
        paths,
    )
}

// --- Schema and series-shape helpers ---

pub fn generate_schema_entries_for_struct(
    type_path: &TokenStream,
    to_df_trait: &syn::Path,
    column_name: &str,
    name_policy: &NestedNamePolicy,
    list_layers: usize,
    paths: &ExternalPaths,
) -> TokenStream {
    generate_for_struct(
        type_path,
        to_df_trait,
        column_name,
        name_policy,
        list_layers,
        EmitMode::SchemaEntries,
        paths,
    )
}

/// Shared runtime emitter for the nested schema-entries / empty-rows pair.
/// Both emissions iterate `T::schema()?`, prefix the inner name with the
/// outer field name, build a per-iteration runtime `DataType` wrapped in
/// `list_layers` `List<>` envelopes, and push the result into a per-mode
/// accumulator. Only the accumulator type/name and the per-iteration push
/// expression vary, captured by [`EmitMode`].
fn generate_for_struct(
    type_path: &TokenStream,
    to_df_trait: &syn::Path,
    column_name: &str,
    name_policy: &NestedNamePolicy,
    list_layers: usize,
    mode: EmitMode,
    paths: &ExternalPaths,
) -> TokenStream {
    let pp = paths.prelude();
    let wrap_layers = gen_wrap_dtype_layers(list_layers, paths);
    let wrapped = idents::schema_wrapped_dtype();
    let output_name =
        super::nested_names::compose_nested_name(name_policy, column_name, &quote! { inner_name });
    match mode {
        EmitMode::SchemaEntries => quote! {
            {
                let mut nested_fields: ::std::vec::Vec<(::std::string::String, #pp::DataType)> = ::std::vec::Vec::new();
                for (inner_name, inner_dtype) in <#type_path as #to_df_trait>::schema()? {
                    let output_name = #output_name;
                    let mut #wrapped: #pp::DataType = inner_dtype;
                    #wrap_layers
                    nested_fields.push((output_name, #wrapped));
                }
                nested_fields
            }
        },
        EmitMode::EmptyRows => quote! {
            {
                let mut nested_series: ::std::vec::Vec<#pp::Column> = ::std::vec::Vec::new();
                for (inner_name, inner_dtype) in <#type_path as #to_df_trait>::schema()? {
                    let output_name = #output_name;
                    let mut #wrapped: #pp::DataType = inner_dtype;
                    #wrap_layers
                    let empty_series = #pp::Series::new_empty(output_name.as_str().into(), &#wrapped);
                    nested_series.push(empty_series.into());
                }
                nested_series
            }
        },
    }
}
