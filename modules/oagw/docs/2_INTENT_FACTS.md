> This is module facts - generated from 1_INTENT.md and guidelines/NEW_MODULE.md, docs/MODKIT_PLUGINS.md, docs/MODKIT_UNIFIED_SYSTEM.md, examples/

# OAGW Module Facts

This document captures all facts required to implement the OAGW (Outbound API Gateway) module for managing outbound traffic to external APIs.

---

## F0 — Context & System Definition

### F00 — Overview

**[F00.001]** – Service Mission Statement
> OAGW is a reusable **outbound API gateway** responsible for invoking external **HTTP APIs** (including **HTTP SSE** streaming) with pluggable protocol, authentication, and streaming implementations. It manages tenant-scoped route/link configuration, delegates actual invocation to selected plugins, and provides integrated circuit breaking, error normalization, and `Retry-After` propagation.
> **OAGW performs no implicit retries** — retries occur only when explicitly requested by the caller via `RetryIntent`.

**[F00.002]** – Bounded Context Definition
> OAGW owns **outbound transport orchestration only**. It does NOT:
> - Parse or interpret protocol-specific payloads (that's the consumer's responsibility)
> - Implement vendor-specific logic (delegated to plugins)
> - Store credentials directly (references `cred_store` by UUID)

**[F00.003]** – Non-Goals / Explicitly Unsupported Behavior
> - Inbound request handling (reverse proxy)
> - Protocol payload parsing (e.g., LLM response parsing)
> - Direct credential storage (uses `cred_store` references)
> - WebSocket bidirectional streaming (v1 scope)
> - **Automatic retry** (caller controls via `RetryIntent`)
> - **Job scheduling or background processing** (OAGW is a synchronous gateway)
> - **Streaming retry** (caller must restart stream if protocol supports resume)

**[F00.004]** – Service Dependencies Map
> | Dependency | Purpose |
> |------------|---------|
> | `types_registry` | GTS schema/instance registration |
> | `cred_store` | Secret material retrieval by UUID reference |
> | `api_ingress` | REST API hosting |
> | `modkit-db` | Database persistence |
> | `modkit-auth` | SecurityCtx authorization |

**[F00.005]** – Data Ownership Boundaries
> - **Owns**: `outbound_api_route`, `outbound_api_link`, `outbound_api_route_limits`, `outbound_api_audit_log`
> - **References**: `cred_store.secret_store.id` (logical FK)
> - **Reads**: `types_registry` for GTS instances

---

### F01 — Project Facts

**[F01.001]** – Programming Language & Runtime
> Rust 2021 edition, async runtime: Tokio

**[F01.002]** – Project Layout Conventions
> SDK pattern: `oagw-sdk/` (public API) + `oagw-gw/` (gateway implementation)
> Plugin: `plugins/oagw-generic` (all in-one implementation)
> See [NEW_MODULE.md](../../guidelines/NEW_MODULE.md)

**[F01.003]** – Coding Standards
> - `#![forbid(unsafe_code)]` in SDK crate
> - All API methods accept `&SecurityCtx`
> - SDK types have NO `serde` derives
> - RFC-9457 Problem Details for errors

**[F01.004]** – Versioning Strategy
> REST API versioned: `/oagw/v1/...`
> GTS schemas versioned: `gts.x.core.oagw.*.v1~`

---

### F01 — Main Rust Entrypoints

**[F01.010]** – SDK Crate (`oagw-sdk`)
> - `OagwApi` trait: `oagw_sdk::api::OagwApi`
> - `OagwError` enum: `oagw_sdk::error::OagwError`
> - `OagwInvokeRequest`: `oagw_sdk::types::OagwInvokeRequest`
> - `OagwInvokeResponse`: `oagw_sdk::types::OagwInvokeResponse`
> - `RetryIntent`: `oagw_sdk::retry::RetryIntent`
> - `RetryBudget`: `oagw_sdk::retry::RetryBudget`

**[F01.011]** – Gateway Crate (`oagw-gw`)
> - Module struct: `oagw_gw::OagwModule`
> - Service impl: `oagw_gw::service::OagwService`
> - REST handlers: `oagw_gw::api::rest::handlers`
> - Domain entities: `oagw_gw::domain::entities`

**[F01.012]** – Plugin Crate (`oagw-generic`)
> - Plugin impl: `oagw_generic::HttpPlugin`
> - Plugin trait: `oagw_sdk::plugin::OagwPluginApi`

