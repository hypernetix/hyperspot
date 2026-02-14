# OAGW examples plan (E2E justification catalog)

Rules for all scenarios:
- Use Management API to create upstreams/routes/plugins unless scenario explicitly tests invalid config.
- Use Proxy API to invoke.
- Assert both:
  - Response semantics (status, headers, body/stream behavior)
  - Side effects (metrics/audit logs, stored config, plugin lifecycle)

Legend (used in checks):
- `ESrc`: `X-OAGW-Error-Source` header.
- `PD`: RFC 9457 Problem Details (`application/problem+json`).

---

## 1) Management API: authentication + authorization

### [x] 1.1 Justification: All management endpoints require bearer auth
- Scenario: [`negative-1.1-all-management-endpoints-require-bearer-auth.md`](management-api/auth/negative-1.1-all-management-endpoints-require-bearer-auth.md)
- Why it matters:
  - Prevents unauthenticated configuration tampering.
- What to check:
  - `POST /api/oagw/v1/upstreams` without `Authorization` returns `401`.
  - `GET /api/oagw/v1/routes` without `Authorization` returns `401`.
  - Errors are `PD` with stable `type`.

### [x] 1.2 Justification: Permission gates for upstream/route/plugin CRUD
- Scenario: [`negative-1.2-permission-gates-upstream-route-plugin-crud.md`](management-api/auth/negative-1.2-permission-gates-upstream-route-plugin-crud.md)
- Why it matters:
  - Prevents privilege escalation between operators/users.
- What to check:
  - Missing upstream permission returns `403`.
  - Missing route permission returns `403`.
  - Missing plugin permission returns `403`.
  - With correct permission, same call succeeds.

### [x] 1.3 Justification: Tenant scoping in management APIs
- Scenario: [`negative-1.3-tenant-scoping-management-apis.md`](management-api/auth/negative-1.3-tenant-scoping-management-apis.md)
- Why it matters:
  - Avoid cross-tenant reads/writes.
- What to check:
  - Principal from tenant A cannot `GET /upstreams/{id}` created by tenant B.
  - Listing endpoints only return tenant-visible resources.

---

## 2) Management API: upstream lifecycle

### [x] 2.1 Justification: Create minimal HTTP upstream (single endpoint)
- Scenario: [`positive-2.1-create-minimal-http-upstream.md`](management-api/upstreams/positive-2.1-create-minimal-http-upstream.md)
- Why it matters:
  - Base onboarding path.
- What to check:
  - `POST /upstreams` returns `201`.
  - Returned upstream has `enabled=true` default.
  - `alias` auto-generation follows rules for standard ports.

### [x] 2.2 Justification: Alias auto-generation for non-standard port
- Scenario: [`positive-2.2-alias-auto-generation-non-standard-port.md`](management-api/upstreams/positive-2.2-alias-auto-generation-non-standard-port.md)
- Why it matters:
  - Prevent collisions and ambiguous routing.
- What to check:
  - Endpoint `host=api.example.com, port=8443` yields alias `api.example.com:8443` (or requires explicit alias, per implementation).

### [x] 2.3 Justification: Explicit alias required for IP-based endpoints
- Scenario: [`negative-2.3-explicit-alias-required-ip-based-endpoints.md`](management-api/upstreams/negative-2.3-explicit-alias-required-ip-based-endpoints.md)
- Why it matters:
  - Avoid unstable aliasing and SSRF ambiguity.
- What to check:
  - Create upstream with `host=10.0.0.1` and no alias is rejected (`400` `PD`).

### [x] 2.4 Justification: Multi-endpoint upstream pool compatibility rules
- Scenario: [`negative-2.4-multi-endpoint-upstream-pool-compatibility-rules.md`](management-api/upstreams/negative-2.4-multi-endpoint-upstream-pool-compatibility-rules.md)
- Why it matters:
  - Load balancing must not mix incompatible protocols/schemes/ports.
- What to check:
  - Creating pool with mismatched `scheme` fails (`400`).
  - Creating pool with mismatched `port` fails (`400`).
  - Creating pool with mismatched `protocol` fails (`400`).

### [x] 2.5 Justification: Update upstream
- Scenario: [`positive-2.5-update-upstream.md`](management-api/upstreams/positive-2.5-update-upstream.md)
- Why it matters:
  - Real systems change endpoints/auth/rate limits.
- What to check:
  - `PUT /upstreams/{id}` updates mutable fields.
  - Versioning/immutability rules are respected (plugins immutable, but upstream references change).

### [x] 2.6 Justification: Disable upstream blocks proxy traffic (including descendants)
- Scenario: [`negative-2.6-disable-upstream-blocks-proxy-traffic.md`](management-api/upstreams/negative-2.6-disable-upstream-blocks-proxy-traffic.md)
- Why it matters:
  - Emergency stop for compromised upstream.
- What to check:
  - With `enabled=false`, proxy requests return a gateway error (`503` `PD`, `ESrc=gateway`).
  - If upstream disabled in ancestor, descendant sees it disabled too.

### [x] 2.7 Justification: Delete upstream cascades routes
- Scenario: [`positive-2.7-delete-upstream-cascades-routes.md`](management-api/upstreams/positive-2.7-delete-upstream-cascades-routes.md)
- Why it matters:
  - Cleanup must not leave dangling config.
- What to check:
  - Deleting upstream removes dependent routes (or rejects delete with a clear error if cascade not implemented).
  - Subsequent proxy requests return `404 UPSTREAM_NOT_FOUND` or `ROUTE_NOT_FOUND`.

### [x] 2.8 Justification: Alias uniqueness enforced per tenant
- Scenario: [`negative-2.8-alias-uniqueness-enforced-per-tenant.md`](management-api/upstreams/negative-2.8-alias-uniqueness-enforced-per-tenant.md)
- Why it matters:
  - Prevents ambiguous routing.
- What to check:
  - Creating two upstreams with same `alias` in same tenant is rejected (conflict).
  - Creating same `alias` in different tenants succeeds.

### [x] 2.9 Justification: Tags support discovery and filtering
- Scenario: [`positive-2.9-tags-support-discovery-filtering.md`](management-api/upstreams/positive-2.9-tags-support-discovery-filtering.md)
- Why it matters:
  - Tenants need to find reusable upstreams (e.g., `openai`, `llm`).
- What to check:
  - Create upstream with `tags`.
  - Listing/filtering returns expected upstreams (implementation-specific query parameter; if only OData `$filter` exists, use that).

### [x] 2.10 Justification: Multi-endpoint load balancing distributes requests
- Scenario: [`positive-2.10-multi-endpoint-load-balancing-distributes-requests.md`](management-api/upstreams/positive-2.10-multi-endpoint-load-balancing-distributes-requests.md)
- Why it matters:
  - HA and capacity.
- What to check:
  - Over N requests, endpoints are selected round-robin (or documented strategy).
  - Endpoint selection is stable across keep-alive connections (or explicitly not).

### [x] 2.11 Justification: Re-enable upstream restores proxy traffic
- Scenario: [`positive-2.11-re-enable-upstream-restores-proxy-traffic.md`](management-api/upstreams/positive-2.11-re-enable-upstream-restores-proxy-traffic.md)
- Why it matters:
  - Maintenance windows should be reversible.
- What to check:
  - Upstream with `enabled=false` blocks traffic.
  - `PUT /upstreams/{id}` with `enabled=true` restores traffic.
  - Subsequent proxy requests succeed.

### [x] 2.12 Justification: List upstreams includes disabled resources
- Scenario: [`positive-2.12-list-upstreams-includes-disabled-resources.md`](management-api/upstreams/positive-2.12-list-upstreams-includes-disabled-resources.md)
- Why it matters:
  - Operators need visibility into all config, not just active.
