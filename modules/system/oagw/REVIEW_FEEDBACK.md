# OAGW Design Review - Combined Feedback

**Date**: 2026-02-05  
**Sources**: Cascade Review, Claude Review

---

## 1. Architectural Weaknesses & Design Flaws

- [ ] 1.1 **[Critical]** Raw SQL vs Secure ORM (Cascade)
    - Location: `DESIGN.md` → Database Persistence → Common Queries (lines 2274+)
    - Issue: Design specifies raw SQL for tenant-scoped lookups while project mandates Secure ORM layer
    - Impact: Tenant boundary violations, injection risks, two parallel "truths" that will drift
    - Recommendation: Treat SQL as illustrative only, define persistence layer via `SecureConn`-scoped SeaORM queries, add invariant: no query executes without `SecurityContext`

- [ ] 1.2 **[High]** Protocol Capability Cache Key Semantics (Cascade)
    - Location: `DESIGN.md` → Architecture / Security Considerations (lines 142-153)
    - Issue: Cache is "per host/IP" with TTL=1h but no defined key semantics (hostname vs resolved IP vs SNI) nor invalidation triggers
    - Impact: Incorrect protocol selection after DNS change/failover, cross-tenant coupling via shared cache
    - Recommendation: Specify key = `(tenant_id?, upstream_id, endpoint_id, resolved_ip, alpn_result)`, ensure caches are per-upstream endpoint, define invalidation on DNS change
      and repeated failures

- [ ] 1.3 **[High]** 3-Layer Model Inconsistency (Cascade)
    - Location: `docs/adr-resource-identification.md` vs `DESIGN.md` → Hierarchical Configuration
    - Issue: ADR introduces 3-layer model (Upstream Definition + Tenant Binding + Request Context), but persistence stores upstream config in single `oagw_upstream` table without
      binding entity
    - Impact: Ambiguous source of truth, broken sharing semantics, hard-to-audit access control
    - Recommendation: Either implement bindings explicitly (table + API) or remove binding concepts from ADR

- [ ] 1.4 **[High]** Mixed Return Types in Alias Resolution (Cascade)
    - Location: `DESIGN.md` → Alias Resolution → pseudocode (lines 372-396)
    - Issue: Pseudocode returns `Response(...)` from function that otherwise returns upstream object
    - Impact: Ad-hoc error handling, potential "fallback to parent" on error paths creating isolation gaps
    - Recommendation: Define strict return type (`Result<Upstream, Problem>`), ensure distinct typed errors for "not found", "missing host header", "incompatible alias selection"

- [ ] 1.5 **[High]** Race Condition in Circuit Breaker Half-Open State (Claude)
    - Location: ADR: Circuit Breaker, Half-Open State Management
    - Issue: Redis Lua script (lines 218-235) performs non-atomic read-check-transition operations, multiple nodes could simultaneously transition to HALF_OPEN
    - Impact: Probe flood to recovering upstream, inconsistent circuit state across nodes
    - Recommendation: Use atomic Redis `WATCH`/`MULTI`/`EXEC` or single Lua script that atomically checks state, time, increments counter, returns permit decision

- [ ] 1.6 **[Medium]** Task Explosion in Queue Consumer (Cascade)
    - Location: `docs/adr-backpressure-queueing.md` → Queue Consumer pseudocode (lines 349-381)
    - Issue: Consumer loop acquires permit and `tokio::spawn`s execution without documented upper bound
    - Impact: Memory/CPU pressure amplification under load
    - Recommendation: Bound execution concurrency explicitly with fixed worker pool, ensure queue consumers cannot spawn more than `max_concurrent` tasks per scope

- [ ] 1.7 **[Medium]** Local-Only Concurrency Limiting Weakness (Cascade)
    - Location: `docs/adr-concurrency-control.md` → Local-only limiting (lines 161-173)
    - Issue: Phase 1/2 "local-only" limiting documented as acceptable but tenant isolation requirement is strong
    - Impact: Single tenant can exceed global limits by spreading traffic across nodes
    - Recommendation: Document as hard constraint: require distributed coordinator for strict isolation, or require load balancer tenant-affinity

- [ ] 1.8 **[Medium]** Plugin Chain Resource Cleanup on Early Termination (Claude)
    - Location: DESIGN.md Plugin Execution Order (lines 541-558), Starlark Context API (lines 562-606)
    - Issue: No specified behavior for cleanup of acquired resources (permits, rate limit tokens) when plugin calls `ctx.respond()` or `ctx.reject()`
    - Impact: Resource leaks if plugins short-circuit before permits are tracked for release
    - Recommendation: Document explicit permit lifecycle: permits acquired before plugin chain execution and released after response, regardless of termination point