---

### F02 — External Dependencies

**[F02.001]** – HTTP Client Library
> `reqwest` for HTTP/1.1 and HTTP/2 requests
> `modkit::http::TracedClient` for OpenTelemetry trace context injection

**[F02.002]** – HTTP/3 Client Library
> `reqwest` with `http3` feature or `h3` crate (future)

**[F02.003]** – SSE Client Library
> `reqwest` + `eventsource-client` or `reqwest-eventsource` for Server-Sent Events parsing

**[F02.004]** – gRPC Client Library (Not in v1)
> Not implemented in the current scope. OAGW supports **HTTP + SSE only** for now.

**[F02.005]** – Circuit Breaker Library
> `failsafe` or `tower` crate for circuit breaker implementation

**[F02.006]** – Rate Limiting Library
> `governor` crate for token bucket rate limiting

**[F02.007]** – Caching Library
> `moka` crate for in-memory LRU caching (token cache, response cache)

**[F02.008]** – OAuth2 Library
> `oauth2` crate for token exchange flows
> `jsonwebtoken` for JWT parsing/validation

---

### F03 — Deployment

**[F03.001]** – Health Check Endpoints
> - `GET /oagw/v1/health` — Liveness probe (no auth)
> - `GET /oagw/v1/ready` — Readiness probe (no auth; checks critical dependencies like DB)
> - `GET /oagw/v1/routes/{routeId}/health` — Route health (requires role)

---

### F05 — Secrets Handling

**[F05.001]** – Secret Reference Model
> OAGW stores `secret_ref: uuid` pointing to `cred_store.secret_store.id`
> Never stores actual secrets in OAGW tables

**[F05.002]** – Secret Resolution
> At invocation time: load secret from `cred_store` via `CredStoreApi`
> Apply to request based on `auth_type_gts_id`

---

### F07 — Configuration Management

**[F07.001]** – Module Config Structure
```rust
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct OagwConfig {
    pub token_cache_max_capacity: usize,      // default: 10_000
    pub token_cache_max_ttl_sec: u64,         // default: 3600
    pub token_cache_safety_margin_sec: u64,   // default: 60
    pub response_cache_max_capacity: usize,   // default: 1_000
    pub max_inflight_bytes_per_invocation: usize, // default: 8_388_608 (8 MiB)
    pub max_stream_chunk_size_bytes: usize,       // default: 65_536 (64 KiB)
    pub default_connection_timeout_ms: u64,   // default: 5_000
    pub default_request_timeout_ms: u64,      // default: 30_000
    pub default_idle_timeout_ms: u64,         // default: 60_000
    pub default_rate_limit_per_min: u32,      // default: 1_000
    pub circuit_breaker_threshold: u32,       // default: 5
    pub circuit_breaker_timeout_sec: u32,     // default: 30
    pub circuit_breaker_success_threshold: u32, // default: 2
    pub audit_guarantee: AuditGuarantee,          // default: BestEffort
    pub max_audit_latency_ms: u64,                // default: 50
}

#[derive(Debug, Clone, Deserialize)]
pub enum AuditGuarantee {
    BestEffort,
    Guaranteed,
    FailClosed,
}
```

---

### F08 — Logging & Tracing

**[F08.001]** – Span Attributes
> All operations create spans with:
> - `tenant_id`, `user_id` (if available)
> - `route_id`, `link_id`
> - `protocol`, `auth_type`
> - `target_url`
> - `http.method`, `http.status_code` (for HTTP)

**[F08.002]** – Tracing Crates
> `tracing`, `tracing-opentelemetry` for distributed tracing

---

## F1 — GTS Types & Instances

### F10 — GTS Schemas (Types)

**[F10.001]** – Protocol Schema
> `gts.x.core.oagw.proto.v1~`
> Fields: `id`, `display_name`, `description?`, `priority?`

**[F10.002]** – Auth Type Schema
> `gts.x.core.oagw.auth_type.v1~`
> Fields: `id`, `display_name`, `description?`

**[F10.003]** – Strategy Schema
> `gts.x.core.oagw.strategy.v1~`
> Fields: `id`, `display_name`, `description?`

---

### F11 — Well-Known Protocol Instances

