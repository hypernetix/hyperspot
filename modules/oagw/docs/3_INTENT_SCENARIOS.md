> This is module scenarios - generated from 1_INTENT.md and guidelines/NEW_MODULE.md, docs/MODKIT_PLUGINS.md, docs/MODKIT_UNIFIED_SYSTEM.md, examples/

# OAGW Module Scenarios

This document provides detailed step-by-step scenarios for the OAGW (Outbound API Gateway) module. Each scenario is versioned v1-v5, where v1 is MVP and v5 is full-featured.

**Version Definitions:**

| Version | Focus | Key Features |
|---------|-------|--------------|
| **v1** | MVP | Basic invocation, route/link CRUD, plugin selection, bearer/API key auth, GTS error taxonomy, no-retry default |
| **v2** | Production | Streaming (no retry), audit logging, metrics, rate limiting, OData pagination, `Retry-After` propagation |
| **v3** | Reliability | Circuit breaker, token caching, OAuth2, health checks, `RetryIntent` support, `RetryBudget` |
| **v4** | Performance | Sticky sessions, round robin, response caching |
| **v5** | Enterprise | Token exchange, advanced caching, per-tenant rate limits |

---

## Design Principles

1. **OAGW is a synchronous gateway** — no background job processing, no in-memory queues
2. **Retry is caller-controlled** — OAGW performs no implicit retries; caller provides a declarative `RetryIntent` per request [F55.001]
3. **Streaming never retries** — caller must restart stream if protocol supports resume [F55.006]
4. **`Retry-After` always propagates** — from downstream, circuit breaker, or rate limiter [F55.005]
5. **All errors have GTS IDs** — enables programmatic error handling [F54.002]
6. **No constants except GTS IDs** — all identifiers are GTS instance IDs


**Notation:** Steps reference `[FXXX]` fact IDs from [FACTS.md](./FACTS.md).

---

## Scenario 1: REST API Invocation

External client invokes downstream API via OAGW REST endpoint.

### 1.1 Request Reception

- [ ] v1 - HTTP request arrives at `POST /oagw/v1/invoke` [F20.011]
- [ ] v1 - `api_ingress` routes request to OAGW handler
- [ ] v1 - Extract `Authorization` header, validate JWT via `modkit-auth`
- [ ] v1 - Construct `SecurityCtx` with `tenant_id`, `user_id` [F41.001]
- [ ] v2 - Validate request against OpenAPI schema (utoipa)
- [ ] v2 - Check caller has `gts.x.core.idp.role.v1~x.oagw.invoke.v1` role [F42.004]
- [ ] v3 - Extract `X-Request-Id` header for correlation
- [ ] v1 - Apply request size limit check before parsing body [F27.009]

### 1.2 Request Parsing & Validation

- [ ] v1 - Deserialize JSON body into `InvokeRequestDto`
- [ ] v1 - Validate required fields: `route_id`, `method`, `path`
- [ ] v1 - **IF** `link_id` provided:
  - [ ] v1 - Use specified link directly
- [ ] v1 - **ELSE**:
  - [ ] v1 - OAGW will select link in step 1.4
- [ ] v2 - Validate `method` is one of: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS
- [ ] v3 - Validate `headers` map doesn't contain forbidden headers (Host, Authorization)
- [ ] v4 - Validate `body` size against `max_request_size_bytes` from route limits [F20.004]

### 1.3 Route Resolution

- [ ] v1 - Query `outbound_api_route` by `route_id` and `tenant_id` [F73.001]
- [ ] v1 - **IF** route not found:
  - [ ] v1 - Return `404 Not Found` with Problem Detail [F27.007]
- [ ] v1 - Load route configuration: `base_url`, `auth_type_gts_id`
- [ ] v2 - Load supported protocols from `outbound_api_route_supported_protocol` [F73.002]
- [ ] v3 - Load route limits from `outbound_api_route_limits` (if exists) [F73.004]
- [ ] v3 - **IF** no route-specific limits:
  - [ ] v3 - Use module config defaults [F07.001]

### 1.4 Link Selection

- [ ] v1 - **IF** `link_id` was provided in request:
  - [ ] v1 - Query `outbound_api_link` by `link_id`, `tenant_id` [F73.003]
  - [ ] v1 - **IF** link not found OR `enabled = false`:
    - [ ] v1 - Return `404 Not Found` [F27.007]
  - [ ] v1 - Verify `link.route_id == route_id`
