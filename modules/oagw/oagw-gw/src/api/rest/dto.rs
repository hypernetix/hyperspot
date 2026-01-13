//! REST DTOs for OAGW.
//!
//! These DTOs have serde and utoipa derives for REST serialization.

use chrono::{DateTime, Utc};
use modkit_db_macros::ODataFilterable;
use oagw_sdk::{
    HttpMethod, Link, LinkPatch, NewLink, NewRoute, OagwInvokeRequest, OagwInvokeResponse,
    RetryIntent, Route, RoutePatch,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

// === Route DTOs ===

/// Response DTO for a route.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ODataFilterable)]
#[serde(rename_all = "camelCase")]
pub struct RouteDto {
    #[odata(filter(kind = "Uuid"))]
    pub id: Uuid,
    #[odata(filter(kind = "Uuid"))]
    pub tenant_id: Uuid,
    #[odata(filter(kind = "String"))]
    pub base_url: String,
    #[odata(filter(kind = "I64"))]
    pub rate_limit_req_per_min: i32,
    #[odata(filter(kind = "String"))]
    pub auth_type_gts_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_ttl_sec: Option<i32>,
    pub supported_protocols: Vec<String>,
    #[odata(filter(kind = "DateTimeUtc"))]
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Route> for RouteDto {
    fn from(route: Route) -> Self {
        Self {
            id: route.id,
            tenant_id: route.tenant_id,
            base_url: route.base_url,
            rate_limit_req_per_min: route.rate_limit_req_per_min,
            auth_type_gts_id: route.auth_type_gts_id,
            cache_ttl_sec: route.cache_ttl_sec,
            supported_protocols: route.supported_protocols,
            created_at: route.created_at,
            updated_at: route.updated_at,
        }
    }
}

/// Request DTO for creating a route.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateRouteRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub base_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_req_per_min: Option<i32>,
    pub auth_type_gts_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_ttl_sec: Option<i32>,
    #[serde(default)]
    pub supported_protocols: Vec<String>,
}

impl CreateRouteRequest {
    /// Convert to domain model with tenant ID from security context.
    pub fn into_new_route(self, tenant_id: Uuid) -> NewRoute {
        NewRoute {
            id: self.id,
            tenant_id,
            base_url: self.base_url,
            rate_limit_req_per_min: self.rate_limit_req_per_min,
            auth_type_gts_id: self.auth_type_gts_id,
            cache_ttl_sec: self.cache_ttl_sec,
            supported_protocols: self.supported_protocols,
        }
    }
}

/// Request DTO for updating a route.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRouteRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate_limit_req_per_min: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_type_gts_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[allow(clippy::option_option)] // Intentional: distinguish set-to-null from not-set
    pub cache_ttl_sec: Option<Option<i32>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_protocols: Option<Vec<String>>,
}

impl From<UpdateRouteRequest> for RoutePatch {
    fn from(req: UpdateRouteRequest) -> Self {
        Self {
            base_url: req.base_url,
            rate_limit_req_per_min: req.rate_limit_req_per_min,
            auth_type_gts_id: req.auth_type_gts_id,
            cache_ttl_sec: req.cache_ttl_sec,
            supported_protocols: req.supported_protocols,
        }
    }
}

// === Link DTOs ===

/// Response DTO for a link.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ODataFilterable)]
#[serde(rename_all = "camelCase")]
pub struct LinkDto {
    #[odata(filter(kind = "Uuid"))]
    pub id: Uuid,
    #[odata(filter(kind = "Uuid"))]
    pub tenant_id: Uuid,
    #[odata(filter(kind = "Uuid"))]
    pub secret_ref: Uuid,
    #[odata(filter(kind = "Uuid"))]
    pub route_id: Uuid,
    #[odata(filter(kind = "String"))]
    pub secret_type_gts_id: String,
    #[odata(filter(kind = "Bool"))]
    pub enabled: bool,
    #[odata(filter(kind = "I64"))]
    pub priority: i32,
    #[odata(filter(kind = "String"))]
    pub strategy_gts_id: String,
    #[odata(filter(kind = "DateTimeUtc"))]
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Link> for LinkDto {
    fn from(link: Link) -> Self {
        Self {
            id: link.id,
            tenant_id: link.tenant_id,
            secret_ref: link.secret_ref,
            route_id: link.route_id,
            secret_type_gts_id: link.secret_type_gts_id,
            enabled: link.enabled,
            priority: link.priority,
            strategy_gts_id: link.strategy_gts_id,
            created_at: link.created_at,
            updated_at: link.updated_at,
        }
    }
}

/// Request DTO for creating a link.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateLinkRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Uuid>,
    pub secret_ref: Uuid,
    pub route_id: Uuid,
    pub secret_type_gts_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    pub strategy_gts_id: String,
}