**[F11.001]** – HTTP/1.1 Protocol
> `gts.x.core.oagw.proto.v1~x.core.http.http11.v1`

**[F11.002]** – HTTP/2 Protocol
> `gts.x.core.oagw.proto.v1~x.core.http.http2.v1`

**[F11.003]** – HTTP/3 Protocol
> `gts.x.core.oagw.proto.v1~x.core.http.http3.v1`

**[F11.004]** – SSE Protocol
> `gts.x.core.oagw.proto.v1~x.core.http.sse.v1`

**[F11.005]** – gRPC Protocol (Not supported)
> Not supported in the current scope.

---

### F12 — Well-Known Strategy Instances

**[F12.001]** – Sticky Session Strategy
> `gts.x.core.oagw.strategy.v1~x.core._.sticky_session.v1`
> Sticks to previously selected link for `(tenant_id, user_id)`

**[F12.002]** – Round Robin Strategy
> `gts.x.core.oagw.strategy.v1~x.core._.round_robin.v1`
> Distributes invocations across links for same route

---

### F13 — Well-Known Auth Type Instances

**[F13.001]** – Bearer Token Auth
> `gts.x.core.oagw.auth_type.v1~x.core.auth.bearer_token.v1`
> Header: `Authorization: Bearer <token>`

**[F13.002]** – API Key Header Auth
> `gts.x.core.oagw.auth_type.v1~x.core.auth.api_key_header.v1`
> Custom header with API key (e.g., `X-API-Key: <key>`)

**[F13.003]** – API Key Query Auth
> `gts.x.core.oagw.auth_type.v1~x.core.auth.api_key_query.v1`
> Query parameter with API key (e.g., `?key=<key>`)

**[F13.004]** – OAuth2 Client Credentials Auth
> `gts.x.core.oagw.auth_type.v1~x.core.auth.oauth2_client_creds.v1`
> OAuth2 client credentials flow

**[F13.005]** – OAuth2 Token Exchange Auth
> `gts.x.core.oagw.auth_type.v1~x.core.auth.oauth2_token_exchange.v1`
> RFC 8693 token exchange

---

## F2 — Domain Model

### F20 — Domain Entities

**[F20.001]** – OutboundApiRoute Entity
> Represents a downstream API endpoint configuration
> Fields: `id`, `tenant_id`, `base_url`, `rate_limit_req_per_min`, `auth_type_gts_id`, `cache_ttl_sec?`

**[F20.002]** – OutboundApiRouteSupportedProtocol Entity
> M:N relation between route and supported protocols
> Fields: `id`, `route_id`, `protocol_gts_id`

**[F20.003]** – OutboundApiLink Entity
> Represents a tenant's configured connection to a route with credentials
> Fields: `id`, `tenant_id`, `secret_ref`, `route_id`, `secret_type_gts_id`, `enabled`, `priority`, `strategy_gts_id`

**[F20.004]** – OutboundApiRouteLimits Entity
> Configurable limits per route or per link
> Fields: `id`, `tenant_id`, `link_id?`, `route_id?`, timeouts, circuit breaker settings, size limits, rate limits

**[F20.005]** – OutboundApiAuditLog Entity
> Audit trail for all outbound invocations
> Fields: `id`, `tenant_id`, `user_id?`, `route_id`, `link_id`, `operation`, `target_url`, `status_code?`, `duration_ms`, `error_message?`, `trace_id`, `timestamp`

---

### F21 — Value Objects

**[F21.001]** – OagwInvokeRequest
```rust
pub struct OagwInvokeRequest {
    pub link_id: Option<Uuid>,
    pub route_id: Uuid,
    pub method: HttpMethod,
    pub path: String,
    pub query: Option<HashMap<String, String>>,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<Bytes>,
    pub timeout_ms: Option<u64>,
    pub retry_intent: RetryIntent,      // Default: max_attempts=1 (no retry)
}
```

**[F21.002]** – OagwInvokeResponse
```rust
pub struct OagwInvokeResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: Bytes,
    pub duration_ms: u64,
    pub link_id: Uuid,
    pub retry_after_sec: Option<u64>,   // Propagated from downstream
    pub attempt_number: u32,            // Which attempt succeeded (1-based)
}
```