- What to check:
  - `GET /upstreams` returns both enabled and disabled upstreams.
  - Response includes `enabled` field for each resource.
  - Optional: `$filter=enabled eq true` can filter to active only.

---

## 3) Management API: route lifecycle + matching fields

### [x] 3.1 Justification: Create HTTP route with method + path
- Scenario: [`positive-3.1-create-http-route-method-path.md`](management-api/routes/positive-3.1-create-http-route-method-path.md)
- Why it matters:
  - Core routing feature.
- What to check:
  - `POST /routes` returns `201`.
  - Route is associated to the target upstream.

### [x] 3.2 Justification: Method allowlist enforcement
- Scenario: [`negative-3.2-method-allowlist-enforcement.md`](management-api/routes/negative-3.2-method-allowlist-enforcement.md)
- Why it matters:
  - Prevents accidental exposure of unsafe methods.
- What to check:
  - Route allows `POST`; invoking `GET` returns `404 ROUTE_NOT_FOUND` or `400` (per spec), as a gateway error.

### [x] 3.3 Justification: Query allowlist enforcement
- Scenario: [`negative-3.3-query-allowlist-enforcement.md`](management-api/routes/negative-3.3-query-allowlist-enforcement.md)
- Why it matters:
  - Prevents parameter smuggling and uncontrolled upstream behavior.
- What to check:
  - Allowed query param passes through.
  - Unknown query param is rejected (`400` `PD`, `ESrc=gateway`).

### [x] 3.4 Justification: Path suffix mode = disabled
- Scenario: [`negative-3.4-path-suffix-mode-disabled.md`](management-api/routes/negative-3.4-path-suffix-mode-disabled.md)
- Why it matters:
  - Ensures strict routing for sensitive endpoints.
- What to check:
  - Supplying any suffix returns a gateway validation error.

### [x] 3.5 Justification: Path suffix mode = append
- Scenario: [`positive-3.5-path-suffix-mode-append.md`](management-api/routes/positive-3.5-path-suffix-mode-append.md)
- Why it matters:
  - Enables prefix routes for REST resources.
- What to check:
  - Suffix is appended to outbound path.
  - Outbound path is normalized (no `//` surprises if suffix begins with `/`).

### [x] 3.6 Justification: Route priority resolves ambiguities
- Scenario: [`positive-3.6-route-priority-resolves-ambiguities.md`](management-api/routes/positive-3.6-route-priority-resolves-ambiguities.md)
- Why it matters:
  - Multiple routes may match; deterministic selection is required.
- What to check:
  - With two candidate routes, higher `priority` wins.

### [x] 3.7 Justification: Disable route blocks proxy traffic
- Scenario: [`negative-3.7-disable-route-blocks-proxy-traffic.md`](management-api/routes/negative-3.7-disable-route-blocks-proxy-traffic.md)
- Why it matters:
  - Granular shutdown.
- What to check:
  - With `route.enabled=false`, request returns `404 ROUTE_NOT_FOUND` or `503` (implementation-defined), `ESrc=gateway`.

### [x] 3.8 Justification: Create gRPC route by service+method
- Scenario: [`positive-3.8-create-grpc-route-service-method.md`](management-api/routes/positive-3.8-create-grpc-route-service-method.md)
- Why it matters:
  - gRPC routing is not path-based at config level.
- What to check:
  - `match.grpc.service` + `match.grpc.method` routes to HTTP/2 `:path` `/Service/Method`.
  - Wrong service/method yields `404 ROUTE_NOT_FOUND` (gateway).

### [x] 3.9 Justification: Re-enable route restores proxy traffic
- Scenario: [`positive-3.9-re-enable-route-restores-proxy-traffic.md`](management-api/routes/positive-3.9-re-enable-route-restores-proxy-traffic.md)
- Why it matters:
  - Route-level maintenance should be reversible.
- What to check:
  - Route with `enabled=false` is skipped in matching.
  - `PUT /routes/{id}` with `enabled=true` restores matching.
  - Subsequent proxy requests succeed.

### [x] 3.10 Justification: List routes includes disabled resources
- Scenario: [`positive-3.10-list-routes-includes-disabled-resources.md`](management-api/routes/positive-3.10-list-routes-includes-disabled-resources.md)
- Why it matters:
  - Operators need visibility into all route config.
- What to check:
  - `GET /routes` returns both enabled and disabled routes.
  - Response includes `enabled` field for each resource.

## 4) Management API: plugin lifecycle

### [x] 4.1 Justification: Create custom Starlark guard plugin
- Scenario: [`positive-4.1-create-custom-starlark-guard-plugin.md`](management-api/plugins/positive-4.1-create-custom-starlark-guard-plugin.md)
- Why it matters:
  - Tenant extensibility.
- What to check:
  - `POST /plugins` returns `201`.
  - Returned plugin is addressable by anonymous GTS id `...~{uuid}`.
  - `GET /plugins/{id}/source` returns `200 text/plain`.

### [x] 4.2 Justification: Plugin immutability (no update)
- Scenario: [`negative-4.2-plugin-immutability.md`](management-api/plugins/negative-4.2-plugin-immutability.md)
- Why it matters:
  - Ensures reproducibility and auditability.
- What to check:
  - `PUT /plugins/{id}` is not available (404/405).

### [x] 4.3 Justification: Delete plugin succeeds only when unreferenced
- Scenario: [`positive-4.3-delete-plugin-succeeds-only-unreferenced.md`](management-api/plugins/positive-4.3-delete-plugin-succeeds-only-unreferenced.md)
- Why it matters:
  - Prevents breaking active upstream/route configs.
- What to check:
  - Referenced plugin deletion returns `409` `PD` with `type` = `...plugin.in_use...`.
  - Unlinked plugin deletion returns `204`.

### [x] 4.4 Justification: Plugin type enforcement
- Scenario: [`negative-4.4-plugin-type-enforcement.md`](management-api/plugins/negative-4.4-plugin-type-enforcement.md)
- Why it matters:
  - Prevents attaching a guard where auth is expected.
- What to check:
  - Attempt to attach `plugin.guard` as upstream auth fails at config validation (`400`).

### [x] 4.5 Justification: Plugin resolution supports builtin named ids and custom UUID ids
- Scenario: [`positive-4.5-plugin-resolution-supports-builtin-named-ids-custom-uuid.md`](management-api/plugins/positive-4.5-plugin-resolution-supports-builtin-named-ids-custom-uuid.md)
- Why it matters:
  - Mix builtin and custom plugins in the same chain.
- What to check:
  - Attaching builtin plugin id resolves to a deterministic UUID and requires a seeded `oagw_plugin` row (seeded on startup/migration).
  - Attaching UUID plugin id (custom plugin) requires a DB row; missing DB row yields `503 PLUGIN_NOT_FOUND` (`PD`, `ESrc=gateway`).

### [x] 4.6 Justification: Plugin sharing modes merge correctly across tenant hierarchy
- Scenario: [`positive-4.6-plugin-sharing-modes-merge-correctly-across-tenant-hierarchy.md`](management-api/plugins/positive-4.6-plugin-sharing-modes-merge-correctly-across-tenant-hierarchy.md)
- Why it matters:
  - Partners share guard/transform baselines; customers extend them.
- What to check:
  - With parent `plugins.sharing=inherit` and child specifies plugins:
    - Effective chain is `parent + child`.
  - With parent `plugins.sharing=enforce`:
    - Child cannot remove or replace parent plugins (only append if allowed by policy).
  - With parent `plugins.sharing=private`:
    - Child does not see parent plugins.

