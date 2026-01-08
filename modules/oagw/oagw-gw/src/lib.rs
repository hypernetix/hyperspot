// Clippy allows for v1 implementation - to be tightened in v2
#![allow(clippy::doc_markdown)] // Many technical terms without backticks
#![allow(clippy::must_use_candidate)] // Will add systematically in v2
#![allow(clippy::str_to_string)] // Prefer consistency over micro-optimization

//! OAGW Gateway Module Implementation
//!
//! The OAGW (Outbound API Gateway) module manages outbound traffic to external APIs.
//! It provides:
//!
//! - Route and link configuration management
//! - Plugin-based protocol and authentication handling
//! - Circuit breaking, rate limiting, and error normalization
//! - Streaming support (SSE)
//!
//! ## Architecture
//!
//! ```text
//!           Consumer Module
//!                 │
//!                 ▼ hub.get::<dyn OagwApi>()
//! ┌────────────────────────────────────┐
//! │          OAGW Gateway              │
//! │  ┌──────────────────────────────┐  │
//! │  │     REST API (/oagw/v1/...)  │  │
//! │  └──────────────────────────────┘  │
//! │               │                    │
//! │               ▼                    │
//! │  ┌──────────────────────────────┐  │
//! │  │     Domain Service           │  │
//! │  │  - Route/Link resolution     │  │
//! │  │  - Plugin selection          │  │
//! │  │  - Circuit breaker           │  │
//! │  └──────────────────────────────┘  │
//! │               │                    │
//! │               ▼ hub.get_scoped()   │
//! └────────────────────────────────────┘
//!                │
//!     ┌──────────┴──────────┐
//!     ▼                     ▼
//! ┌────────┐           ┌────────┐
//! │Plugin A│           │Plugin B│
//! │(HTTP)  │           │(Custom)│
//! └────────┘           └────────┘
//! ```
//!
//! ## Usage
//!
//! The public API is defined in `oagw-sdk` and re-exported here.

// === PUBLIC API (from SDK) ===
pub use oagw_sdk::{
    // GTS types
    get_oagw_base_schemas,
    get_oagw_well_known_instances,
    // Retry types
    BackoffStrategy,
    // Models
    HttpMethod,
    Link,
    LinkPatch,
    NewLink,
    NewRoute,
    // Pagination
    ODataQuery,
    // API traits
    OagwApi,
    OagwAuthTypeV1,
    // Error types
    OagwError,
    OagwInvokeRequest,
    OagwInvokeResponse,
    OagwPluginApi,
    OagwPluginSpecV1,
    OagwProtoV1,
    OagwResponseStream,
    OagwStrategyV1,
    OagwStreamAbort,
    OagwStreamChunk,
    OagwStreamProtoV1,
    Page,
    RetryBudget,
    RetryIntent,
    RetryOn,
    RetryScope,
    Route,
    RoutePatch,
    Secret,
    StatusClass,
    StreamAbortReason,
};

// === MODULE DEFINITION ===
pub mod module;
pub use module::OagwGateway;

// === LOCAL CLIENT ===
pub mod local_client;

// === INTERNAL MODULES ===
#[doc(hidden)]
pub mod api;
#[doc(hidden)]
pub mod config;
#[doc(hidden)]
pub mod domain;
#[doc(hidden)]
pub mod infra;