**[F21.003]** – OagwResponseStream
```rust
pub type OagwResponseStream = Pin<Box<dyn Stream<Item = Result<OagwStreamChunk, OagwStreamAbort>> + Send>>;

pub struct OagwStreamChunk {
    pub data: Bytes,
    pub event_type: Option<String>,
    pub event_id: Option<String>,       // SSE Last-Event-ID for resume
}
```

**[F21.004]** – OagwStreamAbort (for stream failures)
```rust
pub struct OagwStreamAbort {
    pub gts_id: GtsInstanceId,            // gts.x.core.errors.err.v1~x.oagw.stream.aborted.v1
    pub bytes_received: u64,              // How much was received before failure
    pub abort_reason: StreamAbortReason,  // network | protocol | auth | timeout
    pub resumable: bool,
    pub resume_hint: Option<String>,      // e.g., SSE Last-Event-ID
    pub detail: Option<String>,
}

pub enum StreamAbortReason {
    Network,
    Protocol,
    Auth,
    Timeout,
}
```

---

## F2 — REST API

### F20 — REST Handlers

**[F20.001]** – POST /oagw/v1/routes
> Create a new outbound route

**[F20.002]** – GET /oagw/v1/routes
> List routes with OData pagination

**[F20.003]** – GET /oagw/v1/routes/{routeId}
> Get route by ID

**[F20.004]** – PATCH /oagw/v1/routes/{routeId}
> Update route

**[F20.005]** – DELETE /oagw/v1/routes/{routeId}
> Delete route

**[F20.006]** – POST /oagw/v1/links
> Create a new outbound link

**[F20.007]** – GET /oagw/v1/links
> List links with OData pagination

**[F20.008]** – GET /oagw/v1/links/{linkId}
> Get link by ID

**[F20.009]** – PATCH /oagw/v1/links/{linkId}
> Update link

**[F20.010]** – DELETE /oagw/v1/links/{linkId}
> Delete link

**[F20.011]** – POST /oagw/v1/invoke
> Execute outbound invocation via REST

**[F20.012]** – DELETE /oagw/v1/routes/{routeId}/cache/tokens
> Clear cached tokens for route

**[F20.013]** – GET /oagw/v1/health
> Liveness probe (no auth)

**[F20.014]** – GET /oagw/v1/ready
> Readiness probe (no auth)

**[F20.015]** – GET /oagw/v1/routes/{routeId}/health
> Route health check (requires role)

---

### F27 — REST Status Codes

**[F27.001]** – 200 OK
> Successful GET/PATCH operations

**[F27.002]** – 201 Created
> Successful POST (create) operations

**[F27.003]** – 204 No Content
> Successful DELETE operations

**[F27.004]** – 400 Bad Request
> Invalid request payload

**[F27.005]** – 401 Unauthorized
> Missing or invalid authentication

**[F27.006]** – 403 Forbidden
> Insufficient permissions

**[F27.007]** – 404 Not Found
> Route/Link not found

**[F27.008]** – 409 Conflict
> Duplicate resource

**[F27.009]** – 413 Payload Too Large
> Request size exceeds limit

**[F27.010]** – 429 Too Many Requests
> Rate limit exceeded

**[F27.011]** – 502 Bad Gateway
> Downstream error

**[F27.012]** – 503 Service Unavailable
> Circuit breaker open / all links down

**[F27.013]** – 504 Gateway Timeout
> Request/connection timeout

---

## F4 — Security Model

### F41 — Authentication

**[F41.001]** – SecurityCtx Propagation
> All API methods receive `&SecurityCtx` for tenant isolation and audit

### F42 — Authorization (Roles)

**[F42.001]** – Route Health Role
> `gts.x.core.idp.role.v1~x.oagw.route.health.v1` — Required for `/routes/{routeId}/health`

**[F42.002]** – Route Admin Role
> `gts.x.core.idp.role.v1~x.oagw.route.admin.v1` — Full CRUD on routes

**[F42.003]** – Link Admin Role
> `gts.x.core.idp.role.v1~x.oagw.link.admin.v1` — Full CRUD on links

**[F42.004]** – Invoke Role
> `gts.x.core.idp.role.v1~x.oagw.invoke.v1` — Can invoke via REST API

### F43 — Transport Security (Outbound)

**[F43.001]** – TLS / mTLS Requirements
> - Outbound HTTP connections MUST validate TLS certificates by default.
> - The system MUST support configuring trust roots and certificate pinning policies per route/link.
> - The system SHOULD support mTLS (client certificates) per link.