### [x] 4.7 Justification: Plugin usage tracking and GC eligibility behavior
- Scenario: [`positive-4.7-plugin-usage-tracking-gc-eligibility-behavior.md`](management-api/plugins/positive-4.7-plugin-usage-tracking-gc-eligibility-behavior.md)
- Why it matters:
  - Prevents plugin table growth and supports safe cleanup.
- What to check:
  - Link plugin to an upstream/route and invoke proxy:
    - `last_used_at` updated.
    - `gc_eligible_at` cleared.
  - Unlink plugin from all references:
    - `gc_eligible_at` set to `now + TTL`.
  - After GC job runs (or via test hook): plugin deleted only when `gc_eligible_at < now`.

---

## 5) Proxy API: inbound authZ + tenant visibility

### [x] 5.1 Justification: Proxy invoke permission required
- Scenario: [`negative-5.1-proxy-invoke-permission-required.md`](proxy-api/authz/negative-5.1-proxy-invoke-permission-required.md)
- Why it matters:
  - Prevents unauthorized outbound traffic.
- What to check:
  - Missing `gts.x.core.oagw.proxy.v1~:invoke` returns `403`.

### [x] 5.2 Justification: Proxy cannot access upstream not owned or shared
- Scenario: [`negative-5.2-proxy-cannot-access-upstream-not-owned-shared.md`](proxy-api/authz/negative-5.2-proxy-cannot-access-upstream-not-owned-shared.md)
- Why it matters:
  - Tenant isolation.
- What to check:
  - Tenant A token invoking Tenant B private upstream returns `403` or `404` (must not leak existence).

### [x] 5.3 Justification: Upstream sharing via ancestor hierarchy
- Scenario: [`positive-5.3-upstream-sharing-ancestor-hierarchy.md`](proxy-api/authz/positive-5.3-upstream-sharing-ancestor-hierarchy.md)
- Why it matters:
  - Partner → customer model.
- What to check:
  - Ancestor upstream visible to descendant when sharing mode permits.
  - Descendant cannot see ancestor `private` configuration.

---

## 6) Proxy API: alias resolution

### [x] 6.1 Justification: Alias resolves by walking tenant hierarchy (shadowing)
- Scenario: [`positive-6.1-alias-resolves-walking-tenant-hierarchy.md`](proxy-api/alias-resolution/positive-6.1-alias-resolves-walking-tenant-hierarchy.md)
- Why it matters:
  - Override is a first-class multi-tenant capability.
- What to check:
  - Child upstream with same alias overrides parent for routing.

### [x] 6.2 Justification: Enforced ancestor limits still apply when alias shadowed
- Scenario: [`negative-6.2-enforced-ancestor-limits-still-apply-alias-shadowed.md`](proxy-api/alias-resolution/negative-6.2-enforced-ancestor-limits-still-apply-alias-shadowed.md)
- Why it matters:
  - Prevents bypassing enforced policies by shadowing.
- What to check:
  - Parent `rate_limit.sharing=enforce` still constrains child effective limit.

### [x] 6.3 Justification: Multi-endpoint common-suffix alias requires `X-OAGW-Target-Host` header
- Scenario: [`negative-6.3-multi-endpoint-common-suffix-alias-requires-host-header.md`](proxy-api/alias-resolution/negative-6.3-multi-endpoint-common-suffix-alias-requires-host-header.md)
- **Note**: This scenario filename uses legacy "host-header" terminology. The header is `X-OAGW-Target-Host`. See section 6.5 for comprehensive custom header routing scenarios.
- Why it matters:
  - Avoid ambiguous endpoint selection and SSRF-by-host-header tricks.
- What to check:
  - For alias `vendor.com` backed by endpoints `us.vendor.com`, `eu.vendor.com`:
    - Missing `X-OAGW-Target-Host` header yields `400` error, `ESrc=gateway`.
    - With valid `X-OAGW-Target-Host` header routes to that endpoint.
    - With `X-OAGW-Target-Host` header value not in pool rejects.

### [x] 6.4 Justification: Alias not found returns stable 404
- Scenario: [`negative-6.4-alias-not-found-returns-stable-404.md`](proxy-api/alias-resolution/negative-6.4-alias-not-found-returns-stable-404.md)
- Why it matters:
  - Client can distinguish config vs upstream failure.
- What to check:
  - Unknown alias returns `404` `PD`, `ESrc=gateway`, `type` = `...upstream.not_found...` (or `UPSTREAM_NOT_FOUND`).

---

## 6.5) Proxy API: Custom Header Routing (X-OAGW-Target-Host)

### Positive Scenarios

### [x] 6.5.1 Justification: Single-endpoint upstream without X-OAGW-Target-Host header
- Scenario: [`positive-1.1-single-endpoint-no-header.md`](proxy-api/custom-header-routing/positive-1.1-single-endpoint-no-header.md)
- Why it matters:
  - Ensures backward compatibility for most common use case.
- What to check:
  - Single-endpoint upstreams route successfully without the custom header.
  - Behavior unchanged from current implementation.

### [x] 6.5.2 Justification: Single-endpoint upstream with valid X-OAGW-Target-Host header
- Scenario: [`positive-1.2-single-endpoint-with-header.md`](proxy-api/custom-header-routing/positive-1.2-single-endpoint-with-header.md)
- Why it matters:
  - Header is validated but optional for single-endpoint upstreams.
- What to check:
  - Header value must match the single endpoint.
  - Header is stripped before forwarding to upstream.

### [x] 6.5.3 Justification: Multi-endpoint explicit alias without header uses round-robin
- Scenario: [`positive-2.1-multi-endpoint-explicit-alias-no-header.md`](proxy-api/custom-header-routing/positive-2.1-multi-endpoint-explicit-alias-no-header.md)
- Why it matters:
  - Preserves load balancing for explicit aliases.
- What to check:
  - Requests distribute across endpoints via round-robin.
  - No header required for explicit alias.

### [x] 6.5.4 Justification: Multi-endpoint explicit alias with header bypasses load balancing
- Scenario: [`positive-2.2-multi-endpoint-explicit-alias-with-header.md`](proxy-api/custom-header-routing/positive-2.2-multi-endpoint-explicit-alias-with-header.md)
- Why it matters:
  - Allows explicit endpoint selection for debugging/testing.
- What to check:
  - Header value selects specific endpoint.
  - Round-robin is bypassed when header present.

### [x] 6.5.5 Justification: Multi-endpoint common suffix alias with header succeeds
- Scenario: [`positive-3.1-multi-endpoint-common-suffix-with-header.md`](proxy-api/custom-header-routing/positive-3.1-multi-endpoint-common-suffix-with-header.md)
- Why it matters:
  - Core use case for custom header routing.
- What to check:
  - Header disambiguates endpoints with common suffix.
  - Request routes to specified endpoint.

### [x] 6.5.6 Justification: Case-insensitive X-OAGW-Target-Host matching
- Scenario: [`positive-3.2-case-insensitive-matching.md`](proxy-api/custom-header-routing/positive-3.2-case-insensitive-matching.md)
- Why it matters:
  - DNS is case-insensitive; header matching should follow DNS standards.
- What to check:
  - Mixed case header values match endpoints.
  - Comparison follows DNS conventions.

### [x] 6.5.7 Justification: X-OAGW-Target-Host bypasses round-robin load balancing
- Scenario: [`positive-3.3-load-balancing-bypass.md`](proxy-api/custom-header-routing/positive-3.3-load-balancing-bypass.md)
- Why it matters:
  - Explicit routing for operational needs (debugging, region-specific routing).
- What to check:
  - Header consistently routes to same endpoint.
  - Load balancing state preserved for requests without header.

### Negative Scenarios

