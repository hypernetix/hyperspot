//! gRPC client generation macro
//!
//! Generates a tonic-based gRPC client that implements an API trait.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse::ParseStream, punctuated::Punctuated, Data, DeriveInput, Fields, Meta,
    Path, Token,
};

use crate::utils::{parse_path_attribute, parse_string_attribute, validate_path};

/// Configuration for #[`grpc_client`] attribute
pub struct GrpcClientConfig {
    pub api: Path,
    pub tonic: String,
    #[allow(dead_code)]
    pub package: Option<String>,
}

impl Parse for GrpcClientConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut api: Option<Path> = None;
        let mut tonic: Option<String> = None;
        let mut package: Option<String> = None;

        let punctuated: Punctuated<Meta, Token![,]> =
            input.parse_terminated(Meta::parse, Token![,])?;

        for meta in punctuated {
            if let Some(path) = parse_path_attribute("api", &meta)? {
                if api.is_some() {
                    return Err(syn::Error::new_spanned(meta, "duplicate `api` parameter"));
                }
                api = Some(path);
            } else if let Some(s) = parse_string_attribute("tonic", &meta)? {
                if tonic.is_some() {
                    return Err(syn::Error::new_spanned(meta, "duplicate `tonic` parameter"));
                }
                tonic = Some(s);
            } else if let Some(s) = parse_string_attribute("package", &meta)? {
                if package.is_some() {
                    return Err(syn::Error::new_spanned(
                        meta,
                        "duplicate `package` parameter",
                    ));
                }
                package = Some(s);
            } else {
                return Err(syn::Error::new_spanned(
                    meta,
                    "unknown parameter; expected `api`, `tonic`, or `package`",
                ));
            }
        }

        let api = api.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "missing required parameter: api = \"path::to::ApiTrait\"",
            )
        })?;

        let tonic = tonic.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "missing required parameter: tonic = \"module::TonicClient<Channel>\"",
            )
        })?;

        validate_path(&api, "api trait")?;

        Ok(GrpcClientConfig {
            api,
            tonic,
            package,
        })
    }
}

/// Generate the gRPC client implementation
#[allow(clippy::needless_pass_by_value)] // DeriveInput/config consumed by proc-macro pattern
pub fn expand_grpc_client(
    config: GrpcClientConfig,
    input: DeriveInput,
) -> syn::Result<TokenStream> {
    let struct_ident = &input.ident;
    let api_trait = &config.api;
    let tonic_client_str = &config.tonic;
    let vis = &input.vis;

    // Parse the tonic client type
    let tonic_client_type: syn::Type = syn::parse_str(tonic_client_str).map_err(|e| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("invalid tonic client type: {e}"),
        )
    })?;

    // Validate struct
    match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Unit => {}
            Fields::Named(f) if f.named.is_empty() => {}
            _ => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "grpc_client must be applied to an empty struct or unit struct",
                ));
            }
        },
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "grpc_client can only be applied to structs",
            ));
        }
    }

    let expanded = quote! {
        /// gRPC client wrapper with standardized transport stack
        ///
        /// This client automatically includes:
        /// - Configurable timeouts (connect and RPC)
        /// - Retry logic with exponential backoff
        /// - Metrics collection
        /// - Distributed tracing
        ///
        /// When implementing the API trait for this client:
        /// - Each request type must implement `Into<ProtoRequest>`, where `ProtoRequest`
        ///   is the tonic request message type for the corresponding RPC.
        /// - Each domain response type must be constructible from the tonic response
        ///   inner type, typically via `From<ProtoResponse>` or `Into<DomainResponse>`.
        #vis struct #struct_ident {
            inner: #tonic_client_type,
        }

        impl #struct_ident {
            /// Connect to the gRPC service with default configuration
            pub async fn connect(uri: impl Into<String>) -> ::anyhow::Result<Self> {
                let cfg = ::modkit_transport_grpc::client::GrpcClientConfig::new(
                    stringify!(#struct_ident)
                );
                Self::connect_with_config(uri, &cfg).await
            }

            /// Connect to the gRPC service with custom configuration
            pub async fn connect_with_config(
                uri: impl Into<String>,
                cfg: &::modkit_transport_grpc::client::GrpcClientConfig,
            ) -> ::anyhow::Result<Self> {
                let inner = ::modkit_transport_grpc::client::connect_with_stack::<#tonic_client_type>(
                    uri,
                    cfg,
                ).await?;
                Ok(Self { inner })
            }

            /// Create a client from an existing channel (bypasses transport stack)
            pub fn from_channel(channel: ::tonic::transport::Channel) -> Self {
                let inner = <#tonic_client_type>::new(channel);
                Self { inner }
            }

            /// Get a mutable reference to the inner tonic client
            #[doc(hidden)]
            pub fn inner_mut(&mut self) -> &mut #tonic_client_type {
                &mut self.inner
            }
        }

        // Compile-time validation that the API trait is in scope
        const _: () = {
            fn __validate_types_for_grpc_client()
            where
                #struct_ident: #api_trait,
            {
            }
        };
    };

    Ok(expanded)
}