- [ ] v1 - **ELSE** (auto-select link):
  - [ ] v1 - Query all enabled links for `(route_id, tenant_id)` [F73.003]
  - [ ] v1 - **IF** no enabled links:
    - [ ] v1 - Return `503 Service Unavailable` [F27.012]
    - [ ] v1 - Error: `gts.x.core.errors.err.v1~x.oagw.link.unavailable.v1` [F54.002]
  - [ ] v2 - Sort links by `priority` ASC (lower = higher priority)
  - [ ] v3 - Filter out links with open circuit breakers [F54.004]
  - [ ] v3 - **IF** all links filtered out:
    - [ ] v3 - Return `503 Service Unavailable`
    - [ ] v3 - Error: `gts.x.core.errors.err.v1~x.oagw.circuit_breaker.open.v1`
    - [ ] v3 - Set `retry_after_sec` from circuit breaker half-open timeout [F55.005]
  - [ ] v4 - **SWITCH** on `link.strategy_gts_id`:
    - [ ] v4 - **CASE** `sticky_session` [F12.001]:
      - [ ] v4 - Lookup sticky mapping for `(tenant_id, user_id)`
      - [ ] v4 - **IF** mapping exists AND link is available:
        - [ ] v4 - Use mapped link
      - [ ] v4 - **ELSE**:
        - [ ] v4 - Select first available link, store mapping
    - [ ] v4 - **CASE** `round_robin` [F12.002]:
      - [ ] v4 - Get atomic counter for route
      - [ ] v4 - Select `links[counter % links.len()]`
      - [ ] v4 - Increment counter
    - [ ] v4 - **DEFAULT**:
      - [ ] v4 - Select first available link (by priority)

### 1.5 Rate Limit Check

- [ ] v2 - Get rate limiter for `(tenant_id, route_id)` [F56.001]
- [ ] v4 - In multi-instance deployments, use Redis-backed distributed rate limiter for `(tenant_id, route_id)` [F56.002]
- [ ] v2 - **IF** rate limit exceeded:
  - [ ] v2 - Calculate `retry_after_sec` from quota reset time [F55.005]
  - [ ] v2 - Return `429 Too Many Requests` [F27.010]
  - [ ] v2 - Error: `gts.x.core.errors.err.v1~x.oagw.rate_limit.exceeded.v1` [F54.002]
  - [ ] v2 - Set `Retry-After` header (MUST propagate) [F55.005]
  - [ ] v2 - Increment `oagw.errors_total{error_type="rate_limit"}` [F53.003]
- [ ] v3 - Check concurrent request limit for route [F20.004]
- [ ] v3 - **IF** `active_requests >= max_concurrent_requests`:
  - [ ] v3 - Return `429 Too Many Requests`
- [ ] v3 - Increment active request counter
- [ ] v5 - Check per-tenant rate limit (if configured separately)

### 1.6 Plugin Selection

- [ ] v1 - Get route's required protocol from `outbound_api_route_supported_protocol`
- [ ] v1 - Get route's required auth type from `route.auth_type_gts_id`
- [ ] v1 - Query `types_registry` for plugins matching schema `gts.x.core.modkit.plugin.v1~x.core.oagw.plugin.v1~*`
- [ ] v1 - **FOR EACH** plugin instance:
  - [ ] v1 - Parse `properties` as `OagwPluginSpecV1`
  - [ ] v1 - Check `supported_protocols` contains required protocol
  - [ ] v1 - Check `supported_auth_types` contains required auth type
  - [ ] v1 - **IF** both match:
    - [ ] v1 - Add to eligible plugins list
- [ ] v1 - **IF** no eligible plugins:
  - [ ] v1 - Return `503 Service Unavailable`, log error
- [ ] v1 - Sort eligible plugins by `priority` ASC [F60.002]
- [ ] v1 - Select first plugin (lowest priority)
- [ ] v1 - Get plugin client from `ClientHub` via scoped lookup [F60.001]

### 1.7 Secret Resolution

- [ ] v1 - Get `link.secret_ref` UUID
- [ ] v1 - Get `link.secret_type_gts_id`
- [ ] v1 - Call `CredStoreApi::get_secret(ctx, secret_ref)` [F05.002]
- [ ] v1 - **IF** secret not found:
  - [ ] v1 - Return `500 Internal Server Error`
  - [ ] v1 - Error: `gts.x.core.errors.err.v1~x.oagw.secret.not_found.v1` [F54.002]
  - [ ] v1 - Log error (do NOT expose secret details)
