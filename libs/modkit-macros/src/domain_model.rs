//! Proc-macro implementation for `#[domain_model]` attribute.
//!
//! This macro marks structs as domain models and enforces at compile-time
//! that all fields are `DomainSafe` (free of infrastructure dependencies).

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Fields, Type};

/// Expands the `#[domain_model]` attribute macro.
///
/// Generates:
/// - `impl DomainSafe for T {}`
/// - `impl DomainModel for T {}`
/// - Compile-time assertion that all fields implement `DomainSafe`
pub fn expand_domain_model(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Collect field types for compile-time validation
    let field_types: Vec<&Type> = match &input.data {
        syn::Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => fields.named.iter().map(|f| &f.ty).collect(),
            Fields::Unnamed(fields) => fields.unnamed.iter().map(|f| &f.ty).collect(),
            Fields::Unit => vec![],
        },
        syn::Data::Enum(data) => {
            // For enums, collect all variant field types
            data.variants
                .iter()
                .flat_map(|v| match &v.fields {
                    Fields::Named(fields) => fields.named.iter().map(|f| &f.ty).collect::<Vec<_>>(),
                    Fields::Unnamed(fields) => {
                        fields.unnamed.iter().map(|f| &f.ty).collect::<Vec<_>>()
                    }
                    Fields::Unit => vec![],
                })
                .collect()
        }
        syn::Data::Union(_) => {
            return syn::Error::new_spanned(name, "domain_model cannot be applied to unions")
                .to_compile_error();
        }
    };

    // Build the compile-time field validation
    let field_assertions = if field_types.is_empty() {
        quote! {}
    } else {
        quote! {
            const _: () = {
                #[allow(dead_code)]
                fn __assert_field_is_domain_safe<T: ::modkit::domain::DomainSafe>() {}

                #[allow(dead_code)]
                fn __validate_domain_model_fields() {
                    #(
                        __assert_field_is_domain_safe::<#field_types>();
                    )*
                }
            };
        }
    };

    quote! {
        #input

        impl #impl_generics ::modkit::domain::DomainSafe for #name #ty_generics #where_clause {}
        impl #impl_generics ::modkit::domain::DomainModel for #name #ty_generics #where_clause {}

        #field_assertions
    }
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

        assert!(output_str.contains("DomainSafe"));
        assert!(output_str.contains("DomainModel"));
        assert!(output_str.contains("__assert_field_is_domain_safe"));
    }

    #[test]
    fn test_expand_unit_struct() {
        let input: DeriveInput = parse_quote! {
            pub struct Marker;
        };

        let output = expand_domain_model(&input);
        let output_str = output.to_string();

        assert!(output_str.contains("DomainSafe"));
        assert!(output_str.contains("DomainModel"));
        // No field assertions for unit structs
        assert!(!output_str.contains("__validate_domain_model_fields"));
    }

    #[test]
    fn test_expand_tuple_struct() {
        let input: DeriveInput = parse_quote! {
            pub struct UserId(String);
        };

        let output = expand_domain_model(&input);
        let output_str = output.to_string();

        assert!(output_str.contains("DomainSafe"));
        assert!(output_str.contains("__assert_field_is_domain_safe"));
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

        assert!(output_str.contains("DomainSafe"));
        assert!(output_str.contains("DomainModel"));
    }

    #[test]
    fn test_expand_generic_struct() {
        let input: DeriveInput = parse_quote! {
            pub struct Container<T> {
                pub value: T,
            }
        };

        let output = expand_domain_model(&input);
        let output_str = output.to_string();

        assert!(output_str.contains("DomainSafe"));
        assert!(output_str.contains("DomainModel"));
    }
}
