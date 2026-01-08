//! GTS schema types for OAGW.
//!
//! This module defines the GTS schemas for:
//! - Plugin instances (`OagwPluginSpecV1`)
//! - Protocol types (`OagwProtoV1`)
//! - Stream protocol types (`OagwStreamProtoV1`)
//! - Auth types (`OagwAuthTypeV1`)
//! - Strategy types (`OagwStrategyV1`)
//!
//! Well-known instances are defined in submodules:
//! - `protocols` - HTTP/1.1, HTTP/2, HTTP/3, SSE
//! - `stream_protocols` - SSE stream
//! - `auth_types` - Bearer token, API key, OAuth2
//! - `strategies` - Sticky session, round robin, priority

use gts::GtsInstanceId;
pub use gts::GtsSchemaId;
use gts_macros::struct_to_gts_schema;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// For helper functions
use anyhow::Result;

// Re-export BaseModkitPluginV1 from modkit for convenience
pub use modkit::gts::BaseModkitPluginV1;

// Well-known GTS IDs for protocols
pub mod protocols {
    /// HTTP/1.1 protocol.
    pub const HTTP11: &str = "gts.x.core.oagw.proto.v1~x.core.http.http11.v1";
    /// HTTP/2 protocol.
    pub const HTTP2: &str = "gts.x.core.oagw.proto.v1~x.core.http.http2.v1";
    /// HTTP/3 protocol (future).
    pub const HTTP3: &str = "gts.x.core.oagw.proto.v1~x.core.http.http3.v1";
    /// Server-Sent Events protocol.
    pub const SSE: &str = "gts.x.core.oagw.proto.v1~x.core.http.sse.v1";
}

// Well-known GTS IDs for auth types
pub mod auth_types {
    /// Bearer token authentication.
    pub const BEARER_TOKEN: &str = "gts.x.core.oagw.auth_type.v1~x.core.auth.bearer_token.v1";
    /// API key in header.
    pub const API_KEY_HEADER: &str = "gts.x.core.oagw.auth_type.v1~x.core.auth.api_key_header.v1";
    /// API key in query parameter.
    pub const API_KEY_QUERY: &str = "gts.x.core.oagw.auth_type.v1~x.core.auth.api_key_query.v1";
    /// `OAuth2` client credentials flow.
    pub const OAUTH2_CLIENT_CREDS: &str =
        "gts.x.core.oagw.auth_type.v1~x.core.auth.oauth2_client_creds.v1";
    /// `OAuth2` token exchange (RFC 8693).
    pub const OAUTH2_TOKEN_EXCHANGE: &str =
        "gts.x.core.oagw.auth_type.v1~x.core.auth.oauth2_token_exchange.v1";
}

// Well-known GTS IDs for strategies
pub mod strategies {
    /// Sticky session strategy.
    pub const STICKY_SESSION: &str = "gts.x.core.oagw.strategy.v1~x.core._.sticky_session.v1";
    /// Round-robin strategy.
    pub const ROUND_ROBIN: &str = "gts.x.core.oagw.strategy.v1~x.core._.round_robin.v1";
    /// Priority-based selection (default).
    pub const PRIORITY: &str = "gts.x.core.oagw.strategy.v1~x.core._.priority.v1";
}

// Well-known GTS IDs for stream protocols
pub mod stream_protocols {
    /// Server-Sent Events streaming protocol.
    pub const SSE: &str = "gts.x.core.oagw.stream_proto.v1~x.core.http.sse.v1";
}

// Well-known GTS IDs for roles
pub mod roles {
    /// Role for route health checks.
    pub const ROUTE_HEALTH: &str = "gts.x.core.idp.role.v1~x.oagw.route.health.v1";
    /// Role for route administration.
    pub const ROUTE_ADMIN: &str = "gts.x.core.idp.role.v1~x.oagw.route.admin.v1";
    /// Role for link administration.
    pub const LINK_ADMIN: &str = "gts.x.core.idp.role.v1~x.oagw.link.admin.v1";
    /// Role for invocation.
    pub const INVOKE: &str = "gts.x.core.idp.role.v1~x.oagw.api.invoke.v1";
}

// ============================================================================
// BASE GTS SCHEMAS
// ============================================================================