- [ ] 1.9 **[Medium]** Connection Pool Exhaustion Under Multi-Endpoint Load Balancing (Claude)
    - Location: DESIGN.md Multi-Endpoint Load Balancing (lines 459-467), ADR: Concurrency Control (lines 236-260)
    - Issue: Round-robin doesn't coordinate with per-endpoint connection pools, slow endpoint causes pile-up
    - Impact: Increased latency, unfair load distribution, pool exhaustion on slow endpoint
    - Recommendation: Implement weighted round-robin based on pending request count, or adaptive load balancing (least connections)

- [ ] 1.10 **[Medium]** Memory Safety in Starlark Body Access (Claude)
    - Location: DESIGN.md Starlark Context API (lines 571-575, 579-580), Sandbox Restrictions (lines 609-621)
    - Issue: `ctx.request.body` and `ctx.request.json()` require buffering entire body (up to 100MB), multiple concurrent executions could exhaust memory
    - Impact: OOM conditions under load with large request bodies
    - Recommendation: Add per-request memory limit for Starlark execution, document that body access forces buffering

---

## 2. Protocol & Standards Compliance

- [ ] 2.1 **[High]** Connection Header Token Stripping (Cascade)
    - Location: `DESIGN.md` → Headers Transformation (lines 291-310) + `examples/case-7.2-hop-by-hop-headers-stripped.md`
    - Issue: Hop-by-hop stripping uses fixed set, doesn't handle `Connection: <header-names>` nominated headers (RFC 7230)
    - Impact: Header smuggling if client supplies `Connection: Foo` and `Foo: bar` and gateway forwards `Foo` upstream
    - Recommendation: Parse `Connection` tokens and strip those header names too, add negative test

- [ ] 2.2 **[High]** TE Header Handling for HTTP/2 (Cascade)
    - Location: `DESIGN.md` → Headers Transformation table (lines 293-306) + gRPC ADR
    - Issue: Design states `TE` is stripped unconditionally, but `TE: trailers` is allowed for HTTP/2 and gRPC relies on trailer semantics
    - Impact: Breaking gRPC/HTTP/2 semantics
    - Recommendation: For HTTP/2: allow only `TE: trailers` (strip otherwise), add protocol-specific header rules

- [ ] 2.3 **[High]** Body Validation vs Streaming Conflict (Cascade)
    - Location: `DESIGN.md` → Body Validation Rules (lines 327-335)
    - Issue: "Content-Length must match actual size" conflicts with "reject before buffering" + streaming bodies
    - Impact: Either buffer (breaking streaming/backpressure) or can't validate (false rejects/accepts)
    - Recommendation: Define validation by protocol: for streaming use counter for max size; validate Content-Length only when body fully buffered or transport provides reliable
      length

- [ ] 2.4 **[Medium]** Path Normalization Rules Missing (Cascade)
    - Location: `DESIGN.md` → Proxy Endpoint (lines 2054-2063)
    - Issue: No normalization rules for absolute-form, authority-form, path encoding (`..`, `%2f`, `%5c`, double-encoding)
    - Impact: Path traversal, route bypass, inconsistent route matching
    - Recommendation: Specify canonicalization rules, use raw path consistently, reject ambiguous encodings, reject absolute-form unless supported

- [ ] 2.5 **[Medium]** gRPC Detection Incomplete (Cascade)
    - Location: `docs/adr-grpc-support.md`
    - Issue: gRPC detection based only on `content-type` prefix, no validation of HTTP/2 pseudo-headers, `:path` mapping
    - Impact: Misclassification routes non-gRPC into gRPC proxy and vice versa
    - Recommendation: Require: HTTP/2 + `POST` + `content-type` starts with `application/grpc` + `:path` matches `/package.Service/Method`

- [ ] 2.6 **[Medium]** HTTP/2 GOAWAY Handling Not Specified (Claude)
    - Location: DESIGN.md HTTP Version Negotiation (lines 146-154)
    - Issue: Spec mentions "Automatic retry on connection errors" but doesn't specify GOAWAY behavior (connection draining)
    - Impact: Requests may fail with stream reset errors if sent on draining connections
    - Recommendation: Document GOAWAY handling: mark connection as draining, complete in-flight requests, use new connection for new requests

