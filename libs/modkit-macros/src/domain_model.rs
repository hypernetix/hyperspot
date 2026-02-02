//! Proc-macro implementation for `#[domain_model]` attribute.
//!
//! This macro marks structs and enums as domain models and validates that they don't
//! contain infrastructure types. Validation is performed at macro expansion time by
//! checking field type names against forbidden patterns, similar to how `#[api_dto]`
//! validates its arguments.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, Type, TypePath};

/// Forbidden type patterns for domain models.
///
/// These patterns match infrastructure types that should not appear in domain models.
/// The list is synchronized with Dylint lints `DE0301` (`no_infra_in_domain`) and
/// `DE0308` (`no_http_in_domain`).
const FORBIDDEN_PATTERNS: &[&str] = &[
    // Database frameworks
    "sqlx::",
    "sea_orm::",
    // HTTP/Web frameworks
    "http::",
    "axum::",
    "hyper::",
    // External service clients
    "reqwest::",
    "tonic::",
    // Infrastructure/API layer
    "::infra::",
    "::infrastructure::",
    "::api::",
    // File system (should be abstracted)
    "std::fs::",
    "tokio::fs::",
];

/// Forbidden type names that are infrastructure types.
///
/// These are checked as complete type names (not just prefixes).
const FORBIDDEN_TYPE_NAMES: &[&str] = &[
    "PgPool",
    "MySqlPool",
    "SqlitePool",
    "DatabaseConnection",
    "StatusCode",
    "Request",
    "Response",
    "HeaderMap",
    "Method",
];

/// Expands the `#[domain_model]` attribute macro.
///
/// This function:
/// 1. Validates that all field types are free of infrastructure dependencies
/// 2. Returns clear error messages if forbidden types are found
/// 3. Generates `impl DomainModel for T {}`
///
/// Unlike the previous implementation that used trait bounds (which produced
/// generic "trait not satisfied" errors), this validates type names directly
/// during macro expansion, providing clear, actionable error messages.
pub fn expand_domain_model(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Collect all fields with their types and optional names
    let fields_with_context: Vec<FieldContext> = match &input.data {
        syn::Data::Struct(data) => collect_struct_fields(&data.fields),
        syn::Data::Enum(data) => collect_enum_fields(data),
        syn::Data::Union(_) => {
            return syn::Error::new_spanned(name, "domain_model cannot be applied to unions")
                .to_compile_error();
        }
    };

    // Validate each field type
    for field_ctx in &fields_with_context {
        if let Err(err) = validate_field_type(field_ctx.ty, &field_ctx.context) {
            return err.to_compile_error();
        }
    }

    // If validation passed, generate simple impl without assertions
    quote! {
        #input

        #[allow(deprecated)]
        impl #impl_generics ::modkit::domain::DomainSafe for #name #ty_generics #where_clause {}
        impl #impl_generics ::modkit::domain::DomainModel for #name #ty_generics #where_clause {}
    }
}

/// Context information about a field for error reporting.
struct FieldContext<'a> {
    ty: &'a Type,
    context: String,
}

/// Collects fields from a struct with context for error messages.
fn collect_struct_fields(fields: &Fields) -> Vec<FieldContext<'_>> {
    match fields {
        Fields::Named(fields) => fields
            .named
            .iter()
            .map(|f| {
                // Named fields always have an ident by syn's definition
                #[allow(clippy::unwrap_used)]
                let field_name = &f.ident.as_ref().unwrap();
                FieldContext {
                    ty: &f.ty,
                    context: format!("field '{field_name}'"),
                }
            })
            .collect(),
        Fields::Unnamed(fields) => fields
            .unnamed
            .iter()
            .enumerate()
            .map(|(idx, f)| FieldContext {
                ty: &f.ty,
                context: format!("tuple field {idx}"),
            })
            .collect(),
        Fields::Unit => vec![],
    }
}

