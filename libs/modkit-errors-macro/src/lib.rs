//! Proc-macro for generating strongly-typed error catalogs from JSON.
//!
//! This macro reads a JSON file at compile time, validates error definitions,
//! and generates type-safe error code enums and helper macros.
//!
//! ## Usage
//!
//! The macro is self-contained and handles imports automatically.
//!
//! ```rust,ignore
//! declare_errors! {
//!     path = "gts/errors_system.json",
//!     namespace = "system_errors",
//!     vis = "pub"
//! }
//! ```

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
    #[serde(default)]
    alias: Option<String>,
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

    // Compute short names and check for collisions
    let short_names = compute_short_names(&entries)?;

    let namespace_ident = syn::Ident::new(&input.namespace, Span::call_site());
    let vis = &input.vis;
    let json_file_path = &input.path;

    let enum_variants = generate_enum_variants(&entries);
    let const_defs = generate_const_defs(&entries);
    let impl_methods = generate_impl_methods(&entries);
    let short_accessors = generate_short_accessors(&entries, &short_names);
    let from_literal_impl = generate_from_literal(&entries);
    let macro_rules_single = generate_macro_rules_single(&entries, &namespace_ident);
    let macro_rules_double = generate_macro_rules_double(&entries, &namespace_ident);
    let response_macro_rules = generate_response_macro_rules(&entries, &namespace_ident);

    Ok(quote! {
        // Force Cargo to rebuild if errors.json changes
        const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/", #json_file_path));

        // Fully-qualified imports (work both inside and outside modkit)
        use ::modkit_errors::catalog::ErrDef;
        use ::modkit_errors::problem::Problem;

        /// Strongly-typed error codes generated from the JSON catalog
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        #[allow(non_camel_case_types)]
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
            pub fn as_problem(&self, detail: impl Into<String>) -> Problem {
                self.def().as_problem(detail)
            }

            /// Create a Problem with `instance` and optional `trace_id` context.
            pub fn with_context(
                &self,
                detail: impl Into<String>,
                instance: &str,
                trace_id: Option<String>,
            ) -> Problem {
                let mut p = self.as_problem(detail);
                p = p.with_instance(instance);
                if let Some(tid) = trace_id {
                    p = p.with_trace_id(tid);
                }
                p
            }

            // Short ergonomic accessor functions
            #(#short_accessors)*

            /// Internal helper to get ErrorCode from a literal string
            #[doc(hidden)]
            pub fn from_literal(code: &str) -> Self {
                match code {
                    #(#from_literal_impl,)*
                    _ => panic!("Unknown error code literal — must be present in errors.json"),
                }
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
        use problem_from_catalog;

        /// Macro to create a Problem directly from a literal error code with instance/trace
        #[macro_export]
        macro_rules! response_from_catalog {
            #(#response_macro_rules)*

            // Catch-all for unknown codes
            ($unknown:literal, $instance:expr, $trace:expr, $($arg:tt)+) => {
                compile_error!(concat!("Unknown error code: ", $unknown))
            };
            ($unknown:literal, $instance:expr, $trace:expr) => {
                compile_error!(concat!("Unknown error code: ", $unknown))
            };
        }
        use response_from_catalog;
    })
}

fn validate_entries(entries: &[ErrorEntry]) -> syn::Result<()> {
    let mut codes = std::collections::HashSet::new();
    let mut titles_and_statuses = std::collections::HashMap::new();

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

        // Strict GTS validation
        validate_gts_format(&entry.code)?;

        // Optional: Detect redundancy (same title+status)
        let key = (entry.title.trim(), entry.status);
        if let Some(existing_code) = titles_and_statuses.get(&key) {
            eprintln!(
                "Warning: Error codes '{}' and '{}' share identical title+status ({}:{}). Consider consolidating.",
                existing_code, entry.code, entry.title, entry.status
            );
        } else {
            titles_and_statuses.insert(key, entry.code.clone());
        }
    }

    Ok(())
}