- [ ] 2.7 **[Low]** OData $filter on JSONB Fields Not Specified (Claude)
    - Location: DESIGN.md REST API Query Parameters (lines 1968-1972)
    - Issue: Filtering on nested JSONB fields undefined (e.g., `server.endpoints[0].host`)
    - Impact: Users may expect deep filtering but behavior is undefined
    - Recommendation: Document which fields support $filter, restrict to top-level non-JSONB or define JSONB path syntax

- [ ] 2.8 **[Low]** Transfer-Encoding + Content-Length Ambiguity (Claude)
    - Location: DESIGN.md Body Validation Rules (lines 325-334)
    - Issue: Doesn't specify behavior when both headers are present (RFC 7230 says ignore Content-Length)
    - Impact: Potential request smuggling vectors
    - Recommendation: Explicitly state: "If both Transfer-Encoding: chunked and Content-Length present, Content-Length is ignored per RFC 7230"

---

## 3. Security Vulnerabilities

- [ ] 3.1 **[Critical]** SSRF Protections Out of Scope (Cascade)
    - Location: `DESIGN.md` → Out of Scope (lines 121-126) + Security Considerations/SSRF (lines 128-135)
    - Issue: Critical SSRF components declared out-of-scope (DNS resolution, IP pinning, allowed segments), leaves highest-risk vectors unspecified (DNS rebinding, IPv6 literals,
      link-local, private ranges, TOCTOU)
    - Impact: SSRF exploitation against metadata services, internal RFC1918, rebinding targets
    - Recommendation: Make SSRF first-class requirement: specify resolution policy (A/AAAA), deny/allow lists by CIDR, IP pinning per connection, rebinding-safe connect validation

- [ ] 3.2 **[High]** Alias Resolution Host Header Issues (Cascade)
    - Location: `DESIGN.md` → Alias Resolution (lines 383-392)
    - Issue: Multi-endpoint aliasing requires inbound `Host` header, doesn't specify HTTP/2 `:authority` handling or host-header spoofing prevention
    - Impact: Tenant can steer endpoint selection unexpectedly, SSRF-like steering within pool
    - Recommendation: Use dedicated header (e.g., `X-OAGW-Upstream-Host`) validated and stripped from upstream, or bind selection to SNI/`:authority` under TLS rules

- [ ] 3.3 **[High]** Plugin Secret/PII Logging (Cascade)
    - Location: `DESIGN.md` → Audit Logging (lines 2510-2557) + plugin `ctx.log` API (lines 560-607/1684-1731)
    - Issue: Starlark plugins can read `ctx.request.body`/`ctx.request.json()` and log arbitrary data without guardrails
    - Impact: Secret leakage (API keys, OAuth tokens, credentials) into logs, cross-tenant exposure
    - Recommendation: Plugin logs must pass through redaction layer, provide "sensitive" classification API, disable body logging by default with size limits

- [ ] 3.4 **[High]** Header Injection Coverage Incomplete (Cascade)
    - Location: `examples/case-26.2-header-injection-protection.md` + DESIGN.md header transformation
    - Issue: Coverage focuses on `\r\n` but misses Unicode normalization, obs-fold, invalid header names, duplicate `Host`, multiple `Content-Length`, CL/TE smuggling patterns
    - Impact: Request smuggling, upstream desync, SSRF steering, cache poisoning
    - Recommendation: Define strict header parsing: reject invalid names/values, reject multiple `Host`, reject ambiguous CL/TE combinations, add smuggling tests

- [ ] 3.5 **[High]** DNS Rebinding Window in HTTP Version Cache (Claude)
    - Location: DESIGN.md HTTP Version Negotiation (lines 146-154)
    - Issue: Protocol version cached by "host/IP" with 1h TTL, DNS resolution timing not clarified, could bypass SSRF protections
    - Impact: DNS rebinding attacks could bypass SSRF protections
    - Recommendation: DNS resolution must occur per-request, HTTP version cache should key by resolved IP not hostname, resolve "out of scope" DNS/IP pinning rules immediately

- [ ] 3.6 **[High]** Unicode Header Injection (Claude)
    - Location: DESIGN.md Headers Transformation (lines 293-309), examples/case-26.2-header-injection-protection.md
    - Issue: Test only covers CRLF, misses Unicode attacks (U+0085, U+2028, U+2029, zero-width characters, bidirectional override)
    - Impact: Header injection via Unicode edge cases
    - Recommendation: Validate header values as ASCII (or explicit UTF-8 with all line-terminators rejected), add Unicode test cases

