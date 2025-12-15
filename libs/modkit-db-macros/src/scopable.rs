use heck::ToUpperCamelCase;
use proc_macro2::{Span, TokenStream};
use proc_macro_error2::{abort, emit_error};
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput};

/// Configuration parsed from `#[secure(...)]` attributes
#[derive(Default)]
struct SecureConfig {
    // Tenant dimension
    tenant_col: Option<(String, Span)>,
    no_tenant: Option<Span>,

    // Resource dimension
    resource_col: Option<(String, Span)>,
    no_resource: Option<Span>,

    // Owner dimension
    owner_col: Option<(String, Span)>,
    no_owner: Option<Span>,

    // Type dimension
    type_col: Option<(String, Span)>,
    no_type: Option<Span>,

    // Unrestricted flag
    unrestricted: Option<Span>,
}

#[allow(clippy::needless_pass_by_value)] // DeriveInput is consumed by proc-macro pattern
pub fn expand_derive_scopable(input: DeriveInput) -> TokenStream {
    // Verify this is a struct
    if !matches!(&input.data, Data::Struct(_)) {
        abort!(
            input.span(),
            "#[derive(Scopable)] can only be applied to structs"
        );
    }

    // Parse #[secure(...)] attributes
    let config = parse_secure_attrs(&input);

    // Validate configuration
    validate_config(&config, &input);

    let entity_ident = syn::Ident::new("Entity", input.ident.span());

    // If unrestricted, generate simple implementation with all None
    if config.unrestricted.is_some() {
        return quote! {
            impl ::modkit_db::secure::ScopableEntity for #entity_ident {
                const IS_UNRESTRICTED: bool = true;

                fn tenant_col() -> ::core::option::Option<Self::Column> {
                    ::core::option::Option::None
                }

                fn resource_col() -> ::core::option::Option<Self::Column> {
                    ::core::option::Option::None
                }

                fn owner_col() -> ::core::option::Option<Self::Column> {
                    ::core::option::Option::None
                }

                fn type_col() -> ::core::option::Option<Self::Column> {
                    ::core::option::Option::None
                }
            }
        };
    }

    // Generate tenant_col implementation
    let tenant_col_impl =
        generate_col_impl("tenant_col", config.tenant_col.as_ref(), input.ident.span());

    // Generate resource_col implementation
    let resource_col_impl = generate_col_impl(
        "resource_col",
        config.resource_col.as_ref(),
        input.ident.span(),
    );

    // Generate owner_col implementation
    let owner_col_impl =
        generate_col_impl("owner_col", config.owner_col.as_ref(), input.ident.span());

    // Generate type_col implementation
    let type_col_impl = generate_col_impl("type_col", config.type_col.as_ref(), input.ident.span());

    // Generate the implementation
    quote! {
        impl ::modkit_db::secure::ScopableEntity for #entity_ident {
            const IS_UNRESTRICTED: bool = false;

            #tenant_col_impl

            #resource_col_impl

            #owner_col_impl

            #type_col_impl
        }
    }
}

/// Generate a column method implementation
fn generate_col_impl(
    method_name: &str,
    col: Option<&(String, Span)>,
    default_span: Span,
) -> TokenStream {
    let method_ident = syn::Ident::new(method_name, default_span);

    if let Some((col_name, _)) = col {
        let col_variant = snake_to_upper_camel(col_name);
        let col_ident = syn::Ident::new(&col_variant, default_span);
        quote! {
            fn #method_ident() -> ::core::option::Option<Self::Column> {
                ::core::option::Option::Some(Self::Column::#col_ident)
            }
        }
    } else {
        quote! {
            fn #method_ident() -> ::core::option::Option<Self::Column> {
                ::core::option::Option::None
            }
        }
    }
}