- [ ] v2 - Validate secret type matches `secret_type_gts_id`

### 1.8 Authentication Preparation

- [ ] v1 - **SWITCH** on `route.auth_type_gts_id`:
  - [ ] v1 - **CASE** `bearer_token` [F13.001]:
    - [ ] v1 - Set header: `Authorization: Bearer {secret.value}`
  - [ ] v1 - **CASE** `api_key_header` [F13.002]:
    - [ ] v1 - Get header name from secret metadata
    - [ ] v1 - Set header: `{header_name}: {secret.value}`
  - [ ] v2 - **CASE** `api_key_query` [F13.003]:
    - [ ] v2 - Get query param name from secret metadata
    - [ ] v2 - Append to URL: `?{param_name}={secret.value}`
  - [ ] v3 - **CASE** `oauth2_client_creds` [F13.004]:
    - [ ] v3 - Build cache key: `(tenant_id, route_id, auth_type, scopes)` [F52.001]
    - [ ] v3 - Check token cache
    - [ ] v3 - **IF** cache hit:
      - [ ] v3 - Use cached token
      - [ ] v3 - Increment `oagw.token_cache.hit_total` [F53.007]
    - [ ] v3 - **ELSE**:
      - [ ] v3 - Increment `oagw.token_cache.miss_total`
      - [ ] v3 - Call OAuth2 token endpoint with client credentials
      - [ ] v3 - Parse JWT, extract `exp` claim
      - [ ] v3 - Calculate TTL: `min(exp - safety_margin, max_ttl)` [F52.001]
      - [ ] v3 - Store in cache with TTL
    - [ ] v3 - Set header: `Authorization: Bearer {token}`
  - [ ] v4 - **CASE** `oauth2_token_exchange` [F13.005]:
    - [ ] v4 - Build cache key: `(tenant_id, user_id, route_id, auth_type, scopes)`
    - [ ] v4 - Check token cache
    - [ ] v4 - **IF** cache miss:
      - [ ] v4 - Get inbound user token from `SecurityCtx`
      - [ ] v4 - Call RFC 8693 token exchange endpoint
      - [ ] v4 - Cache exchanged token
    - [ ] v4 - Set header: `Authorization: Bearer {exchanged_token}`

### 1.9 Request Construction

- [ ] v1 - Build target URL: `{route.base_url}{request.path}`
- [ ] v1 - **IF** `request.query` provided:
  - [ ] v1 - Append query string to URL
- [ ] v1 - Create HTTP request with:
  - [ ] v1 - Method from `request.method`
  - [ ] v1 - URL from above
  - [ ] v1 - Body from `request.body` (if provided)
- [ ] v1 - Add authentication headers (from step 1.8)
- [ ] v2 - Add custom headers from `request.headers`
- [ ] v3 - Inject OpenTelemetry trace context via `TracedClient` [F02.001]
- [ ] v3 - Set `User-Agent` header with OAGW identifier

### 1.10 Timeout Configuration

- [ ] v1 - Get `connection_timeout_ms` from route limits (or default 5000ms)
- [ ] v1 - Get `request_timeout_ms` from route limits (or default 30000ms)
- [ ] v1 - **IF** `request.timeout_ms` provided AND < `request_timeout_ms`:
  - [ ] v1 - Use `request.timeout_ms`
- [ ] v2 - Configure idle timeout for connection reuse

### 1.11 Plugin Invocation

- [ ] v1 - Start timer for duration tracking
- [ ] v2 - Create tracing span with attributes [F08.001]:
  - [ ] v2 - `tenant_id`, `route_id`, `link_id`
  - [ ] v2 - `target_url`, `http.method`
- [ ] v1 - Call `plugin.invoke_unary(ctx, link, route, secret, request)` [F60.001]
- [ ] v1 - **TRY**:
  - [ ] v1 - Await response
- [ ] v1 - **CATCH** connection timeout:
  - [ ] v1 - Increment circuit breaker failure count
  - [ ] v1 - Return `504 Gateway Timeout` [F27.013]
  - [ ] v1 - Error: `gts.x.core.errors.err.v1~x.oagw.timeout.connection.v1` (retriable=true) [F54.002]
