//! gRPC client generation from API trait definitions with SecurityCtx support
//!
//! This macro is applied to trait definitions and generates strongly-typed gRPC clients
//! with automatic conversion between domain types and protobuf messages, including
//! automatic SecurityCtx propagation for secured APIs.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse::ParseStream, punctuated::Punctuated, FnArg, ItemTrait, ReturnType, Token,
    TraitItem,
};

/// Configuration for generate_clients macro
pub struct GenerateClientsConfig {
    pub grpc_client: String,
}

impl Parse for GenerateClientsConfig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut grpc_client: Option<String> = None;

        if input.is_empty() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "generate_clients: you must specify grpc_client = \"...\"",
            ));
        }

        let punctuated: Punctuated<syn::Meta, Token![,]> =
            input.parse_terminated(syn::Meta::parse, Token![,])?;

        for meta in punctuated {
            match &meta {
                syn::Meta::NameValue(nv) if nv.path.is_ident("grpc_client") => {
                    if grpc_client.is_some() {
                        return Err(syn::Error::new_spanned(
                            meta,
                            "duplicate `grpc_client` parameter",
                        ));
                    }
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(s),
                        ..
                    }) = &nv.value
                    {
                        grpc_client = Some(s.value());
                    } else {
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "grpc_client value must be a string literal",
                        ));
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        meta,
                        "unknown parameter; expected `grpc_client`",
                    ));
                }
            }
        }

        let grpc_client = grpc_client.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "generate_clients: you must specify grpc_client = \"...\"",
            )
        })?;

        Ok(GenerateClientsConfig { grpc_client })
    }
}

/// Method signature information with SecurityCtx awareness
#[derive(Clone)]
pub struct MethodSig {
    pub name: syn::Ident,
    pub ctx_type: Option<syn::Type>,
    pub request_type: syn::Type,
    pub response_type: syn::Type,
    pub error_type: syn::Type,
}

/// Check if a type is SecurityCtx by examining its path segments
fn is_security_ctx_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(type_path) = ty {
        if let Some(last_seg) = type_path.path.segments.last() {
            return last_seg.ident == "SecurityCtx";
        }
    }
    false
}

/// Extract T from &T or &mut T
fn extract_reference_type(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Reference(type_ref) = ty {
        Some(&type_ref.elem)
    } else {
        None
    }
}

/// Extract T and E from Result<T, E>
fn extract_result_types(ty: &syn::Type) -> syn::Result<(syn::Type, syn::Type)> {
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Result" {
                if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                    let generic_args: Vec<_> = args.args.iter().collect();

                    if generic_args.len() == 2 {
                        if let (
                            syn::GenericArgument::Type(ok_type),
                            syn::GenericArgument::Type(err_type),
                        ) = (generic_args[0], generic_args[1])
                        {
                            return Ok((ok_type.clone(), err_type.clone()));
                        }
                    }
                }
            }
        }
    }
    Err(syn::Error::new_spanned(
        ty,
        "Return type must be Result<T, E> with two generic type parameters",
    ))
}

/// Extract methods from trait definition with strict validation
fn extract_methods(trait_def: &ItemTrait) -> syn::Result<Vec<MethodSig>> {
    let mut methods = Vec::new();

    for item in &trait_def.items {
        if let TraitItem::Fn(method) = item {
            // Validate async
            if method.sig.asyncness.is_none() {
                return Err(syn::Error::new_spanned(
                    &method.sig,
                    "API methods must be async",
                ));
            }

            // Collect all inputs
            let inputs: Vec<_> = method.sig.inputs.iter().collect();

            // First must be &self
            if inputs.is_empty() {
                return Err(syn::Error::new_spanned(
                    &method.sig.inputs,
                    "API methods must begin with &self",
                ));
            }

            let first_arg = inputs[0];
            if !matches!(first_arg, FnArg::Receiver(r) if r.reference.is_some() && r.mutability.is_none())
            {
                return Err(syn::Error::new_spanned(
                    first_arg,
                    "API methods must begin with &self (not self or &mut self)",
                ));
            }

            // Count non-self parameters
            let non_self_params: Vec<_> = inputs[1..].iter().collect();

            let (ctx_type, request_type) = match non_self_params.len() {
                1 => {
                    // Case A: Only request parameter
                    if let FnArg::Typed(pat_type) = non_self_params[0] {
                        (None, (*pat_type.ty).clone())
                    } else {
                        return Err(syn::Error::new_spanned(
                            non_self_params[0],
                            "Expected typed parameter",
                        ));
                    }
                }
                2 => {
                    // Case B: SecurityCtx + request parameter
                    let first_param = non_self_params[0];
                    let second_param = non_self_params[1];

                    if let (FnArg::Typed(ctx_param), FnArg::Typed(req_param)) =
                        (first_param, second_param)
                    {
                        // Validate first parameter is &SecurityCtx
                        let ctx_ref_type =
                            extract_reference_type(&ctx_param.ty).ok_or_else(|| {
                                syn::Error::new_spanned(
                                    &ctx_param.ty,
                                    "Second parameter must be &SecurityCtx (a reference)",
                                )
                            })?;

                        if !is_security_ctx_type(ctx_ref_type) {
                            return Err(syn::Error::new_spanned(
                                &ctx_param.ty,
                                "Second parameter must be &SecurityCtx or &<something named SecurityCtx>",
                            ));
                        }

                        // Check for &mut SecurityCtx (not allowed)
                        if let syn::Type::Reference(type_ref) = &*ctx_param.ty {
                            if type_ref.mutability.is_some() {
                                return Err(syn::Error::new_spanned(
                                    &ctx_param.ty,
                                    "SecurityCtx must be immutable reference (&SecurityCtx), not &mut SecurityCtx",
                                ));
                            }
                        }

                        (Some(ctx_ref_type.clone()), (*req_param.ty).clone())
                    } else {
                        return Err(syn::Error::new_spanned(
                            &method.sig.inputs,
                            "Expected typed parameters",
                        ));
                    }
                }
                0 => {
                    return Err(syn::Error::new_spanned(
                        &method.sig.inputs,
                        "API methods must have at least one parameter (request type)",
                    ));
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &method.sig.inputs,
                        "API methods must have either (req) or (ctx, req) - no more than 2 parameters after &self",
                    ));
                }
            };

            // Extract return type
            let (response_type, error_type) = match &method.sig.output {
                ReturnType::Type(_, ty) => extract_result_types(ty)?,
                ReturnType::Default => {
                    return Err(syn::Error::new_spanned(
                        &method.sig,
                        "API methods must return Result<T, E>",
                    ));
                }
            };

            methods.push(MethodSig {
                name: method.sig.ident.clone(),
                ctx_type,
                request_type,
                response_type,
                error_type,
            });
        }
    }

    Ok(methods)
}