/// Validate the configuration for strict compile-time checks
fn validate_config(config: &SecureConfig, input: &DeriveInput) {
    let struct_span = input.span();

    // If unrestricted is set, no other attributes should be present
    if let Some(unrestricted_span) = config.unrestricted {
        let has_error = [
            config.tenant_col.as_ref().map(|(_, span)| {
                emit_error!(
                    unrestricted_span,
                    "Cannot use 'unrestricted' with 'tenant_col'";
                    note = *span => "tenant_col defined here"
                );
            }),
            config.no_tenant.as_ref().map(|span| {
                emit_error!(
                    unrestricted_span,
                    "Cannot use 'unrestricted' with 'no_tenant'";
                    note = *span => "no_tenant defined here"
                );
            }),
            config.resource_col.as_ref().map(|(_, span)| {
                emit_error!(
                    unrestricted_span,
                    "Cannot use 'unrestricted' with 'resource_col'";
                    note = *span => "resource_col defined here"
                );
            }),
            config.no_resource.as_ref().map(|span| {
                emit_error!(
                    unrestricted_span,
                    "Cannot use 'unrestricted' with 'no_resource'";
                    note = *span => "no_resource defined here"
                );
            }),
            config.owner_col.as_ref().map(|(_, span)| {
                emit_error!(
                    unrestricted_span,
                    "Cannot use 'unrestricted' with 'owner_col'";
                    note = *span => "owner_col defined here"
                );
            }),
            config.no_owner.as_ref().map(|span| {
                emit_error!(
                    unrestricted_span,
                    "Cannot use 'unrestricted' with 'no_owner'";
                    note = *span => "no_owner defined here"
                );
            }),
            config.type_col.as_ref().map(|(_, span)| {
                emit_error!(
                    unrestricted_span,
                    "Cannot use 'unrestricted' with 'type_col'";
                    note = *span => "type_col defined here"
                );
            }),
            config.no_type.as_ref().map(|span| {
                emit_error!(
                    unrestricted_span,
                    "Cannot use 'unrestricted' with 'no_type'";
                    note = *span => "no_type defined here"
                );
            }),
        ]
        .iter()
        .any(Option::is_some);

        if has_error {
            abort!(
                unrestricted_span,
                "When using 'unrestricted', no other column attributes are allowed"
            );
        }
        return; // Valid unrestricted config
    }

    // Check each scope dimension has exactly one option
    validate_dimension(
        "tenant",
        config.tenant_col.as_ref(),
        config.no_tenant,
        struct_span,
    );
    validate_dimension(
        "resource",
        config.resource_col.as_ref(),
        config.no_resource,
        struct_span,
    );
    validate_dimension(
        "owner",
        config.owner_col.as_ref(),
        config.no_owner,
        struct_span,
    );
    validate_dimension(
        "type",
        config.type_col.as_ref(),
        config.no_type,
        struct_span,
    );
}

/// Validate a single dimension has exactly one specification
fn validate_dimension(
    name: &str,
    col: Option<&(String, Span)>,
    no_col: Option<Span>,
    struct_span: Span,
) {
    match (col, &no_col) {
        (None, None) => {
            // Missing explicit decision
            let msg = format!(
                "secure: missing explicit decision for {name}:\n  \
                 use `{name}_col = \"column_name\"` or `no_{name}`"
            );
            abort!(struct_span, msg);
        }
        (Some((_, col_span)), Some(no_span)) => {
            // Both specified
            let msg = format!("secure: conflicting attributes for {name}");
            let note_msg = format!("no_{name} also defined here");
            emit_error!(
                *col_span,
                msg;
                note = *no_span => note_msg
            );
            let abort_msg = format!("secure: specify either `{name}_col` or `no_{name}`, not both");
            abort!(struct_span, abort_msg);
        }
        _ => {
            // Valid: exactly one is specified
        }
    }
}

