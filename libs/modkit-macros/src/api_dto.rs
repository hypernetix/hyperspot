use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::punctuated::Punctuated;
use syn::{DeriveInput, Ident, Token};

const ALLOWED_FLAGS: &[&str] = &["request", "response"];

/// Validates `api_dto` flags for unknown or duplicate identifiers.
/// Returns Ok(()) if valid, or Err(TokenStream) with compile error if invalid.
pub fn validate_flags(args: &Punctuated<Ident, Token![,]>) -> Result<(), TokenStream> {
    let mut seen_flags = HashSet::new();

    for ident in args {
        let flag_str = ident.to_string();

        // Check if flag is allowed
        if !ALLOWED_FLAGS.contains(&flag_str.as_str()) {
            let err = syn::Error::new_spanned(
                ident,
                format!(
                    "unknown flag '{flag_str}'; expected one of: {}",
                    ALLOWED_FLAGS.join(", ")
                ),
            );
            return Err(err.to_compile_error());
        }

        // Check for duplicates
        if !seen_flags.insert(flag_str.clone()) {
            let err = syn::Error::new_spanned(ident, format!("duplicate flag '{flag_str}'"));
            return Err(err.to_compile_error());
        }
    }

    Ok(())
}

pub fn expand_api_dto(args: &Punctuated<Ident, Token![,]>, input: &DeriveInput) -> TokenStream {
    if let Err(err) = validate_flags(args) {
        return err;
    }

    let has_request = args.iter().any(|id| id == "request");
    let has_response = args.iter().any(|id| id == "response");

    if !has_request && !has_response {
        return quote! {
            compile_error!("api_dto macro requires at least one of 'request' or 'response' arguments");
        };
    }

    let (serialize, deserialize) = (has_response, has_request);
    let name = &input.ident;
    let ser = if serialize {
        quote! { ::serde::Serialize, }
    } else {
        quote! {}
    };
    let resp_trait_impl = if serialize {
        quote! { impl ::modkit::api::api_dto::ResponseApiDto for #name {} }
    } else {
        quote! {}
    };
    let de = if deserialize {
        quote! { ::serde::Deserialize, }
    } else {
        quote! {}
    };
    let req_trait_impl = if deserialize {
        quote! { impl ::modkit::api::api_dto::RequestApiDto for #name {} }
    } else {
        quote! {}
    };

    let has_serde = serialize || deserialize;
    let serde_attr = if has_serde {
        quote! { #[serde(rename_all = "snake_case")] }
    } else {
        quote! {}
    };

    quote! {
        #[derive(#ser #de utoipa::ToSchema)]
        #serde_attr
        #input
        #req_trait_impl
        #resp_trait_impl
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_validate_flags_valid_request() {
        let args: Punctuated<Ident, Token![,]> = parse_quote!(request);
        assert!(validate_flags(&args).is_ok());
    }

    #[test]
    fn test_validate_flags_valid_response() {
        let args: Punctuated<Ident, Token![,]> = parse_quote!(response);
        assert!(validate_flags(&args).is_ok());
    }

    #[test]
    fn test_validate_flags_valid_both() {
        let args: Punctuated<Ident, Token![,]> = parse_quote!(request, response);
        assert!(validate_flags(&args).is_ok());
    }

    #[test]
    fn test_validate_flags_unknown_flag() {
        let args: Punctuated<Ident, Token![,]> = parse_quote!(unknown);
        let result = validate_flags(&args);
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("unknown flag 'unknown'"));
        assert!(err_str.contains("expected one of: request, response"));
    }

    #[test]
    fn test_validate_flags_duplicate_request() {
        let args: Punctuated<Ident, Token![,]> = parse_quote!(request, request);
        let result = validate_flags(&args);
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("duplicate flag 'request'"));
    }

    #[test]
    fn test_validate_flags_duplicate_response() {
        let args: Punctuated<Ident, Token![,]> = parse_quote!(response, response);
        let result = validate_flags(&args);
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("duplicate flag 'response'"));
    }

    #[test]
    fn test_validate_flags_typo() {
        let args: Punctuated<Ident, Token![,]> = parse_quote!(requets);
        let result = validate_flags(&args);
        assert!(result.is_err());
        let err_str = result.unwrap_err().to_string();
        assert!(err_str.contains("unknown flag 'requets'"));
    }

    #[test]
    fn test_expand_api_dto_request_only() {
        let args: Punctuated<Ident, Token![,]> = parse_quote!(request);
        let input: DeriveInput = parse_quote! {
            pub struct TestDto {
                pub id: String,
            }
        };
        let output = expand_api_dto(&args, &input);
        let output_str = output.to_string();

        assert!(output_str.contains("serde :: Deserialize"));
        assert!(!output_str.contains("serde :: Serialize"));
        assert!(output_str.contains("RequestApiDto"));
        assert!(!output_str.contains("ResponseApiDto"));
        assert!(output_str.contains("utoipa :: ToSchema"));
        assert!(output_str.contains("rename_all = \"snake_case\""));
    }

    #[test]
    fn test_expand_api_dto_response_only() {
        let args: Punctuated<Ident, Token![,]> = parse_quote!(response);
        let input: DeriveInput = parse_quote! {
            pub struct TestDto {
                pub id: String,
            }
        };
        let output = expand_api_dto(&args, &input);
        let output_str = output.to_string();

        assert!(output_str.contains("serde :: Serialize"));
        assert!(!output_str.contains("serde :: Deserialize"));
        assert!(output_str.contains("ResponseApiDto"));
        assert!(!output_str.contains("RequestApiDto"));
        assert!(output_str.contains("utoipa :: ToSchema"));
        assert!(output_str.contains("rename_all = \"snake_case\""));
    }

    #[test]
    fn test_expand_api_dto_both_flags() {
        let args: Punctuated<Ident, Token![,]> = parse_quote!(request, response);
        let input: DeriveInput = parse_quote! {
            pub struct TestDto {
                pub id: String,
            }
        };
        let output = expand_api_dto(&args, &input);
        let output_str = output.to_string();

        assert!(output_str.contains("serde :: Serialize"));
        assert!(output_str.contains("serde :: Deserialize"));
        assert!(output_str.contains("RequestApiDto"));
        assert!(output_str.contains("ResponseApiDto"));
        assert!(output_str.contains("utoipa :: ToSchema"));
        assert!(output_str.contains("rename_all = \"snake_case\""));
    }

    #[test]
    fn test_expand_api_dto_no_flags_error() {
        let args: Punctuated<Ident, Token![,]> = parse_quote!();
        let input: DeriveInput = parse_quote! {
            pub struct TestDto {
                pub id: String,
            }
        };
        let output = expand_api_dto(&args, &input);
        let output_str = output.to_string();

        assert!(output_str.contains("compile_error"));
        assert!(output_str.contains("requires at least one of 'request' or 'response'"));
    }

    #[test]
    fn test_expand_api_dto_unknown_flag_error() {
        let args: Punctuated<Ident, Token![,]> = parse_quote!(invalid);
        let input: DeriveInput = parse_quote! {
            pub struct TestDto {
                pub id: String,
            }
        };
        let output = expand_api_dto(&args, &input);
        let output_str = output.to_string();

        assert!(output_str.contains("unknown flag 'invalid'"));
    }
}