/// Strict GTS format validation
///
/// Valid format: `gts.vendor.package.namespace.type.version~chain1~chain2~...~instanceGTX`
/// Where the final GTX (instance) must have at least 5 segments: vendor.package.namespace.type.version
fn validate_gts_format(code: &str) -> syn::Result<()> {
    // Must start with 'gts.'
    if !code.starts_with("gts.") {
        return Err(syn::Error::new(
            Span::call_site(),
            format!("GTS code '{code}' must start with 'gts.'"),
        ));
    }

    // Split by '~' to get GTX chain
    let parts: Vec<&str> = code.split('~').collect();
    if parts.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            format!("GTS code '{code}' is empty or malformed"),
        ));
    }

    // Validate each GTX in the chain
    for (idx, gtx) in parts.iter().enumerate() {
        let segments: Vec<&str> = gtx.split('.').collect();

        // First GTX must start with 'gts'
        if idx == 0 && segments.first().is_none_or(|s| *s != "gts") {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("GTS code '{code}' must start with 'gts' in the first GTX"),
            ));
        }

        // All GTX segments must be non-empty and lowercase alphanumeric (with underscores)
        for segment in &segments {
            if segment.is_empty() {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!("GTS code '{code}' contains empty segment"),
                ));
            }
            if !segment
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!(
                        "GTS code '{code}' has invalid segment '{segment}': only lowercase letters, digits and underscores are allowed"
                    ),
                ));
            }
        }

        // Final GTX (instance) must have at least 5 segments after 'gts'
        if idx == parts.len() - 1 {
            // Subtract 1 for 'gts' prefix
            let meaningful_segments = if segments.first().is_some_and(|s| *s == "gts") {
                segments.len() - 1
            } else {
                segments.len()
            };

            if meaningful_segments < 5 {
                return Err(syn::Error::new(
                    Span::call_site(),
                    format!(
                        "GTS code '{code}' is expected to have at least 5 segments in final GTX: vendor.package.namespace.type.version (found {meaningful_segments} segments)"
                    ),
                ));
            }

            // Validate that the final segment is a version (vN or vN.M format)
            if let Some(last) = segments.last() {
                let is_version = last.starts_with('v')
                    && last.len() > 1
                    && last[1..].chars().all(|c| c.is_ascii_digit() || c == '.')
                    && last[1..].split('.').all(|t| !t.is_empty());
                if !is_version {
                    return Err(syn::Error::new(
                        Span::call_site(),
                        format!(
                            "GTS code '{code}' final GTX must end with version 'vN' or 'vN.M' (found '{last}')"
                        ),
                    ));
                }
            }
        }
    }

    // Validate that there's at least one chained GTX (format: gts.X~Y or gts.X)
    if parts.is_empty() {
        return Err(syn::Error::new(
            Span::call_site(),
            format!("GTS code '{code}' must have at least one GTX"),
        ));
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
                    $crate::#namespace::ErrorCode::#variant.as_problem("")
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
                    $crate::#namespace::ErrorCode::#variant.as_problem($detail)
                };
            }
        })
        .collect()
}

/// Convert a dotted error code to a valid Rust identifier
fn code_to_ident(code: &str) -> syn::Ident {
    let mut sanitized = code.replace(['.', '-', '/', '~'], "_");

    // Prefix with underscore if it starts with a digit
    if sanitized.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        sanitized = format!("_{sanitized}");
    }

    syn::Ident::new(&sanitized, Span::call_site())
}

/// Extract the final GTX segment (after last `~`) from a GTS identifier.
/// If there is no `~`, use the entire code.
fn last_gtx_segment(code: &str) -> &str {
    if let Some(pos) = code.rfind('~') {
        &code[pos + 1..]
    } else {
        code
    }
}