### [x] 6.5.8 Justification: Missing X-OAGW-Target-Host for common suffix alias returns 400
- Scenario: [`negative-1.1-missing-header-common-suffix.md`](proxy-api/custom-header-routing/negative-1.1-missing-header-common-suffix.md)
- Why it matters:
  - Prevents ambiguous routing and enforces explicit endpoint selection.
- What to check:
  - Missing header for common suffix alias returns `400` `PD`.
  - Error includes list of valid endpoint hosts.
  - Error type: `gts.x.core.errors.err.v1~x.oagw.routing.missing_target_host.v1`.

### [x] 6.5.9 Justification: Invalid X-OAGW-Target-Host format with port returns 400
- Scenario: [`negative-1.2-invalid-format-with-port.md`](proxy-api/custom-header-routing/negative-1.2-invalid-format-with-port.md)
- Why it matters:
  - Port is defined in upstream configuration, not header.
- What to check:
  - Header with port number rejected with `400` `PD`.
  - Error type: `gts.x.core.errors.err.v1~x.oagw.routing.invalid_target_host.v1`.

### [x] 6.5.10 Justification: Invalid X-OAGW-Target-Host format with path returns 400
- Scenario: [`negative-1.3-invalid-format-with-path.md`](proxy-api/custom-header-routing/negative-1.3-invalid-format-with-path.md)
- Why it matters:
  - Path components are not part of host specification.
- What to check:
  - Header with path component rejected with `400` `PD`.
  - Error type: `gts.x.core.errors.err.v1~x.oagw.routing.invalid_target_host.v1`.

### [x] 6.5.11 Justification: Invalid X-OAGW-Target-Host format with special chars returns 400
- Scenario: [`negative-1.4-invalid-format-special-chars.md`](proxy-api/custom-header-routing/negative-1.4-invalid-format-special-chars.md)
- Why it matters:
  - Prevents injection attacks and malformed routing.
- What to check:
  - Header with query params or special characters rejected with `400` `PD`.
  - Error type: `gts.x.core.errors.err.v1~x.oagw.routing.invalid_target_host.v1`.

### [x] 6.5.12 Justification: Unknown X-OAGW-Target-Host not in endpoint list returns 400
- Scenario: [`negative-2.1-unknown-host.md`](proxy-api/custom-header-routing/negative-2.1-unknown-host.md)
- Why it matters:
  - Allowlist validation prevents routing to arbitrary servers (SSRF protection).
- What to check:
  - Header value not matching any endpoint rejected with `400` `PD`.
  - Error includes list of valid hosts.
  - Error type: `gts.x.core.errors.err.v1~x.oagw.routing.unknown_target_host.v1`.

### [x] 6.5.13 Justification: IP address when hostname expected returns 400
- Scenario: [`negative-2.2-ip-address-when-hostname-expected.md`](proxy-api/custom-header-routing/negative-2.2-ip-address-when-hostname-expected.md)
- Why it matters:
  - Type mismatch (IP vs hostname) treated as unknown host.
- What to check:
  - IP address when endpoints use hostnames rejected with `400` `PD`.
  - Error type: `gts.x.core.errors.err.v1~x.oagw.routing.unknown_target_host.v1`.

---

## 7) Proxy API: request transformation invariants

### [x] 7.1 Justification: Inbound → outbound path + query mapping for HTTP
- Scenario: [`positive-7.1-inbound-outbound-path-query-mapping-http.md`](proxy-api/request-transforms/positive-7.1-inbound-outbound-path-query-mapping-http.md)
- Why it matters:
  - Correctness and security of upstream requests.
- What to check:
  - Outbound path equals `route.match.http.path` (+ suffix if enabled).
  - Only allowlisted query params are forwarded.

### [x] 7.2 Justification: Hop-by-hop headers stripped
- Scenario: [`positive-7.2-hop-hop-headers-stripped.md`](proxy-api/request-transforms/positive-7.2-hop-hop-headers-stripped.md)
- Why it matters:
  - Prevents connection manipulation and request smuggling.
- What to check:
  - Inbound `Connection`, `Upgrade`, `Transfer-Encoding`, `TE`, etc do not reach upstream unless explicitly allowed by protocol handler.

### [x] 7.3 Justification: `Host` header replaced by upstream host
- Scenario: [`positive-7.3-host-header-replaced-upstream-host.md`](proxy-api/request-transforms/positive-7.3-host-header-replaced-upstream-host.md)
- Why it matters:
  - Prevents host header injection.
- What to check:
  - Upstream sees correct `Host`.

### [x] 7.4 Justification: Well-known header validation errors are `400`
- Scenario: [`negative-7.4-well-known-header-validation-errors-400.md`](proxy-api/request-transforms/negative-7.4-well-known-header-validation-errors-400.md)
- Why it matters:
  - Fail fast on malformed requests.
- What to check:
  - Invalid `Content-Length` returns `400` `PD`.
  - `Content-Length` mismatch with actual body returns `400` `PD`.

### [x] 7.5 Justification: Upstream `headers` config applies simple transformations
- Scenario: [`positive-7.5-upstream-headers-config-applies-simple-transformations.md`](proxy-api/request-transforms/positive-7.5-upstream-headers-config-applies-simple-transformations.md)
- Why it matters:
  - Common use case without writing plugins.
- What to check:
  - `upstream.headers.request.set` adds/overwrites headers.
  - Header removal rules (if supported) apply.
  - Invalid header names/values are rejected with `400` `PD`.

### [x] 7.6 Justification: Request correlation headers propagate end-to-end
- Scenario: [`positive-7.6-request-correlation-headers-propagate-end-end.md`](proxy-api/request-transforms/positive-7.6-request-correlation-headers-propagate-end-end.md)
- Why it matters:
  - Debuggability across systems.
- What to check:
  - If client sends `X-Request-ID`, upstream receives it and response includes it.
  - If client does not send `X-Request-ID`, gateway generates one (if implemented) and uses it consistently in logs.

---

## 8) Body validation (core)

### [x] 8.1 Justification: Maximum body size limit enforced (100MB)
- Scenario: [`negative-8.1-maximum-body-size-limit-enforced.md`](proxy-api/body-validation/negative-8.1-maximum-body-size-limit-enforced.md)
- Why it matters:
  - Prevents memory exhaustion.
- What to check:
  - Body > 100MB rejected early with `413` `PD`, `ESrc=gateway`.

### [x] 8.2 Justification: Transfer-Encoding support limited to `chunked`
- Scenario: [`negative-8.2-transfer-encoding-support-limited-chunked.md`](proxy-api/body-validation/negative-8.2-transfer-encoding-support-limited-chunked.md)
- Why it matters:
  - Avoid unsupported encodings and ambiguous parsing.
- What to check:
  - Unsupported transfer-encoding rejected with `400` `PD`.

### [x] 8.3 Justification: Streaming request bodies are not buffered (where supported)
- Scenario: [`positive-8.3-streaming-request-bodies-not-buffered.md`](proxy-api/body-validation/positive-8.3-streaming-request-bodies-not-buffered.md)
- Why it matters:
  - Large uploads should not OOM gateway.
- What to check:
  - For endpoints supporting streaming body, memory does not grow with body size (assert via metrics/observability hooks if available).

---

## 9) Outbound authentication plugins (OAGW → upstream)

### [x] 9.1 Justification: `noop` auth plugin forwards without credential injection
- Scenario: [`positive-9.1-noop-auth-plugin-forwards-credential-injection.md`](proxy-api/authentication/positive-9.1-noop-auth-plugin-forwards-credential-injection.md)
- Why it matters:
  - Public upstreams.
- What to check:
  - No auth headers are added.

### [x] 9.2 Justification: API key injection (header + optional prefix)
- Scenario: [`positive-9.2-api-key-injection.md`](proxy-api/authentication/positive-9.2-api-key-injection.md)
- Why it matters:
  - Common vendor auth.
