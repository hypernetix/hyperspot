//! Proc-macro for generating strongly-typed error catalogs from JSON.
//!
//! This macro reads a JSON file at compile time, validates error definitions,
//! and generates type-safe error code enums and helper macros.

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use serde::Deserialize;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, LitStr, Token};

/// JSON schema for a single error definition
#[derive(Debug, Clone, Deserialize)]
struct ErrorEntry {
    status: u16,
    title: String,
    code: String,
    #[serde(rename = "type")]
    type_url: Option<String>,
}

/// Parsed macro input
struct DeclareErrorsInput {
    path: String,
    namespace: String,
    vis: syn::Visibility,
}

impl Parse for DeclareErrorsInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut path = None;
        let mut namespace = None;
        let mut vis = syn::Visibility::Inherited;

        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match key.to_string().as_str() {
                "path" => {
                    let lit: LitStr = input.parse()?;
                    path = Some(lit.value());
                }
                "namespace" => {
                    let lit: LitStr = input.parse()?;
                    namespace = Some(lit.value());
                }
                "vis" => {
                    let lit: LitStr = input.parse()?;
                    vis = match lit.value().as_str() {
                        "pub" => syn::Visibility::Public(syn::token::Pub::default()),
                        _ => syn::Visibility::Inherited,
                    };
                }
                _ => return Err(syn::Error::new(key.span(), "Unknown parameter")),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(DeclareErrorsInput {
            path: path.ok_or_else(|| input.error("Missing 'path' parameter"))?,
            namespace: namespace.ok_or_else(|| input.error("Missing 'namespace' parameter"))?,
            vis,
        })
    }
}

/// Main proc-macro entry point
#[proc_macro]
pub fn declare_errors(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeclareErrorsInput);

    match generate_errors(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

fn generate_errors(input: &DeclareErrorsInput) -> syn::Result<TokenStream2> {
    // Load and parse JSON file
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| syn::Error::new(Span::call_site(), "CARGO_MANIFEST_DIR not set"))?;
    let json_path = std::path::Path::new(&manifest_dir).join(&input.path);

    let json_content = std::fs::read_to_string(&json_path).map_err(|e| {
        syn::Error::new(
            Span::call_site(),
            format!(
                "Failed to read error catalog at {}: {}",
                json_path.display(),
                e
            ),
        )
    })?;

    let entries: Vec<ErrorEntry> = serde_json::from_str(&json_content).map_err(|e| {
        syn::Error::new(
            Span::call_site(),
            format!(
                "Failed to parse error catalog JSON at {}: {}",
                json_path.display(),
                e
            ),
        )
    })?;

    // Validate entries
    validate_entries(&entries)?;

    let namespace_ident = syn::Ident::new(&input.namespace, Span::call_site());
    let vis = &input.vis;
    let json_file_path = &input.path;

    let enum_variants = generate_enum_variants(&entries);
    let const_defs = generate_const_defs(&entries);
    let impl_methods = generate_impl_methods(&entries);
    let macro_rules_single = generate_macro_rules_single(&entries, &namespace_ident);
    let macro_rules_double = generate_macro_rules_double(&entries, &namespace_ident);

    Ok(quote! {
        // Force Cargo to rebuild if errors.json changes
        const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #json_file_path));

        use modkit_errors::ErrDef;
        use http::StatusCode;
        use modkit::api::problem::Problem;

        /// Strongly-typed error codes from the catalog
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        #vis enum ErrorCode {
            #(#enum_variants),*
        }

        impl ErrorCode {
            /// Get the HTTP status code for this error
            pub const fn status(&self) -> u16 {
                match self {
                    #(#const_defs),*
                }
            }

            /// Get the error definition for this error code
            pub const fn def(&self) -> ErrDef {
                match self {
                    #(#impl_methods),*
                }
            }

            /// Convert to Problem with detail (without instance/trace)
            pub fn to_problem(&self, detail: impl Into<String>) -> Problem {
                self.def().to_problem(detail)
            }

            /// Create a full ProblemResponse with context (recommended for handlers)
            pub fn to_response(
                &self,
                detail: impl Into<String>,
                instance: &str,
                trace_id: Option<String>,
            ) -> modkit::api::problem::ProblemResponse {
                let mut p = self.to_problem(detail);
                p = p.with_instance(instance);
                if let Some(tid) = trace_id {
                    p = p.with_trace_id(tid);
                }
                p.into()
            }
        }

        /// Macro to create a Problem from a literal error code (compile-time validated)
        #[macro_export]
        macro_rules! problem_from_catalog {
            #(#macro_rules_single)*
            #(#macro_rules_double)*

            // Catch-all for unknown codes
            ($unknown:literal) => {
                compile_error!(concat!("Unknown error code: ", $unknown))
            };
            ($unknown:literal, $detail:expr) => {
                compile_error!(concat!("Unknown error code: ", $unknown))
            };
        }
    })
}