- [ ] 3.7 **[Medium]** Secret Access Control Error Code (Cascade)
    - Location: `DESIGN.md` → Secret Access Control (lines 843-859)
    - Issue: `cred_store` denial mapped to `401 Unauthorized` instead of `403 Forbidden`
    - Impact: Information leaks (distinguishing secret existence), confusing client behavior
    - Recommendation: Map denial to `403 Forbidden`, treat "secret not found" separately

- [ ] 3.8 **[Medium]** Error Source Header Stripping (Cascade)
    - Location: `docs/adr-error-source-distinction.md` + DESIGN.md error handling
    - Issue: `X-OAGW-Error-Source` header may be stripped with no fallback for security-sensitive flows
    - Impact: Clients mis-handle errors causing retry storms or hiding gateway rejections
    - Recommendation: Ensure body is always RFC 9457 with stable `type` values, document clients should treat non-problem+json as "likely upstream"

- [ ] 3.9 **[Medium]** Tenant ID in Circuit Breaker Redis Keys (Claude)
    - Location: ADR: Circuit Breaker (lines 212-218)
    - Issue: Redis keys include `tenant_id` in plaintext: `oagw:cb:{tenant_id}:{upstream_id}:state`
    - Impact: Information disclosure about tenant configuration and usage patterns if Redis compromised
    - Recommendation: Hash tenant_id in Redis keys: `oagw:cb:{hash(tenant_id)}:{upstream_id}:state`

- [ ] 3.10 **[Medium]** Plugin Source Code Exposure via API (Claude)
    - Location: DESIGN.md Plugin Endpoints (lines 1998-2010)
    - Issue: `GET /api/oagw/v1/plugins/{id}/source` returns Starlark source as `text/plain`, may contain sensitive business logic
    - Impact: Information disclosure if permissions misconfigured
    - Recommendation: Consider removing from public API, require explicit `include_source=true`, audit all source access

- [ ] 3.11 **[Medium]** Starlark ctx.log May Leak Sensitive Data (Claude)
    - Location: DESIGN.md Audit Logging (lines 2546-2549)
    - Issue: Audit logging excludes bodies/secrets, but `ctx.log.info(msg, data)` can log arbitrary data including parsed request bodies
    - Impact: Sensitive data in logs despite audit policy
    - Recommendation: Add log sanitization for plugin-generated logs, maximum log message size limit

---

## 4. Operational & Reliability Issues

- [ ] 4.1 **[High]** Rate Limiting Redis Failure Mode (Cascade)
    - Location: `docs/adr-rate-limiting.md` + DESIGN.md → rate limit schema
    - Issue: Failure mode is "fallback to local-only" without strict bound, worst-case overshoot scales with `node_count` and burst capacity
    - Impact: Real-world quota breaches and SLA violations, fairness bugs during partial outages
    - Recommendation: Define "overshoot bound" and expose operationally, provide strict mode that fails closed for enforce budgets when Redis unavailable

- [ ] 4.2 **[High]** Circuit Breaker Metrics vs Cardinality Rules Conflict (Cascade)
    - Location: `docs/adr-circuit-breaker.md` + DESIGN.md metrics cardinality rules
    - Issue: ADR proposes metrics with `tenant_id` label (line ~289), DESIGN.md forbids per-tenant labels (lines 2481-2486)
    - Impact: Either violate cardinality requirement or lose observability for multi-tenant debugging
    - Recommendation: Decide: no tenant labels in Prometheus with tenant-level debugging via logs/traces, or enable tenant labels behind gated debug endpoint

- [ ] 4.3 **[High]** Token Bucket Drift with Distributed Sync (Claude)
    - Location: ADR: Rate Limiting, Hybrid Sync (lines 183-218)
    - Issue: Sync algorithm has race condition between push and pull, global count may double-count requests, can exceed limit 2-3x
    - Impact: Rate limits significantly exceeded during high concurrency
    - Recommendation: Use Redis `INCRBY` returning new value in single command, implement proper distributed rate limiting (Redis Cell or Sliding Window Log)