- What to check:
  - Header name exactness (case-insensitive match, but sent as configured).
  - Prefix handling (`"Bearer "` etc).
  - Secret value not logged.

### [x] 9.3 Justification: Basic auth injection
- Scenario: [`positive-9.3-basic-auth-injection.md`](proxy-api/authentication/positive-9.3-basic-auth-injection.md)
- Why it matters:
  - Legacy systems.
- What to check:
  - Correct `Authorization: Basic ...` formatting.

### [x] 9.4 Justification: Bearer token passthrough/injection
- Scenario: [`positive-9.4-bearer-token-passthrough-injection.md`](proxy-api/authentication/positive-9.4-bearer-token-passthrough-injection.md)
- Why it matters:
  - Static bearer tokens and service tokens.
- What to check:
  - `Authorization: Bearer ...` set from secret.

### [x] 9.5 Justification: OAuth2 client credentials (body-based)
- Scenario: [`positive-9.5-oauth2-client-credentials.md`](proxy-api/authentication/positive-9.5-oauth2-client-credentials.md)
- Why it matters:
  - Standard machine-to-machine auth.
- What to check:
  - Token is fetched via OAuth2 flow.
  - Token cached.
  - On upstream `401`, plugin refreshes token.
  - Plugin does not retry the original request beyond auth refresh policy (per design note).

### [x] 9.6 Justification: OAuth2 client credentials (basic-auth variant)
- Scenario: [`positive-9.6-oauth2-client-credentials.md`](proxy-api/authentication/positive-9.6-oauth2-client-credentials.md)
- Why it matters:
  - Some token endpoints require `client_id/client_secret` via basic auth.
- What to check:
  - Token request uses correct client authentication.

### [x] 9.7 Justification: Secret access control via `cred_store`
- Scenario: [`negative-9.7-secret-access-control-cred-store.md`](proxy-api/authentication/negative-9.7-secret-access-control-cred-store.md)
- Why it matters:
  - Prevent secret exfiltration across tenants.
- What to check:
  - If `cred_store` denies access, proxy returns `401 AuthenticationFailed` (`PD`, `ESrc=gateway`).
  - If secret missing, proxy returns `500 SecretNotFound` (`PD`).

### [x] 9.8 Justification: Hierarchical auth sharing modes behave as specified
- Scenario: [`positive-9.8-hierarchical-auth-sharing-modes-behave-specified.md`](proxy-api/authentication/positive-9.8-hierarchical-auth-sharing-modes-behave-specified.md)
- Why it matters:
  - Partners may share auth, or enforce a corporate credential.
- What to check:
  - Parent `auth.sharing=inherit`, child provides auth:
    - Effective auth is child override (only if child has override permission).
  - Parent `auth.sharing=inherit`, child does not provide auth:
    - Effective auth is parent.
  - Parent `auth.sharing=enforce`:
    - Child cannot override auth.
  - Parent `auth.sharing=private`:
    - Child must provide its own auth.

### [x] 9.9 Justification: Descendant override permissions are enforced
- Scenario: [`negative-9.9-descendant-override-permissions-enforced.md`](proxy-api/authentication/negative-9.9-descendant-override-permissions-enforced.md)
- Why it matters:
  - Prevents unauthorized weakening of shared configs.
- What to check:
  - Without `oagw:upstream:override_auth`, child cannot override inherited auth even when `sharing=inherit`.
  - Without `oagw:upstream:override_rate`, child cannot set a custom rate limit.
  - Without `oagw:upstream:add_plugins`, child cannot append plugins.

---

## 10) Guard plugins (policy enforcement)

### [x] 10.1 Justification: Timeout guard plugin enforces request timeout
- Scenario: [`negative-10.1-timeout-guard-plugin-enforces-request-timeout.md`](plugins/guards/negative-10.1-timeout-guard-plugin-enforces-request-timeout.md)
- Why it matters:
  - Prevents hung upstream calls.
- What to check:
  - Request exceeding timeout returns `504` gateway timeout (`PD`, `ESrc=gateway`).

### [x] 10.2 Justification: Built-in CORS handling (preflight)
- Scenario: [`positive-10.2-built-cors-handling.md`](plugins/guards/positive-10.2-built-cors-handling.md)
- Why it matters:
  - Browser clients.
- What to check:
  - OPTIONS preflight handled locally (`204` with correct `Access-Control-*`).
  - Preflight bypasses upstream call.
  - Origin/method/header not allowed yields `403` `PD`.

### [x] 10.3 Justification: CORS credentials + wildcard is rejected by config validation
- Scenario: [`negative-10.3-cors-credentials-wildcard-rejected-config-validation.md`](plugins/guards/negative-10.3-cors-credentials-wildcard-rejected-config-validation.md)
- Why it matters:
  - Prevents insecure misconfiguration.
- What to check:
  - `allow_credentials=true` + `allowed_origins=['*']` rejected (`400`), or preflight rejected.

### [x] 10.4 Justification: Custom Starlark guard rejects based on headers/body
- Scenario: [`negative-10.4-custom-starlark-guard-rejects-based-headers-body.md`](plugins/guards/negative-10.4-custom-starlark-guard-rejects-based-headers-body.md)
- Why it matters:
  - Tenant-specific compliance.
- What to check:
  - Missing required header rejected with plugin-defined status/code.
  - Body-too-large rejected with `413`.
  - Ensure guard only runs `on_request`.

---

## 11) Transform plugins (request/response/error mutation)

### [x] 11.1 Justification: Request path rewrite transform
- Scenario: [`positive-11.1-request-path-rewrite-transform.md`](plugins/transforms/positive-11.1-request-path-rewrite-transform.md)
- Why it matters:
  - Versioning and upstream compatibility.
- What to check:
  - Outbound `path` is rewritten.
  - Transform emits audit-safe logs (no secrets).

### [x] 11.2 Justification: Query mutation transform
- Scenario: [`positive-11.2-query-mutation-transform.md`](plugins/transforms/positive-11.2-query-mutation-transform.md)
- Why it matters:
  - Inject upstream-required parameters.
- What to check:
  - New query param added.
  - Internal query params removed.
  - Query allowlist rules remain enforced before transform (or explicitly document order via test).

### [x] 11.3 Justification: Header mutation transform
- Scenario: [`positive-11.3-header-mutation-transform.md`](plugins/transforms/positive-11.3-header-mutation-transform.md)
- Why it matters:
  - Vendor headers, tracing, feature flags.
- What to check:
  - `headers.set/add/remove` changes reflected upstream.
  - Hop-by-hop headers remain stripped even if plugin tries to set them (define expected behavior and lock it with test).

### [x] 11.4 Justification: Response JSON redaction transform
- Scenario: [`positive-11.4-response-json-redaction-transform.md`](plugins/transforms/positive-11.4-response-json-redaction-transform.md)
- Why it matters:
  - Prevents PII leakage.
- What to check:
  - Target fields replaced with placeholder.
  - Non-JSON response triggers defined behavior (reject or no-op).

### [x] 11.5 Justification: `on_error` transform handles gateway errors
- Scenario: [`positive-11.5-error-transform-handles-gateway-errors.md`](plugins/transforms/positive-11.5-error-transform-handles-gateway-errors.md)
- Why it matters:
  - Standardize tenant error formats.
- What to check:
  - For a gateway error (e.g., rate limit), `on_error` can rewrite `title/detail` and status if allowed.

### [x] 11.6 Justification: Plugin ordering and layering (upstream before route)
- Scenario: [`positive-11.6-plugin-ordering-layering.md`](plugins/transforms/positive-11.6-plugin-ordering-layering.md)
- Why it matters:
  - Predictable composition.
- What to check:
  - Upstream plugins run before route plugins.
  - Auth runs before guards, before transforms.