- [ ] v1 - **CATCH** request timeout:
  - [ ] v1 - Increment circuit breaker failure count
  - [ ] v1 - Return `504 Gateway Timeout`
  - [ ] v1 - Error: `gts.x.core.errors.err.v1~x.oagw.timeout.request.v1` (retriable=true) [F54.002]
- [ ] v2 - **CATCH** protocol error:
  - [ ] v2 - Return `502 Bad Gateway` [F27.011]
  - [ ] v2 - Error: `gts.x.core.errors.err.v1~x.oagw.protocol.error.v1` (retriable=false) [F54.002]
- [ ] v1 - Stop timer, record duration

### 1.12 Circuit Breaker Update

- [ ] v3 - **IF** invocation succeeded (2xx status):
  - [ ] v3 - Reset consecutive failure count for link
  - [ ] v3 - **IF** circuit was half-open:
    - [ ] v3 - Increment success count
    - [ ] v3 - **IF** success_count >= `circuit_breaker_success_threshold`:
      - [ ] v3 - Close circuit breaker
      - [ ] v3 - Update `oagw.circuit_breaker.state` gauge [F53.005]
- [ ] v3 - **ELSE IF** invocation failed (5xx status or error):
  - [ ] v3 - Increment consecutive failure count
  - [ ] v3 - **IF** failure_count >= `circuit_breaker_threshold`:
    - [ ] v3 - Open circuit breaker
    - [ ] v3 - Set half-open timer for `circuit_breaker_timeout_sec`
    - [ ] v3 - Increment `oagw.circuit_breaker.opened` counter [F53.005]
    - [ ] v3 - Update state gauge

### 1.13 Audit Logging

- [ ] v2 - Insert row into `outbound_api_audit_log` [F73.005]:
  - [ ] v2 - `tenant_id` from `SecurityCtx`
  - [ ] v2 - `user_id` from `SecurityCtx` (if available)
  - [ ] v2 - `route_id`, `link_id`
  - [ ] v2 - `operation` = HTTP method
  - [ ] v2 - `target_url`
  - [ ] v2 - `status_code` from response
  - [ ] v2 - `duration_ms` from timer
  - [ ] v2 - `error_message` (if error occurred)
  - [ ] v2 - `trace_id` from current span [F08.001]
  - [ ] v2 - `timestamp` = now

### 1.14 Metrics Recording

- [ ] v2 - Increment `oagw.invocations_total{tenant_id, route_id, status}` [F53.001]
- [ ] v2 - Record `oagw.request.duration_msec{route_id, protocol}` histogram [F53.002]
- [ ] v2 - **IF** error occurred:
  - [ ] v2 - Increment `oagw.errors_total{error_type, route_id}` [F53.003]
- [ ] v3 - Record `oagw.bytes_sent_total` (request body size) [F53.006]
- [ ] v3 - Record `oagw.bytes_received_total` (response body size)
- [ ] v3 - Decrement active request counter

### 1.15 Response Construction

- [ ] v1 - **IF** success (plugin returned response):
  - [ ] v1 - Map `OagwInvokeResponse` to `InvokeResponseDto`
  - [ ] v1 - Return `200 OK` with JSON body [F27.001]
- [ ] v1 - **ELSE IF** downstream error (4xx/5xx from downstream):
  - [ ] v1 - Propagate status code
  - [ ] v1 - Include downstream body in response
  - [ ] v1 - **IF** downstream returned `Retry-After` header:
    - [ ] v1 - Propagate `retry_after_sec` in response [F55.005]
- [ ] v2 - Add response headers:
  - [ ] v2 - `X-Request-Id` (correlation)
  - [ ] v2 - `X-OAGW-Duration-Ms` (processing time)
  - [ ] v2 - `X-OAGW-Link-Id` (which link was used)
  - [ ] v2 - `Retry-After` (if applicable, MUST propagate) [F55.005]

### 1.16 Response Caching (Optional)

- [ ] v5 - **IF** request method was GET AND `route.cache_ttl_sec` is set:
  - [ ] v5 - **IF** response does NOT have `Cache-Control: no-store`:
    - [ ] v5 - Build cache key: `(route_id, path, query, relevant_headers)` [F52.002]
    - [ ] v5 - Store response in cache with TTL