- [ ] 4.4 **[Medium]** Metrics Host Label Ambiguity with Alias Shadowing (Cascade)
    - Location: `DESIGN.md` → Metrics labels: `{host, path, method, status_class}` (lines 2428-2466)
    - Issue: `host` label uses hostname, but alias shadowing means same `host` might represent different tenant configs/endpoints
    - Impact: Misleading metrics for incident response
    - Recommendation: Add stable low-cardinality identifier label (e.g., `upstream_key` = first 8 chars of UUID), keep `host` as informational

- [ ] 4.5 **[Medium]** Queueing vs Streaming Request Conflict (Cascade)
    - Location: `docs/adr-backpressure-queueing.md`
    - Issue: Queueing interacts with streaming requests ("metadata only queued") but doesn't specify how body stream is preserved during wait
    - Impact: Memory spike from buffering or streaming not actually supported
    - Recommendation: Define support matrix: disallow queueing for streaming bodies or require explicit buffering policy

- [ ] 4.6 **[Medium]** Audit Logging Sampling May Lose Security Events (Cascade)
    - Location: `DESIGN.md` → Audit Logging (lines 2537-2549)
    - Issue: Sampling guidance doesn't define where applied (pre/post plugins) or how to avoid sampling away security-relevant events
    - Impact: Loss of forensic data during attacks
    - Recommendation: Define non-sampleable event classes (authZ failures, config changes, circuit transitions)

- [ ] 4.7 **[Medium]** Sliding Window Boundary Condition (Claude)
    - Location: ADR: Rate Limiting, Algorithm Selection (lines 30-40)
    - Issue: Sliding window with `window: "day"` has 24h window, at UTC midnight entire previous day's count may slide out causing burst
    - Impact: Burst allowed at window boundaries
    - Recommendation: Use true sliding window with sub-window counting, document precision vs storage trade-off

- [ ] 4.8 **[Medium]** Circuit Breaker Opens for Single-Endpoint Failures (Claude)
    - Location: ADR: Circuit Breaker, `scope: "global"` default (line 116)
    - Issue: With `scope: "global"` default and multiple endpoints, one failing endpoint opens entire upstream circuit
    - Impact: Single unhealthy endpoint takes down entire upstream, negates redundancy
    - Recommendation: Consider `scope: "per_endpoint"` as default for multi-endpoint upstreams, document behavior prominently

- [ ] 4.9 **[Medium]** Metrics Cardinality Explosion with Many Routes (Claude)
    - Location: DESIGN.md Cardinality Management (lines 2479-2485)
    - Issue: `path` label uses route config which is bounded per upstream, but no max routes limit specified
    - Impact: Prometheus overload with high route counts
    - Recommendation: Add configurable `max_routes_per_upstream` limit, consider aggregating into categories

- [ ] 4.10 **[Medium]** Queue Timeout Cascade Risk (Claude)
    - Location: ADR: Backpressure (lines 106-107, 606-609)
    - Issue: Default queue timeout is 5s, same as common client timeout, requests may timeout while queued
    - Impact: Queue fills with abandoned requests, reduced effective throughput
    - Recommendation: Default queue timeout should be 2-3s, add guidance `queue.timeout < expected_client_timeout * 0.5`

---

## 5. Configuration & API Design

- [ ] 5.1 **[High]** Dual Permission Systems (Cascade)
    - Location: `DESIGN.md` → Permissions and Access Control (lines 860-872) vs Inbound AuthZ permissions table (lines 159-174)
    - Issue: Two permission systems: GTS permission strings (`gts.x.core...`) and separate `oagw:*` permissions, relationship unspecified
    - Impact: Authorization gaps, inconsistent policy across management/proxy/override operations
    - Recommendation: Define single permission model or deterministic mapping, specify which API operation checks which permissions, add bypass tests

- [ ] 5.2 **[High]** Plugin Reference Type Confusion (Cascade)
    - Location: `DESIGN.md` → Upstream schema `plugins.items` (lines 1039-1056) vs plugin identification (lines 481-494)
    - Issue: Plugin references described as always GTS identifiers, but schema allows raw UUIDs as alternative
    - Impact: Type confusion, inconsistent parsing, potential injection vector
    - Recommendation: Choose one canonical representation (prefer GTS identifier strings), document exact normalization

- [ ] 5.3 **[Medium]** CORS Origin Validation Too Permissive (Cascade)
    - Location: `DESIGN.md` → CORS schema `allowed_origins` uses `format: uri` (lines 1204-1208)
    - Issue: Origin should be scheme+host+optional port, `format: uri` may accept paths, userinfo, fragments, non-HTTP schemes
    - Impact: CORS bypasses or misconfigurations
    - Recommendation: Validate origin as origin tuple, reject paths/query/fragment/userinfo, restrict scheme to http/https