### [x] 11.7 Justification: Plugin control flow (`next`, `reject`, `respond`)
- Scenario: [`positive-11.7-plugin-control-flow.md`](plugins/transforms/positive-11.7-plugin-control-flow.md)
- Why it matters:
  - Plugins must be able to short-circuit.
- What to check:
  - `ctx.reject` stops chain and returns gateway error.
  - `ctx.respond` returns custom success response without calling upstream.

### [x] 11.8 Justification: Starlark sandbox restrictions
- Scenario: [`negative-11.8-starlark-sandbox-restrictions.md`](plugins/transforms/negative-11.8-starlark-sandbox-restrictions.md)
- Why it matters:
  - Prevents plugin-based SSRF/file reads.
- What to check:
  - Network I/O attempt fails.
  - File I/O attempt fails.
  - Infinite loop times out.
  - Large allocation is blocked.

---

## 12) Protocol coverage: HTTP (plain)

### [x] 12.1 Justification: Plain HTTP request/response passthrough
- Scenario: [`positive-12.1-plain-http-request-response-passthrough.md`](protocols/http/positive-12.1-plain-http-request-response-passthrough.md)
- Why it matters:
  - Baseline behavior.
- What to check:
  - Status, headers, body forwarded.
  - Gateway adds rate-limit headers if enabled.

### [x] 12.2 Justification: Upstream error passthrough with `ESrc=upstream`
- Scenario: [`negative-12.2-upstream-error-passthrough-esrc-upstream.md`](protocols/http/negative-12.2-upstream-error-passthrough-esrc-upstream.md)
- Why it matters:
  - Clients need to distinguish who produced the failure.
- What to check:
  - Upstream returns `500` with JSON body.
  - OAGW response keeps body intact and sets `ESrc=upstream`.

### [x] 12.3 Justification: Gateway error uses `PD` + `ESrc=gateway`
- Scenario: [`negative-12.3-gateway-error-uses-pd-esrc-gateway.md`](protocols/http/negative-12.3-gateway-error-uses-pd-esrc-gateway.md)
- Why it matters:
  - Consistent client handling.
- What to check:
  - Induce gateway error (route not found / validation error).
  - Response is `PD`, `Content-Type=application/problem+json`, `ESrc=gateway`.

### [x] 12.4 Justification: HTTP version negotiation (HTTP/2 attempt + fallback)
- Scenario: [`positive-12.4-http-version-negotiation.md`](protocols/http/positive-12.4-http-version-negotiation.md)
- Why it matters:
  - Correctness and performance across mixed upstreams.
- What to check:
  - First call attempts HTTP/2; on failure falls back to HTTP/1.1.
  - Subsequent calls use cached decision (TTL behavior).

### [x] 12.6 Justification: No automatic retries for upstream failures
- Scenario: [`negative-12.6-no-automatic-retries-upstream-failures.md`](protocols/http/negative-12.6-no-automatic-retries-upstream-failures.md)
- Why it matters:
  - Prevents duplicate side effects on non-idempotent calls.
- What to check:
  - Upstream returns transient `5xx` or connection close:
    - OAGW returns error once.
    - Upstream observes a single request attempt (requires controllable upstream).

### [x] 12.7 Justification: Scheme/protocol mismatches fail explicitly
- Scenario: [`negative-12.7-scheme-protocol-mismatches-fail-explicitly.md`](protocols/http/negative-12.7-scheme-protocol-mismatches-fail-explicitly.md)
- Why it matters:
  - Prevents silent downgrade or undefined behavior.
- What to check:
  - Upstream `protocol=http` with endpoint `scheme=grpc` rejected at config validation.
  - Upstream `protocol=grpc` with endpoint `scheme=https` rejected (or yields `502 ProtocolError` on invoke; lock expected behavior).

---

## 13) Protocol coverage: SSE (HTTP streaming)

### [x] 13.1 Justification: SSE stream is forwarded without buffering
- Scenario: [`positive-13.1-sse-stream-forwarded-buffering.md`](protocols/sse/positive-13.1-sse-stream-forwarded-buffering.md)
- Why it matters:
  - Streaming correctness and memory bounds.
- What to check:
  - Response `Content-Type: text/event-stream`.
  - Events arrive incrementally.
  - Rate limit headers present in initial headers.

### [x] 13.2 Justification: Client disconnect aborts upstream stream
- Scenario: [`positive-13.2-client-disconnect-aborts-upstream-stream.md`](protocols/sse/positive-13.2-client-disconnect-aborts-upstream-stream.md)
- Why it matters:
  - Resource cleanup.
- What to check:
  - Disconnect client mid-stream.
  - Upstream stream is closed.
  - Gateway may emit `StreamAborted` internally; ensure no leaked in-flight metrics.

---

## 14) Protocol coverage: WebSocket (wss)

### [x] 14.1 Justification: WebSocket upgrade is proxied
- Scenario: [`positive-14.1-websocket-upgrade-proxied.md`](protocols/websocket/positive-14.1-websocket-upgrade-proxied.md)
- Why it matters:
  - Real-time APIs.
- What to check:
  - `101 Switching Protocols` handshake forwarded.
  - Required WS headers forwarded/validated.

### [x] 14.2 Justification: Auth injected during handshake (not per-message)
- Scenario: [`positive-14.2-auth-injected-during-handshake.md`](protocols/websocket/positive-14.2-auth-injected-during-handshake.md)
- Why it matters:
  - Security model consistency.
- What to check:
  - Upstream sees auth header on upgrade request.
  - Subsequent WS frames are forwarded unchanged.

### [x] 14.3 Justification: Rate limit applies to connection establishment
- Scenario: [`negative-14.3-rate-limit-applies-connection-establishment.md`](protocols/websocket/negative-14.3-rate-limit-applies-connection-establishment.md)
- Why it matters:
  - Avoid per-message accounting surprises.
- What to check:
  - Exceeding rate limit rejects upgrade with `429` (`PD`, `ESrc=gateway`).

### [x] 14.4 Justification: WS connection idle timeout enforced
- Scenario: [`negative-14.4-ws-connection-idle-timeout-enforced.md`](protocols/websocket/negative-14.4-ws-connection-idle-timeout-enforced.md)
- Why it matters:
  - Prevents stale connections consuming resources.
- What to check:
  - Idle connection closed after configured timeout.

---

## 15) Protocol coverage: gRPC

### [x] 15.1 Justification: gRPC unary request proxied (native)
- Scenario: [`positive-15.1-grpc-unary-request-proxied.md`](protocols/grpc/positive-15.1-grpc-unary-request-proxied.md)
- Why it matters:
  - Service-to-service API support.
- What to check:
  - `content-type: application/grpc*` detection routes to gRPC handler.
  - Metadata headers preserved.

### [x] 15.2 Justification: gRPC server streaming proxied
- Scenario: [`positive-15.2-grpc-server-streaming-proxied.md`](protocols/grpc/positive-15.2-grpc-server-streaming-proxied.md)
- Why it matters:
  - Common for list APIs.
- What to check:
  - Stream forwarded without buffering.
  - Backpressure respects HTTP/2 flow control.

### [x] 15.3 Justification: gRPC JSON transcoding for HTTP clients
- Scenario: [`positive-15.3-grpc-json-transcoding-http-clients.md`](protocols/grpc/positive-15.3-grpc-json-transcoding-http-clients.md)
- Why it matters:
  - Enables REST clients to call gRPC services.
- What to check:
  - HTTP JSON request converted to gRPC protobuf.
  - Server streaming results returned as `application/x-ndjson`.