**[F43.002]** – Secret Rotation & Long-Lived Connections
> - Secrets referenced via `cred_store` MAY rotate.
> - OAGW MUST bound connection reuse (idle timeout) and SHOULD avoid holding credentials indefinitely in memory.
> - For long-lived SSE streams, credential refresh semantics MUST be explicit (caller-initiated stream restart).

---

## F5 — Service Layer

### F50 — Service Methods (OagwApi Trait)

**[F50.001]** – invoke_unary
```rust
async fn invoke_unary(&self, ctx: &SecurityCtx, req: OagwInvokeRequest) -> Result<OagwInvokeResponse, OagwError>;
```

**[F50.002]** – invoke_stream
```rust
async fn invoke_stream(&self, ctx: &SecurityCtx, req: OagwInvokeRequest) -> Result<OagwResponseStream, OagwError>;
```

**[F50.003]** – Route CRUD
```rust
async fn create_route(&self, ctx: &SecurityCtx, req: NewRoute) -> Result<Route, OagwError>;
async fn get_route(&self, ctx: &SecurityCtx, id: Uuid) -> Result<Route, OagwError>;
async fn list_routes(&self, ctx: &SecurityCtx, query: ODataQuery) -> Result<Page<Route>, OagwError>;
async fn update_route(&self, ctx: &SecurityCtx, id: Uuid, patch: RoutePatch) -> Result<Route, OagwError>;
async fn delete_route(&self, ctx: &SecurityCtx, id: Uuid) -> Result<(), OagwError>;
```

**[F50.004]** – Link CRUD
```rust
async fn create_link(&self, ctx: &SecurityCtx, req: NewLink) -> Result<Link, OagwError>;
async fn get_link(&self, ctx: &SecurityCtx, id: Uuid) -> Result<Link, OagwError>;
async fn list_links(&self, ctx: &SecurityCtx, query: ODataQuery) -> Result<Page<Link>, OagwError>;
async fn update_link(&self, ctx: &SecurityCtx, id: Uuid, patch: LinkPatch) -> Result<Link, OagwError>;
async fn delete_link(&self, ctx: &SecurityCtx, id: Uuid) -> Result<(), OagwError>;
```

---

### F51 — Concurrency Model

**[F51.001]** – Token Cache Deduplication
> Deduplicate concurrent token exchanges for same cache key using `tokio::sync::OnceCell` or similar

**[F51.002]** – Circuit Breaker State
> Thread-safe circuit breaker state scoped per **(provider_id, route_id, tenant_id, link_id)**.
> Optional aggregate state may be maintained per **(tenant_id, route_id)** strictly for reporting/health aggregation.
> Circuit breakers MUST NOT be shared across tenants.

---

### F52 — Caching

**[F52.001]** – Token Cache
> Key: `(tenant_id, user_id, route_id, auth_type_gts_id, scopes)`
> TTL: `token.exp - safety_margin`
> Max TTL: 3600s
> Max capacity: 10,000 entries

**[F52.002]** – Response Cache (Optional)
> Key: `(route_id, request_path, query_params, relevant_headers)`
> TTL: from `route.cache_ttl_sec`
> Bypass on `Cache-Control: no-store`

---

### F56 — Rate Limiting Governance

**[F56.001]** – Scope
> Rate limiting MUST be tenant-isolated and at minimum support per **(tenant_id, route_id)**.
> Optional tighter scopes may be added (e.g., per link, per user) but MUST NOT cross tenant boundaries.

**[F56.002]** – Distributed Rate Limiting (v4+)
> For multi-instance deployments, rate limiting MUST support a Redis-backed distributed implementation.

---

### F53 — Observability (Metrics)

**[F53.001]** – Invocation Counter
> `oagw.invocations_total` (counter, labels: tenant_id, route_id, status)

**[F53.002]** – Request Duration Histogram
> `oagw.request.duration_msec` (histogram, labels: route_id, protocol)

**[F53.003]** – Error Counter
> `oagw.errors_total` (counter, labels: error_type, route_id)

**[F53.004]** – Active Connections Gauge
> `oagw.active_connections` (gauge, labels: route_id)

**[F53.005]** – Circuit Breaker Metrics
> `oagw.circuit_breaker.opened` (counter, labels: link_id)
> `oagw.circuit_breaker.state` (gauge, labels: link_id)