/// Generate gRPC client implementation with SecurityCtx support
fn generate_grpc_client(
    trait_name: &syn::Ident,
    grpc_client_type_str: &str,
    methods: &[MethodSig],
) -> syn::Result<TokenStream> {
    let client_name = quote::format_ident!("{}GrpcClient", trait_name);
    let grpc_client_type: syn::Type = syn::parse_str(grpc_client_type_str)?;

    let method_impls = methods.iter().map(|method| {
        let name = &method.name;
        let req_type = &method.request_type;
        let resp_type = &method.response_type;
        let err_type = &method.error_type;

        if let Some(ctx_type) = method.ctx_type.as_ref() {
            // Case A: Method has SecurityCtx
            quote! {
                async fn #name(
                    &self,
                    ctx: &#ctx_type,
                    req: #req_type,
                ) -> Result<#resp_type, #err_type> {
                    let mut client = self.inner.clone();
                    let mut request = ::tonic::Request::new(req.into());
                    // Attach SecurityCtx metadata to gRPC request
                    ::modkit_transport_grpc::attach_secctx(request.metadata_mut(), ctx)
                        .map_err(|e| {
                            // Convert tonic Status to error type
                            let status = ::tonic::Status::internal(format!("Failed to attach secctx: {}", e));
                            #err_type::from(status)
                        })?;
                    let response = client
                        .#name(request)
                        .await
                        .map_err(#err_type::from)?;
                    Ok(response.into_inner().into())
                }
            }
        } else {
            // Case B: Method without SecurityCtx
            quote! {
                async fn #name(&self, req: #req_type) -> Result<#resp_type, #err_type> {
                    let mut client = self.inner.clone();
                    let request = ::tonic::Request::new(req.into());
                    let response = client
                        .#name(request)
                        .await
                        .map_err(#err_type::from)?;
                    Ok(response.into_inner().into())
                }
            }
        }
    });

    Ok(quote! {
        /// gRPC client implementation with standardized transport stack
        ///
        /// This client automatically includes:
        /// - Configurable timeouts (connect and RPC)
        /// - Retry logic with exponential backoff
        /// - Metrics collection
        /// - Distributed tracing
        /// - Automatic SecurityCtx propagation (for secured APIs)
        ///
        /// For each API method:
        /// - The request type must implement `Into<ProtoRequest>`, where `ProtoRequest`
        ///   is the tonic request message type for this RPC.
        /// - The domain response type must be constructible from the tonic response
        ///   inner type, typically via `From<ProtoResponse>` or `Into<DomainResponse>`.
        /// - The error type must implement `From<tonic::Status>` for error conversion.
        ///
        /// If these conversions are missing, this impl will fail to compile at the call site.
        pub struct #client_name {
            inner: #grpc_client_type,
        }

        impl #client_name {
            /// Connect to the gRPC service with default configuration
            pub async fn connect(uri: impl Into<String>) -> ::anyhow::Result<Self> {
                let cfg = ::modkit_transport_grpc::client::GrpcClientConfig::new(
                    stringify!(#trait_name)
                );
                Self::connect_with_config(uri, &cfg).await
            }

            /// Connect to the gRPC service with custom configuration
            pub async fn connect_with_config(
                uri: impl Into<String>,
                cfg: &::modkit_transport_grpc::client::GrpcClientConfig,
            ) -> ::anyhow::Result<Self> {
                let uri_string = uri.into();

                // Create endpoint with timeouts from config
                let endpoint = ::tonic::transport::Endpoint::from_shared(uri_string)?
                    .connect_timeout(cfg.connect_timeout)
                    .timeout(cfg.rpc_timeout);

                // Connect to the service
                let channel = endpoint.connect().await?;

                Ok(Self {
                    inner: <#grpc_client_type>::new(channel),
                })
            }

            /// Create from an existing channel (useful for testing or custom setups)
            pub fn from_channel(channel: ::tonic::transport::Channel) -> Self {
                Self {
                    inner: <#grpc_client_type>::new(channel),
                }
            }
        }

        #[::async_trait::async_trait]
        impl #trait_name for #client_name {
            #(#method_impls)*
        }
    })
}

/// Main expansion function
pub fn expand_generate_clients(
    config: GenerateClientsConfig,
    trait_def: ItemTrait,
) -> syn::Result<TokenStream> {
    let trait_name = &trait_def.ident;
    let methods = extract_methods(&trait_def)?;

    if methods.is_empty() {
        return Err(syn::Error::new_spanned(
            &trait_def,
            "API trait must declare at least one async method",
        ));
    }

    let grpc_client = generate_grpc_client(trait_name, &config.grpc_client, &methods)?;

    Ok(quote! {
        #trait_def

        #grpc_client
    })
}
