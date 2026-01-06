use heck::ToUpperCamelCase;
use proc_macro2::{Span, TokenStream};
use proc_macro_error2::{abort, emit_error};
use quote::quote;
use syn::{spanned::Spanned, Data, DeriveInput, Fields, Ident, Lit};

/// Configuration for a single filterable field
struct FilterableField {
    /// The field identifier in the struct
    field_ident: Ident,
    /// The field name as a string (for API)
    field_name: String,
    /// The `FieldKind` variant name (e.g., "String", "Uuid", "`DateTimeUtc`")
    kind: String,
    /// Span for error reporting
    span: Span,
}

/// Parse #[odata(filter(kind = "..."))] attributes on struct fields
fn parse_field_attrs(field: &syn::Field) -> Option<FilterableField> {
    let field_ident = field.ident.as_ref()?.clone();
    let field_name = field_ident.to_string();
    let span = field.span();

    let mut found_kind: Option<String> = None;

    for attr in &field.attrs {
        // Look for #[odata(...)]
        if !attr.path().is_ident("odata") {
            continue;
        }

        // Parse using syn v2 API
        let result = attr.parse_nested_meta(|meta| {
            // Check for filter(...) nested group
            if meta.path.is_ident("filter") {
                // Parse the contents of filter(...)
                meta.parse_nested_meta(|filter_meta| {
                    // Check for kind = "..."
                    if filter_meta.path.is_ident("kind") {
                        let value = filter_meta.value()?;
                        let lit: Lit = value.parse()?;
                        if let Lit::Str(lit_str) = lit {
                            found_kind = Some(lit_str.value());
                        } else {
                            emit_error!(
                                filter_meta.path.span(),
                                "kind value must be a string literal"
                            );
                        }
                    }
                    Ok(())
                })?;
            }
            Ok(())
        });

        if let Err(e) = result {
            emit_error!(attr.span(), "Failed to parse #[odata] attribute: {}", e);
        }
    }

    found_kind.map(|kind| FilterableField {
        field_ident,
        field_name,
        kind,
        span,
    })
}

#[allow(clippy::needless_pass_by_value)] // DeriveInput is consumed by proc-macro pattern
pub fn expand_derive_odata_filterable(input: DeriveInput) -> TokenStream {
    // Verify this is a struct with named fields
    let fields = match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                abort!(
                    input.span(),
                    "#[derive(ODataFilterable)] requires a struct with named fields"
                );
            }
        },
        _ => {
            abort!(
                input.span(),
                "#[derive(ODataFilterable)] can only be applied to structs"
            );
        }
    };

    // Extract filterable fields
    let mut filterable_fields = Vec::new();
    for field in fields {
        if let Some(filterable) = parse_field_attrs(field) {
            filterable_fields.push(filterable);
        }
    }

    if filterable_fields.is_empty() {
        emit_error!(
            input.span(),
            "No filterable fields found. Add #[odata(filter(kind = \"...\"))] to at least one field."
        );
        return quote! {};
    }

    // Generate the filter field enum name
    let dto_name = &input.ident;
    let filter_enum_name = Ident::new(&format!("{dto_name}FilterField"), input.span());

    // Generate enum variants
    let enum_variants: Vec<_> = filterable_fields
        .iter()
        .map(|f| Ident::new(&f.field_ident.to_string().to_upper_camel_case(), f.span))
        .collect();

    // Generate the FIELDS constant array
    let fields_array = enum_variants.iter().map(|variant| {
        quote! { #filter_enum_name::#variant }
    });

    // Generate name() match arms
    let name_match_arms = filterable_fields
        .iter()
        .zip(&enum_variants)
        .map(|(f, variant)| {
            let name = &f.field_name;
            quote! {
                #filter_enum_name::#variant => #name
            }
        });

    // Generate kind() match arms
    let kind_match_arms = filterable_fields
        .iter()
        .zip(&enum_variants)
        .map(|(f, variant)| {
            let kind_str = &f.kind;
            let kind_ident = Ident::new(kind_str, f.span);
            quote! {
                #filter_enum_name::#variant => ::modkit_odata::filter::FieldKind::#kind_ident
            }
        });

    // Generate the full implementation
    quote! {
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
        #[allow(non_camel_case_types)]
        pub enum #filter_enum_name {
            #(#enum_variants),*
        }

        impl ::modkit_odata::filter::FilterField for #filter_enum_name {
            const FIELDS: &'static [Self] = &[
                #(#fields_array),*
            ];

            fn name(&self) -> &'static str {
                match self {
                    #(#name_match_arms),*
                }
            }

            fn kind(&self) -> ::modkit_odata::filter::FieldKind {
                match self {
                    #(#kind_match_arms),*
                }
            }
        }
    }
}