**[F53.006]** – Bytes Transferred
> `oagw.bytes_sent_total` (counter, labels: tenant_id, route_id, link_id)
> `oagw.bytes_received_total` (counter, labels: tenant_id, route_id, link_id)

**[F53.007]** – Token Cache Metrics
> `oagw.token_cache.hit_total` (counter)
> `oagw.token_cache.miss_total` (counter)
> `oagw.token_cache.size` (gauge)

---

### F54 — Error Handling

**[F54.001]** – Error GTS Schema
> `gts.x.core.errors.err.v1~` — Base error schema
> All OAGW errors are registered as GTS instances under this schema

**[F54.002]** – Error GTS Instances (Error Taxonomy)
> | Error Type | HTTP | GTS Instance ID | Retriable |
> |------------|------|-----------------|--------|
> | RouteNotFound | 404 | `gts.x.core.errors.err.v1~x.oagw.route.not_found.v1` | No |
> | LinkNotFound | 404 | `gts.x.core.errors.err.v1~x.oagw.link.not_found.v1` | No |
> | LinkUnavailable | 503 | `gts.x.core.errors.err.v1~x.oagw.link.unavailable.v1` | Yes |
> | CircuitBreakerOpen | 503 | `gts.x.core.errors.err.v1~x.oagw.circuit_breaker.open.v1` | Yes |
> | ConnectionTimeout | 504 | `gts.x.core.errors.err.v1~x.oagw.timeout.connection.v1` | Yes |
> | RequestTimeout | 504 | `gts.x.core.errors.err.v1~x.oagw.timeout.request.v1` | Yes |
> | IdleTimeout | 504 | `gts.x.core.errors.err.v1~x.oagw.timeout.idle.v1` | Yes |
> | RateLimitExceeded | 429 | `gts.x.core.errors.err.v1~x.oagw.rate_limit.exceeded.v1` | Yes* |
> | PayloadTooLarge | 413 | `gts.x.core.errors.err.v1~x.oagw.payload.too_large.v1` | No |
> | ProtocolError | 502 | `gts.x.core.errors.err.v1~x.oagw.protocol.error.v1` | No |
> | AuthenticationFailed | 401 | `gts.x.core.errors.err.v1~x.oagw.auth.failed.v1` | No |
> | SecretNotFound | 500 | `gts.x.core.errors.err.v1~x.oagw.secret.not_found.v1` | No |
> | PluginNotFound | 503 | `gts.x.core.errors.err.v1~x.oagw.plugin.not_found.v1` | No |
> | DownstreamError | 502 | `gts.x.core.errors.err.v1~x.oagw.downstream.error.v1` | Depends |
> | StreamAborted | 502 | `gts.x.core.errors.err.v1~x.oagw.stream.aborted.v1` | No** |
> | ValidationError | 400 | `gts.x.core.errors.err.v1~x.oagw.validation.error.v1` | No |
>
> \* Retriable after `Retry-After` delay
> \** Streaming errors are never auto-retried; caller may restart stream

**[F54.003]** – Error Response Structure (RFC-9457)
```rust
pub struct OagwError {
    pub gts_id: GtsInstanceId,      // e.g., gts.x.core.errors.err.v1~x.oagw.timeout.connection.v1
    pub status: u16,
    pub title: String,
    pub detail: Option<String>,
    pub retry_after_sec: Option<u64>,  // Propagated from downstream or circuit breaker
    pub retriable: bool,               // Hint for caller's RetryIntent
    pub link_id: Option<Uuid>,         // Which link failed (if applicable)
    pub downstream_status: Option<u16>, // Original status from downstream
}
```

**[F54.004]** – Circuit Breaker Configuration
> - Failure threshold: 5 consecutive failures
> - Half-open timeout: 30 seconds
> - Success threshold to close: 2 successes

---

### F55 — Retry Policy (Client-Controlled)