/// GTS schema for OAGW protocol types.
///
/// Schema ID: `gts.x.core.oagw.proto.v1~`
///
/// Defines the shape of protocol instances (HTTP/1.1, HTTP/2, HTTP/3, etc.)
/// that routes can support.
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.core.oagw.proto.v1~",
    description = "OAGW protocol type specification",
    properties = "id, display_name, description, priority"
)]
#[derive(Debug, Clone)]
pub struct OagwProtoV1 {
    /// Full GTS instance ID (e.g., `gts.x.core.oagw.proto.v1~x.core.http.http2.v1`).
    pub id: GtsInstanceId,
    /// Human-readable display name.
    pub display_name: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Priority for protocol selection (lower wins).
    #[serde(default)]
    pub priority: i16,
}

/// GTS schema for OAGW stream protocol types.
///
/// Schema ID: `gts.x.core.oagw.stream_proto.v1~`
///
/// Defines the shape of streaming protocol instances (SSE, WebSocket, etc.)
/// that routes can support for streaming responses.
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.core.oagw.stream_proto.v1~",
    description = "OAGW stream protocol type specification",
    properties = "id, display_name, description, priority"
)]
#[derive(Debug, Clone)]
pub struct OagwStreamProtoV1 {
    /// Full GTS instance ID.
    pub id: GtsInstanceId,
    /// Human-readable display name.
    pub display_name: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Priority for protocol selection (lower wins).
    #[serde(default)]
    pub priority: i16,
}

/// GTS schema for OAGW auth type definitions.
///
/// Schema ID: `gts.x.core.oagw.auth_type.v1~`
///
/// Defines the shape of authentication type instances (bearer token, API key, OAuth2, etc.)
/// that routes can use for outbound authentication.
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.core.oagw.auth_type.v1~",
    description = "OAGW auth type specification",
    properties = "id, display_name, description, priority"
)]
#[derive(Debug, Clone)]
pub struct OagwAuthTypeV1 {
    /// Full GTS instance ID.
    pub id: GtsInstanceId,
    /// Human-readable display name.
    pub display_name: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Priority for auth type selection (lower wins).
    #[serde(default)]
    pub priority: i16,
}

/// GTS schema for OAGW link selection strategy types.
///
/// Schema ID: `gts.x.core.oagw.strategy.v1~`
///
/// Defines the shape of strategy instances (sticky session, round robin, priority, etc.)
/// that links can use for load balancing.
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = true,
    schema_id = "gts.x.core.oagw.strategy.v1~",
    description = "OAGW link selection strategy specification",
    properties = "id, display_name, description, priority"
)]
#[derive(Debug, Clone)]
pub struct OagwStrategyV1 {
    /// Full GTS instance ID.
    pub id: GtsInstanceId,
    /// Human-readable display name.
    pub display_name: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Priority for strategy selection (lower wins).
    #[serde(default)]
    pub priority: i16,
}

// ============================================================================
// PLUGIN SCHEMA
// ============================================================================

/// GTS schema for OAGW plugin instances.
///
/// Schema ID: `gts.x.core.modkit.plugin.v1~x.core.oagw.plugin.v1~`
#[struct_to_gts_schema(
    dir_path = "schemas",
    base = BaseModkitPluginV1,
    schema_id = "gts.x.core.modkit.plugin.v1~x.core.oagw.plugin.v1~",
    description = "OAGW plugin specification",
    properties = "supported_protocols, supported_stream_protocols, supported_auth_types, supported_strategies"
)]
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::struct_field_names)] // Fields prefixed for clarity in GTS schema
pub struct OagwPluginSpecV1 {
    /// Supported unary protocol GTS IDs.
    pub supported_protocols: Vec<String>,
    /// Supported streaming protocol GTS IDs.
    pub supported_stream_protocols: Vec<String>,
    /// Supported auth type GTS IDs.
    pub supported_auth_types: Vec<String>,
    /// Supported strategy GTS IDs.
    pub supported_strategies: Vec<String>,
}

// ============================================================================
// WELL-KNOWN INSTANCES HELPERS
// ============================================================================

