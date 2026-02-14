use heck::ToUpperCamelCase;
use proc_macro_error2::abort;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, spanned::Spanned};

/// Well-known property names that are auto-derived from dimension columns.
///
/// These mirror `modkit_security::access_scope::properties` but are duplicated
/// here because proc-macro crates cannot depend on runtime crates.
const PEP_PROP_OWNER_TENANT_ID: &str = "owner_tenant_id";
const PEP_PROP_RESOURCE_ID: &str = "id";
const PEP_PROP_OWNER_ID: &str = "owner_id";

/// Reserved property names that must not be used in `pep_prop(...)`.
const RESERVED_PROPERTIES: &[(&str, &str)] = &[
    (PEP_PROP_OWNER_TENANT_ID, "tenant_col"),
    (PEP_PROP_RESOURCE_ID, "resource_col"),
    (PEP_PROP_OWNER_ID, "owner_col"),
];

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

    // Custom PEP property mappings: (property_name, column_name, span)
    pep_props: Vec<(String, String, Span)>,
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

                fn resolve_property(_property: &str) -> ::core::option::Option<Self::Column> {
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

    // Generate resolve_property implementation
    let resolve_property_impl = generate_resolve_property(&config, input.ident.span());

    // Generate the implementation
    quote! {
        impl ::modkit_db::secure::ScopableEntity for #entity_ident {
            const IS_UNRESTRICTED: bool = false;

            #tenant_col_impl

            #resource_col_impl

            #owner_col_impl

            #type_col_impl

            #resolve_property_impl
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

/// Generate the `resolve_property` match arms from dimension columns and `pep_prop` entries.
fn generate_resolve_property(config: &SecureConfig, span: Span) -> TokenStream {
    let mut arms = Vec::new();

    // Auto-map dimension columns to their well-known property names
    if let Some((col_name, _)) = &config.tenant_col {
        let col_variant = snake_to_upper_camel(col_name);
        let col_ident = syn::Ident::new(&col_variant, span);
        arms.push(quote! {
            #PEP_PROP_OWNER_TENANT_ID => ::core::option::Option::Some(Self::Column::#col_ident),
        });
    }

    if let Some((col_name, _)) = &config.resource_col {
        let col_variant = snake_to_upper_camel(col_name);
        let col_ident = syn::Ident::new(&col_variant, span);
        arms.push(quote! {
            #PEP_PROP_RESOURCE_ID => ::core::option::Option::Some(Self::Column::#col_ident),
        });
    }

    if let Some((col_name, _)) = &config.owner_col {
        let col_variant = snake_to_upper_camel(col_name);
        let col_ident = syn::Ident::new(&col_variant, span);
        arms.push(quote! {
            #PEP_PROP_OWNER_ID => ::core::option::Option::Some(Self::Column::#col_ident),
        });
    }

    // Custom pep_prop entries
    for (property, column, _) in &config.pep_props {
        let col_variant = snake_to_upper_camel(column);
        let col_ident = syn::Ident::new(&col_variant, span);
        arms.push(quote! {
            #property => ::core::option::Option::Some(Self::Column::#col_ident),
        });
    }

    quote! {
        fn resolve_property(property: &str) -> ::core::option::Option<Self::Column> {
            match property {
                #(#arms)*
                _ => ::core::option::Option::None,
            }
        }
    }
}

/// Validate the configuration for strict compile-time checks
fn validate_config(config: &SecureConfig, input: &DeriveInput) {
    let struct_span = input.span();

    // If unrestricted is set, no other attributes should be present
    if let Some(unrestricted_span) = config.unrestricted {
        let has_other = config.tenant_col.is_some()
            || config.no_tenant.is_some()
            || config.resource_col.is_some()
            || config.no_resource.is_some()
            || config.owner_col.is_some()
            || config.no_owner.is_some()
            || config.type_col.is_some()
            || config.no_type.is_some();

        if has_other {
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

    // Validate pep_prop entries
    validate_pep_props(config);
}

/// Validate `pep_prop` entries for reserved names, duplicates, and empty values.
fn validate_pep_props(config: &SecureConfig) {
    let mut seen = std::collections::HashSet::new();

    for (property, column, span) in &config.pep_props {
        // Check for reserved property names
        for (reserved, use_instead) in RESERVED_PROPERTIES {
            if property == reserved {
                abort!(
                    *span,
                    "pep_prop: '{}' is a reserved property name; use `{}` instead",
                    reserved,
                    use_instead
                );
            }
        }

        // Check for empty property or column
        if property.is_empty() {
            abort!(*span, "pep_prop: property name must not be empty");
        }
        if column.is_empty() {
            abort!(*span, "pep_prop: column name must not be empty");
        }

        // Check for duplicate property names
        if !seen.insert(property.clone()) {
            abort!(*span, "pep_prop: duplicate property name '{}'", property);
        }
    }
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
        (Some((_, col_span)), Some(_no_span)) => {
            // Both specified
            let abort_msg = format!("secure: specify either `{name}_col` or `no_{name}`, not both");
            abort!(*col_span, abort_msg);
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
                if config.unrestricted.is_some() {
                    abort!(span, "duplicate attribute 'unrestricted'");
                }
                config.unrestricted = Some(span);
                return Ok(());
            }

            if meta.path.is_ident("no_tenant") {
                if config.unrestricted.is_some() {
                    abort!(span, "Cannot use 'no_tenant' with 'unrestricted'");
                }
                if config.no_tenant.is_some() {
                    abort!(span, "duplicate attribute 'no_tenant'");
                }
                if config.tenant_col.is_some() {
                    abort!(
                        span,
                        "secure: specify either `tenant_col` or `no_tenant`, not both"
                    );
                }
                config.no_tenant = Some(span);
                return Ok(());
            }

            if meta.path.is_ident("no_resource") {
                if config.unrestricted.is_some() {
                    abort!(span, "Cannot use 'no_resource' with 'unrestricted'");
                }
                if config.no_resource.is_some() {
                    abort!(span, "duplicate attribute 'no_resource'");
                }
                if config.resource_col.is_some() {
                    abort!(
                        span,
                        "secure: specify either `resource_col` or `no_resource`, not both"
                    );
                }
                config.no_resource = Some(span);
                return Ok(());
            }

            if meta.path.is_ident("no_owner") {
                if config.unrestricted.is_some() {
                    abort!(span, "Cannot use 'no_owner' with 'unrestricted'");
                }
                if config.no_owner.is_some() {
                    abort!(span, "duplicate attribute 'no_owner'");
                }
                if config.owner_col.is_some() {
                    abort!(
                        span,
                        "secure: specify either `owner_col` or `no_owner`, not both"
                    );
                }
                config.no_owner = Some(span);
                return Ok(());
            }

            if meta.path.is_ident("no_type") {
                if config.unrestricted.is_some() {
                    abort!(span, "Cannot use 'no_type' with 'unrestricted'");
                }
                if config.no_type.is_some() {
                    abort!(span, "duplicate attribute 'no_type'");
                }
                if config.type_col.is_some() {
                    abort!(
                        span,
                        "secure: specify either `type_col` or `no_type`, not both"
                    );
                }
                config.no_type = Some(span);
                return Ok(());
            }

            // Check for pep_prop(name = "column") — nested meta with parentheses
            if meta.path.is_ident("pep_prop") {
                if config.unrestricted.is_some() {
                    abort!(span, "Cannot use 'pep_prop' with 'unrestricted'");
                }
                meta.parse_nested_meta(|pep_meta| {
                    let property = pep_meta
                        .path
                        .get_ident()
                        .map(ToString::to_string)
                        .unwrap_or_default();
                    let column: String = pep_meta.value()?.parse::<syn::LitStr>()?.value();
                    config
                        .pep_props
                        .push((property, column, pep_meta.path.span()));
                    Ok(())
                })?;
                return Ok(());
            }

            parse_key_value_attr(&mut config, meta);
            Ok(())
        });

        if let Err(err) = result {
            abort!(err.span(), "{}", err);
        }
    }

    config
}

/// Parse a key-value attribute like `tenant_col = "column_name"`.
#[allow(clippy::needless_pass_by_value)] // ParseNestedMeta is consumed by .value()
fn parse_key_value_attr(config: &mut SecureConfig, meta: syn::meta::ParseNestedMeta<'_>) {
    let span = meta.path.span();
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
            if config.unrestricted.is_some() {
                abort!(span, "Cannot use 'tenant_col' with 'unrestricted'");
            }
            if config.tenant_col.is_some() {
                abort!(span, "duplicate attribute 'tenant_col'");
            }
            if config.no_tenant.is_some() {
                abort!(
                    span,
                    "secure: specify either `tenant_col` or `no_tenant`, not both"
                );
            }
            config.tenant_col = Some((value, span));
        }
        "resource_col" => {
            if config.unrestricted.is_some() {
                abort!(span, "Cannot use 'resource_col' with 'unrestricted'");
            }
            if config.resource_col.is_some() {
                abort!(span, "duplicate attribute 'resource_col'");
            }
            if config.no_resource.is_some() {
                abort!(
                    span,
                    "secure: specify either `resource_col` or `no_resource`, not both"
                );
            }
            config.resource_col = Some((value, span));
        }
        "owner_col" => {
            if config.unrestricted.is_some() {
                abort!(span, "Cannot use 'owner_col' with 'unrestricted'");
            }
            if config.owner_col.is_some() {
                abort!(span, "duplicate attribute 'owner_col'");
            }
            if config.no_owner.is_some() {
                abort!(
                    span,
                    "secure: specify either `owner_col` or `no_owner`, not both"
                );
            }
            config.owner_col = Some((value, span));
        }
        "type_col" => {
            if config.unrestricted.is_some() {
                abort!(span, "Cannot use 'type_col' with 'unrestricted'");
            }
            if config.type_col.is_some() {
                abort!(span, "duplicate attribute 'type_col'");
            }
            if config.no_type.is_some() {
                abort!(
                    span,
                    "secure: specify either `type_col` or `no_type`, not both"
                );
            }
            config.type_col = Some((value, span));
        }
        _ => {
            abort!(
                span,
                "Unknown attribute '{}'. Valid attributes: tenant_col, no_tenant, \
                 resource_col, no_resource, owner_col, no_owner, type_col, no_type, \
                 unrestricted, pep_prop",
                key
            );
        }
    }
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