/// Collects fields from enum variants with context for error messages.
fn collect_enum_fields(data: &syn::DataEnum) -> Vec<FieldContext<'_>> {
    data.variants
        .iter()
        .flat_map(|variant| {
            let variant_name = &variant.ident;
            match &variant.fields {
                Fields::Named(fields) => fields
                    .named
                    .iter()
                    .map(|f| {
                        // Named fields always have an ident by syn's definition
                        #[allow(clippy::unwrap_used)]
                        let field_name = &f.ident.as_ref().unwrap();
                        FieldContext {
                            ty: &f.ty,
                            context: format!("field '{field_name}' in variant '{variant_name}'"),
                        }
                    })
                    .collect::<Vec<_>>(),
                Fields::Unnamed(fields) => fields
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(idx, f)| FieldContext {
                        ty: &f.ty,
                        context: format!("tuple field {idx} in variant '{variant_name}'"),
                    })
                    .collect::<Vec<_>>(),
                Fields::Unit => vec![],
            }
        })
        .collect()
}

/// Validates that a type doesn't contain forbidden infrastructure types.
///
/// This function checks type paths against `FORBIDDEN_PATTERNS` and `FORBIDDEN_TYPE_NAMES`.
/// It recursively checks generic arguments (e.g., `Option<http::StatusCode>`).
///
/// Returns Ok(()) if the type is valid, or Err with a descriptive error.
fn validate_field_type(ty: &Type, context: &str) -> syn::Result<()> {
    match ty {
        // Check path types (most common case)
        Type::Path(type_path) => validate_type_path(type_path, context),

        // Recursively check inner types
        Type::Reference(type_ref) => validate_field_type(&type_ref.elem, context),
        Type::Slice(type_slice) => validate_field_type(&type_slice.elem, context),
        Type::Array(type_array) => validate_field_type(&type_array.elem, context),
        Type::Ptr(type_ptr) => validate_field_type(&type_ptr.elem, context),
        Type::Tuple(type_tuple) => {
            for elem_ty in &type_tuple.elems {
                validate_field_type(elem_ty, context)?;
            }
            Ok(())
        }

        // TraitObject: check trait bounds
        Type::TraitObject(trait_obj) => {
            for bound in &trait_obj.bounds {
                if let syn::TypeParamBound::Trait(trait_bound) = bound {
                    let path_str = type_path_to_string(&trait_bound.path);
                    if let Some(pattern) = find_forbidden_pattern(&path_str) {
                        return Err(syn::Error::new_spanned(
                            ty,
                            format!(
                                "{context} uses forbidden trait '{path_str}' (matches pattern '{pattern}'). \
                                 Domain models must be free of infrastructure dependencies. \
                                 Move infrastructure types to the infra/ or api/ layers."
                            ),
                        ));
                    }
                }
            }
            Ok(())
        }

        // Other type kinds are typically safe or will be caught by other means
        _ => Ok(()),
    }
}

/// Validates a type path (e.g., `http::StatusCode`, `Option<String>`).
fn validate_type_path(type_path: &TypePath, context: &str) -> syn::Result<()> {
    let path = &type_path.path;
    let path_str = type_path_to_string(path);

    // Check if the type path matches any forbidden pattern
    if let Some(pattern) = find_forbidden_pattern(&path_str) {
        return Err(syn::Error::new_spanned(
            type_path,
            format!(
                "{context} has type '{path_str}' which matches forbidden pattern '{pattern}'. \
                 Domain models must be free of infrastructure dependencies like database types (sqlx, sea_orm), \
                 HTTP types (http, axum, hyper), or external service clients. \
                 Move infrastructure types to the infra/ or api/ layers."
            ),
        ));
    }

    // Check if the last segment matches forbidden type names
    if let Some(last_segment) = path.segments.last() {
        let type_name = last_segment.ident.to_string();
        if FORBIDDEN_TYPE_NAMES.contains(&type_name.as_str()) {
            return Err(syn::Error::new_spanned(
                type_path,
                format!(
                    "{context} has type '{type_name}' which is a forbidden infrastructure type. \
                     Domain models must be transport-agnostic and persistence-agnostic. \
                     Use domain-specific types instead."
                ),
            ));
        }

        // Recursively check generic arguments (e.g., Option<http::StatusCode>)
        if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
            for arg in &args.args {
                if let syn::GenericArgument::Type(inner_ty) = arg {
                    validate_field_type(inner_ty, context)?;
                }
            }
        }
    }

    Ok(())
}

