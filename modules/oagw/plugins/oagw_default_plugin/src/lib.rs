// Clippy allows for v1 implementation - to be tightened in v2
#![allow(clippy::doc_markdown)] // Many technical terms without backticks
#![allow(clippy::must_use_candidate)] // Will add systematically in v2
#![allow(clippy::str_to_string)] // Prefer consistency over micro-optimization
#![allow(clippy::unused_self)] // Stub implementations have unused self
#![allow(clippy::redundant_clone)] // Some clones needed for future async use

//! OAGW Default HTTP Plugin
//!
//! This plugin provides HTTP/1.1, HTTP/2, and SSE support for the OAGW gateway.
//! It implements the `OagwPluginApi` trait and registers itself with the types-registry.
//!
//! ## Supported Features
//!
//! - **Protocols**: HTTP/1.1, HTTP/2, SSE
//! - **Auth Types**: Bearer token, API key (header)
//! - **Strategies**: Priority-based selection
//!
//! ## v1 Scope
//!
//! - Basic HTTP invocation with bearer token and API key auth
//! - Request/response handling
//! - Timeout support
//!
//! ## Future Versions (TODOs)
//!
//! - v2: SSE streaming support
//! - v3: OAuth2 client credentials, token caching
//! - v4: Sticky sessions, round-robin strategies
//! - v5: OAuth2 token exchange

// === MODULE DEFINITION ===
pub mod module;
pub use module::OagwDefaultPlugin;

// === INTERNAL MODULES ===
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod service;