**[F55.001]** – Retry Intent Structure
> Retry is **fully controlled by the caller**, never automatic in OAGW.
```rust
pub struct RetryIntent {
    pub max_attempts: u32,              // 1 = no retry (default)
    pub retry_on: Vec<RetryOn>,         // Declarative retry conditions
    pub scope: RetryScope,              // same_link | different_link | reroute
    pub allow_strategy_reselect: bool,  // Can re-run strategy when switching links
    pub backoff: BackoffStrategy,       // Exponential, linear, constant
    pub budget: Option<Arc<RetryBudget>>, // Optional shared budget
}

pub enum RetryOn {
    Timeout,
    ConnectError,
    StatusClass(StatusClass),
    ErrorGtsId(GtsInstanceId),
}

pub enum StatusClass {
    C5xx,
    C429,
}

pub enum RetryScope {
    SameLink,
    DifferentLink,
    Reroute,
}

pub enum BackoffStrategy {
    None,
    Constant { delay_ms: u64 },
    Linear { initial_ms: u64, increment_ms: u64, max_ms: u64 },
    Exponential { initial_ms: u64, multiplier: f64, max_ms: u64 },
}
```

**[F55.002]** – Default Retry Intent
> Default is **no retry**: `RetryIntent { max_attempts: 1, .. }`
> Caller must explicitly opt-in to retries.

**[F55.003]** – Retry Budget
> Shared budget to limit total retries across multiple calls.
```rust
pub struct RetryBudget {
    pub max_retries: AtomicU32,         // Total retries allowed in window
    pub time_window_sec: u64,           // Budget replenishes after window
    pub min_retries_per_sec: f64,       // Minimum retry rate guarantee
}
```

**[F55.004]** – Retry Decision Flow (Caller-Side)
> OAGW may perform retries **within the same invocation** when requested by the caller via `RetryIntent`.
> - Retry state is **ephemeral** and lives only in the invocation context.
> - `attempt_number` is **semantic** (1-based) and MUST reflect the attempt that produced the returned result.

**[F55.005]** – Retry-After Propagation
> OAGW **always propagates** `Retry-After` from:
> - Downstream `429` responses
> - Downstream `503` responses with `Retry-After` header
> - Circuit breaker half-open timeout
> - Rate limiter quota reset time

**[F55.006]** – Streaming Retry Policy
> **OAGW never retries streaming requests.**
> - If stream fails mid-way, the stream yields a terminal `OagwStreamAbort`
> - Abort includes `bytes_received` and `resume_hint` (if SSE)
> - Caller may restart stream using protocol-specific resume (e.g., SSE `Last-Event-ID`)
> - Resume capability is protocol-dependent, not OAGW's responsibility

**[F55.007]** – Route-Level Retry Policy
> Routes can define **allowed** retry behavior (limits, not defaults):
```rust
pub struct RouteRetryPolicy {
    pub allow_retry: bool,              // false = reject any RetryIntent with max_attempts > 1
    pub max_attempts_limit: u32,        // Cap on caller's max_attempts
    pub allowed_error_classes: Vec<GtsInstanceId>,  // Errors that may be retried
}
```

## F6 — Outbound Gateways (Plugin Interface)

### F60 — Plugin API Trait

**[F60.001]** – OagwPluginApi Trait
```rust
#[async_trait]
pub trait OagwPluginApi: Send + Sync {
    fn supported_protocols(&self) -> &[GtsInstanceId];
    fn supported_stream_protocols(&self) -> &[GtsInstanceId];
    fn supported_auth_types(&self) -> &[GtsInstanceId];
    fn supported_strategies(&self) -> &[GtsInstanceId];
    fn priority(&self) -> i16;

    async fn invoke_unary(
        &self,
        ctx: &SecurityCtx,
        link: &Link,
        route: &Route,
        secret: &Secret,
        req: OagwInvokeRequest,
    ) -> Result<OagwInvokeResponse, OagwError>;

    async fn invoke_stream(
        &self,
        ctx: &SecurityCtx,
        link: &Link,
        route: &Route,
        secret: &Secret,
        req: OagwInvokeRequest,
    ) -> Result<OagwResponseStream, OagwError>;
}
```

**[F60.002]** – Plugin Selection
> Filter plugins by: protocol + auth_type
> Select lowest priority plugin

---

## F7 — Persistence

### F70 — DB Connectivity

**[F70.001]** – Database Backend
> PostgreSQL/MariaDB/SQLite via SeaORM

### F73 — Schemas