- [ ] v5 - **BEFORE** step 1.11, check response cache:
  - [ ] v5 - **IF** cache hit for GET request:
    - [ ] v5 - Return cached response immediately
    - [ ] v5 - Skip plugin invocation

---

## Scenario 2: Direct Rust API Invocation

Internal module invokes downstream API via `OagwApi` trait (ClientHub).

### 2.1 Client Acquisition

- [ ] v1 - Caller module gets `OagwApi` client from `ClientHub`:
  ```rust
  let oagw = ctx.client_hub().get::<dyn OagwApi>()?;
  ```
- [ ] v1 - `OagwApi` is registered by OAGW gateway during `init()` [F50.001]

### 2.2 Request Construction

- [ ] v1 - Caller constructs `OagwInvokeRequest` [F21.001]:
  ```rust
  let req = OagwInvokeRequest {
      route_id,
      link_id: None, // or Some(specific_link)
      method: HttpMethod::Post,
      path: "/v1/chat/completions".to_string(),
      query: None,
      headers: Some(custom_headers),
      body: Some(request_body),
      timeout_ms: Some(30000),
      retry_intent: RetryIntent::default(), // No retry by default [F55.002]
  };
  ```
- [ ] v3 - **IF** caller wants retry, configure `RetryIntent` [F55.001]:
  ```rust
  let req = OagwInvokeRequest {
      // ...
      retry_intent: RetryIntent {
          max_attempts: 3,
          retry_on: vec![
              GtsInstanceId::from("gts.x.core.errors.err.v1~x.oagw.timeout.connection.v1"),
              GtsInstanceId::from("gts.x.core.errors.err.v1~x.oagw.timeout.request.v1"),
          ],
          backoff: BackoffStrategy::Exponential {
              initial_ms: 100,
              multiplier: 2.0,
              max_ms: 5000,
          },
          budget: Some(shared_budget.clone()), // Optional shared budget [F55.003]
      },
  };
  ```

### 2.3 Unary Invocation

- [ ] v1 - Call `invoke_unary`:
  ```rust
  let response = oagw.invoke_unary(&ctx, req).await?;
  ```
- [ ] v1 - Service validates `SecurityCtx` has valid `tenant_id`
- [ ] v1 - **EXECUTE** steps 1.3 through 1.15 (same as REST scenario)
- [ ] v1 - Return `OagwInvokeResponse` [F21.002]:
  ```rust
  OagwInvokeResponse {
      status_code: 200,
      headers: response_headers,
      body: response_body,
      duration_ms: 150,
      link_id: selected_link_id,
  }
  ```

### 2.4 Streaming Invocation

- [ ] v2 - Call `invoke_stream` for SSE/streaming responses [F50.002]:
  ```rust
  let stream = oagw.invoke_stream(&ctx, req).await?;
  ```
- [ ] v2 - Service selects plugin supporting stream protocol [F11.004]
- [ ] v2 - Plugin returns `OagwResponseStream` [F21.003]
- [ ] v2 - Caller consumes stream:
  ```rust
  while let Some(chunk) = stream.next().await {
      match chunk {
          Ok(data) => process_chunk(data),
          Err(e) => handle_stream_error(e),
      }
  }
  ```

### 2.5 Stream Protocol Selection

- [ ] v2 - **IF** route supports SSE protocol [F11.004]:
  - [ ] v2 - Select plugin with `supported_stream_protocols` containing SSE
  - [ ] v2 - Plugin initiates HTTP request with SSE headers
  - [ ] v2 - Plugin parses `text/event-stream` frames
  - [ ] v2 - Each SSE event becomes `OagwStreamChunk`

### 2.6 Stream Error Handling (No Retry)

- [ ] v2 - **OAGW never retries streaming requests** [F55.006]
- [ ] v2 - **IF** stream connection drops mid-stream:
  - [ ] v2 - Stream yields terminal `OagwStreamAbort` [F21.004]
  - [ ] v2 - Error: `gts.x.core.errors.err.v1~x.oagw.stream.aborted.v1` (retriable=false)
  - [ ] v2 - Include `bytes_received` in error
  - [ ] v2 - Include `resume_hint` (if SSE) for caller to resume
  - [ ] v2 - Record error in audit log
- [ ] v3 - **IF** stream idle timeout exceeded [F20.004]:
  - [ ] v3 - Terminate stream
  - [ ] v3 - Error: `gts.x.core.errors.err.v1~x.oagw.timeout.idle.v1`