### [x] 15.4 Justification: gRPC status mapping and error source
- Scenario: [`negative-15.4-grpc-status-mapping-error-source.md`](protocols/grpc/negative-15.4-grpc-status-mapping-error-source.md)
- Why it matters:
  - Clients need consistent failure semantics.
- What to check:
  - Upstream gRPC `RESOURCE_EXHAUSTED` maps to rate limit error (status + `type`).
  - Upstream gRPC failures are marked `ESrc=upstream` when passed through.

---

## 17) Protocol coverage: WebTransport (`wt`) (future-facing)

### [x] 17.1 Justification: WT session establishment + auth
- Scenario: [`positive-17.1-wt-session-establishment-auth.md`](protocols/webtransport/positive-17.1-wt-session-establishment-auth.md)
- Why it matters:
  - QUIC-based real-time transports are explicitly in schema.
- What to check:
  - Upstream scheme `wt` accepted.
  - Failure mode clearly documented in behavior tests if feature is not implemented yet (e.g., `502 ProtocolError`, `ESrc=gateway`).

---

## 18) Rate limiting

### [x] 18.1 Justification: Token bucket sustained + burst
- Scenario: [`positive-18.1-token-bucket-sustained-burst.md`](rate-limiting/positive-18.1-token-bucket-sustained-burst.md)
- Why it matters:
  - Default algorithm.
- What to check:
  - Burst allows short spike.
  - Sustained rate enforced.
  - `429` includes `Retry-After` and `X-RateLimit-*` headers.

### [x] 18.1.1 Justification: Rate limit response headers can be disabled
- Scenario: [`positive-18.1.1-rate-limit-response-headers-can-be-disabled.md`](rate-limiting/positive-18.1.1-rate-limit-response-headers-can-be-disabled.md)
- Why it matters:
  - Some clients don’t want header overhead; operators may reduce leakage.
- What to check:
  - With `rate_limit.response_headers=false`, success responses omit `X-RateLimit-*`.
  - Error responses still include `Retry-After`.

### [x] 18.2 Justification: Sliding window strictness
- Scenario: [`negative-18.2-sliding-window-strictness.md`](rate-limiting/negative-18.2-sliding-window-strictness.md)
- Why it matters:
  - Prevents boundary bursts.
- What to check:
  - Requests across a window boundary do not allow 2x burst.

### [x] 18.3 Justification: Rate limit scope variants
- Scenario: [`positive-18.3-rate-limit-scope-variants.md`](rate-limiting/positive-18.3-rate-limit-scope-variants.md)
- Why it matters:
  - Operators need different fairness models.
- What to check (one scenario each):
  - `scope=global`.
  - `scope=tenant`.
  - `scope=user`.
  - `scope=ip`.
  - `scope=route`.

### [x] 18.4 Justification: Weighted cost per route
- Scenario: [`positive-18.4-weighted-cost-per-route.md`](rate-limiting/positive-18.4-weighted-cost-per-route.md)
- Why it matters:
  - Expensive endpoints should consume more budget.
- What to check:
  - Route with `cost=10` consumes 10 tokens.

### [x] 18.5 Justification: Strategy variants when limit exceeded
- Scenario: [`negative-18.5-strategy-variants-limit-exceeded.md`](rate-limiting/negative-18.5-strategy-variants-limit-exceeded.md)
- Why it matters:
  - Operators choose UX vs strictness.
- What to check:
  - `strategy=reject` returns `429`.
  - `strategy=queue` delays then succeeds or times out with `503 queue.timeout`.
  - `strategy=degrade` (if present) uses configured fallback behavior.

### [x] 18.6 Justification: Hierarchical min() merge for descendant overrides
- Scenario: [`positive-18.6-hierarchical-min-merge-descendant-overrides.md`](rate-limiting/positive-18.6-hierarchical-min-merge-descendant-overrides.md)
- Why it matters:
  - Parent caps child.
- What to check:
  - Parent `enforce 1000/min`, child `500/min` => effective `500/min`.

### [x] 18.7 Justification: Budget modes (allocated/shared/unlimited) behave as specified
- Scenario: [`positive-18.7-budget-modes-behave-specified.md`](rate-limiting/positive-18.7-budget-modes-behave-specified.md)
- Why it matters:
  - Partner/tenant quota allocation is a core multi-tenant requirement.
- What to check:
  - `budget.mode=allocated` rejects creation/override when sum(child) exceeds `total * overcommit_ratio`.
  - `budget.mode=shared` shares a single pool between tenants.
  - `budget.mode=unlimited` does not enforce budget validation.

---

## 19) Concurrency limiting + backpressure queueing (future-facing)

### [x] 19.1 Justification: Concurrency limit rejects when max in-flight reached
- Why it matters:
  - Protect upstream and gateway.
- What to check:
  - When `max_concurrent` exceeded, return `503 concurrency_limit.exceeded` with `Retry-After`.

### [x] 19.2 Justification: Queue strategy buffers requests up to max depth
- Why it matters:
  - Smooths spikes.
- What to check:
  - FIFO order.
  - `queue.full` returned when depth exceeded.
  - `queue.timeout` returned when wait exceeds config.
  - Memory limit enforcement for queued requests.

### [x] 19.3 Justification: Streaming requests hold permits until completion
- Why it matters:
  - Prevents infinite concurrency leakage via streams.
- What to check:
  - SSE and WS hold concurrency permit until closed.

---

## 20) Circuit breaker (future-facing)

### [x] 20.1 Justification: Circuit opens after consecutive failures
- Why it matters:
  - Prevents cascading failures.
- What to check:
  - After threshold, requests fail fast with `503 circuit_breaker.open` + `Retry-After`.

### [x] 20.2 Justification: Half-open probing closes circuit on recovery
- Why it matters:
  - Self-healing.
- What to check:
  - After open timeout, limited probes allowed.
  - Success threshold closes circuit.

### [x] 20.3 Justification: Per-endpoint circuit scope
- Why it matters:
  - Multi-endpoint pool should not be entirely blocked by one bad node.
- What to check:
  - With `scope=per_endpoint`, failures isolate to single endpoint.

---

## 21) CORS (built-in)

### [x] 21.1 Justification: Actual request adds CORS headers on response
- Why it matters:
  - Browser consumption.
- What to check:
  - Response includes `Access-Control-Allow-Origin` (specific origin, not `*` when credentials).
  - Includes `Vary: Origin`.

### [x] 21.2 Justification: Hierarchical merge/union for CORS allowed origins
- Why it matters:
  - Partner adds baseline origins, customer adds own.
- What to check:
  - `inherit` merges by union.
  - `enforce` forbids child adding origins.

---

## 22) Observability: metrics endpoint

### [x] 22.1 Justification: Metrics endpoint is auth-protected
- Why it matters:
  - Metrics can leak topology.
- What to check:
  - Unauthenticated `GET /metrics` is rejected.
  - Non-admin rejected.

### [x] 22.2 Justification: Request metrics increment on success
- Why it matters:
  - SLIs/alerting.
- What to check:
  - `oagw_requests_total` increments with correct labels (host, path, method, status_class).
  - Histogram `phase=total/upstream/plugins` recorded.

### [x] 22.3 Justification: Errors metrics increment on gateway errors
- Why it matters:
  - Detect regressions.
- What to check:
  - Induce gateway error; `oagw_errors_total{error_type=...}` increments.

### [x] 22.4 Justification: Cardinality rules (no tenant labels)
- Why it matters:
  - Prevents Prometheus overload.
- What to check:
  - Metrics output does not include `tenant_id` label.
  - Path label uses configured route path, not dynamic suffix.

### [x] 22.5 Justification: Status metrics use status class grouping
- Why it matters:
  - Prevents high-cardinality status labels.
- What to check:
  - Metrics export uses `status_class=2xx/3xx/4xx/5xx` labels.
  - Per-status-code labels are absent (or explicitly documented).