**[F73.001]** – outbound_api_route Table
```sql
CREATE TABLE outbound_api_route (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    base_url TEXT NOT NULL,
    rate_limit_req_per_min INT NOT NULL DEFAULT 1000,
    auth_type_gts_id TEXT NOT NULL,
    cache_ttl_sec INT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**[F73.002]** – outbound_api_route_supported_protocol Table
```sql
CREATE TABLE outbound_api_route_supported_protocol (
    id UUID PRIMARY KEY,
    route_id UUID NOT NULL REFERENCES outbound_api_route(id) ON DELETE CASCADE,
    protocol_gts_id TEXT NOT NULL
);
```

**[F73.003]** – outbound_api_link Table
```sql
CREATE TABLE outbound_api_link (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    secret_ref UUID NOT NULL,
    route_id UUID NOT NULL REFERENCES outbound_api_route(id),
    secret_type_gts_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    priority INT NOT NULL DEFAULT 0,
    strategy_gts_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**[F73.004]** – outbound_api_route_limits Table
```sql
CREATE TABLE outbound_api_route_limits (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    link_id UUID NULL UNIQUE,
    route_id UUID NULL UNIQUE,
    connection_timeout_ms INT NOT NULL DEFAULT 5000,
    request_timeout_ms INT NOT NULL DEFAULT 30000,
    idle_timeout_ms INT NOT NULL DEFAULT 60000,
    circuit_breaker_threshold INT NOT NULL DEFAULT 5,
    circuit_breaker_timeout_sec INT NOT NULL DEFAULT 30,
    circuit_breaker_success_threshold INT NOT NULL DEFAULT 2,
    max_concurrent_requests INT NOT NULL DEFAULT 100,
    max_request_size_bytes INT NOT NULL DEFAULT 10485760,
    max_response_size_bytes INT NOT NULL DEFAULT 104857600,
    rate_limit_per_min INT NOT NULL DEFAULT 1000,
    CHECK (link_id IS NOT NULL OR route_id IS NOT NULL)
);
```

**[F73.005]** – outbound_api_audit_log Table
```sql
CREATE TABLE outbound_api_audit_log (
    id UUID PRIMARY KEY,
    tenant_id UUID NOT NULL,
    user_id UUID NULL,
    route_id UUID NOT NULL,
    link_id UUID NOT NULL,
    operation TEXT NOT NULL,
    target_url TEXT NOT NULL,
    status_code INT NULL,
    duration_ms INT NOT NULL,
    error_message TEXT NULL,
    trace_id TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_tenant_ts ON outbound_api_audit_log(tenant_id, timestamp DESC);
CREATE INDEX idx_audit_route_ts ON outbound_api_audit_log(route_id, timestamp DESC);
CREATE INDEX idx_audit_link_ts ON outbound_api_audit_log(link_id, timestamp DESC);
```

---

## F8 — Provider-Specific Facts (Reference)

### F80 — OpenAI API Facts

**[F80.001]** – Base URL
> `https://api.openai.com/v1`

**[F80.002]** – Authentication
> Bearer token: `Authorization: Bearer <api_key>`

**[F80.003]** – Completion Endpoint
> `POST /chat/completions`

**[F80.004]** – Embedding Endpoint
> `POST /embeddings`

**[F80.005]** – Streaming
> SSE via `stream: true` parameter

---

### F81 — Anthropic API Facts

**[F81.001]** – Base URL
> `https://api.anthropic.com`

**[F81.002]** – Authentication
> API key header: `x-api-key: <api_key>`
> Version header: `anthropic-version: 2023-06-01`

**[F81.003]** – Completion Endpoint
> `POST /v1/messages`

**[F81.004]** – Streaming
> SSE via `stream: true` parameter

**[F81.005]** – No Embedding Support
> Anthropic does not provide embedding API

---

### F82 — Gemini API Facts

**[F82.001]** – Base URL
> `https://generativelanguage.googleapis.com`

**[F82.002]** – Authentication
> Query parameter: `?key=<api_key>`

**[F82.003]** – Completion Endpoint
> `POST /v1beta/models/{model}:generateContent`

**[F82.004]** – Embedding Endpoint
> `POST /v1beta/models/{model}:batchEmbedContents`

**[F82.005]** – Streaming
> SSE via `?alt=sse` query parameter

---

## F9 — SLOs

**[F90.001]** – Availability Target
> 99.9% (excluding planned maintenance and downstream failures)

**[F90.002]** – Latency Targets
> - p50: < 100ms overhead
> - p95: < 200ms overhead
> - p99: < 500ms overhead

**[F90.003]** – Error Rate Target
> < 0.1% (excluding downstream errors)
