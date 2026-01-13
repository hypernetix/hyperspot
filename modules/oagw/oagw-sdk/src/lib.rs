//! OAGW SDK (Outbound API Gateway)
//!
//! This crate provides the public API contract for the OAGW gateway
//! and its plugin implementations.
//!
//! ## API Traits
//!
//! - `OagwApi` - Public API exposed by the gateway to other modules
//! - `OagwPluginApi` - Internal API implemented by plugins
//!
//! ## GTS Types
//!
//! - `OagwPluginSpecV1` - Plugin instance schema
//! - Protocol, auth type, and strategy schemas
//!
//! ## Usage
//!
//! ```ignore
//! use oagw_sdk::OagwApi;
//!
//! // Get the client from ClientHub
//! let client = hub.get::<dyn OagwApi>()?;
//!
//! // Invoke an outbound API
//! let response = client.invoke_unary(&ctx, request).await?;
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod api;
pub mod error;
pub mod gts;
pub mod models;
pub mod retry;

// API traits
pub use api::{OagwApi, OagwPluginApi};

// Error types
pub use error::OagwError;

// GTS schema types
pub use gts::{
    get_oagw_base_schemas, get_oagw_well_known_instances, OagwAuthTypeV1, OagwPluginSpecV1,
    OagwProtoV1, OagwStrategyV1, OagwStreamProtoV1,
};

// Models
pub use models::{
    HttpMethod, Link, LinkPatch, NewLink, NewRoute, OagwInvokeRequest, OagwInvokeResponse,
    OagwResponseStream, OagwStreamAbort, OagwStreamChunk, Route, RoutePatch, Secret,
    StreamAbortReason,
};

// Retry types
pub use retry::{BackoffStrategy, RetryBudget, RetryIntent, RetryOn, RetryScope, StatusClass};

// GTS types (re-exported for convenience)
pub use gts::GtsSchemaId;

// Pagination primitives (re-exported for convenience)
pub use modkit_odata::{ODataQuery, Page, PageInfo};
