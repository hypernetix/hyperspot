use syn::{DeriveInput, Ident, Token};
use syn::punctuated::Punctuated;
use quote::quote;
use proc_macro2::TokenStream;

pub fn expand_api_dto(args: &Punctuated<Ident, Token![,]>, input: &DeriveInput) -> TokenStream {
    let has_request = args.iter().any(|id| id == "request");
    let has_response = args.iter().any(|id| id == "response");
    
    let (serialize, deserialize) = if !has_request && !has_response {
        (false, false)
    } else {
        (has_response, has_request)
    };
    let name = &input.ident;
    let ser = if serialize { quote! { serde::Serialize, } } else { quote! {} };
    let resp_trait_impl = if serialize { quote! { impl ::modkit::api::api_dto::ResponseApiDto for #name {} } } else { quote! {} };
    let de = if deserialize { quote! { serde::Deserialize, } } else { quote! {} };
    let req_trait_impl = if deserialize { quote! { impl ::modkit::api::api_dto::RequestApiDto for #name {} } } else { quote! {} };
    quote! {
        #[derive(#ser #de utoipa::ToSchema)]
        #[serde(rename_all = "snake_case")]
        #input
        #req_trait_impl
        #resp_trait_impl
    }
}