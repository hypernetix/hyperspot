use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::{DeriveInput, Ident, Token};

pub fn expand_api_dto(args: &Punctuated<Ident, Token![,]>, input: &DeriveInput) -> TokenStream {
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
        quote! { serde::Serialize, }
    } else {
        quote! {}
    };
    let resp_trait_impl = if serialize {
        quote! { impl ::modkit::api::api_dto::ResponseApiDto for #name {} }
    } else {
        quote! {}
    };
    let de = if deserialize {
        quote! { serde::Deserialize, }
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