- [ ] v3 - **IF** response size limit exceeded:
  - [ ] v3 - Abort stream
  - [ ] v3 - Error: `gts.x.core.errors.err.v1~x.oagw.payload.too_large.v1`
- [ ] v3 - **Caller is responsible for stream restart**:
  - [ ] v3 - Use `resume_hint` (e.g., `Last-Event-ID`) for SSE resume (if protocol supports)
  - [ ] v3 - Resume capability is protocol-dependent, NOT OAGW's responsibility

### 2.7 Error Propagation

- [ ] v1 - **IF** OAGW returns error:
  - [ ] v1 - Error is typed `OagwError` with GTS ID [F54.002]
  - [ ] v1 - Error includes `retriable` hint and `retry_after_sec` [F54.003]
  - [ ] v1 - Caller can match on GTS error ID:
    ```rust
    match &result {
        Err(e) if e.gts_id == GTS_OAGW_LINK_UNAVAILABLE => fallback_logic(),
        Err(e) if e.gts_id == GTS_OAGW_CIRCUIT_BREAKER_OPEN => {
            // Use retry_after_sec if available
            if let Some(delay) = e.retry_after_sec {
                sleep(Duration::from_secs(delay)).await;
            }
            retry_with_different_link()
        },
        Err(e) if e.gts_id == GTS_OAGW_RATE_LIMIT_EXCEEDED => {
            // MUST respect retry_after_sec [F55.005]
            let delay = e.retry_after_sec.unwrap_or(60);
            sleep(Duration::from_secs(delay)).await;
        },
        Err(e) => return Err(e.into()),
    }
    ```

### 2.8 Retry Decision (Caller-Side) [F55.004]

- [ ] v1 - **Default: No retry** — `RetryIntent::default()` has `max_attempts: 1`
- [ ] v3 - **IF** `RetryIntent.max_attempts > 1`, OAGW MAY perform retries within the same invocation [F55.004]
- [ ] v3 - `attempt_number` in `OagwInvokeResponse` is semantic and shows which attempt produced the returned response
- [ ] v1 - **Streaming requests are NEVER retried by OAGW** [F55.006]

---

## Scenario 3: Route & Link Management (REST API)

Admin configures routes and links via REST API.

### 3.1 Create Route

- [ ] v1 - `POST /oagw/v1/routes` [F20.001]
- [ ] v1 - Validate caller has `gts.x.core.idp.role.v1~x.oagw.route.admin.v1` role [F42.002]
- [ ] v1 - Validate request body:
  - [ ] v1 - `base_url` is valid URL
  - [ ] v1 - `auth_type_gts_id` exists in `types_registry`
  - [ ] v2 - `protocol_gts_ids[]` all exist in `types_registry`
- [ ] v1 - Generate UUID for route
- [ ] v1 - Insert into `outbound_api_route` [F73.001]
- [ ] v2 - Insert supported protocols [F73.002]
- [ ] v1 - Return `201 Created` with route [F27.002]

### 3.2 Create Link

- [ ] v1 - `POST /oagw/v1/links` [F20.006]
- [ ] v1 - Validate caller has `gts.x.core.idp.role.v1~x.oagw.link.admin.v1` role [F42.003]
- [ ] v1 - Validate `route_id` exists and belongs to tenant
- [ ] v1 - Validate `secret_ref` exists in `cred_store`
- [ ] v1 - Validate `strategy_gts_id` exists
- [ ] v1 - Generate UUID for link
- [ ] v1 - Insert into `outbound_api_link` [F73.003]
- [ ] v1 - Return `201 Created` with link

### 3.3 List Routes (with OData)

- [ ] v2 - `GET /oagw/v1/routes?$skip=0&$top=20&$filter=...` [F20.002]
- [ ] v2 - Apply tenant isolation filter
- [ ] v2 - Apply OData `$filter` to query
- [ ] v2 - Apply `$orderby` sorting
- [ ] v2 - Apply pagination (`$skip`, `$top`)
- [ ] v3 - Apply `$select` field projection
- [ ] v2 - Return `Page<RouteDto>` with `@odata.nextLink`

### 3.4 Update Route Limits

- [ ] v3 - `PUT /oagw/v1/routes/{routeId}/limits`
- [ ] v3 - Validate caller has `gts.x.core.idp.role.v1~x.oagw.route.admin.v1` role
- [ ] v3 - Upsert into `outbound_api_route_limits` [F73.004]
- [ ] v3 - Invalidate any cached configuration
- [ ] v3 - Return updated limits