fn validate_entries(entries: &[ErrorEntry]) -> syn::Result<()> {
    let mut codes = std::collections::HashSet::new();

    for entry in entries {
        // Validate status code
        if !(100..=599).contains(&entry.status) {
            return Err(syn::Error::new(
                Span::call_site(),
                format!(
                    "Invalid HTTP status code {} for error '{}'",
                    entry.status, entry.code
                ),
            ));
        }

        // Validate non-empty title
        if entry.title.trim().is_empty() {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("Empty title for error '{}'", entry.code),
            ));
        }

        // Check for duplicate codes
        if !codes.insert(&entry.code) {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("Duplicate error code: '{}'", entry.code),
            ));
        }
    }

    Ok(())
}

fn generate_enum_variants(entries: &[ErrorEntry]) -> Vec<TokenStream2> {
    entries
        .iter()
        .map(|e| {
            let variant = code_to_ident(&e.code);
            let code = &e.code;
            quote! {
                #[doc = #code]
                #variant
            }
        })
        .collect()
}

fn generate_const_defs(entries: &[ErrorEntry]) -> Vec<TokenStream2> {
    entries
        .iter()
        .map(|e| {
            let variant = code_to_ident(&e.code);
            let status = e.status;
            quote! {
                ErrorCode::#variant => #status
            }
        })
        .collect()
}

fn generate_impl_methods(entries: &[ErrorEntry]) -> Vec<TokenStream2> {
    entries
        .iter()
        .map(|e| {
            let variant = code_to_ident(&e.code);
            let status = e.status;
            let title = &e.title;
            let code = &e.code;
            let type_url = match &e.type_url {
                Some(s) => s.clone(),
                None => format!("https://errors.example.com/{}", e.code),
            };

            quote! {
                ErrorCode::#variant => ErrDef {
                    status: #status,
                    title: #title,
                    code: #code,
                    type_url: #type_url,
                }
            }
        })
        .collect()
}

fn generate_macro_rules_single(
    entries: &[ErrorEntry],
    namespace: &syn::Ident,
) -> Vec<TokenStream2> {
    entries
        .iter()
        .map(|e| {
            let code_lit = &e.code;
            let variant = code_to_ident(&e.code);

            quote! {
                (#code_lit) => {
                    $crate::#namespace::ErrorCode::#variant.to_problem("")
                };
            }
        })
        .collect()
}

fn generate_macro_rules_double(
    entries: &[ErrorEntry],
    namespace: &syn::Ident,
) -> Vec<TokenStream2> {
    entries
        .iter()
        .map(|e| {
            let code_lit = &e.code;
            let variant = code_to_ident(&e.code);

            quote! {
                (#code_lit, $detail:expr) => {
                    $crate::#namespace::ErrorCode::#variant.to_problem($detail)
                };
            }
        })
        .collect()
}

/// Convert a dotted error code to a valid Rust identifier
fn code_to_ident(code: &str) -> syn::Ident {
    let mut sanitized = code.replace(['.', '-', '/'], "_");

    // Prefix with underscore if it starts with a digit
    if sanitized
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        sanitized = format!("_{}", sanitized);
    }

    syn::Ident::new(&sanitized, Span::call_site())
}