- [ ] 5.4 **[Medium]** OData Validation Rules Not Specified (Cascade)
    - Location: `DESIGN.md` → OData list query parameters (lines 1964-1973)
    - Issue: Design lists OData options but doesn't specify validation rules (length/field count/duplicates) required by `case-25.2`
    - Impact: DoS vector via very long `$select`, inconsistent errors
    - Recommendation: Reference `docs/ODATA_SELECT.md` explicitly, define shared validator

- [ ] 5.5 **[Medium]** Rate Limit Merge Incomplete (Cascade)
    - Location: `DESIGN.md` → Merge Strategies (lines 660-668)
    - Issue: Only sustained rate described in pseudocode, but schema includes burst, scope, strategy, cost, response_headers, budget
    - Impact: Child can weaken burst capacity or change scope violating parent expectations
    - Recommendation: Define merge per field: sustained rate/window, burst, scope, budget, headers; use "stricter of each dimension"

- [ ] 5.6 **[Medium]** CORS vs Rate Limit Merge Strategy Inconsistency (Claude)
    - Location: DESIGN.md Rate Limit Merge (lines 661-667), ADR: CORS (lines 357-362)
    - Issue: Rate limits merge with `min()` (stricter wins), CORS `allowed_origins` merges with union (more permissive)
    - Impact: Users may assume all configs follow same pattern
    - Recommendation: Document rationale clearly, consider renaming CORS merge to differentiate

- [ ] 5.7 **[Medium]** Ambiguous `enabled: false` Inheritance (Claude)
    - Location: DESIGN.md Upstream Schema `enabled` field (lines 945-948), SQL query (lines 2283-2295)
    - Issue: Unclear if child creating shadowing upstream with `enabled: true` bypasses parent disable
    - Impact: Confusing behavior when shadowing disabled ancestor upstream
    - Recommendation: Clarify: shadowing bypasses disable or parent disable applies regardless, add test case

- [ ] 5.8 **[Low]** burst.capacity < sustained.rate Warning Missing (Claude)
    - Location: DESIGN.md rate_limit schema (lines 1125-1188)
    - Issue: `burst.capacity` can be set lower than rate, effectively reducing limit
    - Impact: Misconfiguration leading to lower-than-expected limits
    - Recommendation: Warn when `burst.capacity < sustained.rate`, document effective rate formula

- [ ] 5.9 **[Low]** Pool Compatibility Validation Not Specified (Claude)
    - Location: DESIGN.md Multi-Endpoint Load Balancing (lines 459-467)
    - Issue: Validation rules for pool compatibility not specified, unclear what happens when updating protocol/scheme/port with established connections
    - Impact: Runtime errors during upstream updates
    - Recommendation: Document validation: updates require no active connections or rejected with `409 Conflict`, add `connection_drain_timeout`

---

## 6. Implementation Gaps

- [ ] 6.1 **[High]** Duplicate Heading in Guard Rules (Cascade)
    - Location: `DESIGN.md` → Guard Rules (lines 311-314)
    - Issue: Duplicate section headings suggest unfinished editing, may hide contradictions
    - Impact: Missing requirements during implementation
    - Recommendation: Do spec consistency pass, remove duplicates, ensure each rule exists once with definitive behavior

- [ ] 6.2 **[High]** Error Types Table Incomplete (Cascade)
    - Location: `DESIGN.md` → Error Types table (lines 2562-2581)
    - Issue: Footnotes (`Yes*`, `No**`) without definitions, `DownstreamError` without clear protocol variant mapping
    - Impact: Clients cannot reliably implement retry logic
    - Recommendation: Make retriable semantics explicit per error type and protocol, link to reproducer test cases

- [ ] 6.3 **[Medium]** Plugin Lock Poisoning Risk (Cascade)
    - Location: `docs/adr-rust-abi-client-library.md` → Starlark integration uses `Arc<RwLock<PluginContext>>` with `unwrap()` (lines ~751-786)
    - Issue: Lock poisoning or panics can crash request handling, single lock is contention hotspot
    - Impact: Availability risk and latency spikes
    - Recommendation: Replace `unwrap()` with structured errors, avoid holding write locks across expensive operations

