use heck::ToUpperCamelCase;
use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, emit_error};
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput};

/// Configuration parsed from `#[secure(...)]` attributes
#[derive(Default)]
struct SecureConfig {
    tenant_col: Option<(String, Span)>,
    resource_col: Option<(String, Span)>,
    owner_col: Option<(String, Span)>,
    entity: Option<(String, Span)>,
    unrestricted: Option<Span>,
}

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

    // Determine entity type name (default: "Entity")
    let entity_name = config
        .entity
        .as_ref()
        .map(|(s, _)| s.as_str())
        .unwrap_or("Entity");
    let entity_ident = syn::Ident::new(entity_name, input.ident.span());

    // Check for conflicting flags
    if config.unrestricted.is_some() && config.tenant_col.is_some() {
        emit_error!(
            config.unrestricted.unwrap(),
            "Cannot use both 'unrestricted' and 'tenant_col' attributes";
            note = config.tenant_col.as_ref().unwrap().1 => "tenant_col defined here"
        );
    }

    // Determine column names with defaults
    let resource_col = config
        .resource_col
        .as_ref()
        .map(|(s, _)| s.as_str())
        .unwrap_or("id");

    // Convert column names from snake_case to UpperCamelCase for enum variants
    let resource_variant = snake_to_upper_camel(resource_col);
    let tenant_variant = config
        .tenant_col
        .as_ref()
        .map(|(s, _)| snake_to_upper_camel(s));
    let owner_variant = config
        .owner_col
        .as_ref()
        .map(|(s, _)| snake_to_upper_camel(s));

    // Generate column identifiers
    let resource_col_ident = syn::Ident::new(&resource_variant, input.ident.span());

    // Generate tenant_col implementation
    let tenant_col_impl = if let Some(tenant_var) = tenant_variant {
        let tenant_col_ident = syn::Ident::new(&tenant_var, input.ident.span());
        quote! {
            fn tenant_col() -> ::core::option::Option<Self::Column> {
                ::core::option::Option::Some(Self::Column::#tenant_col_ident)
            }
        }
    } else {
        quote! {
            fn tenant_col() -> ::core::option::Option<Self::Column> {
                ::core::option::Option::None
            }
        }
    };

    // Generate owner_col implementation
    let owner_col_impl = if let Some(owner_var) = owner_variant {
        let owner_col_ident = syn::Ident::new(&owner_var, input.ident.span());
        quote! {
            fn owner_col() -> ::core::option::Option<Self::Column> {
                ::core::option::Option::Some(Self::Column::#owner_col_ident)
            }
        }
    } else {
        quote! {
            // Use default implementation (returns None)
        }
    };

    // Generate IS_UNRESTRICTED constant
    let is_unrestricted = config.unrestricted.is_some();
    let is_unrestricted_impl = quote! {
        const IS_UNRESTRICTED: bool = #is_unrestricted;
    };

    // Generate the implementation
    quote! {
        impl ::modkit_db::secure::ScopableEntity for #entity_ident {
            #is_unrestricted_impl

            fn id_col() -> Self::Column {
                Self::Column::#resource_col_ident
            }

            #tenant_col_impl

            #owner_col_impl
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
            // Check if this is a flag (unrestricted) or key-value pair
            if meta.path.is_ident("unrestricted") {
                let span = meta.path.span();
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

            // Key-value pair
            let key = meta
                .path
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_default();
            let span = meta.path.span();

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
                "entity" => {
                    if let Some((_, prev_span)) = config.entity {
                        abort!(
                            span,
                            "duplicate attribute 'entity'";
                            note = prev_span => "first defined here"
                        );
                    }
                    config.entity = Some((value, span));
                }
                _ => {
                    abort!(
                        span,
                        "Unknown attribute '{}'. Valid attributes: tenant_col, resource_col, owner_col, entity, unrestricted",
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

/// Convert snake_case to UpperCamelCase for enum variant names
fn snake_to_upper_camel(s: &str) -> String {
    s.to_upper_camel_case()
}

#[cfg(test)]
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