/// Parse all `#[secure(...)]` attributes with duplicate detection
fn parse_secure_attrs(input: &DeriveInput) -> SecureConfig {
    let mut config = SecureConfig::default();

    for attr in &input.attrs {
        if !attr.path().is_ident("secure") {
            continue;
        }

        let result = attr.parse_nested_meta(|meta| {
            let span = meta.path.span();

            // Check if this is a flag (no_* or unrestricted)
            if meta.path.is_ident("unrestricted") {
                if let Some(prev_span) = config.unrestricted {
                    abort!(
                        span,
                        "duplicate attribute 'unrestricted'";
                        note = prev_span => "first defined here"
                    );
                }
                config.unrestricted = Some(span);
                return Ok(());
            }

            if meta.path.is_ident("no_tenant") {
                if let Some(prev_span) = config.no_tenant {
                    abort!(
                        span,
                        "duplicate attribute 'no_tenant'";
                        note = prev_span => "first defined here"
                    );
                }
                config.no_tenant = Some(span);
                return Ok(());
            }

            if meta.path.is_ident("no_resource") {
                if let Some(prev_span) = config.no_resource {
                    abort!(
                        span,
                        "duplicate attribute 'no_resource'";
                        note = prev_span => "first defined here"
                    );
                }
                config.no_resource = Some(span);
                return Ok(());
            }

            if meta.path.is_ident("no_owner") {
                if let Some(prev_span) = config.no_owner {
                    abort!(
                        span,
                        "duplicate attribute 'no_owner'";
                        note = prev_span => "first defined here"
                    );
                }
                config.no_owner = Some(span);
                return Ok(());
            }

            if meta.path.is_ident("no_type") {
                if let Some(prev_span) = config.no_type {
                    abort!(
                        span,
                        "duplicate attribute 'no_type'";
                        note = prev_span => "first defined here"
                    );
                }
                config.no_type = Some(span);
                return Ok(());
            }

            // Key-value pair
            let key = meta
                .path
                .get_ident()
                .map(ToString::to_string)
                .unwrap_or_default();

            if key.is_empty() {
                abort!(span, "Expected attribute name");
            }

            let value: String = match meta.value() {
                Ok(v) => match v.parse::<syn::LitStr>() {
                    Ok(lit) => lit.value(),
                    Err(_) => abort!(span, "Expected string literal"),
                },
                Err(_) => abort!(span, "Expected '=' followed by a string value"),
            };

            match key.as_str() {
                "tenant_col" => {
                    if let Some((_, prev_span)) = config.tenant_col {
                        abort!(
                            span,
                            "duplicate attribute 'tenant_col'";
                            note = prev_span => "first defined here"
                        );
                    }
                    config.tenant_col = Some((value, span));
                }
                "resource_col" => {
                    if let Some((_, prev_span)) = config.resource_col {
                        abort!(
                            span,
                            "duplicate attribute 'resource_col'";
                            note = prev_span => "first defined here"
                        );
                    }
                    config.resource_col = Some((value, span));
                }
                "owner_col" => {
                    if let Some((_, prev_span)) = config.owner_col {
                        abort!(
                            span,
                            "duplicate attribute 'owner_col'";
                            note = prev_span => "first defined here"
                        );
                    }
                    config.owner_col = Some((value, span));
                }
                "type_col" => {
                    if let Some((_, prev_span)) = config.type_col {
                        abort!(
                            span,
                            "duplicate attribute 'type_col'";
                            note = prev_span => "first defined here"
                        );
                    }
                    config.type_col = Some((value, span));
                }
                _ => {
                    abort!(
                        span,
                        "Unknown attribute '{}'. Valid attributes: tenant_col, no_tenant, resource_col, no_resource, owner_col, no_owner, type_col, no_type, unrestricted",
                        key
                    );
                }
            }

            Ok(())
        });

        if let Err(err) = result {
            emit_error!(err.span(), "{}", err);
        }
    }

    config
}

/// Convert `snake_case` to `UpperCamelCase` for enum variant names
fn snake_to_upper_camel(s: &str) -> String {
    s.to_upper_camel_case()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_snake_to_upper_camel() {
        assert_eq!(snake_to_upper_camel("tenant_id"), "TenantId");
        assert_eq!(snake_to_upper_camel("id"), "Id");
        assert_eq!(snake_to_upper_camel("owner_user_id"), "OwnerUserId");
        assert_eq!(snake_to_upper_camel("custom_col"), "CustomCol");
    }
}
