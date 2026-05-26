use crate::ir::DateTimeUnit;
use proc_macro2::Span;
use syn::spanned::Spanned as SynSpanned;

use super::Spanned;
use super::decimal::parse_decimal_attr;
use super::field_conflicts::{FieldAttr, set_override};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlattenConfig {
    pub prefix: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LeafOverride {
    AsStr,
    AsString,
    Decimal { precision: u8, scale: u8 },
    TimeUnit(DateTimeUnit),
}

#[derive(Clone, Debug)]
pub enum FieldDisposition {
    Skip,
    Include(FieldConversion),
}

#[derive(Clone, Debug)]
pub enum FieldConversion {
    Default,
    LeafOverride(Spanned<LeafOverride>),
    Binary { span: Span },
    Flatten(Spanned<FlattenConfig>),
}

fn parse_time_unit_attr(meta: &syn::meta::ParseNestedMeta<'_>) -> Result<DateTimeUnit, syn::Error> {
    let lit: syn::LitStr = meta.value()?.parse()?;
    match lit.value().as_str() {
        "ms" => Ok(DateTimeUnit::Milliseconds),
        "us" => Ok(DateTimeUnit::Microseconds),
        "ns" => Ok(DateTimeUnit::Nanoseconds),
        other => Err(syn::Error::new_spanned(
            &lit,
            format!("invalid `time_unit` value `{other}`; expected one of \"ms\", \"us\", \"ns\""),
        )),
    }
}

fn duplicate_flatten_prefix_error(
    existing: (String, Span),
    incoming_value: &str,
    incoming_span: Span,
) -> syn::Error {
    let (existing_value, existing_span) = existing;
    let message = if existing_value == incoming_value {
        "`flatten(...)` declares duplicate `prefix` key; remove one".to_owned()
    } else {
        format!(
            "`flatten(...)` declares duplicate `prefix` keys with different values; \
             first is `{existing_value}`, second is `{incoming_value}`; pick one"
        )
    };

    let mut error = syn::Error::new(incoming_span, message);
    error.combine(syn::Error::new(
        existing_span,
        "first `prefix` key declared here",
    ));
    error
}

fn parse_flatten_attr(meta: &syn::meta::ParseNestedMeta<'_>) -> Result<FlattenConfig, syn::Error> {
    if !meta.input.peek(syn::token::Paren) {
        return Ok(FlattenConfig { prefix: None });
    }

    let mut prefix: Option<(String, Span)> = None;
    meta.parse_nested_meta(|sub| {
        if sub.path.is_ident("prefix") {
            let key_span = sub.path.span();
            let lit: syn::LitStr = sub.value()?.parse()?;
            let value = lit.value();
            if value.is_empty() {
                return Err(syn::Error::new_spanned(
                    lit,
                    "`flatten(prefix = \"...\")` requires a non-empty prefix",
                ));
            }
            if let Some(existing) = prefix.take() {
                return Err(duplicate_flatten_prefix_error(existing, &value, key_span));
            }
            prefix = Some((value, key_span));
            Ok(())
        } else {
            Err(sub.error("unknown key inside `flatten(...)`; expected `prefix = \"...\"`"))
        }
    })?;

    Ok(FlattenConfig {
        prefix: prefix.map(|(value, _)| value),
    })
}

pub fn parse_field_disposition(
    field: &syn::Field,
    field_display_name: &str,
) -> Result<FieldDisposition, syn::Error> {
    let mut override_: Option<(FieldAttr, Span)> = None;
    for attr in &field.attrs {
        if attr.path().is_ident("df_derive") {
            attr.parse_nested_meta(|meta| {
                let incoming_span = meta.path.span();
                if meta.path.is_ident("skip") {
                    set_override(
                        field_display_name,
                        &mut override_,
                        FieldAttr::Skip,
                        incoming_span,
                    )
                } else if meta.path.is_ident("as_string") {
                    set_override(
                        field_display_name,
                        &mut override_,
                        FieldAttr::Leaf(LeafOverride::AsString),
                        incoming_span,
                    )
                } else if meta.path.is_ident("as_str") {
                    set_override(
                        field_display_name,
                        &mut override_,
                        FieldAttr::Leaf(LeafOverride::AsStr),
                        incoming_span,
                    )
                } else if meta.path.is_ident("as_binary") {
                    set_override(
                        field_display_name,
                        &mut override_,
                        FieldAttr::Binary,
                        incoming_span,
                    )
                } else if meta.path.is_ident("decimal") {
                    let (precision, scale) = parse_decimal_attr(&meta)?;
                    set_override(
                        field_display_name,
                        &mut override_,
                        FieldAttr::Leaf(LeafOverride::Decimal { precision, scale }),
                        incoming_span,
                    )
                } else if meta.path.is_ident("time_unit") {
                    let unit = parse_time_unit_attr(&meta)?;
                    set_override(
                        field_display_name,
                        &mut override_,
                        FieldAttr::Leaf(LeafOverride::TimeUnit(unit)),
                        incoming_span,
                    )
                } else if meta.path.is_ident("flatten") {
                    let config = parse_flatten_attr(&meta)?;
                    set_override(
                        field_display_name,
                        &mut override_,
                        FieldAttr::Flatten(config),
                        incoming_span,
                    )
                } else {
                    Err(meta.error(
                        "unknown key in #[df_derive(...)] field attribute; expected `skip`, `flatten`, `as_str`, `as_string`, `as_binary`, `decimal(precision = N, scale = N)`, or `time_unit = \"ms\"|\"us\"|\"ns\"`",
                    ))
                }
            })?;
        }
    }
    Ok(override_.map_or(
        FieldDisposition::Include(FieldConversion::Default),
        |(value, span)| value.into_disposition(span),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_disposition(field: &syn::Field) -> syn::Result<FieldDisposition> {
        parse_field_disposition(field, "value")
    }

    fn flatten_config(field: &syn::Field) -> FlattenConfig {
        let disposition = parse_disposition(field).expect("field disposition should parse");
        let FieldDisposition::Include(FieldConversion::Flatten(config)) = disposition else {
            panic!("field flatten config should be present");
        };
        config.value
    }

    fn leaf_override_value(field: &syn::Field) -> LeafOverride {
        let disposition = parse_disposition(field).expect("field disposition should parse");
        let FieldDisposition::Include(FieldConversion::LeafOverride(leaf_override)) = disposition
        else {
            panic!("field leaf override should be present");
        };
        leaf_override.value
    }

    #[test]
    fn parses_string_and_decimal_field_overrides() {
        let as_str = leaf_override_value(&syn::parse_quote! {
            #[df_derive(as_str)]
            value: String
        });
        assert!(matches!(as_str, LeafOverride::AsStr));

        let as_string = leaf_override_value(&syn::parse_quote! {
            #[df_derive(as_string)]
            value: DisplayType
        });
        assert!(matches!(as_string, LeafOverride::AsString));

        let decimal = leaf_override_value(&syn::parse_quote! {
            #[df_derive(decimal(precision = 10, scale = 2))]
            value: Decimal
        });
        assert!(matches!(
            decimal,
            LeafOverride::Decimal {
                precision: 10,
                scale: 2,
            }
        ));
    }

    #[test]
    fn rejects_duplicate_decimal_keys_and_bad_time_units() {
        let duplicate_decimal = parse_disposition(&syn::parse_quote! {
            #[df_derive(decimal(precision = 10, precision = 11, scale = 2))]
            value: Decimal
        });
        assert!(duplicate_decimal.is_err());

        let bad_time_unit = parse_disposition(&syn::parse_quote! {
            #[df_derive(time_unit = "bad")]
            value: chrono::NaiveDateTime
        });
        assert!(bad_time_unit.is_err());
    }

    #[test]
    fn rejects_conflicting_string_overrides() {
        let result = parse_disposition(&syn::parse_quote! {
            #[df_derive(as_str, as_string)]
            value: String
        });
        assert!(result.is_err());
    }

    #[test]
    fn conflicting_overrides_report_the_first_override() {
        let error = parse_disposition(&syn::parse_quote! {
            #[df_derive(as_str, as_string)]
            value: String
        })
        .expect_err("conflicting field overrides should fail");
        let rendered = error.into_compile_error().to_string();

        assert!(rendered.contains("has both `as_str` and `as_string`"));
        assert!(rendered.contains("first `as_str` override declared here"));
    }

    #[test]
    fn parses_flatten_with_optional_prefix() {
        let bare = flatten_config(&syn::parse_quote! {
            #[df_derive(flatten)]
            value: Key
        });
        assert_eq!(bare.prefix, None);

        let prefixed = flatten_config(&syn::parse_quote! {
            #[df_derive(flatten(prefix = "contract"))]
            value: Key
        });
        assert_eq!(prefixed.prefix.as_deref(), Some("contract"));
    }

    #[test]
    fn rejects_bad_flatten_prefix() {
        let empty = parse_disposition(&syn::parse_quote! {
            #[df_derive(flatten(prefix = ""))]
            value: Key
        });
        assert!(empty.is_err());

        let duplicate = parse_disposition(&syn::parse_quote! {
            #[df_derive(flatten(prefix = "a", prefix = "b"))]
            value: Key
        });
        assert!(duplicate.is_err());
    }
}
