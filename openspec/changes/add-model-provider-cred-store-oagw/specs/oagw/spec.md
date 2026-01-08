## ADDED Requirements

### Requirement: oagw module responsibility
The system SHALL provide an `oagw` gateway module with **pluggable co-loaded protocol/auth/streaming implementations** responsible for invoking outbound remote API calls in a reusable way.

The `oagw` module SHALL:
- Manage tenant-scoped outbound route/link configuration.
- Delegate actual invocation to a selected plugin implementation.
- Provide error normalization for known protocols.

#### Scenario: Reusable outbound invocation
- **WHEN** a module needs to call an external HTTP API
- **THEN** it can delegate to `oagw` instead of implementing its own transport/auth logic

### Requirement: oagw GTS schemas and instances registration
The system SHALL register the following GTS schemas in `types_registry` during gateway startup:
- `gts.x.core.oagw.proto.v1~`
- `gts.x.core.oagw.stream_proto.v1~`
- `gts.x.core.oagw.auth_type.v1~`
- `gts.x.core.oagw.strategy.v1~`

Each schema SHALL define:
- `id: GtsInstanceId`
- `display_name: string`
- `description?: string`
- `priority?: int` (lower wins)`

Well-known instances SHALL be loaded from `oagw/.../gts/*.instances.json` and registered during gateway `init()`.

priority is used to select the best protocol for a given route, e.g. HTTP/3 if supported by downstream would have lower priority than HTTP/2.

Well-known route protocol instances SHOULD include:
- `gts.x.core.oagw.proto.v1~x.core.http.http11.v1`
- `gts.x.core.oagw.proto.v1~x.core.http.http2.v1`
- `gts.x.core.oagw.proto.v1~x.core.http.http3.v1`
- `gts.x.core.oagw.proto.v1~x.core.http.sse.v1`

Well-known strategy instances SHOULD include:
- `gts.x.core.oagw.strategy.v1~x.core._.sticky_session.v1`
- `gts.x.core.oagw.strategy.v1~x.core._.round_robin.v1`

Auth type schemas and instances SHOULD be treated analogously to protocol/strategy schemas:
- The `gts.x.core.oagw.auth_type.v1~` schema defines the shape of an auth mechanism.
- Well-known auth type instances define a stable set of supported auth behaviors.
- An `outbound_api_route` references exactly one `auth_type_gts_id`, allowing tenants to select behavior without hardcoding.

#### Scenario: Startup registration
- **WHEN** `oagw` gateway `init()` runs
- **THEN** it registers the listed schemas and shipped instances in `types_registry`

#### Scenario: Sticky session strategy
- **GIVEN** a link is configured with `strategy_gts_id` = `gts.x.core.oagw.strategy.v1~x.core._.sticky_session.v1`
- **WHEN** `oagw` selects a link for a given tenant
- **THEN** it SHOULD try to stick to the previously selected link for that tenant

#### Scenario: Round robin strategy
- **GIVEN** a link is configured with `strategy_gts_id` = `gts.x.core.oagw.strategy.v1~x.core._.round_robin.v1`
- **WHEN** `oagw` selects a link for a given tenant
- **THEN** it SHOULD distribute invocations across links for the same route

### Requirement: oagw persistence model
The system SHALL persist anonymous objects (UUID IDs) in the `oagw` database.

#### Database tables (logical), managed by OAGW gateway module
- `outbound_api_route`
  - `id uuid pk`
  - `tenant_id uuid not null`
  - `base_url text not null`
  - `rate_limit_req_per_min int not null default 1000`
  - `auth_type_gts_id text not null` (schema: `gts.x.core.oagw.auth_type.v1~`)

- `outbound_api_route_supported_protocol`
  - `id uuid pk`
  - `route_id uuid not null fk outbound_api_route(id)`
  - `protocol_gts_id text not null` (list of supported protocols, schema: `gts.x.core.oagw.proto.v1~`)

- `outbound_api_link`
  - `id uuid pk`
  - `tenant_id uuid not null`
  - `secret_ref uuid not null` (logical/not-physical FK to `cred_store.secret_store.id`)
  - `route_id uuid not null fk outbound_api_route(id)`
  - `secret_type_gts_id text not null` (schema: `gts.x.core.cred_store.secret_type.v1~`)
  - `enabled bool not null`
  - `priority int not null`
  - `strategy_gts_id text not null` (schema: `gts.x.core.oagw.strategy.v1~`)

- `outbound_api_route_limits`
  - `id uuid pk`
  - `tenant_id uuid not null`
  - `link_id uuid null unique (logical FK to outbound_api_link.id)` (can be per link)
  - `route_id uuid null unique (logical FK to outbound_api_route.id)` (... or per route)
  - `connection_timeout_ms int not null default 5000` (5s to establish connection)
  - `request_timeout_ms int not null default 30000` (30s total request duration)
  - `idle_timeout_ms int not null default 60000` (60s idle connection reuse)
  - `circuit_breaker_threshold int not null default 5`
  - `circuit_breaker_timeout_sec int not null default 30`
  - `circuit_breaker_success_threshold int not null default 2`
  - `max_concurrent_requests int not null default 100`
  - `max_request_size_bytes int not null default 10485760` (10 MB)
  - `max_response_size_bytes int not null default 104857600` (100 MB)
  - `rate_limit_per_min int not null default 1000`

#### Scenario: Configure link for a tenant
- **WHEN** a tenant configures an outbound link
- **THEN** the link references a `cred_store` secret by UUID

### Requirement: oagw gateway API (Rust-native)
The system SHALL expose a Rust-native API (ClientHub) named `OagwApi`.

The API SHALL:
- Accept `&SecurityCtx` for every method.
- Allow selecting an outbound link and invoking requests through it.
- Support non-streaming and streaming responses.

The Rust-native API SHOULD expose two shapes:
- Unary (synchronous) invocation: request/response completes with a single response value.
- Streaming (asynchronous) invocation: response is a stream of items.

The Rust-native API SHOULD support streaming over:
- HTTP SSE (`gts.x.core.oagw.proto.v1~x.core.http.sse.v1`) as a server-to-client stream.

#### Scenario: Rust-native unary invocation
- **WHEN** a consumer calls `OagwApi::invoke_unary(ctx, request)`
- **THEN** the gateway resolves an eligible plugin and executes a single request/response exchange

#### Scenario: Rust-native streaming invocation
- **WHEN** a consumer calls `OagwApi::invoke_stream(ctx, request)`
- **THEN** the gateway resolves an eligible plugin and returns a stream of response items

#### Scenario: Rust-native invocation
- **WHEN** a consumer calls `OagwApi::invoke(ctx, request)`
- **THEN** the gateway resolves an eligible plugin and executes the request

### Requirement: oagw unary vs streaming Rust API shape
The system SHALL define a contract model for outbound invocations that supports both unary and streaming responses.

The Rust-native API SHOULD define method shapes similar to:
- `invoke_unary(&self, ctx: &SecurityCtx, req: OagwInvokeRequest) -> Result<OagwInvokeResponse, OagwError>`
- `invoke_stream(&self, ctx: &SecurityCtx, req: OagwInvokeRequest) -> Result<OagwResponseStream, OagwError>`

The streaming response type SHOULD be compatible with `futures::Stream`, enabling transport-independent composition.

#### Scenario: Streaming response as futures::Stream
- **WHEN** a consumer uses `invoke_stream`
- **THEN** it can process response items incrementally using backpressure-aware `Stream` combinators

### Requirement: oagw REST API
The system SHALL expose REST endpoints:
- `GET/POST /oagw/v1/routes`
- `GET/PATCH/DELETE /oagw/v1/routes/{routeId}`
- `GET/POST /oagw/v1/links`
- `GET/PATCH/DELETE /oagw/v1/links/{linkId}`
- `POST /oagw/v1/invoke`

#### Scenario: REST invoke
- **WHEN** a client POSTs to `/oagw/v1/invoke` with a link id and request payload
- **THEN** `oagw` executes the request via the selected plugin
- **AND** returns normalized errors using RFC-9457 Problem Details

### Requirement: oagw plugin interface and selection
The system SHALL define an `oagw` plugin interface.

Each plugin SHALL report:
- Supported route protocols (instances of `gts.x.core.oagw.proto.v1~`).
- Supported stream protocols (instances of `gts.x.core.oagw.stream_proto.v1~`).
- Supported auth types (instances of `gts.x.core.oagw.auth_type.v1~`).
- Supported strategies (instances of `gts.x.core.oagw.strategy.v1~`).
- Plugin priority (lower wins).

The gateway SHALL choose the eligible plugin with the lowest priority.

The gateway SHOULD implement sticky session selection keyed by:
- `(tenant_id, user_id)` when available.

#### Scenario: Eligible plugin selection
- **GIVEN** an outbound link requiring protocol P and auth type A
- **WHEN** `oagw` resolves its plugin
- **THEN** it filters to plugins supporting (P, A)
- **AND** selects the lowest-priority plugin

### Requirement: oagw protocol and transport implementation libraries
The system SHALL define the Rust libraries used for outbound transport per selected protocol.

For outbound HTTP (`http11`, `http2`) the gateway SHOULD use:
- `reqwest` for request construction and execution.
- `modkit::http::TracedClient` to inject OpenTelemetry trace context into outbound calls.

For outbound HTTP/3 (`http3`) the gateway SHOULD use an HTTP/3-capable Rust client library.

For outbound SSE (`sse`) the gateway SHOULD:
- Use `reqwest` for initiating the HTTP request.
- Use an SSE-compatible client implementation to parse `text/event-stream` frames into stream items.

### Requirement: oagw auth mechanisms, token exchange, and caching
The system SHALL define how outbound authentication is produced for each outbound call.

The gateway SHALL treat outbound auth configuration as data:
- A route references an auth type instance via `auth_type_gts_id`.
- A link references secret material via `secret_ref` and `secret_type_gts_id`.

Outbound auth material SHOULD be resolved using `cred_store` and SHOULD NOT be embedded directly into `oagw` tables.

#### Token exchange model
When the inbound request is authenticated with an end-user token, the gateway MAY need to exchange it for an outbound token.

If token exchange is required, the gateway SHOULD implement RFC 8693 OAuth 2.0 Token Exchange against a configured identity provider.

The gateway SHOULD cache exchanged tokens.

Caching behavior SHOULD:
- Use an in-memory cache keyed by `(tenant_id, user_id, route_id, auth_type_gts_id, requested_scopes)`.
- Store tokens with TTL derived from the token `exp` claim minus a safety skew.
- Deduplicate concurrent exchanges for the same cache key.

#### Scenario: Exchanged token cache hit
- **GIVEN** an exchanged outbound token exists in cache for `(tenant_id, user_id, route_id, auth_type, scopes)`
- **WHEN** the tenant invokes the same route again
- **THEN** `oagw` reuses the cached token
- **AND** does not call the identity provider

#### Scenario: Exchanged token cache miss
- **GIVEN** no cached outbound token exists
- **WHEN** the tenant invokes the route
- **THEN** `oagw` exchanges the inbound identity token for an outbound token
- **AND** caches the result until its effective expiry

#### Auth library mapping (non-exhaustive)
The system SHOULD implement outbound auth using common, audited Rust libraries:
- OAuth2 client credentials / token exchange: `oauth2` (for token acquisition flows)
- JWT validation/parsing (when needed for caching/expiry decisions): `jsonwebtoken`
- Static headers / API keys: `http` header types and `reqwest` request builders

#### Scenario: Outbound auth derived from link secret
- **GIVEN** a link references `secret_ref` and `secret_type_gts_id`
- **WHEN** the gateway prepares the outbound request
- **THEN** it loads secret material from `cred_store` using the reference
- **AND** applies it to the request according to `auth_type_gts_id`

### Requirement: Caller-controlled retry semantics
The system SHALL treat retries as an explicit caller-provided policy.

The gateway MUST NOT perform implicit retries.

The gateway MAY perform retries **within a single invocation** only when explicitly requested by the caller via a declarative `RetryIntent`.

The retry state MUST be ephemeral and live only within the invocation context.

#### Scenario: No implicit retry
- **WHEN** the caller provides no `RetryIntent` (or `max_attempts = 1`)
- **THEN** the gateway executes exactly one outbound attempt

#### Scenario: Retry intent enables bounded retries
- **GIVEN** a caller provides `RetryIntent.max_attempts > 1`
- **WHEN** the first attempt fails with a condition matching `RetryIntent.retry_on`
- **THEN** the gateway MAY retry within the same invocation until attempts are exhausted
- **AND** the response MUST include the attempt number that produced the returned result

### Requirement: oagw audit logging
The system SHALL maintain an audit log for all outbound invocations.

#### Database table: outbound_api_audit_log
- `id uuid pk`
- `tenant_id uuid not null`
- `user_id uuid null`
- `route_id uuid not null`
- `link_id uuid not null`
- `operation text not null` (HTTP method)
- `target_url text not null`
- `status_code int null` (HTTP status)
- `duration_ms int not null`
- `error_message text null`
- `trace_id text not null`
- `timestamp timestamptz not null`

**Indexes:**
- `(tenant_id, timestamp desc)` for tenant audit queries
- `(route_id, timestamp desc)` for route-specific audit trail
- `(link_id, timestamp desc)` for link performance analysis

NOTE: table parititioning and archiving and retention policies are not specified in this document. It's a subject for future improvements.

The system SHALL define audit logging guarantees.

Guarantee options are:
- best-effort
- guaranteed
- fail-closed

If audit logging is synchronous, the system SHALL enforce an upper bound on audit log latency.

#### Scenario: Audit latency budget
- **WHEN** the audit subsystem exceeds the maximum configured latency budget
- **THEN** the gateway applies the configured guarantee mode (best-effort drop, guaranteed wait, or fail-closed)

#### Scenario: Audit successful invocation
- **WHEN** oagw invokes external API successfully
- **THEN** logs (tenant_id, route_id, link_id, status_code, duration_ms, timestamp)
- **AND** includes OpenTelemetry trace_id from current span

#### Scenario: Audit failed invocation
- **WHEN** oagw invocation fails (timeout, 500 error)
- **THEN** logs error details for troubleshooting
- **AND** records error_message and status_code

### Requirement: oagw observability
The system SHALL expose Prometheus-compatible metrics.

Required metrics (using `metrics` crate):
- `oagw.invocations_total` (counter, labels: tenant_id, route_id, status)
- `oagw.request.duration_msec` (histogram, labels: route_id, protocol)
- `oagw.errors_total` (counter, labels: error_type, route_id)
- `oagw.active_connections` (gauge, labels: route_id)
- `oagw.circuit_breaker.opened` (counter, labels: link_id)
- `oagw.circuit_breaker.state` (gauge, labels: link_id)
- `oagw.bytes_sent_total` (counter, labels: tenant_id, route_id, link_id)
- `oagw.bytes_received_total` (counter, labels: tenant_id, route_id, link_id)
- `oagw.token_cache.hit_total` (counter)
- `oagw.token_cache.miss_total` (counter)
- `oagw.token_cache.size` (gauge)

Implementation SHALL use `metrics` and `metrics-exporter-prometheus` crates.

#### Scenario: Metrics collection
- **WHEN** outbound invocation completes
- **THEN** system increments appropriate counters
- **AND** records latency histogram
- **AND** metrics are exposed at GET /metrics endpoint

### Requirement: oagw health checks
The system SHALL expose health check endpoints.

REST endpoints:
- `GET /oagw/v1/health` - Liveness probe
- `GET /oagw/v1/ready` - Readiness probe (checks core dependencies like DB)
- `GET /oagw/v1/routes/{routeId}/health` - Route-specific health check

The `/oagw/v1/health` and `/oagw/v1/ready` endpoints access is not protected by any role.

#### Scenario: Health check
- **WHEN** `GET /oagw/v1/health` is called
- **AND** no role is required
- **THEN** returns 200 OK if system is health

#### Scenario: Readiness check
- **WHEN** `GET /oagw/v1/ready` is called
- **AND** no role is required
- **THEN** returns 200 OK if system can serve traffic
- **AND** includes results of core dependency checks

#### Scenario: Check downstream health
- **WHEN** `GET /oagw/v1/routes/{routeId}/health` is called
- **THEN** access is protected by role `gts.x.core.idp.role.v1~x.oagw.route.health.v1` assigned
- **AND** system tests connectivity to all links for that route
- **AND** returns aggregated health status

### Requirement: oagw tracing integration
The system SHALL integrate with OpenTelemetry for distributed tracing.

All operations SHALL create spans with attributes:
- `tenant_id`
- `user_id` (if available)
- `route_id`
- `link_id`
- `protocol`
- `auth_type`
- `target_url`
- `http.method` (for HTTP)
- `http.status_code` (for HTTP)

Implementation SHALL use `tracing` and `tracing-opentelemetry` crates.

#### Scenario: Trace outbound invocation
- **WHEN** outbound invocation is performed
- **THEN** system creates OpenTelemetry span with all relevant attributes
- **AND** span is linked to parent trace context
- **AND** errors are recorded as span events

### Requirement: oagw circuit breaker
The system SHALL implement circuit breaker state globally or per tenant-scoped link to prevent cascade failures.

Circuit breaker state MUST be hierarchically scoped per `(provider_id, route_id, tenant_id, link_id)`.

The system MAY maintain an aggregate breaker signal per `(tenant_id, route_id)` only for reporting and health aggregation.

#### Extended table: outbound_api_link
- Add: `circuit_breaker_threshold int not null default 5`
- Add: `circuit_breaker_timeout_sec int not null default 30`
- Add: `circuit_breaker_success_threshold int not null default 2`

Circuit breaker configuration:
- **Failure threshold:** 5 consecutive failures (configurable per link)
- **Timeout:** 30 seconds (half-open state)
- **Success threshold:** 2 successes to close
- **Configurable:** module config file parameters can override these defaults

Implementation SHOULD use `failsafe` or `tower` crate.

#### Scenario: Circuit breaker opens
- **GIVEN** link L fails 5 consecutive times
- **WHEN** circuit breaker opens
- **THEN** subsequent requests to L fail-fast (503)
- **AND** system tries next eligible link (if available)
- **AND** after 30s, circuit enters half-open state

#### Scenario: Fallback to alternate link
- **GIVEN** primary link has open circuit breaker
- **WHEN** oagw selects link for invocation
- **THEN** gateway skips primary and selects next highest-priority link

#### Scenario: Circuit breaker recovery
- **GIVEN** circuit breaker is half-open
- **WHEN** 2 consecutive calls succeed
- **THEN** circuit breaker closes
- **AND** normal operation resumes

### Requirement: oagw resource limits
The system SHALL enforce resource limits per route.

The system SHALL define explicit backpressure and buffering limits for unary and streaming requests.

#### Scenario: Enforce per-invocation inflight bytes
- **WHEN** the response body stream exceeds the configured per-invocation inflight byte budget
- **THEN** the gateway aborts the stream and returns a stream abort error

#### Scenario: Enforce concurrent request limit
- **GIVEN** route R has 100 active requests
- **WHEN** 101st request arrives
- **THEN** gateway returns 429 Too Many Requests
- **AND** includes Retry-After header

#### Scenario: Enforce request size limit
- **WHEN** client attempts to send 20 MB request payload
- **AND** route limit is 10 MB
- **THEN** gateway returns 413 Payload Too Large before sending to downstream

NOTE: limits above are per route, configurable globally in module config or per tenant in database (`outbound_api_route_limits`).

#### Scenario: Enforce response size limit
- **WHEN** downstream returns response exceeding max_response_size_bytes
- **THEN** gateway aborts response stream
- **AND** returns 502 Bad Gateway to client

### Requirement: oagw timeout configuration
The system SHALL support granular timeout settings per link.

#### Scenario: Connection timeout
- **WHEN** TCP connection to link takes > 5s (configured in `outbound_api_link.connection_timeout_ms`)
- **THEN** gateway aborts connection attempt
- **AND** tries next link or returns 504 Gateway Timeout

#### Scenario: Request timeout
- **WHEN** request takes > 30s (including all retries, configured in `outbound_api_link`)
- **THEN** gateway cancels request
- **AND** returns 504 Gateway Timeout to client

#### Scenario: Idle connection reuse
- **WHEN** HTTP connection is idle for > 60s (configured in `outbound_api_link`)
- **THEN** gateway closes connection
- **AND** establishes new connection for next request

### Requirement: oagw streaming error handling
The system SHALL handle mid-stream errors gracefully.

#### Scenario: SSE stream interruption
- **WHEN** SSE stream from downstream breaks mid-stream
- **THEN** gateway closes client stream with error event
- **AND** logs partial stream duration and bytes received

The streaming contract SHALL be treated as a stream of results.

Stream items MUST be shaped as `Result<Chunk, StreamAbort>`.

`StreamAbort` MUST include:
- abort_reason (network | protocol | auth | timeout)
- resumable: bool
- resume_hint (e.g., Last-Event-ID)

#### Scenario: Stream timeout
- **WHEN** streaming response stalls (no data for > configured in `outbound_api_link.idle_timeout_ms`)
- **THEN** gateway terminates stream
- **AND** returns timeout error to client

### Requirement: oagw token cache implementation
The system SHALL implement token caching using in-memory LRU cache.

Implementation SHOULD use `moka` crate for high-performance caching.

Cache configuration:
- **Max capacity:** 10,000 entries
- **TTL:** Derived from token `exp` claim minus 60s safety margin
- **Max TTL:** 3600s (1 hour)

Cache key SHALL include:
- `(tenant_id, user_id, route_id, auth_type_gts_id, scopes)`

#### Scenario: Token TTL respects JWT exp claim
- **WHEN** token exchange returns JWT with exp = 1 hour
- **THEN** cache TTL = exp - 60s (safety margin)
- **AND** token is evicted before actual expiry

#### Scenario: Token cache eviction
- **WHEN** cache reaches max capacity
- **THEN** least recently used tokens are evicted
- **AND** next access triggers fresh token exchange

### Requirement: oagw token cache invalidation
The system SHALL support manual token cache invalidation.

REST endpoint:
- `DELETE /oagw/v1/routes/{routeId}/cache/tokens` - Clear cached tokens for route

#### Scenario: Clear cached tokens after credential rotation
- **WHEN** admin rotates credentials for route R
- **THEN** admin calls `DELETE /oagw/v1/routes/{R}/cache/tokens`
- **AND** all cached tokens for R are evicted
- **AND** next invocation triggers fresh token exchange

#### Scenario: Clear all tokens for tenant
- **WHEN** tenant credentials are compromised
- **THEN** admin can clear all tenant tokens via cache flush
- **AND** all active tokens are invalidated immediately

### Requirement: oagw protocol version selection
The system SHALL automatically select optimal protocol version.

#### Scenario: Automatic HTTP/2 vs HTTP/3 selection
- **GIVEN** route supports both `http2` and `http3` protocols
- **WHEN** oagw initiates connection based on protocol GTS id with lower priority
- **THEN** attempts HTTP/3 (QUIC) first if supported by downstream
- **AND** falls back to HTTP/2 if HTTP/3 negotiation fails

#### Scenario: Force protocol version
- **WHEN** link explicitly specifies `protocol_gts_id = ... http11`
- **THEN** gateway MUST use HTTP/1.1 only (no automatic upgrade)

#### Scenario: Protocol negotiation failure
- **WHEN** client requires protocol P but downstream doesn't support it
- **THEN** gateway returns 502 Bad Gateway
- **AND** error detail specifies protocol mismatch

### Requirement: oagw response caching (optional)
The system MAY support response caching for idempotent GET requests.

#### Extended table: outbound_api_route
- Add: `cache_ttl_sec int null` (NULL = no caching)

Cache key SHALL include:
- `(route_id, request_path, query_params, relevant_headers)`

Implementation SHOULD use `moka` crate.

#### Scenario: Cache GET response
- **GIVEN** route has `cache_ttl_sec = 300` (5 min)
- **WHEN** client performs GET request
- **THEN** gateway caches response for 5 minutes
- **AND** subsequent identical GET requests return cached response

#### Scenario: Cache invalidation on mutation
- **WHEN** client performs POST/PUT/DELETE on cached route
- **THEN** gateway invalidates cached GET responses for that route

#### Scenario: Respect Cache-Control headers
- **WHEN** downstream response includes `Cache-Control: no-store`
- **THEN** gateway bypasses cache regardless of route configuration

### Requirement: oagw error taxonomy
The system SHALL define comprehensive error types using RFC 9457 Problem Details defined in the modkit unified system.

Error types SHALL include errors with appropriate GTS identifier:
- `LinkUnavailable`: All links for route are down
- `CircuitBreakerOpen`: Circuit breaker prevents invocation
- `ConnectionTimeout`: Cannot establish connection
- `RequestTimeout`: Request exceeded timeout
- `RateLimitExceeded`: Route rate limit hit
- `PayloadTooLarge`: Request or response size exceeded
- `ProtocolError`: Protocol negotiation or parsing failed
- `AuthenticationFailed`: Cannot obtain or refresh token
- `DownstreamError`: Downstream returned error status

#### Scenario: Link unavailable error
- **WHEN** all eligible links are unavailable (circuit breakers open)
- **THEN** gateway returns 503 Service Unavailable
- **AND** includes Problem Detail with type `urn:oagw:error:link-unavailable`
- **AND** lists affected link IDs

#### Scenario: Rate limit error
- **WHEN** route rate limit is exceeded
- **THEN** gateway returns 429 Too Many Requests
- **AND** includes `Retry-After` header with seconds until reset

### Requirement: oagw rate limiting
The system SHALL enforce rate limits per route using token bucket algorithm.

Implementation SHOULD use `governor` crate.

Rate limits SHALL be configurable per route (default: 1000 req/min).
There must be global limit and per-tenant limit provided in `outbound_api_route`.

The system SHALL support distributed rate limiting for multi-instance deployments.

#### Scenario: Distributed rate limiting with Redis
- **GIVEN** the deployment runs multiple gateway instances
- **WHEN** rate limiting is enforced for `(tenant_id, route_id)`
- **THEN** the system uses a Redis-backed shared limiter to ensure consistent quotas across instances

### Requirement: outbound TLS and credential lifecycle
The system SHALL define outbound TLS and credential refresh behavior.

#### Scenario: TLS certificate validation
- **WHEN** the gateway makes an outbound HTTPS request
- **THEN** it validates the remote certificate chain by default

#### Scenario: Credential rotation for long-lived streams
- **WHEN** credentials referenced from `cred_store` rotate
- **THEN** new invocations use the latest secret material
- **AND** long-lived SSE streams require an explicit caller-initiated restart to pick up the new credentials

#### Scenario: Rate limit enforcement
- **GIVEN** route R has rate limit 1000 req/min
- **WHEN** client exceeds limit
- **THEN** gateway returns 429 Too Many Requests
- **AND** includes Retry-After header

#### Scenario: Per-tenant rate limiting
- **GIVEN** route R has per-tenant limit configured
- **WHEN** tenant T exceeds its quota
- **THEN** only tenant T is rate limited
- **AND** other tenants continue normally

### Requirement: oagw OData support
The system SHALL implement OData query capabilities per ModKit conventions.

REST endpoints SHALL support:
- `$skip` and `$top` for pagination
- `$filter` for filtering
- `$orderby` for sorting
- `$select` for field projection

Implementation SHALL use `modkit::api::OperationBuilder` OData support.

#### Scenario: Paginated route list
- **WHEN** GET /oagw/v1/routes?$skip=20&$top=10
- **THEN** returns routes 21-30
- **AND** includes `@odata.nextLink` for pagination

#### Scenario: Filter links by route
- **WHEN** GET /oagw/v1/links?$filter=route_id eq '550e8400-e29b-41d4-a716-446655440000'
- **THEN** returns only links for that route

### Requirement: oagw SLOs (Service Level Objectives)
The system SHOULD target:
- **Availability:** 99.9% (excluding planned maintenance and downstream failures)
- **Latency:**
  - p50: < 100ms overhead (excluding downstream latency)
  - p95: < 200ms overhead
  - p99: < 500ms overhead
- **Error rate:** < 0.1% (excluding downstream errors)

#### Scenario: SLO monitoring
- **WHEN** operations team reviews SLO dashboard
- **THEN** metrics show p95 latency overhead, error rate, and uptime
- **AND** alerts trigger if SLO thresholds are breached