---

## 23) Observability: audit logging

### [x] 23.1 Justification: Proxy requests produce structured audit log
- Why it matters:
  - Forensics and compliance.
- What to check:
  - Log includes request_id, tenant_id, principal_id, host, path, status, duration.

### [x] 23.2 Justification: Sensitive data not logged
- Why it matters:
  - Prevents secret/PII leaks.
- What to check:
  - No request bodies.
  - No query params.
  - No auth headers / secret values.

### [x] 23.3 Justification: Config change operations are logged
- Why it matters:
  - Admin audit trail.
- What to check:
  - Upstream create/update/delete yields log entry.
  - Route create/update/delete yields log entry.
  - Plugin create/delete yields log entry.

---

## 24) Error handling: gateway vs upstream

### [x] 24.1 Justification: Gateway errors always use RFC 9457 PD
- Why it matters:
  - Stable client parsing.
- What to check:
  - For each gateway error class (400/401/403/404/409/413/429/5xx), body is `PD`.

### [x] 24.2 Justification: `ESrc` header is set for both gateway and upstream failures
- Why it matters:
  - Client retry policy depends on source.
- What to check:
  - Gateway error includes `ESrc=gateway`.
  - Upstream error includes `ESrc=upstream`.

### [x] 24.3 Justification: Retry-After is present for retriable gateway errors
- Why it matters:
  - Client backoff guidance.
- What to check:
  - `429`, `503 link unavailable`, `503 circuit breaker open`, `504` timeouts include `Retry-After` (where specified).

### [x] 24.4 Justification: Stream aborted is classified distinctly
- Why it matters:
  - Streaming clients need different handling.
- What to check:
  - Abort SSE/WS mid-flight; classify as `StreamAborted`.

### [x] 24.5 Justification: Every documented error type has at least one reproducer
- Why it matters:
  - Prevents untested error paths.
- What to check:
  - `ValidationError` (bad query param / bad header)
  - `RouteNotFound` (no matching route)
  - `AuthenticationFailed` (bad/missing secret or outbound auth failure)
  - `PayloadTooLarge` (body limit)
  - `RateLimitExceeded` (429)
  - `SecretNotFound` (missing secret ref)
  - `ProtocolError` (protocol mismatch / invalid upgrade)
  - `DownstreamError` (upstream TLS failure / connect error mapped)
  - `StreamAborted` (client disconnect)
  - `LinkUnavailable` (upstream unavailable)
  - `CircuitBreakerOpen` (open circuit)
  - `ConnectionTimeout` (connect timeout)
  - `RequestTimeout` (overall timeout)
  - `IdleTimeout` (idle connection timeout)
  - `PluginNotFound` (missing plugin reference)
  - `PluginInUse` (delete referenced plugin)

---

## 25) REST list endpoints: OData `$filter/$orderby/$top/$skip/$select`

### [x] 25.1 Justification: `$select` projects fields for upstream list
- Why it matters:
  - Reduce payload.
- What to check:
  - `GET /upstreams?$select=id,alias` returns only those fields.

### [x] 25.2 Justification: `$select` validation
- Why it matters:
  - Prevent abuse.
- What to check:
  - Too long `$select` returns `400` `PD`.
  - Too many fields returns `400` `PD`.
  - Duplicate fields returns `400` `PD`.

### [x] 25.3 Justification: `$filter` on alias
- Why it matters:
  - Discoverability.
- What to check:
  - `GET /upstreams?$filter=alias eq 'api.openai.com'` returns only matches.

### [x] 25.4 Justification: Pagination (`$top/$skip`) stable ordering
- Why it matters:
  - UI and automation.
- What to check:
  - Paging produces consistent sets.

### [x] 25.5 Justification: `$select` works for routes and plugins lists
- Why it matters:
  - Large configs should be listable efficiently.
- What to check:
  - `GET /routes?$select=id,upstream_id,match` returns only those fields.
  - `GET /plugins?$select=id,plugin_type,name` returns only those fields.

### [x] 25.6 Justification: `$orderby` is applied and validated
- Why it matters:
  - Stable results for paging and UIs.
- What to check:
  - `created_at desc` ordering changes item order.
  - Invalid `$orderby` yields `400` `PD`.

### [x] 25.7 Justification: `$top` max and `$skip` bounds enforced
- Why it matters:
  - Prevent abuse and runaway responses.
- What to check:
  - `$top` above max clamps or rejects (lock expected behavior).
  - Negative `$skip` rejected.

---

## 26) Security-focused proxy scenarios

### [x] 26.1 Justification: SSRF prevention by strict upstream host selection
- Why it matters:
  - Gateway must not be a generic open proxy.
- What to check:
  - Client cannot override upstream destination via inbound `Host` (except the explicit common-suffix selection mode).
  - Absolute-form URLs in request line are rejected or normalized.

### [x] 26.2 Justification: Header injection protections
- Why it matters:
  - Prevent response/request splitting.
- What to check:
  - Newline characters in header values rejected.

### [x] 26.3 Justification: Protocol mismatch errors are explicit
- Why it matters:
  - Prevent silent downgrades.
- What to check:
  - Using HTTP route against `protocol=grpc` upstream fails with `502 ProtocolError` (`PD`).

---

## 27) Cross-surface consistency checks

### [x] 27.1 Justification: IDs are anonymous GTS identifiers on API surface
- Why it matters:
  - Stable API contract.
- What to check:
  - `GET /upstreams/{id}` accepts `gts.x.core.oagw.upstream.v1~{uuid}`.
  - Same for routes/plugins.

### [x] 27.2 Justification: Examples in `modules/system/oagw/examples/*.md` remain valid
- Scenarios:
  - [`positive-example-04-grpc-unary-proxy.md`](protocols/grpc/examples/positive-example-04-grpc-unary-proxy.md)
  - [`positive-example-01-http-request-response.md`](protocols/http/examples/positive-example-01-http-request-response.md)
  - [`positive-example-02-sse-streaming.md`](protocols/sse/examples/positive-example-02-sse-streaming.md)
  - [`positive-example-03-websocket-upgrade.md`](protocols/websocket/examples/positive-example-03-websocket-upgrade.md)
- Why it matters:
  - Documentation drift should be detected.
- What to check:
- Execute the flows described in:
    - `protocols/http/examples/positive-example-01-http-request-response.md`
    - `protocols/sse/examples/positive-example-02-sse-streaming.md`
    - `protocols/websocket/examples/positive-example-03-websocket-upgrade.md`
    - `protocols/grpc/examples/positive-example-04-grpc-unary-proxy.md`
  - If proxy path format differs from design (`/proxy/{alias}/*`), lock behavior via explicit failing test + updated expectation.

---

## 28) Minimum scenario matrix (coverage checklist)

For E2E coverage, ensure at least one scenario exists for each cell:

### 28.1 Protocol × scheme
- HTTP:
  - `https`
- Streaming:
  - SSE over `https`
  - WebSocket over `wss`
  - WebTransport over `wt` (feature or explicit-not-supported behavior)
- gRPC:
  - `grpc` (and HTTP/2 multiplexing on shared ingress port)

### 28.2 Outbound auth plugin × protocol
- `noop`: HTTP
- `apikey`: HTTP, WebSocket, SSE, gRPC
- `basic`: HTTP
- `bearer`: HTTP
- `oauth2.client_cred`: HTTP
- `oauth2.client_cred_basic`: HTTP

### 28.3 Plugin chain phases
- Guard-only (`on_request`)
- Transform (`on_request`)
- Transform (`on_response`)
- Transform (`on_error`)

### 28.4 Error source
- Gateway-generated (validation/rate-limit/etc)
- Upstream passthrough (4xx/5xx)