- [ ] 6.4 **[Medium]** In-Flight Counter Not Decremented on Error (Cascade)
    - Location: `docs/adr-rust-abi-client-library.md` → `DirectClient::execute` (lines 424-443)
    - Issue: Metrics decrement shown only on success path, error paths not shown to decrement
    - Impact: Metrics drift and "stuck in-flight" gauges, incorrect autoscaling/alerting
    - Recommendation: Ensure decrement happens in `drop` guard, add tests for error paths

- [ ] 6.5 **[Medium]** HTTP/3 and WebTransport Scope Mismatch (Cascade)
    - Location: `DESIGN.md` + examples around HTTP/3 / WebTransport
    - Issue: Prompt lists HTTP/3 and WebTransport but DESIGN.md states HTTP/3 is future work
    - Impact: Misaligned expectations for supported protocols
    - Recommendation: Add explicit "supported in v1" list in DESIGN.md, ensure examples/tests match only those

- [ ] 6.6 **[Medium]** Missing Test: Plugin Type Mismatch (Claude)
    - Location: DESIGN.md Plugin Resolution Algorithm (lines 526-532)
    - Issue: No test for what happens when upstream `plugins.items` contains guard plugin ID but record has `plugin_type: "transform"`
    - Impact: Unclear error behavior, could cause runtime panic
    - Recommendation: Add test case `case-4.x-plugin-type-mismatch.md` expecting `400` or `409`

- [ ] 6.7 **[Medium]** Missing Test: Concurrent Circuit Breaker Transitions (Claude)
    - Location: ADR: Circuit Breaker
    - Issue: No test covers concurrent requests racing to transition circuit state
    - Impact: Untested race conditions may cause production issues
    - Recommendation: Add test `case-20.x-circuit-concurrent-transitions.md`

- [ ] 6.8 **[Low]** Auth Plugin Immutability vs Override Confusion (Claude)
    - Location: DESIGN.md Auth Config Merge (lines 656-659), Plugin Lifecycle (lines 498-509)
    - Issue: Plugins are immutable but auth config can be overridden, relationship between plugin immutability and config override unclear
    - Impact: Confusion about whether auth override requires new plugin
    - Recommendation: Clarify: auth plugins referenced by ID with per-upstream `config` block, plugin code immutable but config mutable

- [ ] 6.9 **[Low]** Missing: Plugin Execution Time Metrics (Claude)
    - Location: DESIGN.md Metrics (lines 2420-2505)
    - Issue: `oagw_request_duration_seconds{phase}` includes "plugins" but doesn't break down by individual plugin
    - Impact: Difficult to identify slow plugins in production
    - Recommendation: Add `oagw_plugin_duration_seconds{plugin_id, phase}` histogram

- [ ] 6.10 **[Low]** Unspecified: Streaming Permit Lifetime (Claude)
    - Location: ADR: Concurrency Control (lines 158-159)
    - Issue: "Permit held until stream completes or client disconnects" - for SSE/WebSocket, what defines "completes"?
    - Impact: Long-lived streams could exhaust concurrency permits
    - Recommendation: Add `streaming_permit_timeout` option, document streaming should use separate pool

---

## Summary

| Category       | Critical | High   | Medium | Low   |
|----------------|----------|--------|--------|-------|
| Architecture   | 1        | 4      | 5      | 0     |
| Protocol       | 0        | 3      | 3      | 2     |
| Security       | 1        | 5      | 5      | 0     |
| Operations     | 0        | 3      | 7      | 0     |
| Configuration  | 0        | 2      | 5      | 2     |
| Implementation | 0        | 2      | 5      | 3     |
| **Total**      | **2**    | **19** | **30** | **7** |

---

## Priority Action Items

1. **Immediate (Critical)**
    - [ ] 1.1 - Raw SQL vs Secure ORM
    - [ ] 3.1 - SSRF Protections Out of Scope

2. **High Priority**
    - [ ] 1.2, 1.3, 1.4, 1.5 - Architecture weaknesses
    - [ ] 2.1, 2.2, 2.3 - Protocol compliance
    - [ ] 3.2, 3.3, 3.4, 3.5, 3.6 - Security vulnerabilities
    - [ ] 4.1, 4.2, 4.3 - Operational issues
    - [ ] 5.1, 5.2 - API design
    - [ ] 6.1, 6.2 - Implementation gaps

3. **Before Production (Medium)**
    - All medium severity items

4. **Enhancement (Low)**
    - All low severity items