/// Returns all well-known OAGW GTS schemas as JSON values for registration.
///
/// This includes:
/// - `gts.x.core.oagw.proto.v1~`
/// - `gts.x.core.oagw.stream_proto.v1~`
/// - `gts.x.core.oagw.auth_type.v1~`
/// - `gts.x.core.oagw.strategy.v1~`
///
/// # Errors
///
/// Returns an error if schema JSON cannot be parsed.
pub fn get_oagw_base_schemas() -> Result<Vec<serde_json::Value>> {
    let mut schemas = Vec::new();

    // Protocol schema
    let proto_schema: serde_json::Value =
        serde_json::from_str(&OagwProtoV1::gts_schema_with_refs_as_string())?;
    schemas.push(proto_schema);

    // Stream protocol schema
    let stream_proto_schema: serde_json::Value =
        serde_json::from_str(&OagwStreamProtoV1::gts_schema_with_refs_as_string())?;
    schemas.push(stream_proto_schema);

    // Auth type schema
    let auth_type_schema: serde_json::Value =
        serde_json::from_str(&OagwAuthTypeV1::gts_schema_with_refs_as_string())?;
    schemas.push(auth_type_schema);

    // Strategy schema
    let strategy_schema: serde_json::Value =
        serde_json::from_str(&OagwStrategyV1::gts_schema_with_refs_as_string())?;
    schemas.push(strategy_schema);

    Ok(schemas)
}

/// Returns all well-known OAGW instances as JSON values for registration.
///
/// This includes well-known instances for:
/// - Protocols (HTTP/1.1, HTTP/2, HTTP/3, SSE)
/// - Stream protocols (SSE)
/// - Auth types (bearer token, API key header/query, OAuth2)
/// - Strategies (sticky session, round robin, priority)
pub fn get_oagw_well_known_instances() -> Vec<serde_json::Value> {
    let mut instances = Vec::new();

    // Protocol instances
    instances.push(serde_json::json!({
        "id": protocols::HTTP11,
        "display_name": "HTTP/1.1",
        "description": "HTTP/1.1 protocol for outbound requests",
        "priority": 30
    }));
    instances.push(serde_json::json!({
        "id": protocols::HTTP2,
        "display_name": "HTTP/2",
        "description": "HTTP/2 protocol for outbound requests with multiplexing",
        "priority": 20
    }));
    instances.push(serde_json::json!({
        "id": protocols::HTTP3,
        "display_name": "HTTP/3",
        "description": "HTTP/3 (QUIC) protocol for outbound requests",
        "priority": 10
    }));
    instances.push(serde_json::json!({
        "id": protocols::SSE,
        "display_name": "Server-Sent Events",
        "description": "HTTP-based Server-Sent Events for streaming responses",
        "priority": 25
    }));

    // Stream protocol instances
    instances.push(serde_json::json!({
        "id": stream_protocols::SSE,
        "display_name": "SSE Stream",
        "description": "Server-Sent Events streaming protocol",
        "priority": 10
    }));

    // Auth type instances
    instances.push(serde_json::json!({
        "id": auth_types::BEARER_TOKEN,
        "display_name": "Bearer Token",
        "description": "Bearer token authentication via Authorization header",
        "priority": 10
    }));
    instances.push(serde_json::json!({
        "id": auth_types::API_KEY_HEADER,
        "display_name": "API Key (Header)",
        "description": "API key authentication via custom header",
        "priority": 20
    }));
    instances.push(serde_json::json!({
        "id": auth_types::API_KEY_QUERY,
        "display_name": "API Key (Query)",
        "description": "API key authentication via query parameter",
        "priority": 30
    }));
    instances.push(serde_json::json!({
        "id": auth_types::OAUTH2_CLIENT_CREDS,
        "display_name": "OAuth2 Client Credentials",
        "description": "OAuth2 client credentials flow for machine-to-machine auth",
        "priority": 40
    }));
    instances.push(serde_json::json!({
        "id": auth_types::OAUTH2_TOKEN_EXCHANGE,
        "display_name": "OAuth2 Token Exchange",
        "description": "RFC 8693 OAuth2 token exchange for user delegation",
        "priority": 50
    }));

    // Strategy instances
    instances.push(serde_json::json!({
        "id": strategies::PRIORITY,
        "display_name": "Priority",
        "description": "Select link with lowest priority value (default strategy)",
        "priority": 10
    }));
    instances.push(serde_json::json!({
        "id": strategies::ROUND_ROBIN,
        "display_name": "Round Robin",
        "description": "Distribute requests across links in round-robin fashion",
        "priority": 20
    }));
    instances.push(serde_json::json!({
        "id": strategies::STICKY_SESSION,
        "display_name": "Sticky Session",
        "description": "Route requests from same tenant/user to same link",
        "priority": 30
    }));

    instances
}