### 3.5 Clear Token Cache

- [ ] v3 - `DELETE /oagw/v1/routes/{routeId}/cache/tokens` [F20.012]
- [ ] v3 - Validate caller has `gts.x.core.idp.role.v1~x.oagw.route.admin.v1` role
- [ ] v3 - Evict all cache entries matching `route_id` [F52.001]
- [ ] v3 - Log cache clear operation
- [ ] v3 - Return `204 No Content` [F27.003]

---

## Scenario 4: Health Checks

### 4.1 Liveness Probe

- [ ] v1 - `GET /oagw/v1/health` [F20.013]
- [ ] v1 - No authentication required [F03.001]
- [ ] v1 - Return `200 OK` if service is running
- [ ] v1 - Response body: `{ "status": "healthy" }`

### 4.2 Readiness Probe

- [ ] v2 - `GET /oagw/v1/ready` [F20.014]
- [ ] v2 - No authentication required
- [ ] v2 - Check database connectivity
- [ ] v2 - **IF** all checks pass:
  - [ ] v2 - Return `200 OK`

### 4.3 Route Health Check

- [ ] v3 - `GET /oagw/v1/routes/{routeId}/health` [F20.015]
- [ ] v3 - Require `gts.x.core.idp.role.v1~x.oagw.route.health.v1` role [F42.001]
- [ ] v3 - Get all links for route
- [ ] v3 - **FOR EACH** link:
  - [ ] v3 - Check circuit breaker state
  - [ ] v3 - **IF** route has `healthcheck_url`:
    - [ ] v4 - Perform lightweight health check request
- [ ] v3 - Return aggregated health status:
  ```json
  {
    "route_id": "...",
    "status": "degraded",
    "links": [
      { "link_id": "...", "status": "healthy" },
      { "link_id": "...", "status": "circuit_open" }
    ]
  }
  ```

---

## Scenario 5: Plugin Lifecycle

### 5.1 Gateway Module Initialization

- [ ] v1 - OAGW gateway `init()` runs
- [ ] v1 - Register GTS schemas in `types_registry` [F10.001-F10.003]:
  - [ ] v1 - `gts.x.core.oagw.proto.v1~`
  - [ ] v1 - `gts.x.core.oagw.stream_proto.v1~`
  - [ ] v1 - `gts.x.core.oagw.auth_type.v1~`
  - [ ] v1 - `gts.x.core.oagw.strategy.v1~`
- [ ] v1 - Load well-known instances from `gts/*.instances.json`
- [ ] v1 - Register instances in `types_registry` [F11.001-F13.005]
- [ ] v1 - Create `OagwService` with dependencies
- [ ] v1 - Register `OagwApi` client in `ClientHub` (unscoped)

### 5.2 Plugin Module Initialization

- [ ] v1 - Plugin module `init()` runs (after OAGW gateway)
- [ ] v1 - Generate stable GTS instance ID:
  ```rust
  let instance_id = OagwPluginSpecV1::gts_make_instance_id(
      "x.core.oagw.http_plugin.v1"
  );
  ```
- [ ] v1 - Register plugin instance in `types_registry`:
  ```rust
  let instance = BaseModkitPluginV1::<OagwPluginSpecV1> {
      id: instance_id.clone(),
      vendor: "x".into(),
      priority: 10,
      properties: OagwPluginSpecV1 {
          supported_protocols: vec![HTTP11, HTTP2, SSE],
          supported_auth_types: vec![BEARER, API_KEY_HEADER],
          // ...
      },
  };
  registry.register(&ctx, vec![instance_json]).await?;
  ```
- [ ] v1 - Create plugin service
- [ ] v1 - Register scoped client in `ClientHub`:
  ```rust
  ctx.client_hub().register_scoped::<dyn OagwPluginApi>(
      ClientScope::gts_id(&instance_id),
      plugin_service,
  );
  ```

### 5.3 Plugin Discovery at Runtime

- [ ] v1 - When OAGW needs to select plugin (step 1.6):
- [ ] v1 - Query `types_registry` for plugin instances
- [ ] v1 - Filter by capability requirements
- [ ] v1 - Select by priority
- [ ] v1 - Resolve from `ClientHub` via scoped lookup