impl CreateLinkRequest {
    /// Convert to domain model with tenant ID from security context.
    pub fn into_new_link(self, tenant_id: Uuid) -> NewLink {
        NewLink {
            id: self.id,
            tenant_id,
            secret_ref: self.secret_ref,
            route_id: self.route_id,
            secret_type_gts_id: self.secret_type_gts_id,
            enabled: self.enabled,
            priority: self.priority,
            strategy_gts_id: self.strategy_gts_id,
        }
    }
}

/// Request DTO for updating a link.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLinkRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_ref: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_type_gts_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy_gts_id: Option<String>,
}

impl From<UpdateLinkRequest> for LinkPatch {
    fn from(req: UpdateLinkRequest) -> Self {
        Self {
            secret_ref: req.secret_ref,
            secret_type_gts_id: req.secret_type_gts_id,
            enabled: req.enabled,
            priority: req.priority,
            strategy_gts_id: req.strategy_gts_id,
        }
    }
}

// === Invocation DTOs ===

/// HTTP method for invocation requests.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethodDto {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl From<HttpMethodDto> for HttpMethod {
    fn from(dto: HttpMethodDto) -> Self {
        match dto {
            HttpMethodDto::Get => Self::Get,
            HttpMethodDto::Post => Self::Post,
            HttpMethodDto::Put => Self::Put,
            HttpMethodDto::Patch => Self::Patch,
            HttpMethodDto::Delete => Self::Delete,
            HttpMethodDto::Head => Self::Head,
            HttpMethodDto::Options => Self::Options,
        }
    }
}

impl From<HttpMethod> for HttpMethodDto {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => Self::Get,
            HttpMethod::Post => Self::Post,
            HttpMethod::Put => Self::Put,
            HttpMethod::Patch => Self::Patch,
            HttpMethod::Delete => Self::Delete,
            HttpMethod::Head => Self::Head,
            HttpMethod::Options => Self::Options,
        }
    }
}

/// Request DTO for invoking an outbound API.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InvokeRequest {
    /// Optional link ID to use (if not provided, OAGW selects based on route).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_id: Option<Uuid>,
    /// Route ID identifying the downstream API configuration.
    pub route_id: Uuid,
    /// HTTP method for the request.
    pub method: HttpMethodDto,
    /// Path to append to the route's base URL.
    pub path: String,
    /// Optional query parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<HashMap<String, String>>,
    /// Optional custom headers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
    /// Optional request body (base64 encoded for binary).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Optional timeout override in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

impl From<InvokeRequest> for OagwInvokeRequest {
    fn from(req: InvokeRequest) -> Self {
        Self {
            link_id: req.link_id,
            route_id: req.route_id,
            method: req.method.into(),
            path: req.path,
            query: req.query,
            headers: req.headers,
            body: req.body.map(bytes::Bytes::from),
            timeout_ms: req.timeout_ms,
            retry_intent: RetryIntent::default(), // No retry by default for REST
        }
    }
}

/// Response DTO for an invocation.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct InvokeResponse {
    /// HTTP status code from downstream.
    pub status_code: u16,
    /// Response headers.
    pub headers: HashMap<String, String>,
    /// Response body (base64 encoded for binary).
    pub body: String,
    /// Total duration in milliseconds.
    pub duration_ms: u64,
    /// Link ID that was used for the invocation.
    pub link_id: Uuid,
    /// Retry-After hint propagated from downstream (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_sec: Option<u64>,
    /// Which attempt succeeded (1-based).
    pub attempt_number: u32,
}

impl From<OagwInvokeResponse> for InvokeResponse {
    fn from(resp: OagwInvokeResponse) -> Self {
        Self {
            status_code: resp.status_code,
            headers: resp.headers,
            body: String::from_utf8_lossy(&resp.body).to_string(),
            duration_ms: resp.duration_ms,
            link_id: resp.link_id,
            retry_after_sec: resp.retry_after_sec,
            attempt_number: resp.attempt_number,
        }
    }
}

// === Health DTOs ===

/// Health status response.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub status: String,
}

impl Default for HealthResponse {
    fn default() -> Self {
        Self {
            status: "healthy".to_string(),
        }
    }
}