/// Given a GTX segment "vendor.package.namespace.type.version",
/// produce alias "`package_namespace_type_version`".
///
/// - Drops the vendor (first path segment)
/// - Replaces dots with underscores
/// - Ensures a valid Rust identifier (prefix '_' if starts with a digit)
fn derive_alias_from_gts(code: &str) -> syn::Result<String> {
    let gtx = last_gtx_segment(code);
    // Expect vendor.package.namespace.type.version
    let parts: Vec<&str> = gtx.split('.').collect();
    if parts.len() < 5 {
        return Err(syn::Error::new(
            Span::call_site(),
            format!(
                "GTS code '{code}' is expected to have at least 5 segments in final GTX: vendor.package.namespace.type.version"
            ),
        ));
    }
    // parts[0] = vendor; we drop it
    let rest = &parts[1..]; // package, namespace, type, version, (optionally extra minor parts are already in version)
    let alias_raw = rest.join("_");

    // Ensure valid Rust identifier (lowercase is already per spec)
    let mut ident = alias_raw.replace(['-', '/', '~'], "_"); // just in case
    if ident.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        ident = format!("_{ident}");
    }
    Ok(ident)
}

/// Compute short names for all entries, detecting collisions
fn compute_short_names(entries: &[ErrorEntry]) -> syn::Result<Vec<String>> {
    use std::collections::HashMap;

    let mut name_to_codes: HashMap<String, Vec<&str>> = HashMap::new();

    // Collect all short names (alias or derived via GTS)
    for entry in entries {
        let short = if let Some(alias) = &entry.alias {
            alias.clone()
        } else {
            // Use new GTS-aware derivation
            derive_alias_from_gts(&entry.code)?
        };

        name_to_codes.entry(short).or_default().push(&entry.code);
    }

    // Collision detection
    for (name, codes) in &name_to_codes {
        if codes.len() > 1 {
            return Err(syn::Error::new(
                Span::call_site(),
                format!(
                    "Short name collision: '{}' would be used by multiple error codes: {}. \
                     Please add explicit 'alias' fields in errors.json to resolve this.",
                    name,
                    codes.join(", ")
                ),
            ));
        }
    }

    // Return short names in the same order as entries
    entries
        .iter()
        .map(|e| {
            if let Some(alias) = &e.alias {
                Ok(alias.clone())
            } else {
                derive_alias_from_gts(&e.code)
            }
        })
        // Turn Vec<Result<String>> into Result<Vec<String>>
        .collect::<syn::Result<Vec<String>>>()
}

/// Generate short ergonomic accessor functions
fn generate_short_accessors(entries: &[ErrorEntry], short_names: &[String]) -> Vec<TokenStream2> {
    entries
        .iter()
        .zip(short_names.iter())
        .map(|(entry, short_name)| {
            let full_variant = code_to_ident(&entry.code);
            let short_ident = syn::Ident::new(short_name, Span::call_site());
            let code = &entry.code;

            quote! {
                #[doc = concat!("Returns the error code for `", #code, "`.")]
                pub const fn #short_ident() -> Self {
                    Self::#full_variant
                }
            }
        })
        .collect()
}

/// Generate `from_literal` match arms
fn generate_from_literal(entries: &[ErrorEntry]) -> Vec<TokenStream2> {
    entries
        .iter()
        .map(|e| {
            let code_lit = &e.code;
            let variant = code_to_ident(&e.code);

            quote! {
                #code_lit => Self::#variant
            }
        })
        .collect()
}

/// Generate `response_from_catalog`! macro rules (with format support)
fn generate_response_macro_rules(
    entries: &[ErrorEntry],
    namespace: &syn::Ident,
) -> Vec<TokenStream2> {
    let mut rules = Vec::new();

    for entry in entries {
        let code_lit = &entry.code;
        let variant = code_to_ident(&entry.code);

        // Rule with formatted detail
        rules.push(quote! {
            (#code_lit, $instance:expr, $trace:expr, $($arg:tt)+) => {
                $crate::#namespace::ErrorCode::#variant.with_context(
                    format!($($arg)+),
                    $instance,
                    $trace
                )
            };
        });

        // Rule with static/empty detail
        rules.push(quote! {
            (#code_lit, $instance:expr, $trace:expr) => {
                $crate::#namespace::ErrorCode::#variant.with_context("", $instance, $trace)
            };
        });
    }

    rules
}