/// Converts a `syn::Path` to a string (e.g., `http::StatusCode`).
fn type_path_to_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// Checks if a type path matches any forbidden pattern.
///
/// Returns `Some(pattern)` if a match is found, `None` otherwise.
fn find_forbidden_pattern(path_str: &str) -> Option<&'static str> {
    FORBIDDEN_PATTERNS
        .iter()
        .find(|&&pattern| path_str.starts_with(pattern) || path_str.contains(pattern))
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_expand_simple_struct() {
        let input: DeriveInput = parse_quote! {
            pub struct User {
                pub id: String,
                pub name: String,
            }
        };

        let output = expand_domain_model(&input);
        let output_str = output.to_string();

        assert!(output_str.contains("DomainModel"));
        // Should NOT contain const assertions
        assert!(!output_str.contains("__assert_field_is_domain_safe"));
        assert!(!output_str.contains("const _"));
    }

    #[test]
    fn test_expand_unit_struct() {
        let input: DeriveInput = parse_quote! {
            pub struct Marker;
        };

        let output = expand_domain_model(&input);
        let output_str = output.to_string();

        assert!(output_str.contains("DomainModel"));
        assert!(!output_str.contains("__validate_domain_model_fields"));
    }

    #[test]
    fn test_forbidden_http_status_code() {
        let input: DeriveInput = parse_quote! {
            pub struct BadModel {
                pub status: http::StatusCode,
            }
        };

        let output = expand_domain_model(&input);
        let output_str = output.to_string();

        // Should contain compile error
        assert!(output_str.contains("compile_error"));
        assert!(output_str.contains("http::"));
    }

    #[test]
    fn test_forbidden_sqlx_pool() {
        let input: DeriveInput = parse_quote! {
            pub struct BadModel {
                pub pool: sqlx::PgPool,
            }
        };

        let output = expand_domain_model(&input);
        let output_str = output.to_string();

        assert!(output_str.contains("compile_error"));
        assert!(output_str.contains("sqlx::"));
    }

    #[test]
    fn test_forbidden_type_in_option() {
        let input: DeriveInput = parse_quote! {
            pub struct BadModel {
                pub maybe_status: Option<http::StatusCode>,
            }
        };

        let output = expand_domain_model(&input);
        let output_str = output.to_string();

        assert!(output_str.contains("compile_error"));
        assert!(output_str.contains("http::"));
    }

    #[test]
    fn test_forbidden_type_by_name() {
        let input: DeriveInput = parse_quote! {
            pub struct BadModel {
                pub pool: PgPool,
            }
        };

        let output = expand_domain_model(&input);
        let output_str = output.to_string();

        assert!(output_str.contains("compile_error"));
        assert!(output_str.contains("PgPool"));
    }

    #[test]
    fn test_allowed_types() {
        let input: DeriveInput = parse_quote! {
            pub struct GoodModel {
                pub id: uuid::Uuid,
                pub name: String,
                pub count: i32,
                pub items: Vec<String>,
                pub metadata: Option<serde_json::Value>,
            }
        };

        let output = expand_domain_model(&input);
        let output_str = output.to_string();

        // Should NOT contain compile_error
        assert!(!output_str.contains("compile_error"));
        assert!(output_str.contains("DomainModel"));
    }

    #[test]
    fn test_expand_enum() {
        let input: DeriveInput = parse_quote! {
            pub enum Status {
                Active,
                Inactive { reason: String },
                Pending(i32),
            }
        };

        let output = expand_domain_model(&input);
        let output_str = output.to_string();

        assert!(output_str.contains("DomainModel"));
        assert!(!output_str.contains("const _"));
    }

    #[test]
    fn test_enum_with_forbidden_type() {
        let input: DeriveInput = parse_quote! {
            pub enum BadStatus {
                Ok,
                HttpError(http::StatusCode),
            }
        };

        let output = expand_domain_model(&input);
        let output_str = output.to_string();

        assert!(output_str.contains("compile_error"));
        assert!(output_str.contains("http::"));
    }

    #[test]
    fn test_generic_struct() {
        let input: DeriveInput = parse_quote! {
            pub struct Container<T> {
                pub value: T,
            }
        };

        let output = expand_domain_model(&input);
        let output_str = output.to_string();

        // Generic types are not validated at macro time (validated at use site)
        assert!(output_str.contains("DomainModel"));
        assert!(!output_str.contains("compile_error"));
    }

    #[test]
    fn test_union_rejected() {
        let input: DeriveInput = parse_quote! {
            pub union BadUnion {
                x: i32,
                y: f32,
            }
        };

        let output = expand_domain_model(&input);
        let output_str = output.to_string();

        assert!(output_str.contains("compile_error"));
        assert!(output_str.contains("union"));
    }
}
