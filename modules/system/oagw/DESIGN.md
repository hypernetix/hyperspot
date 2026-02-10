# Technical Design — Outbound API Gateway (OAGW)

<!-- TOC START -->
## Table of Contents

- [Context](#context)
- [Architecture](#architecture)
  - [Key Concepts](#key-concepts)
  - [Out of Scope](#out-of-scope)
  - [Security Considerations](#security-considerations)
- [Core Subsystems](#core-subsystems)
  - [Request Routing](#request-routing)
    - [Routing Flow](#routing-flow)
    - [Route Matching Algorithm](#route-matching-algorithm)
    - [Headers Transformation](#headers-transformation)
    - [Guard Rules](#guard-rules)
    - [Body Validation Rules](#body-validation-rules)
    - [Transformation Rules](#transformation-rules)
  - [Alias Resolution](#alias-resolution)
    - [Alias Generation Rules](#alias-generation-rules)
    - [Resolution Algorithm](#resolution-algorithm)
    - [Shadowing Behavior](#shadowing-behavior)
    - [Alias Uniqueness](#alias-uniqueness)
    - [Multi-Endpoint Load Balancing](#multi-endpoint-load-balancing)
  - [Plugin System](#plugin-system)
    - [Plugin Types](#plugin-types)
    - [Plugin Identification](#plugin-identification)
    - [Plugin Lifecycle Management](#plugin-lifecycle-management)
    - [Plugin Layering](#plugin-layering)
    - [Plugin Execution Order](#plugin-execution-order)
    - [Starlark Context API](#starlark-context-api)
    - [Sandbox Restrictions](#sandbox-restrictions)
- [Hierarchical Configuration](#hierarchical-configuration)
  - [Configuration Sharing Modes](#configuration-sharing-modes)
  - [Shareable Configuration Fields](#shareable-configuration-fields)
  - [Merge Strategies](#merge-strategies)
  - [Configuration Resolution Algorithm](#configuration-resolution-algorithm)
  - [Example: Partner Shares OpenAI Upstream with Customer](#example-partner-shares-openai-upstream-with-customer)
  - [Secret Access Control](#secret-access-control)
  - [Permissions and Access Control](#permissions-and-access-control)
  - [Schema Updates](#schema-updates)
- [Type System](#type-system)
  - [Upstream](#upstream)
  - [Route](#route)
  - [Auth Plugin](#auth-plugin)
  - [Guard Plugin](#guard-plugin)
  - [Transform Plugin](#transform-plugin)
- [REST API](#rest-api)
  - [Error Response Format](#error-response-format)
  - [Error Source Distinction](#error-source-distinction)
  - [Management API](#management-api)
    - [Upstream Endpoints](#upstream-endpoints)
    - [Route Endpoints](#route-endpoints)
    - [Plugin Endpoints](#plugin-endpoints)
  - [Proxy API](#proxy-api)
    - [Proxy Endpoint](#proxy-endpoint)
    - [API Call Examples](#api-call-examples)
- [Database Persistence](#database-persistence)
  - [Data Model](#data-model)
    - [Resource Identification Pattern](#resource-identification-pattern)
    - [Entity Relationship](#entity-relationship)
    - [Upstream Table](#upstream-table)
    - [Route Table](#route-table)
    - [Plugin Table](#plugin-table)
  - [Common Queries](#common-queries)
    - [Find Upstream by Alias (with tenant hierarchy and enabled inheritance)](#find-upstream-by-alias-with-tenant-hierarchy-and-enabled-inheritance)
    - [List Upstreams for Tenant (with shadowing and enabled inheritance)](#list-upstreams-for-tenant-with-shadowing-and-enabled-inheritance)
    - [Find Matching Route for Request](#find-matching-route-for-request)
    - [Resolve Effective Configuration](#resolve-effective-configuration)
    - [List Routes by Upstream](#list-routes-by-upstream)
    - [Track Plugin Usage](#track-plugin-usage)
    - [Delete Garbage-Collected Plugins](#delete-garbage-collected-plugins)
- [Metrics and Observability](#metrics-and-observability)
  - [Core Metrics](#core-metrics)
  - [Cardinality Management](#cardinality-management)
  - [Histogram Buckets](#histogram-buckets)
  - [Metrics Endpoint](#metrics-endpoint)
- [Audit Logging](#audit-logging)
  - [Log Format](#log-format)
  - [What is Logged](#what-is-logged)
  - [What is NOT Logged](#what-is-not-logged)
  - [Log Levels](#log-levels)
- [Error Handling](#error-handling)
  - [Error Types](#error-types)
- [Review](#review)
- [Future Developments](#future-developments)

<!-- TOC END -->

## 1. Architecture Overview

### 1.1 Architectural Vision

OAGW provides a centralized proxy layer for all outbound API requests from CyberFabric to external services. The architecture emphasizes security (SSRF protection, credential isolation), hierarchical multi-tenancy (configuration inheritance with shadowing), and extensibility (plugin system for auth, validation, transformation).

Key architectural decisions: plugin-based modularity for credential injection and request mutation, alias-based upstream resolution with tenant hierarchy search, immutable plugin definitions for deterministic behavior, and configuration layering (upstream → route → tenant) for flexible policy enforcement.

### 1.2 Architecture Drivers

#### Functional Drivers

| Requirement | Design Response |
|-------------|-----------------|
| `fdd-oagw-fr-proxy-endpoint-v1` | RESTful proxy API with alias-based routing |
| `fdd-oagw-fr-alias-resolution-v1` | Tenant hierarchy search algorithm for upstream resolution |
| `fdd-oagw-fr-plugin-types-v1` | Plugin system with Auth, Guard, Transform types |
| `fdd-oagw-fr-config-merge-v1` | Configuration layering: Upstream < Route < Tenant |

#### NFR Allocation

| NFR ID | NFR Summary | Allocated To | Design Response | Verification Approach |
|--------|-------------|--------------|-----------------|----------------------|
| `fdd-oagw-nfr-latency-v1` | <10ms gateway overhead (p95) | Connection pooling, minimal plugin execution | HTTP/2 connection reuse, plugin timeout 100ms | Benchmark tests with Prometheus metrics |
| `fdd-oagw-nfr-availability-v1` | 99.9% uptime, circuit breaker | Circuit breaker per upstream | Trip at 50% error rate, auto-recovery after 30s | Chaos testing, availability monitoring |
| `fdd-oagw-nfr-ssrf-protection-v1` | DNS validation, IP pinning | Request routing layer | Allowlist/denylist validation, header stripping | Security scan, penetration testing |
| `fdd-oagw-nfr-credential-isolation-v1` | Zero credential exposure | Credential store integration | UUID-only references, no logging | Audit log validation, secret scanning |
| `fdd-oagw-nfr-observability-v1` | Request logging, metrics | Logging middleware, Prometheus | Correlation ID per request, histogram metrics | Log analysis, Grafana dashboards |

### 1.3 Service Dependencies

| Dependency       | Purpose                                     |
|------------------|---------------------------------------------|
| `type-registry` | GTS schema/instance registration            |
| `cred_store`     | Secret material retrieval by UUID reference |
| `api_ingress`    | REST API hosting                            |
| `modkit-db`      | Database persistence                        |
| `modkit-auth`    | Authorization                               |

## 2. Principles & Constraints

### 2.1 Design Principles

#### Hierarchical Configuration Inheritance

- [ ] `p1` - **ID**: `fdd-oagw-principle-hierarchical-config-v1`

Configuration defined by ancestor tenants can be inherited, overridden, or enforced by descendants based on sharing modes (`private`, `inherit`, `enforce`). This enables policy control at organizational boundaries while allowing tenant-specific customization.

**Rationale**: Multi-tenancy requires balancing centralized policy enforcement with tenant autonomy.

#### Plugin Immutability

- [ ] `p1` - **ID**: `fdd-oagw-principle-plugin-immutability-v1`

Plugin definitions are immutable after creation. Updates require creating a new plugin version and re-binding upstream/route references.

**Rationale**: Immutability guarantees deterministic behavior for attached routes/upstreams, improves auditability, and avoids in-place source mutation risks.

#### Alias-Based Routing

- [ ] `p1` - **ID**: `fdd-oagw-principle-alias-routing-v1`

Upstreams are identified by human-readable alias in proxy URLs, resolved through tenant hierarchy search with descendant shadowing. Alias defaults to hostname for single-endpoint, common domain suffix for multi-endpoint, or explicit for IP-based upstreams.

**Rationale**: Simplifies client integration, enables tenant-specific overrides, and provides intuitive upstream discovery.

#### No Automatic Retries

- [ ] `p2` - **ID**: `fdd-oagw-principle-no-retries-v1`

OAGW performs at most one upstream attempt per inbound request. Retry logic is client responsibility.

**Rationale**: Gateway cannot determine idempotency of operations; client-managed retries prevent duplicate operations and enable request-specific retry strategies.

### 2.2 Constraints

#### Circuit Breaker Core Policy

- [ ] `p1` - **ID**: `fdd-oagw-constraint-circuit-breaker-core-v1`

Circuit breaker is implemented as core gateway resilience capability, not as a plugin.

**Rationale**: Circuit breaker is fundamental failure handling mechanism; plugin overhead would violate latency requirements.

#### HTTP Family Protocol Focus

- [ ] `p1` - **ID**: `fdd-oagw-constraint-http-family-v1`

Main protocol focus is HTTP family traffic (HTTP, SSE, WebSocket, WebTransport). gRPC support planned for later phase (p4).

**Rationale**: HTTP family protocols cover majority of external API integrations; gRPC deferred to reduce initial complexity.

#### No Response Caching

- [ ] `p2` - **ID**: `fdd-oagw-constraint-no-caching-v1`

OAGW does not cache responses. Caching is client or upstream responsibility.

**Rationale**: Gateway layer caching introduces complexity (cache invalidation, multi-tenancy, staleness policies); better handled at application or CDN layer.

#### IP Pinning and DNS Security

- [ ] `p1` - **ID**: `fdd-oagw-constraint-ip-pinning-v1`

DNS resolution uses IP pinning rules and allowed segment matching for SSRF protection.

**Rationale**: Prevents DNS rebinding attacks targeting internal infrastructure.

**ADRs**: See [ADR: SSRF Protection](./docs/adr-ssrf-protection.md)

#### CORS Preflight Local Handling

- [ ] `p1` - **ID**: `fdd-oagw-constraint-cors-local-v1`

CORS preflight OPTIONS requests are handled locally without upstream round-trip, configured per upstream/route.

**Rationale**: Reduces latency and upstream load for CORS preflight checks.

**ADRs**: See [ADR: CORS](./docs/adr-cors.md)

## 3. Technical Architecture

### 3.1 Component Model

#### Request Routing Component

- [ ] `p1` - **ID**: `fdd-oagw-component-routing-v1`

**Responsibility**: Resolves inbound proxy requests to upstream services through alias resolution, route matching, configuration merging, and plugin chain execution.

**Interface**: HTTP endpoints (`{METHOD} /api/oagw/v1/proxy/{alias}/{path}`), internal routing APIs

**Interactions**:
- Alias Resolution → finds upstream by alias from tenant hierarchy
- Route Matching → matches route by (upstream_id, method, path)
- Config Layering → merges upstream/route/tenant configurations
- Plugin Chain → executes Auth → Guards → Transform pipeline

#### Alias Resolution Component

- [ ] `p1` - **ID**: `fdd-oagw-component-alias-resolution-v1`

**Responsibility**: Maps human-readable upstream alias to upstream configuration, supporting tenant hierarchy search and descendant shadowing.

**Interface**: `resolve_upstream_by_alias(tenant_id, alias) → (upstream, enforced_ancestors)`

**Interactions**:
- Searches tenant hierarchy from descendant to root
- Returns closest match (shadowing winner) plus enforced ancestors
- Validates multi-endpoint Host header requirements

#### Plugin System Component

- [ ] `p1` - **ID**: `fdd-oagw-component-plugin-system-v1`

**Responsibility**: Executes modular Auth, Guard, and Transform plugins in deterministic order with Starlark sandbox isolation.

**Interface**: Plugin execution API, Starlark context API for request/response mutation

**Interactions**:
- Auth plugins → credential injection from `cred_store`
- Guard plugins → request validation, CORS checks, timeout enforcement
- Transform plugins → header/body mutation, logging, metrics
- Execution order: Auth → Guards → Transform(request) → Upstream → Transform(response/error)

#### Hierarchical Configuration Component

- [ ] `p1` - **ID**: `fdd-oagw-component-hierarchical-config-v1`

**Responsibility**: Manages configuration inheritance across tenant hierarchy with sharing modes (private, inherit, enforce) and merge strategies.

**Interface**: `resolve_effective_config(tenant_id, route_id, upstream_id) → final_config`

**Interactions**:
- Merges upstream < route < tenant configurations
- Enforces ancestor constraints (rate limits, enforced plugins)
- Resolves auth override permissions
- Computes effective limits: `min(selected, route, all_enforced_ancestors)`

### 3.2 Core Subsystems

The following sections detail implementation algorithms and data flows for key architectural components.

#### Request Routing

**Routing Flow**:

Routing resolves an inbound proxy request to an upstream service through configuration layering and request transformation.

```
       Inbound Request
              ▼
     ┌─────────────────┐
     │ Alias Resolution│ ─── Resolve upstream by alias from URL path
     └────────┬────────┘
              ▼
     ┌─────────────────┐
     │  Route Matching │ ─── Match route by (upstream_id, method, path)
     └────────┬────────┘
              ▼
     ┌─────────────────┐
     │  Config Layer   │ ─── Upstream → Route → Tenant
     └────────┬────────┘
              ▼
     ┌─────────────────┐
     │  Authorization  │ ─── Inbound request authN/Z
     └────────┬────────┘
              ▼
     ┌─────────────────┐
     │  Request Build  │ ─── Transform inbound → outbound request
     └────────┬────────┘
              ▼
     ┌─────────────────┐
     │  Plugin Chain   │ ─── Execute pre/post transformations
     └────────┬────────┘
              ▼
         Upstream Call
```

```
def handle_request(req):
    # 1. Resolve upstream by alias from URL path
    upstream, ok = resolve_upstream_by_alias(req.tenant, req.alias)
    if not ok:
        return Response(status=404)

    # 2. Match route by (upstream_id, method, path)
    route, ok = match_route(upstream.id, req.method, req.path_suffix)
    if not ok:
        return Response(status=404)

    # 3. Check inbound authentication/authorization
    if not authorize_request(req, route):
        return Response(status=403)

    # 4. Get tenant-specific configuration
    tenant_config = get_tenant_config(req.tenant, route.id, upstream.id)

    # 5. Apply configuration layering: Upstream < Route < Tenant
    final_config = merge_configs(
        upstream.config(),
        route.config(),
        tenant_config
    )

    # 6. Build plugin chain based on final configuration
    plugin_chain = build_plugin_chain(final_config.plugins)

    # 7. Prepare outbound request based on final configuration
    outbound_req = prepare_request(req, final_config)

    # 8. Execute plugin chain with outbound request
    return plugin_chain.execute(outbound_req)
```

#### Route Matching Algorithm

**Request Transformation**

How inbound requests map to outbound:

```
    Inbound:  `POST /api/oagw/v1/proxy/api.openai.com/v1/chat/completions/models/gpt-4?version=2`
                                      └───────┬─────┘└────────┬─────────┘└─────┬─────┘└───┬────┘
                                      upstream.alias    rooute.path      path_suffix    query
```

**Route Config**:

- match.http.path: `/v1/chat/completions`
- match.http.path_suffix_mode: `append`
- match.http.query_allowlist: `[version]`

```
    Outbound: POST https://api.openai.com/v1/chat/completions/models/gpt-4?version=2
                          └──────┬──────┘└────────┬─────────┘└─────┬─────┘└───┬────┘
                          upstream.host     route.path      path_suffix   allowed query
```

#### Headers Transformation

Hop-by-hop headers are stripped by default.

| Inbound Header        | Rule                      |
|-----------------------|---------------------------|
| `Host`                | Replaced by upstream host |
| `Connection`          | Stripped                  |
| `Keep-Alive`          | Stripped                  |
| `Proxy-Authenticate`  | Stripped                  |
| `Proxy-Authorization` | Stripped                  |
| `TE`                  | Stripped                  |
| `Trailer`             | Stripped                  |
| `Transfer-Encoding`   | Stripped                  |
| `Upgrade`             | Stripped                  |

Simple header transformations are defined in the upstream `headers` configuration.
Complex header transformations can be defined in corresponding upstream/route plugins.
Well-known headers e.g., `Content-Length`, `Content-Type` must be validated, set or adjusted; invalid headers should result in `400 Bad Request`.

#### Guard Rules

#### Guard Rules

Validation rules that can reject request:

| Inbound      | Rule                                                             |
|--------------|------------------------------------------------------------------|
| Method       | Must be in `match.http.methods`; reject if not allowed           |
| Query params | Validate against `match.http.query_allowlist`; reject if unknown |
| Path suffix  | Reject if `path_suffix_mode`: `disabled` and suffix provided     |
| Body         | See body validation rules below                                  |
| CORS         | Reject if CORS policy validation fails (rules TBD)               |

#### Body Validation Rules

Default validation checks (no configuration required):

| Check             | Rule                                                     | Error                 |
|-------------------|----------------------------------------------------------|-----------------------|
| Content-Length    | Must be valid integer if present; must match actual size | `400 ValidationError` |
| Max size          | Hard limit 100MB; reject before buffering                | `413 PayloadTooLarge` |
| Transfer-Encoding | Reject unsupported encodings (only `chunked` supported)  | `400 ValidationError` |

Additional validation (JSON Schema, content-type checks, custom rules) implemented via guard plugins.

#### Transformation Rules

Rules that mutate inbound → outbound:

| Inbound      | Outbound | Rule                                                                        |
|--------------|----------|-----------------------------------------------------------------------------|
| Method       | Method   | Passthrough                                                                 |
| Path suffix  | Path     | Append to `match.http.path` if `path_suffix_mode`: `append`; plugin mutable |
| Query params | Query    | Passthrough allowed params; plugin mutable                                  |
| Headers      | Headers  | Apply `upstream.headers` transformation rules; plugin mutable               |
| Body         | Body     | Passthrough by default; plugin mutable                                      |

### Alias Resolution

Upstreams are identified by alias in proxy requests: `{METHOD} /api/oagw/v1/proxy/{alias}/{path}`.

#### Alias Generation Rules

| Scenario                            | Generated Alias      | Example                                             |
|-------------------------------------|----------------------|-----------------------------------------------------|
| Single host, standard port          | hostname (no port)   | `api.openai.com:443` → `api.openai.com`             |
| Single host, non-standard port      | hostname:port        | `api.openai.com:8443` → `api.openai.com:8443`       |
| Multiple hosts with common suffix   | common domain suffix | `us.vendor.com`, `eu.vendor.com` → `vendor.com`     |
| IP addresses or heterogeneous hosts | must be explicit     | `10.0.1.1`, `10.0.1.2` → user provides `my-service` |

**Standard ports** (omitted from alias):

- HTTP: 80
- HTTPS: 443
- WebSocket: 80 (ws), 443 (wss)
- WebTransport: 443
- gRPC: 443

**Non-standard ports** (included in alias): Any port not in standard list.

#### Resolution Algorithm

```
def resolve_upstream_by_alias(tenant_id, alias, req):
    # Walk tenant hierarchy from descendant to root
    hierarchy = get_tenant_hierarchy(tenant_id)  # [child, parent, grandparent, root]
    matches = []

    for tid in hierarchy:
        upstream = find_upstream_by_alias(tid, alias)
        if upstream is not None:
            matches.append(upstream)

    if len(matches) == 0:
        return Response(status=404)  # Not found

    # Closest tenant wins for routing target
    selected = matches[0]

    # Multiple endpoints with common suffix alias require Host header
    if len(selected.endpoints) > 1:
        has_common_suffix = any(
            ep.host != alias and ep.host.endswith("." + alias)
            for ep in selected.endpoints
        )
        if has_common_suffix and "Host" not in req.headers:
            return Response(status=400)  # Missing Host header

    # Shadowing does not bypass ancestor sharing="enforce" constraints.
    enforced_ancestors = []
    for ancestor in matches[1:]:
        if has_enforced_constraints(ancestor):
            enforced_ancestors.append(ancestor)

    return ResolvedAlias(
        upstream=selected,
        enforced_ancestors=enforced_ancestors
    )
```

#### Shadowing Behavior

When resolving alias, OAGW walks tenant hierarchy from descendant to root. Closest match wins.

```
Request from: subsub-tenant
Alias: "vendor.com"

Search order:
1. subsub-tenant upstreams  ← wins if found
2. sub-tenant upstreams
3. root-tenant upstreams
```

**Shadowing allows intentional override**: Descendant tenant can create upstream with same alias as ancestor to override behavior (e.g., point to different server, use different
auth).

**Clarification - shadowing does not bypass enforced ancestor policy**:

- Shadowing selects the routing target only.
- Ancestor constraints configured with `sharing: enforce` remain active.
- Effective limits are computed with enforced ancestors included, for example:
  `effective_rate = min(selected_rate, route_rate, all_ancestor_enforced_rates)`.

```
Root:  alias="api.openai.com", rate_limit={sharing:"enforce", rate:10000/min}
Child: alias="api.openai.com", rate_limit={sharing:"private", rate:500/min}  # shadowing winner

Effective for child requests: min(10000, 500) = 500/min
```

#### Alias Uniqueness

**Decision**: Alias is unique **per tenant**, not globally unique.

**Rationale**:

- ✅ **Tenant isolation**: Tenants can independently manage their upstreams without namespace collisions
- ✅ **Hierarchical override**: Descendants can shadow ancestor aliases for controlled customization
- ✅ **Simplicity**: No cross-tenant coordination needed to create upstreams

**Database constraint**: `CONSTRAINT uq_upstream_tenant_alias UNIQUE (tenant_id, alias)`

**Examples**:

*Valid - same alias in different tenants*:

```
Tenant A: alias="api.openai.com" → server: api.openai.com:443
Tenant B: alias="api.openai.com" → server: api.openai.com:443 (independent config)
```

*Valid - child shadows parent*:

```
Parent:  alias="api.example.com" → server: prod.example.com:443
Child:   alias="api.example.com" → server: staging.example.com:443 (override)
```

*Valid - same host, different ports*:

```
Tenant A: alias="api.openai.com"      → server: api.openai.com:443 (standard port)
Tenant A: alias="api.openai.com:8443" → server: api.openai.com:8443 (non-standard port)
```

*Invalid - duplicate alias within same tenant*:

```
Tenant A: alias="my-service" → server: 10.0.1.1:443
Tenant A: alias="my-service" → server: 10.0.1.2:443  ❌ CONFLICT
```

For multi-endpoint load balancing, use single upstream with multiple endpoints (not multiple upstreams with same alias).

#### Multi-Endpoint Load Balancing

Multiple endpoints in same upstream form a pool. Requests are distributed across endpoints (round-robin). All endpoints must have:

- Same `protocol`
- Same `scheme` (https, wss, etc.)
- Same `port`

For detailed alias resolution and compatibility rules, see [ADR: Resource Identification and Discovery](./docs/adr-resource-identification.md).

### Plugin System

#### Plugin Types

- `gts.x.core.oagw.plugin.auth.v1~*` - Authentication plugin for credential injection. Only upstream level. One per upstream.
- `gts.x.core.oagw.plugin.guard.v1~*` - Validation and policy enforcement plugin. Can reject requests. Upstream/Route levels. Multiple per level.
- `gts.x.core.oagw.plugin.transform.v1~*` - Request/response transformation plugin. Upstream/Route levels. Multiple per level.

Plugins can be built-in or custom Starlark scripts.

#### Plugin Identification

All plugins are identified using **anonymous GTS identifiers** in the API, but stored as UUIDs in the database.

**Builtin Plugins** (system-provided):

- API: Named GTS identifier `gts.x.core.oagw.plugin.{type}.v1~x.core.oagw.{name}.v1`
- Hardcoded in Rust, no database storage
- Example: `gts.x.core.oagw.plugin.transform.v1~x.core.oagw.logging.v1`

**Custom Plugins** (tenant-defined Starlark):

- API: Anonymous GTS identifier `gts.x.core.oagw.plugin.{type}.v1~{uuid}`
- Database: UUID only (without GTS prefix)
- Example API: `gts.x.core.oagw.plugin.guard.v1~550e8400-e29b-41d4-a716-446655440000`
- Example DB: `550e8400-e29b-41d4-a716-446655440000`

#### Plugin Lifecycle Management

**Custom Plugins** (Starlark):

- **Immutable**: Plugins cannot be updated after creation
- **Versioning**: Create new plugin for changes, update upstream/route references
- **Deletion**: Only unlinked plugins can be deleted
- **Garbage Collection**: Unlinked plugins are automatically deleted after TTL (default: 30 days)

**Builtin Plugins**:

- **Versioning**: Version in GTS identifier (v1, v2, etc.)
- **Updates**: Deployed with OAGW releases
- **Backward Compatibility**: Old versions remain available

**Plugin Reference in Configuration**:

```json
{
  "upstream": {
    "plugins": {
      "items": [
        "gts.x.core.oagw.plugin.transform.v1~x.core.oagw.logging.v1",
        "gts.x.core.oagw.plugin.guard.v1~550e8400-e29b-41d4-a716-446655440000"
      ]
    }
  }
}
```

**Resolution Algorithm**:

1. Parse GTS identifier to extract instance part (after `~`)
2. If instance is UUID → extract UUID, lookup in `oagw_plugin` table
3. If instance is named (e.g., `x.core.oagw.logging.v1`) → lookup in builtin registry
4. Plugin type in GTS schema must match `plugin_type` in database

#### Plugin Layering

Plugins can be applied at different levels:

- **Upstream Level**: Plugins that apply to all requests sent to a specific upstream service.
- **Route Level**: Plugins that apply to requests for a specific route.

#### Plugin Execution Order

Plugins execute in defined phases during request processing:

1. **Auth plugin** - Credential injection (one per upstream)
2. **Guard plugins** - Validation and policy enforcement (can reject)
3. **Transform plugins (on_request)** - Mutate outbound request
4. **Upstream call** - Forward request to external service
5. **Transform plugins (on_response)** - Mutate response on success
6. **Transform plugins (on_error)** - Mutate error response on failure

Plugin chain composition follows layering: upstream plugins execute before route plugins.

```
  Final Plugin Chain Composition (config-resolution time)

  Upstream.plugins    Route.plugins
  [U1, U2]         +  [R1, R2]    =>  [U1, U2, R1, R2]
```

#### Starlark Context API

```starlark
# ctx.request (on_request phase)
ctx.request.method              # str: "GET", "POST", etc. (read-only)
ctx.request.path                # str: "/v1/chat/completions"
ctx.request.set_path(path)      # Modify outbound path
ctx.request.query               # dict: {"version": "2"}
ctx.request.set_query(dict)     # Replace query parameters
ctx.request.add_query(key, val) # Add/append query parameter
ctx.request.headers             # Headers object
ctx.request.body                # bytes: raw body
ctx.request.json()              # dict: parsed JSON body
ctx.request.set_json(obj)       # Replace body with JSON
ctx.request.tenant_id           # str: authenticated tenant

# ctx.response (on_response phase)
ctx.response.status             # int: HTTP status code
ctx.response.headers            # Headers object
ctx.response.body               # bytes: raw body
ctx.response.json()             # dict: parsed JSON body
ctx.response.set_json(obj)      # Replace body with JSON
ctx.response.set_status(code)   # Override status code

# ctx.error (on_error phase)
ctx.error.status                # int: error status
ctx.error.code                  # str: error code
ctx.error.message               # str: error message
ctx.error.upstream              # bool: true if upstream error

# Headers object
headers.get("Name")             # str | None
headers.set("Name", "value")    # Set/overwrite
headers.add("Name", "value")    # Append (multi-value)
headers.remove("Name")          # Delete
headers.keys()                  # list[str]

# Utilities
ctx.config                      # dict: plugin instance config
ctx.route.id                    # str: route identifier
ctx.log.info(msg, data)         # Logging
ctx.time.elapsed_ms()           # int: ms since request start

# Control flow
ctx.next()                      # Continue to next plugin
ctx.reject(status, code, msg)   # Halt chain, return error
ctx.respond(status, body)       # Halt chain, return custom response
```

#### Sandbox Restrictions

| Feature                | Allowed                   |
|------------------------|---------------------------|
| Network I/O            | ❌                         |
| File I/O               | ❌                         |
| Imports                | ❌                         |
| Infinite loops         | ❌ (timeout enforced)      |
| Large allocations      | ❌ (memory limit enforced) |
| JSON manipulation      | ✅                         |
| String/Math operations | ✅                         |
| Logging (`ctx.log`)    | ✅                         |
| Time (`ctx.time`)      | ✅                         |

## Hierarchical Configuration

OAGW supports multi-tenant hierarchies where ancestor tenants (partners, root) can define upstreams and routes that descendant tenants (customers, leaf tenants) can inherit and
selectively override.

### Configuration Sharing Modes

Each configuration field in an upstream or route can specify a sharing mode that controls visibility and override behavior across the tenant hierarchy:

| Mode      | Behavior                                                        |
|-----------|-----------------------------------------------------------------|
| `private` | Not visible to descendant tenants (default)                     |
| `inherit` | Visible to descendants; descendant can override if specified    |
| `enforce` | Visible to descendants; descendant cannot override (hard limit) |

### Shareable Configuration Fields

The following configuration fields support sharing modes:

- **Auth** (`auth.sharing`): Authentication configuration including credential references
- **Rate Limits** (`rate_limit.sharing`): Rate limiting rules (sustained rate, burst capacity, scope). See [ADR: Rate Limiting](./docs/adr-rate-limiting.md) for algorithm details.
- **CORS** (`cors.sharing`): Cross-origin resource sharing configuration (allowed origins, methods, headers). See [ADR: CORS](./docs/adr-cors.md) for details.
- **Plugins** (`plugins.sharing`): Plugin chains for guards and transforms
- **Tags** (`tags`): Discovery metadata uses additive merge (top-to-bottom union). No `sharing` field; inherited tags cannot be removed by descendants.

### Merge Strategies

When a descendant tenant creates a binding to an ancestor's upstream, configurations merge according to their sharing mode:

**Auth Configuration**:

| Ancestor Sharing | Descendant Specifies | Effective Config                      |
|------------------|----------------------|---------------------------------------|
| `private`        | —                    | Descendant must provide auth          |
| `inherit`        | No                   | Use ancestor's auth                   |
| `inherit`        | Yes                  | Use descendant's auth (override)      |
| `enforce`        | —                    | Use ancestor's auth (cannot override) |

**Rate Limit Configuration**:

| Ancestor Sharing | Descendant Specifies | Effective Limit                        |
|------------------|----------------------|----------------------------------------|
| `private`        | —                    | Descendant's limit only                |
| `inherit`        | No                   | Use ancestor's limit                   |
| `inherit`        | Yes                  | `min(ancestor, descendant)` (stricter) |
| `enforce`        | Any                  | `min(ancestor, descendant)` (stricter) |

When alias shadowing occurs (child and ancestor define same alias), ancestor `sharing: enforce` rate limits are still included in the `min(...)` merge and cannot be bypassed by shadowing.

**Plugins Configuration**:

| Ancestor Sharing | Descendant Specifies | Effective Plugin Chain                  |
|------------------|----------------------|-----------------------------------------|
| `private`        | —                    | Descendant's plugins only               |
| `inherit`        | No                   | Use ancestor's plugins                  |
| `inherit`        | Yes                  | `ancestor.plugins + descendant.plugins` |
| `enforce`        | Any                  | `ancestor.plugins + descendant.plugins` |

**Tags Metadata (Discovery/UI)**:

- Effective tags are merged top-to-bottom with add-only semantics:
  `effective_tags = union(ancestor_tags..., descendant_tags)`.
- Descendant tenants can add local tags but cannot remove inherited tags.
- If create-upstream resolves to an existing upstream definition (binding flow), request tags are treated as local binding additions and do not mutate ancestor tags.
- Tags are metadata only (discovery/UI), not authorization or routing policy inputs.

### Configuration Resolution Algorithm

```
def resolve_effective_config(tenant_id, upstream_id):
    # 1. Walk tenant hierarchy from descendant to root
    hierarchy = get_tenant_hierarchy(tenant_id)  # [child, parent, grandparent, root]

    # 2. Collect bindings for this upstream across hierarchy
    bindings = []
    for tid in hierarchy:
        b = find_binding(tid, upstream_id)
        if b is not None:
            bindings.append(b)

    # 3. Merge from root to child (root is base, child overrides)
    result = EffectiveConfig()
    for i in range(len(bindings) - 1, -1, -1):
        b = bindings[i]
        is_own = (i == 0)

        # Auth - check sharing mode
        if b.auth is not None and b.auth.sharing != "private":
            if is_own and b.auth.secret_ref != "":
                result.auth = b.auth  # descendant overrides
            elif result.auth is None or b.auth.sharing == "enforce":
                result.auth = b.auth  # ancestor's auth applies

        # Rate limit - merge with min() strategy
        result.rate_limit = merge_rate_limit(result.rate_limit, b.rate_limit, is_own)

        # Plugins - concatenate chains
        result.plugins = merge_plugins(result.plugins, b.plugins, is_own)

        # Tags - additive union, no descendant removal of inherited tags
        result.tags = merge_tags(result.tags, b.tags)

    return result


def merge_rate_limit(ancestor, descendant, is_own):
    if ancestor is None:
        return descendant
    if descendant is None:
        if ancestor.sharing == "private" and not is_own:
            return None
        return ancestor

    # Both exist - take stricter (minimum rate)
    if ancestor.sharing == "enforce" or ancestor.sharing == "inherit":
        return RateLimitConfig(
            rate=min(ancestor.rate, descendant.rate),
            window=ancestor.window
        )
    return descendant


def merge_plugins(ancestor, descendant, is_own):
    result = []

    # Add ancestor plugins if shared
    if ancestor is not None and ancestor.sharing != "private":
        result.extend(ancestor.items)

    # Add descendant plugins
    if descendant is not None:
        result.extend(descendant.items)

    return result


def merge_tags(ancestor_tags, descendant_tags):
    # Add-only metadata merge for discovery and UI
    result = set()
    if ancestor_tags is not None:
        result.update(ancestor_tags)
    if descendant_tags is not None:
        result.update(descendant_tags)
    return sorted(result)
```

### Example: Partner Shares OpenAI Upstream with Customer

**Partner Tenant** (ancestor) creates upstream:

```json
{
  "server": {
    "endpoints": [ { "scheme": "https", "host": "api.openai.com", "port": 443 } ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "alias": "api.openai.com",
  "auth": {
    "type": "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.apikey.v1",
    "sharing": "inherit",
    "config": {
      "header": "Authorization",
      "prefix": "Bearer ",
      "secret_ref": "cred://partner-openai-key"
    }
  },
  "rate_limit": {
    "sharing": "enforce",
    "algorithm": "token_bucket",
    "sustained": {
      "rate": 10000,
      "window": "minute"
    },
    "burst": {
      "capacity": 15000
    }
  },
  "plugins": {
    "sharing": "inherit",
    "items": [
      "gts.x.core.oagw.plugin.transform.v1~x.core.oagw.logging.v1"
    ]
  }
}
```

**Customer Tenant** (descendant) creates binding with override:

```json
{
  "server": {
    "endpoints": [ { "scheme": "https", "host": "api.openai.com", "port": 443 } ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "auth": {
    "type": "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.apikey.v1",
    "config": {
      "header": "Authorization",
      "prefix": "Bearer ",
      "secret_ref": "cred://my-own-openai-key"
    }
  },
  "rate_limit": {
    "sustained": {
      "rate": 100,
      "window": "minute"
    }
  },
  "plugins": {
    "items": [
      "gts.x.core.oagw.plugin.transform.v1~x.core.oagw.metrics.v1"
    ]
  }
}
```

**Effective Configuration** for customer tenant:

```json
{
  "auth": {
    "type": "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.apikey.v1",
    "config": {
      "secret_ref": "cred://my-own-openai-key"
    },
    "note": "Customer overrode partner's auth (sharing: inherit)"
  },
  "rate_limit": {
    "sustained": {
      "rate": 100,
      "window": "minute"
    },
    "note": "min(partner.enforce:10000/min, customer:100/min) = 100/min"
  },
  "plugins": {
    "items": [
      "gts.x.core.oagw.plugin.transform.v1~x.core.oagw.logging.v1",
      "gts.x.core.oagw.plugin.transform.v1~x.core.oagw.metrics.v1"
    ],
    "note": "partner.plugins + customer.plugins (sharing: inherit)"
  }
}
```

### Secret Access Control

Auth configuration references secrets via `secret_ref` (e.g., `cred://partner-openai-key`). OAGW does not manage secret sharing - this is handled by `cred_store`.

**Resolution flow**:

1. OAGW resolves `secret_ref` via `cred_store` API
2. `cred_store` checks if secret is accessible to current tenant (own or shared by ancestor)
3. If accessible → return secret material
4. If not → return error, OAGW returns 401 Unauthorized

This means:

- Ancestor can share a secret with descendants via `cred_store` policies
- Descendant references same `secret_ref` - `cred_store` handles access check
- Descendant can also use own secret with different `secret_ref`

### Permissions and Access Control

Descendant's ability to override configurations depends on permissions granted by ancestors:

| Permission                    | Allows Descendant To                       |
|-------------------------------|--------------------------------------------|
| `oagw:upstream:bind`          | Create binding to ancestor's upstream      |
| `oagw:upstream:override_auth` | Override auth config (if sharing: inherit) |
| `oagw:upstream:override_rate` | Specify own rate limits (subject to min()) |
| `oagw:upstream:add_plugins`   | Append own plugins to inherited chain      |

Without appropriate permissions, descendant must use ancestor's configuration as-is (even with `sharing: inherit`).

### Schema Updates

**Upstream Schema** - add sharing fields:

```json
{
  "auth": {
    "type": "object",
    "properties": {
      "type": { "type": "string", "format": "gts-identifier" },
      "sharing": {
        "type": "string",
        "enum": [ "private", "inherit", "enforce" ],
        "default": "private"
      },
      "config": { "type": "object" }
    }
  },
  "rate_limit": {
    "type": "object",
    "properties": {
      "sharing": {
        "type": "string",
        "enum": [ "private", "inherit", "enforce" ],
        "default": "private"
      },
      "rate": { "type": "integer" },
      "window": { "type": "string" }
    }
  },
  "plugins": {
    "type": "object",
    "properties": {
      "sharing": {
        "type": "string",
        "enum": [ "private", "inherit", "enforce" ],
        "default": "private"
      },
      "items": {
        "type": "array",
        "items": { "type": "string", "format": "gts-identifier" }
      }
    }
  }
}
```

**Route Schema** - similar sharing fields for route-level overrides.

For detailed resource identification and binding model, see [ADR: Resource Identification and Discovery](./docs/adr-resource-identification.md).

## Type System

### Upstream

**Base type**: `gts.x.core.oagw.upstream.v1~`

**Schema**:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "OAGW Upstream Service",
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "format": "uuid",
      "readOnly": true,
      "description": "System-generated unique identifier."
    },
    "enabled": {
      "type": "boolean",
      "default": true,
      "description": "Whether this upstream is enabled. When disabled, all requests to this upstream are rejected. If a parent tenant disables an upstream, it is disabled for all descendant tenants."
    },
    "alias": {
      "type": "string",
      "pattern": "^[a-z0-9]([a-z0-9.:-]*[a-z0-9])?$",
      "description": "Human-readable routing identifier. Auto-generated if not specified: single host with standard port (80,443) → hostname; single host with non-standard port → hostname:port; multiple hosts with common suffix → common suffix (e.g., us.vendor.com + eu.vendor.com → vendor.com); IP addresses or heterogeneous hosts → explicit alias required."
    },
    "tags": {
      "type": "array",
      "items": {
        "type": "string",
        "pattern": "^[a-z0-9_-]+$"
      },
      "description": "Flat tags for categorization and discovery (e.g., openai, llm). Effective tags are additive across hierarchy (ancestor + descendant union); descendants can add, not remove inherited tags."
    },
    "server": {
      "type": "object",
      "properties": {
        "endpoints": {
          "type": "array",
          "items": {
            "type": "object",
            "properties": {
              "scheme": {
                "enum": [ "https", "wss", "wt", "grpc" ],
                "type": "string",
                "default": "https"
              },
              "host": {
                "type": "string",
                "format": "hostname",
                "description": "Hostname or IP address of the upstream service."
              },
              "port": {
                "type": "integer",
                "default": 443,
                "minimum": 1,
                "maximum": 65535
              }
            },
            "additionalProperties": false,
            "required": [ "scheme", "host" ]
          }
        }
      }
    },
    "protocol": {
      "type": "string",
      "enum": [
        "gts.x.core.oagw.protocol.v1~x.core.http.v1",
        "gts.x.core.oagw.protocol.v1~x.core.grpc.v1"
      ],
      "format": "gts-identifier",
      "description": "Protocol used to connect to the upstream service."
    },
    "auth": {
      "type": "object",
      "properties": {
        "type": {
          "type": "string",
          "format": "gts-identifier",
          "examples": [
            "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.apikey.v1"
          ],
          "description": "Authentication plugin type for the upstream service."
        },
        "sharing": {
          "type": "string",
          "enum": [ "private", "inherit", "enforce" ],
          "default": "private",
          "description": "Sharing mode for hierarchical configuration. private: not visible to descendants; inherit: descendants can override; enforce: descendants cannot override."
        },
        "config": {
          "type": "object",
          "description": "Authentication plugin configuration."
        }
      }
    },
    "headers": {
      "$ref": "#/definitions/headers",
      "description": "Header transformation rules for requests/responses."
    },
    "plugins": {
      "type": "object",
      "properties": {
        "sharing": {
          "type": "string",
          "enum": [ "private", "inherit", "enforce" ],
          "default": "private",
          "description": "Sharing mode for plugin chain."
        },
        "items": {
          "type": "array",
          "items": {
            "oneOf": [
              {
                "type": "string",
                "format": "gts-identifier",
                "description": "Builtin plugin GTS identifier"
              },
              {
                "type": "string",
                "format": "uuid",
                "description": "Custom plugin UUID"
              }
            ]
          },
          "description": "List of plugins applied to this upstream service. Builtin plugins referenced by GTS ID, custom plugins by UUID."
        }
      }
    },
    "rate_limit": {
      "$ref": "#/definitions/rate_limit",
      "description": "Rate limiting configuration for the upstream."
    }
  },
  "required": [ "server", "protocol" ],
  "definitions": {
    "headers": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "request": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "set": {
              "type": "object",
              "additionalProperties": { "type": "string" },
              "description": "Headers to set (overwrite if exists)."
            },
            "add": {
              "type": "object",
              "additionalProperties": { "type": "string" },
              "description": "Headers to add (append, allow duplicates)."
            },
            "remove": {
              "type": "array",
              "items": { "type": "string" },
              "description": "Header names to remove from inbound request."
            },
            "passthrough": {
              "type": "string",
              "enum": [ "none", "allowlist", "all" ],
              "default": "none",
              "description": "Which inbound headers to forward."
            },
            "passthrough_allowlist": {
              "type": "array",
              "items": { "type": "string" },
              "description": "Headers to forward when passthrough is 'allowlist'."
            }
          }
        },
        "response": {
          "type": "object",
          "additionalProperties": false,
          "properties": {
            "set": {
              "type": "object",
              "additionalProperties": { "type": "string" },
              "description": "Headers to set on response to client."
            },
            "add": {
              "type": "object",
              "additionalProperties": { "type": "string" },
              "description": "Headers to add to response."
            },
            "remove": {
              "type": "array",
              "items": { "type": "string" },
              "description": "Headers to strip from upstream response."
            }
          }
        }
      }
    },
    "rate_limit": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "sharing": {
          "type": "string",
          "enum": [ "private", "inherit", "enforce" ],
          "default": "private",
          "description": "Sharing mode for rate limits. enforce: descendants cannot exceed this limit."
        },
        "algorithm": {
          "type": "string",
          "enum": [ "token_bucket", "sliding_window" ],
          "default": "token_bucket",
          "description": "Rate limiting algorithm. token_bucket allows bursts; sliding_window prevents boundary bursts."
        },
        "sustained": {
          "type": "object",
          "properties": {
            "rate": {
              "type": "integer",
              "minimum": 1,
              "description": "Tokens replenished per window."
            },
            "window": {
              "type": "string",
              "enum": [ "second", "minute", "hour", "day" ],
              "default": "second",
              "description": "Time window for sustained rate."
            }
          },
          "required": [ "rate" ]
        },
        "burst": {
          "type": "object",
          "properties": {
            "capacity": {
              "type": "integer",
              "minimum": 1,
              "description": "Maximum burst size (bucket capacity). Defaults to sustained.rate if not specified."
            }
          }
        },
        "scope": {
          "type": "string",
          "enum": [ "global", "tenant", "user", "ip", "route" ],
          "default": "tenant",
          "description": "Scope for rate limit counters."
        },
        "strategy": {
          "type": "string",
          "enum": [ "reject", "queue", "degrade" ],
          "default": "reject",
          "description": "Behavior when limit exceeded."
        },
        "cost": {
          "type": "integer",
          "minimum": 1,
          "default": 1,
          "description": "Tokens consumed per request. Useful for weighted endpoints."
        }
      },
      "required": [ "sustained" ]
    },
    "cors": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "sharing": {
          "type": "string",
          "enum": [ "private", "inherit", "enforce" ],
          "default": "private",
          "description": "Sharing mode for CORS configuration."
        },
        "enabled": {
          "type": "boolean",
          "default": false,
          "description": "Enable CORS for this upstream."
        },
        "allowed_origins": {
          "type": "array",
          "items": { "type": "string", "format": "uri" },
          "description": "Allowed origins. Use ['*'] for any origin (not recommended with credentials)."
        },
        "allowed_methods": {
          "type": "array",
          "items": { "type": "string", "enum": [ "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS" ] },
          "default": [ "GET", "POST" ],
          "description": "Allowed HTTP methods."
        },
        "allowed_headers": {
          "type": "array",
          "items": { "type": "string" },
          "default": [ "Content-Type", "Authorization" ],
          "description": "Allowed request headers (case-insensitive)."
        },
        "expose_headers": {
          "type": "array",
          "items": { "type": "string" },
          "default": [ ],
          "description": "Headers exposed to browser (beyond CORS-safelisted headers)."
        },
        "max_age": {
          "type": "integer",
          "minimum": 0,
          "maximum": 86400,
          "default": 86400,
          "description": "Preflight cache duration in seconds (max 24h)."
        },
        "allow_credentials": {
          "type": "boolean",
          "default": false,
          "description": "Allow credentials (cookies, auth headers). Requires specific origins (not '*')."
        }
      },
      "required": [ "enabled" ]
    }
  }
}
```

### Route

**Base type**: `gts.x.core.oagw.route.v1~`
Examples:

- `gts.x.core.oagw.route.v1~openai.api.chat.completions.v1`
- `gts.x.core.oagw.route.v1~weather.api.current.v1`

**Schema**:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "OAGW Route",
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "format": "uuid",
      "readOnly": true,
      "description": "System-generated unique identifier."
    },
    "tags": {
      "type": "array",
      "items": {
        "type": "string",
        "pattern": "^[a-z0-9_-]+$"
      },
      "description": "Flat tags for categorization and discovery."
    },
    "upstream_id": {
      "type": "string",
      "format": "uuid",
      "description": "Reference to the upstream service for this route."
    },
    "match": {
      "type": "object",
      "description": "Protocol-scoped inbound matching rules. Exactly one of {http|grpc} must be present.",
      "additionalProperties": false,
      "properties": {
        "http": { "$ref": "#/definitions/http_match" },
        "grpc": { "$ref": "#/definitions/grpc_match" }
      },
      "oneOf": [
        { "required": [ "http" ] },
        { "required": [ "grpc" ] }
      ]
    },
    "plugins": {
      "type": "object",
      "properties": {
        "sharing": {
          "type": "string",
          "enum": [ "private", "inherit", "enforce" ],
          "default": "private",
          "description": "Sharing mode for plugin chain."
        },
        "items": {
          "type": "array",
          "items": {
            "type": "string",
            "format": "gts-identifier"
          },
          "default": [ ],
          "description": "List of plugins applied to this route."
        }
      }
    },
    "rate_limit": {
      "$ref": "#/definitions/rate_limit",
      "description": "Rate limiting configuration for the route."
    }
  },
  "required": [ "upstream_id", "match" ],
  "definitions": {
    "http_match": {
      "type": "object",
      "additionalProperties": false,
      "description": "HTTP match rules (used when the upstream protocol is HTTP).",
      "properties": {
        "methods": {
          "type": "array",
          "minItems": 1,
          "items": {
            "type": "string",
            "enum": [ "GET", "POST", "PUT", "DELETE", "PATCH" ]
          },
          "description": "HTTP methods supported by this route."
        },
        "path": {
          "type": "string",
          "minLength": 1,
          "description": "Path pattern for the route."
        },
        "query_allowlist": {
          "type": "array",
          "items": { "type": "string" },
          "default": [ ],
          "description": "White-listed query parameters. If empty, allow none."
        },
        "path_suffix_mode": {
          "type": "string",
          "enum": [ "disabled", "append" ],
          "default": "append",
          "description": "How to treat /{path_suffix} from the proxy URL. 'disabled' rejects path_suffix usage; 'append' appends it to path."
        }
      },
      "required": [ "methods", "path" ]
    },
    "grpc_match": {
      "type": "object",
      "additionalProperties": false,
      "description": "gRPC match rules (used when the upstream protocol is gRPC).",
      "properties": {
        "service": {
          "type": "string",
          "minLength": 1,
          "description": "Fully qualified gRPC service name (e.g., 'foo.v1.UserService')."
        },
        "method": {
          "type": "string",
          "minLength": 1,
          "description": "RPC method name (e.g., 'GetUser')."
        }
      },
      "required": [ "service", "method" ]
    },
    
    "rate_limit": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "sharing": {
          "type": "string",
          "enum": [ "private", "inherit", "enforce" ],
          "default": "private",
          "description": "Sharing mode for rate limits. enforce: descendants cannot exceed this limit."
        },
        "algorithm": {
          "type": "string",
          "enum": [ "token_bucket", "sliding_window" ],
          "default": "token_bucket",
          "description": "Rate limiting algorithm. token_bucket allows bursts; sliding_window prevents boundary bursts."
        },
        "sustained": {
          "type": "object",
          "properties": {
            "rate": {
              "type": "integer",
              "minimum": 1,
              "description": "Tokens replenished per window."
            },
            "window": {
              "type": "string",
              "enum": [ "second", "minute", "hour", "day" ],
              "default": "second",
              "description": "Time window for sustained rate."
            }
          },
          "required": [ "rate" ]
        },
        "burst": {
          "type": "object",
          "properties": {
            "capacity": {
              "type": "integer",
              "minimum": 1,
              "description": "Maximum burst size (bucket capacity). Defaults to sustained.rate if not specified."
            }
          }
        },
        "scope": {
          "type": "string",
          "enum": [ "global", "tenant", "user", "ip", "route" ],
          "default": "tenant",
          "description": "Scope for rate limit counters."
        },
        "strategy": {
          "type": "string",
          "enum": [ "reject", "queue", "degrade" ],
          "default": "reject",
          "description": "Behavior when limit exceeded."
        },
        "cost": {
          "type": "integer",
          "minimum": 1,
          "default": 1,
          "description": "Tokens consumed per request. Useful for weighted endpoints."
        }
      },
      "required": [ "sustained" ]
    },
    "cors": {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "sharing": {
          "type": "string",
          "enum": [ "private", "inherit", "enforce" ],
          "default": "private",
          "description": "Sharing mode for CORS configuration."
        },
        "enabled": {
          "type": "boolean",
          "default": false,
          "description": "Enable CORS for this route."
        },
        "allowed_origins": {
          "type": "array",
          "items": { "type": "string", "format": "uri" },
          "description": "Allowed origins. Use ['*'] for any origin (not recommended with credentials)."
        },
        "allowed_methods": {
          "type": "array",
          "items": { "type": "string", "enum": [ "GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS" ] },
          "default": [ "GET", "POST" ],
          "description": "Allowed HTTP methods."
        },
        "allowed_headers": {
          "type": "array",
          "items": { "type": "string" },
          "default": [ "Content-Type", "Authorization" ],
          "description": "Allowed request headers (case-insensitive)."
        },
        "expose_headers": {
          "type": "array",
          "items": { "type": "string" },
          "default": [ ],
          "description": "Headers exposed to browser (beyond CORS-safelisted headers)."
        },
        "max_age": {
          "type": "integer",
          "minimum": 0,
          "maximum": 86400,
          "default": 86400,
          "description": "Preflight cache duration in seconds (max 24h)."
        },
        "allow_credentials": {
          "type": "boolean",
          "default": false,
          "description": "Allow credentials (cookies, auth headers). Requires specific origins (not '*')."
        }
      },
      "required": [ "enabled" ]
    }
  }
}
```

### Auth Plugin

**Base type**: `gts.x.core.oagw.plugin.auth.v1~`

Auth plugins handle outbound authentication to upstream services. Only one auth plugin per upstream.

**Note**: This schema describes builtin auth plugin metadata. Custom auth plugins are stored in `oagw_plugin` table with UUID identification.

**Builtin Plugin Metadata**:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "OAGW Auth Plugin",
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "format": "gts-identifier",
      "examples": [
        "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.apikey.v1",
        "gts.x.core.oagw.plugin.auth.v1~acme.billing.custom_auth.v1"
      ]
    },
    "type": {
      "type": "string",
      "format": "gts-identifier",
      "enum": [
        "gts.x.core.oagw.plugin.type.v1~x.core.oagw.builtin.v1",
        "gts.x.core.oagw.plugin.type.v1~x.core.oagw.starlark.v1"
      ]
    },
    "config_schema": {
      "type": "object",
      "description": "JSON Schema validated when plugin is attached to upstream."
    },
    "source_ref": {
      "type": "string",
      "format": "uri",
      "pattern": "^/api/oagw/v1/plugins/.+/source$",
      "description": "Derived from plugin id. Starlark source fetched via GET {source_ref}."
    }
  },
  "required": [ "id", "type", "config_schema" ]
}
```

**Builtin Authentication Plugins**

- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.noop.v1`
- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.apikey.v1`
- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.basic.v1`
- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.oauth2.client_cred.v1`
- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.oauth2.client_cred_basic.v1`
- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.bearer.v1`

### Guard Plugin

**Base type**: `gts.x.core.oagw.plugin.guard.v1~`

Guard plugins validate requests and enforce policies. Can reject requests before they reach upstream. Multiple guard plugins per upstream/route.

**Schema**:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "OAGW Guard Plugin",
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "format": "gts-identifier",
      "examples": [
        "gts.x.core.oagw.plugin.guard.v1~x.core.oagw.timeout.v1",
        "gts.x.core.oagw.plugin.guard.v1~acme.security.request_validator.v1"
      ]
    },
    "type": {
      "type": "string",
      "format": "gts-identifier",
      "enum": [
        "gts.x.core.oagw.plugin.type.v1~x.core.oagw.builtin.v1",
        "gts.x.core.oagw.plugin.type.v1~x.core.oagw.starlark.v1"
      ]
    },
    "config_schema": {
      "type": "object",
      "description": "JSON Schema validated when plugin is attached to upstream/route."
    },
    "source_ref": {
      "type": "string",
      "format": "uri",
      "pattern": "^/api/oagw/v1/plugins/.+/source$",
      "description": "Derived from plugin id. Starlark source fetched via GET {source_ref}."
    }
  },
  "required": [ "id", "type", "config_schema" ]
}
```

**Builtin Guard Plugins**:

| Plugin ID                                                | Description                 |
|----------------------------------------------------------|-----------------------------|
| `gts.x.core.oagw.plugin.guard.v1~x.core.oagw.timeout.v1` | Request timeout enforcement |
| `gts.x.core.oagw.plugin.guard.v1~x.core.oagw.cors.v1`    | CORS preflight validation   |

**Note**: Circuit breaker is **core functionality** (not a plugin). See [ADR: Circuit Breaker](./docs/adr-circuit-breaker.md) for configuration and fallback strategies.

### Transform Plugin

**Base type**: `gts.x.core.oagw.plugin.transform.v1~`

Transform plugins mutate requests and responses. Multiple transform plugins per upstream/route, executed in order.

**Schema**:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "OAGW Transform Plugin",
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "format": "gts-identifier",
      "examples": [
        "gts.x.core.oagw.plugin.transform.v1~x.core.oagw.logging.v1",
        "gts.x.core.oagw.plugin.transform.v1~acme.billing.redact_pii.v1"
      ]
    },
    "type": {
      "type": "string",
      "format": "gts-identifier",
      "enum": [
        "gts.x.core.oagw.plugin.type.v1~x.core.oagw.builtin.v1",
        "gts.x.core.oagw.plugin.type.v1~x.core.oagw.starlark.v1"
      ]
    },
    "phase": {
      "type": "array",
      "items": {
        "enum": [ "on_request", "on_response", "on_error" ]
      },
      "minItems": 1
    },
    "config_schema": {
      "type": "object",
      "description": "JSON Schema validated when plugin is attached to upstream/route."
    },
    "source_ref": {
      "type": "string",
      "format": "uri",
      "pattern": "^/api/oagw/v1/plugins/.+/source$",
      "description": "Derived from plugin id. Starlark source fetched via GET {source_ref}."
    }
  },
  "required": [ "id", "type", "phase", "config_schema" ]
}
```

**Builtin Transform Plugins**:

| Plugin ID                                                       | Phase                    | Description                        |
|-----------------------------------------------------------------|--------------------------|------------------------------------|
| `gts.x.core.oagw.plugin.transform.v1~x.core.oagw.logging.v1`    | request, response, error | Request/response logging           |
| `gts.x.core.oagw.plugin.transform.v1~x.core.oagw.metrics.v1`    | request, response        | Prometheus metrics                 |
| `gts.x.core.oagw.plugin.transform.v1~x.core.oagw.request_id.v1` | request, response        | X-Request-ID injection/propagation |

**Starlark Plugin Context API**:

```starlark
# ctx.request (on_request phase)
ctx.request.method              # str: "GET", "POST", etc. (read-only)
ctx.request.path                # str: "/v1/chat/completions"
ctx.request.set_path(path)      # Modify outbound path
ctx.request.query               # dict: {"version": "2"}
ctx.request.set_query(dict)     # Replace query parameters
ctx.request.add_query(key, val) # Add/append query parameter
ctx.request.headers             # Headers object
ctx.request.body                # bytes: raw body
ctx.request.json()              # dict: parsed JSON body
ctx.request.set_json(obj)       # Replace body with JSON
ctx.request.tenant_id           # str: authenticated tenant

# ctx.response (on_response phase)
ctx.response.status             # int: HTTP status code
ctx.response.headers            # Headers object
ctx.response.body               # bytes: raw body
ctx.response.json()             # dict: parsed JSON body
ctx.response.set_json(obj)      # Replace body with JSON
ctx.response.set_status(code)   # Override status code

# ctx.error (on_error phase)
ctx.error.status                # int: error status
ctx.error.code                  # str: error code
ctx.error.message               # str: error message
ctx.error.upstream              # bool: true if upstream error

# Headers object
headers.get("Name")             # str | None
headers.set("Name", "value")    # Set/overwrite
headers.add("Name", "value")    # Append (multi-value)
headers.remove("Name")          # Delete
headers.keys()                  # list[str]

# Utilities
ctx.config                      # dict: plugin instance config
ctx.route.id                    # str: route identifier
ctx.log.info(msg, data)         # Logging
ctx.time.elapsed_ms()           # int: ms since request start

# Control flow
ctx.next()                      # Continue to next plugin
ctx.reject(status, code, msg)   # Halt chain, return error
ctx.respond(status, body)       # Halt chain, return custom response
```

**Starlark Sandbox Restrictions**:

| Feature                | Allowed                   |
|------------------------|---------------------------|
| Network I/O            | ❌                         |
| File I/O               | ❌                         |
| Imports                | ❌                         |
| Infinite loops         | ❌ (timeout enforced)      |
| Large allocations      | ❌ (memory limit enforced) |
| JSON manipulation      | ✅                         |
| String/Math operations | ✅                         |
| Logging (`ctx.log`)    | ✅                         |
| Time (`ctx.time`)      | ✅                         |

**Example: Custom Guard Plugin Definition**:

```json
{
  "id": "gts.x.core.oagw.plugin.guard.v1~550e8400-e29b-41d4-a716-446655440000",
  "tenant_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "name": "request_validator",
  "description": "Validates request headers and body size",
  "plugin_type": "guard",
  "config_schema": {
    "type": "object",
    "properties": {
      "max_body_size": { "type": "integer", "default": 1048576 },
      "required_headers": { "type": "array", "items": { "type": "string" } }
    }
  },
  "source_code": "..."
}
```

**Plugin Source** (stored in `source_code` field or fetched via `GET /api/oagw/v1/plugins/gts.x.core.oagw.plugin.guard.v1~550e8400-e29b-41d4-a716-446655440000/source`):

```starlark
def on_request(ctx):
    # Guards only implement on_request phase
    for h in ctx.config.get("required_headers", []):
        if not ctx.request.headers.get(h):
            return ctx.reject(400, "MISSING_HEADER", "Required header: " + h)

    if len(ctx.request.body) > ctx.config.get("max_body_size", 1048576):
        return ctx.reject(413, "BODY_TOO_LARGE", "Body exceeds limit")

    return ctx.next()
```

**Example: Custom Transform Plugin Definition**:

```json
{
  "id": "gts.x.core.oagw.plugin.transform.v1~6ba7b810-9dad-11d1-80b4-00c04fd430c8",
  "tenant_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "name": "redact_pii",
  "description": "Redacts PII fields from response",
  "plugin_type": "transform",
  "phases": [ "on_response" ],
  "config_schema": {
    "type": "object",
    "properties": {
      "fields": { "type": "array", "items": { "type": "string" } }
    }
  },
  "source_code": "..."
}
```

**Plugin Source** (stored in `source_code` field or fetched via `GET /api/oagw/v1/plugins/gts.x.core.oagw.plugin.transform.v1~6ba7b810-9dad-11d1-80b4-00c04fd430c8/source`):

```starlark
def on_response(ctx):
    # Redact PII fields from JSON response
    data = ctx.response.json()
    for field in ctx.config.get("fields", []):
        if field in data:
            data[field] = "[REDACTED]"
    ctx.response.set_json(data)
    return ctx.next()
```

**Example: Path and Query Transformation Plugin**:

```json
{
  "id": "gts.x.core.oagw.plugin.transform.v1~8f8e8400-e29b-41d4-a716-446655440001",
  "tenant_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "name": "path_rewriter",
  "description": "Rewrites request paths and adds API version",
  "plugin_type": "transform",
  "phases": [ "on_request" ],
  "config_schema": {
    "type": "object",
    "properties": {
      "path_prefix": { "type": "string" },
      "add_api_version": { "type": "boolean" }
    }
  },
  "source_code": "..."
}
```

**Plugin Source** (stored in `source_code` field):

```starlark
def on_request(ctx):
    # Transform path: prepend custom prefix
    prefix = ctx.config.get("path_prefix", "")
    if prefix:
        new_path = prefix + ctx.request.path
        ctx.request.set_path(new_path)
        ctx.log.info("Rewrote path", {"old": ctx.request.path, "new": new_path})
    
    # Transform query: add API version if configured
    if ctx.config.get("add_api_version", False):
        ctx.request.add_query("api_version", "2024-01")
    
    # Transform query: remove internal parameters
    query = ctx.request.query
    if "internal_debug" in query:
        del query["internal_debug"]
        ctx.request.set_query(query)
    
    return ctx.next()
```

## REST API

OAGW exposes two main API surfaces:

1. **Management API**: CRUD operations for upstreams, routes, and plugins
2. **Proxy API**: Proxying requests to upstream services

### Error Response Format

All OAGW errors follow **RFC 9457 Problem Details** format (`application/problem+json`):

```json
{
  "type": "gts.x.core.errors.err.v1~x.oagw.rate_limit.exceeded.v1",
  "title": "Rate Limit Exceeded",
  "status": 429,
  "detail": "Rate limit exceeded for upstream api.openai.com",
  "instance": "/api/oagw/v1/proxy/api.openai.com/v1/chat/completions",
  "upstream_id": "uuid-123",
  "host": "api.openai.com",
  "retry_after_seconds": 15,
  "trace_id": "01J..."
}
```

**Response Headers**:

```http
Content-Type: application/problem+json
Retry-After: 15
```

**Standard Fields** (RFC 9457):

- `type`: GTS identifier for the error type (used for programmatic error handling)
- `title`: Human-readable summary
- `status`: HTTP status code
- `detail`: Human-readable explanation specific to this occurrence
- `instance`: URI reference identifying the specific occurrence

**Extension Fields** (OAGW-specific):

- `upstream_id`, `host`, `path`: Request context
- `retry_after_seconds`: Retry guidance
- `trace_id`: For distributed tracing correlation

### Error Source Distinction

OAGW distinguishes between **gateway errors** (originated by OAGW) and **upstream errors** (passthrough from upstream service) using the `X-OAGW-Error-Source` header.

See [ADR: Error Source Distinction](./docs/adr-error-source-distinction.md) for detailed analysis of alternatives.

**Gateway Error**:

```http
HTTP/1.1 429 Too Many Requests
X-OAGW-Error-Source: gateway
Content-Type: application/problem+json
Retry-After: 15

{
  "type": "gts.x.core.errors.err.v1~x.oagw.rate_limit.exceeded.v1",
  "title": "Rate Limit Exceeded",
  "status": 429,
  "detail": "Rate limit exceeded for upstream api.openai.com",
  "instance": "/api/oagw/v1/proxy/api.openai.com/v1/chat",
  "host": "api.openai.com",
  "retry_after_seconds": 15
}
```

**Upstream Error** (passthrough):

```http
HTTP/1.1 500 Internal Server Error
X-OAGW-Error-Source: upstream
Content-Type: application/json

<upstream error body as-is>
```

**Benefits**:

- ✅ Simple to implement and consume
- ✅ Non-invasive (does not modify response body)
- ✅ Works with any content type (JSON, binary, streaming)
- ✅ Industry standard (Kong: `X-Kong-Upstream-Status`, Apigee: `X-Apigee-fault-source`)

**Note**: Header may be stripped by intermediaries. For critical error handling, clients should combine header check with error response structure inspection.

### Management API

#### Upstream Endpoints

| Method   | Path                          | Description        | Request Body                 | Response                 |
|----------|-------------------------------|--------------------|------------------------------|--------------------------|
| `POST`   | `/api/oagw/v1/upstreams`      | Create upstream    | [Upstream Schema](#upstream) | `201 Created` + Upstream |
| `GET`    | `/api/oagw/v1/upstreams`      | List upstreams     | -                            | `200 OK` + Upstream[]    |
| `GET`    | `/api/oagw/v1/upstreams/{id}` | Get upstream by ID | -                            | `200 OK` + Upstream      |
| `PUT`    | `/api/oagw/v1/upstreams/{id}` | Update upstream    | [Upstream Schema](#upstream) | `200 OK` + Upstream      |
| `DELETE` | `/api/oagw/v1/upstreams/{id}` | Delete upstream    | -                            | `204 No Content`         |

**Note**: `{id}` is anonymous GTS identifier: `gts.x.core.oagw.upstream.v1~{uuid}` (e.g., `gts.x.core.oagw.upstream.v1~7c9e6679-7425-40de-944b-e07fc1f90ae7`)

**Query Parameters (List):**

| Parameter  | Type    | Description                                                 |
|------------|---------|-------------------------------------------------------------|
| `$filter`  | string  | OData filter expression (e.g., `alias eq 'api.openai.com'`) |
| `$select`  | string  | Fields to return (e.g., `id,alias,server`)                  |
| `$orderby` | string  | Sort order (e.g., `created_at desc`)                        |
| `$top`     | integer | Max results (default: 50, max: 100)                         |
| `$skip`    | integer | Offset for pagination                                       |

#### Route Endpoints

| Method   | Path                       | Description     | Request Body           | Response              |
|----------|----------------------------|-----------------|------------------------|-----------------------|
| `POST`   | `/api/oagw/v1/routes`      | Create route    | [Route Schema](#route) | `201 Created` + Route |
| `GET`    | `/api/oagw/v1/routes`      | List routes     | -                      | `200 OK` + Route[]    |
| `GET`    | `/api/oagw/v1/routes/{id}` | Get route by ID | -                      | `200 OK` + Route      |
| `PUT`    | `/api/oagw/v1/routes/{id}` | Update route    | [Route Schema](#route) | `200 OK` + Route      |
| `DELETE` | `/api/oagw/v1/routes/{id}` | Delete route    | -                      | `204 No Content`      |

**Note**: `{id}` is anonymous GTS identifier: `gts.x.core.oagw.route.v1~{uuid}` (e.g., `gts.x.core.oagw.route.v1~550e8400-e29b-41d4-a716-446655440000`)

**Query Parameters (List):**

| Parameter  | Type    | Description                                    |
|------------|---------|------------------------------------------------|
| `$filter`  | string  | OData filter (e.g., `upstream_id eq '{uuid}'`) |
| `$select`  | string  | Fields to return                               |
| `$orderby` | string  | Sort order                                     |
| `$top`     | integer | Max results (default: 50, max: 100)            |
| `$skip`    | integer | Offset for pagination                          |

#### Plugin Endpoints

| Method   | Path                               | Description         | Request Body                  | Response               |
|----------|------------------------------------|---------------------|-------------------------------|------------------------|
| `POST`   | `/api/oagw/v1/plugins`             | Create plugin       | [Plugin Schema](#auth-plugin) | `201 Created` + Plugin |
| `GET`    | `/api/oagw/v1/plugins`             | List plugins        | -                             | `200 OK` + Plugin[]    |
| `GET`    | `/api/oagw/v1/plugins/{id}`        | Get plugin by ID    | -                             | `200 OK` + Plugin      |
| `DELETE` | `/api/oagw/v1/plugins/{id}`        | Delete plugin       | -                             | `204 No Content`       |
| `GET`    | `/api/oagw/v1/plugins/{id}/source` | Get Starlark source | -                             | `200 OK` + text/plain  |

**Note**:

- `{id}` is anonymous GTS identifier: `gts.x.core.oagw.plugin.{type}.v1~{uuid}` (e.g., `gts.x.core.oagw.plugin.guard.v1~6ba7b810-9dad-11d1-80b4-00c04fd430c8`)
- Plugins are **immutable** - no PUT/UPDATE endpoint
- DELETE fails with `409 Conflict` if plugin is referenced by any upstream/route

**Plugin Deletion Behavior**:

```http
DELETE /api/oagw/v1/plugins/gts.x.core.oagw.plugin.guard.v1~550e8400-e29b-41d4-a716-446655440000
```

**Success** (plugin not in use):

```http
HTTP/1.1 204 No Content
```

**Failure** (plugin in use):

```http
HTTP/1.1 409 Conflict
Content-Type: application/problem+json

{
  "type": "gts.x.core.errors.err.v1~x.oagw.plugin.in_use.v1",
  "title": "Plugin In Use",
  "status": 409,
  "detail": "Plugin is referenced by 3 upstream(s) and 2 route(s)",
  "plugin_id": "gts.x.core.oagw.plugin.guard.v1~550e8400-e29b-41d4-a716-446655440000",
  "referenced_by": {
    "upstreams": ["gts.x.core.oagw.upstream.v1~..."],
    "routes": ["gts.x.core.oagw.route.v1~..."]
  }
}
```

**Query Parameters (List):**

| Parameter | Type    | Description                            |
|-----------|---------|----------------------------------------|
| `$filter` | string  | OData filter (e.g., `type eq 'guard'`) |
| `$select` | string  | Fields to return                       |
| `$top`    | integer | Max results                            |
| `$skip`   | integer | Offset for pagination                  |

### Proxy API

#### Proxy Endpoint

`{METHOD} /api/oagw/v1/proxy/{alias}[/{path_suffix}][?{query_parameters}]`

Where:

- `{alias}` - Upstream alias (e.g., `api.openai.com` or `my-internal-service`)
- `{path_suffix}` - Path to match against route's `match.http.path` pattern
- `{query_parameters}` - Query params validated against route's `match.http.query_allowlist`

#### API Call Examples

- [Plain HTTP Request/Response](./examples/1.http.positive.md)
- [Server-Sent Events (SSE)](./examples/2.sse.positive.md)
- [Streaming WebSockets](./examples/3.websocket.positive.md)
- [Streaming gRPC](./examples/4.grpc.positive.md)

## Database Persistence

### Data Model

OAGW uses three main tables for configuration storage, all tenant-scoped via `tenant_id`.

#### Resource Identification Pattern

All resources use **anonymous GTS identifiers** in the REST API but store UUIDs in the database:

| Resource | API Identifier                            | Database | Example API                                                            | Example DB                             |
|----------|-------------------------------------------|----------|------------------------------------------------------------------------|----------------------------------------|
| Upstream | `gts.x.core.oagw.upstream.v1~{uuid}`      | UUID     | `gts.x.core.oagw.upstream.v1~7c9e6679-7425-40de-944b-e07fc1f90ae7`     | `7c9e6679-7425-40de-944b-e07fc1f90ae7` |
| Route    | `gts.x.core.oagw.route.v1~{uuid}`         | UUID     | `gts.x.core.oagw.route.v1~550e8400-e29b-41d4-a716-446655440000`        | `550e8400-e29b-41d4-a716-446655440000` |
| Plugin   | `gts.x.core.oagw.plugin.{type}.v1~{uuid}` | UUID     | `gts.x.core.oagw.plugin.guard.v1~6ba7b810-9dad-11d1-80b4-00c04fd430c8` | `6ba7b810-9dad-11d1-80b4-00c04fd430c8` |

**API Layer**: Parses anonymous GTS identifier, extracts UUID after `~`, uses UUID for database operations.

**Exception**: Builtin plugins use named GTS identifiers (e.g., `gts.x.core.oagw.plugin.transform.v1~x.core.oagw.logging.v1`) and are not stored in database.

#### Entity Relationship

```
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│    Upstream     │       │      Route      │       │     Plugin      │
├─────────────────┤       ├─────────────────┤       ├─────────────────┤
│ id (PK)         │◄──────│ upstream_id(FK) │       │ id (PK)         │
│ tenant_id       │       │ id (PK)         │       │ tenant_id       │
│ alias           │       │ tenant_id       │       │ plugin_type     │
│ server (JSONB)  │       │ match (JSONB)   │       │ config_schema   │
│ protocol        │       │ plugins (JSONB) │       │ source_code     │
│ auth (JSONB)    │       │ rate_limit      │       │ ...             │
│ headers (JSONB) │       │ ...             │       └─────────────────┘
│ plugins (JSONB) │       └─────────────────┘
│ rate_limit      │
│ ...             │
└─────────────────┘
```

#### Upstream Table

```sql

CREATE TABLE oagw_upstream
(
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id  UUID         NOT NULL REFERENCES tenant (id),

    -- Identity
    alias      VARCHAR(255) NOT NULL,
    tags       TEXT[] DEFAULT '{}', -- Tenant-local tags; effective tags may include inherited tags at read time

    -- Server configuration (JSONB for flexibility)
    server     JSONB        NOT NULL,
    -- Example: {"endpoints": [{"scheme": "https", "host": "api.openai.com", "port": 443}]}

    protocol   VARCHAR(100) NOT NULL,
    -- Example: "gts.x.core.oagw.protocol.v1~x.core.http.v1"

    -- Auth configuration (JSONB)
    auth       JSONB,
    -- Example: {"type": "...", "sharing": "inherit", "config": {"header": "Authorization", ...}}

    -- Header transformation rules
    headers    JSONB,

    -- Plugin references
    plugins    JSONB,
    -- Example: {"sharing": "inherit", "items": ["gts.x.core.oagw.plugin.transform.v1~x.core.oagw.logging.v1"]}

    -- Rate limiting
    rate_limit JSONB,
    -- Example: {"sharing": "enforce", "rate": 10000, "window": "minute", "scope": "tenant"}

    -- Metadata
    enabled    BOOLEAN          DEFAULT TRUE,
    created_at TIMESTAMPTZ      DEFAULT NOW(),
    updated_at TIMESTAMPTZ      DEFAULT NOW(),
    created_by UUID REFERENCES principal (id),
    updated_by UUID REFERENCES principal (id),

    -- Constraints
    CONSTRAINT uq_upstream_tenant_alias UNIQUE (tenant_id, alias)
);

-- Indexes
CREATE INDEX idx_upstream_tenant ON oagw_upstream (tenant_id);
CREATE INDEX idx_upstream_alias ON oagw_upstream (alias);
CREATE INDEX idx_upstream_tags ON oagw_upstream USING GIN(tags);
CREATE INDEX idx_upstream_enabled ON oagw_upstream (tenant_id, enabled) WHERE enabled = TRUE;

-- Note: Upstreams are addressed in API as anonymous GTS identifiers:
-- Example: gts.x.core.oagw.upstream.v1~7c9e6679-7425-40de-944b-e07fc1f90ae7
-- The UUID after ~ maps to this table's id column
```

#### Route Table

```sql
CREATE TABLE oagw_route
(
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID  NOT NULL REFERENCES tenant (id),
    upstream_id UUID  NOT NULL REFERENCES oagw_upstream (id) ON DELETE CASCADE,

    -- Tags for categorization
    tags        TEXT[] DEFAULT '{}',

    -- Match rules (JSONB, one of http/grpc)
    match       JSONB NOT NULL,
    -- HTTP example: {"http": {"methods": ["GET", "POST"], "path": "/v1/chat/completions", ...}}
    -- gRPC example: {"grpc": {"service": "foo.v1.UserService", "method": "GetUser"}}

    -- Plugin references
    plugins     JSONB,

    -- Rate limiting (route-level)
    rate_limit  JSONB,

    -- Metadata
    enabled     BOOLEAN          DEFAULT TRUE,
    priority    INTEGER          DEFAULT 0, -- Higher priority routes match first
    created_at  TIMESTAMPTZ      DEFAULT NOW(),
    updated_at  TIMESTAMPTZ      DEFAULT NOW(),
    created_by  UUID REFERENCES principal (id),
    updated_by  UUID REFERENCES principal (id)
);

-- Indexes
CREATE INDEX idx_route_tenant ON oagw_route (tenant_id);
CREATE INDEX idx_route_upstream ON oagw_route (upstream_id);
CREATE INDEX idx_route_enabled ON oagw_route (tenant_id, enabled) WHERE enabled = TRUE;
CREATE INDEX idx_route_priority ON oagw_route (upstream_id, priority DESC);

-- Partial index for HTTP route matching
CREATE INDEX idx_route_http_path ON oagw_route ((match -> 'http' - >> 'path')) 
    WHERE match ? 'http';

-- Note: Routes are addressed in API as anonymous GTS identifiers:
-- Example: gts.x.core.oagw.route.v1~550e8400-e29b-41d4-a716-446655440000
-- The UUID after ~ maps to this table's id column
```

#### Plugin Table

```sql
CREATE TABLE oagw_plugin
(
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id      UUID         NOT NULL REFERENCES tenant (id),

    -- Human-readable name (unique per tenant)
    name           VARCHAR(255) NOT NULL,
    description    TEXT,

    -- Plugin classification
    plugin_type    VARCHAR(20)  NOT NULL CHECK (plugin_type IN ('auth', 'guard', 'transform')),

    -- For transform plugins: which phases are supported
    phases         TEXT[] DEFAULT '{}',
    -- Example: ['on_request', 'on_response', 'on_error']

    -- Configuration schema (JSON Schema)
    config_schema  JSONB        NOT NULL,

    -- Starlark source (required for custom plugins)
    source_code    TEXT         NOT NULL,

    -- Lifecycle management
    enabled        BOOLEAN          DEFAULT TRUE,
    last_used_at   TIMESTAMPTZ,
    -- Updated when upstream/route references this plugin
    -- NULL if never used

    gc_eligible_at TIMESTAMPTZ,
    -- Set to (NOW() + TTL) when plugin becomes unlinked
    -- NULL if plugin is linked to any upstream/route
    -- Background job deletes plugins where gc_eligible_at < NOW()

    -- Metadata
    created_at     TIMESTAMPTZ      DEFAULT NOW(),
    updated_at     TIMESTAMPTZ      DEFAULT NOW(),
    created_by     UUID REFERENCES principal (id),
    updated_by     UUID REFERENCES principal (id),

    -- Constraints
    CONSTRAINT uq_plugin_tenant_name UNIQUE (tenant_id, name)
);

-- Indexes
CREATE INDEX idx_plugin_tenant ON oagw_plugin (tenant_id);
CREATE INDEX idx_plugin_type ON oagw_plugin (plugin_type);
CREATE INDEX idx_plugin_enabled ON oagw_plugin (tenant_id, enabled) WHERE enabled = TRUE;
CREATE INDEX idx_plugin_gc ON oagw_plugin (gc_eligible_at) WHERE gc_eligible_at IS NOT NULL;

-- Note: Plugins are addressed in API as anonymous GTS identifiers:
-- Example: gts.x.core.oagw.plugin.guard.v1~550e8400-e29b-41d4-a716-446655440000
-- The UUID after ~ maps to this table's id column
```

### Common Queries

#### Find Upstream by Alias (with tenant hierarchy and enabled inheritance)

```sql
-- Find first matching upstream walking tenant hierarchy from child to root
-- An upstream is only returned if enabled=TRUE for all ancestors in hierarchy
-- $1: alias, $2: tenant_hierarchy array [child_id, parent_id, ..., root_id]

-- First, check if any ancestor has disabled an upstream with this alias
WITH upstream_chain AS (SELECT u.*, array_position($2, u.tenant_id) as pos
                        FROM oagw_upstream u
                        WHERE u.alias = $1
                          AND u.tenant_id = ANY ($2))
SELECT c.*
FROM upstream_chain c
WHERE NOT EXISTS (
    -- Check if any ancestor (higher in hierarchy) has disabled this alias
    SELECT 1
    FROM upstream_chain ancestor
    WHERE ancestor.pos > c.pos -- ancestor is higher in hierarchy (closer to root)
      AND ancestor.enabled = FALSE)
  AND c.enabled = TRUE
ORDER BY c.pos LIMIT 1;
```

**Clarification**: this query selects the routing winner only (closest alias match).
Effective policy resolution must also evaluate ancestor rows for the same alias and apply any `sharing: enforce` constraints.

#### List Upstreams for Tenant (with shadowing and enabled inheritance)

```sql
-- Returns upstreams visible to tenant, respecting:
-- 1. Shadowing: closest tenant's upstream wins
-- 2. Enabled inheritance: if any ancestor disabled the upstream, it's not visible
-- $1: tenant_hierarchy array, $2: limit, $3: offset

WITH ranked_upstreams AS (SELECT u.*,
                                 array_position($1, u.tenant_id)         as pos,
                                 -- Check if any ancestor has disabled this alias
                                 EXISTS (SELECT 1
                                         FROM oagw_upstream ancestor
                                         WHERE ancestor.alias = u.alias
                                           AND ancestor.tenant_id = ANY ($1)
                                           AND array_position($1, ancestor.tenant_id) > array_position($1, u.tenant_id)
                                           AND ancestor.enabled = FALSE) as ancestor_disabled
                          FROM oagw_upstream u
                          WHERE u.tenant_id = ANY ($1))
SELECT DISTINCT
ON (alias) *
FROM ranked_upstreams
WHERE enabled = TRUE
  AND ancestor_disabled = FALSE
ORDER BY alias, pos
    LIMIT $2
OFFSET $3;
```

**Clarification**: list/discovery returns the visible winner per alias.
Ancestor `sharing: enforce` constraints can still affect runtime effective configuration.
For tags, effective discovery should use additive union across hierarchy (`ancestor ∪ descendant`) rather than mutating ancestor rows.

#### Find Matching Route for Request

```sql
-- Match route by upstream, HTTP method, and path prefix
-- $1: upstream_id, $2: method (e.g., 'POST'), $3: request path
SELECT *
FROM oagw_route
WHERE upstream_id = $1
          AND enabled = TRUE
          AND match - > 'http' - > 'methods'
    ? $2
  AND $3 LIKE (match -
    >'http'->>'path' || '%')
ORDER BY
    priority DESC,
    length (match ->'http'->>'path') DESC -- Longest path wins
    LIMIT 1;
```

#### Resolve Effective Configuration

```sql
-- Fetch upstream and route config for merge resolution
-- $1: upstream_id, $2: route_id
SELECT u.auth       as upstream_auth,
       u.rate_limit as upstream_rate_limit,
       u.plugins    as upstream_plugins,
       u.headers    as upstream_headers,
       r.rate_limit as route_rate_limit,
       r.plugins    as route_plugins,
       u.tenant_id  as upstream_tenant_id,
       r.tenant_id  as route_tenant_id
FROM oagw_upstream u
         JOIN oagw_route r ON r.upstream_id = u.id
WHERE u.id = $1
  AND r.id = $2;
```

#### List Routes by Upstream

```sql
-- $1: upstream_id, $2: limit, $3: offset
SELECT *
FROM oagw_route
WHERE upstream_id = $1
  AND enabled = TRUE
ORDER BY priority DESC, created_at
    LIMIT $2
OFFSET $3;
```

#### Track Plugin Usage

```sql
-- Find all plugins referenced by upstreams and routes
-- Used to update last_used_at and gc_eligible_at
WITH referenced_plugins AS (SELECT DISTINCT jsonb_array_elements_text(plugins - > 'items') as plugin_ref
                            FROM oagw_upstream
                            WHERE plugins IS NOT NULL

                            UNION

                            SELECT DISTINCT jsonb_array_elements_text(plugins - > 'items') as plugin_ref
                            FROM oagw_route
                            WHERE plugins IS NOT NULL),
     plugin_uuids AS (SELECT substring(plugin_ref from '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}') ::UUID as plugin_id
                      FROM referenced_plugins
                      WHERE plugin_ref ~ '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}'
    )
-- Update linked plugins (clear gc_eligible_at, update last_used_at)
UPDATE oagw_plugin
SET gc_eligible_at = NULL,
    last_used_at   = NOW()
WHERE id IN (SELECT plugin_id FROM plugin_uuids);

-- Mark unlinked plugins for garbage collection
-- $1: TTL in seconds (default: 2592000 = 30 days)
UPDATE oagw_plugin
SET gc_eligible_at = NOW() + ($1 || ' seconds')::INTERVAL
WHERE id NOT IN (SELECT plugin_id FROM plugin_uuids WHERE plugin_id IS NOT NULL)
  AND gc_eligible_at IS NULL;
```

#### Delete Garbage-Collected Plugins

```sql
-- Background job: delete plugins past their GC eligibility date
DELETE
FROM oagw_plugin
WHERE gc_eligible_at IS NOT NULL
  AND gc_eligible_at < NOW();
```

## Metrics and Observability

OAGW exposes Prometheus metrics at `/metrics` endpoint for monitoring performance, errors, and resource usage.

### Core Metrics

**Request Metrics**:

```promql
# Total requests by host, path, method, status class (2xx, 3xx, 4xx, 5xx)
oagw_requests_total{host, path, method, status_class} counter

# Request duration (histogram with P50, P95, P99)
oagw_request_duration_seconds{host, path, phase} histogram
# phase: "total", "upstream", "plugins"

# In-flight requests
oagw_requests_in_flight{host} gauge
```

**Error Metrics**:

```promql
# Errors by type
oagw_errors_total{host, path, error_type} counter
# error_type: "TIMEOUT", "UPSTREAM_ERROR", "CIRCUIT_BREAKER_OPEN", etc.
```

**Circuit Breaker Metrics**:

```promql
# Circuit breaker state (0=CLOSED, 1=HALF_OPEN, 2=OPEN)
oagw_circuit_breaker_state{host} gauge

# Circuit breaker state changes
oagw_circuit_breaker_transitions_total{host, from_state, to_state} counter
```

**Rate Limit Metrics**:

```promql
# Rate limit rejections
oagw_rate_limit_exceeded_total{host, path} counter

# Rate limit consumption (0.0 to 1.0)
oagw_rate_limit_usage_ratio{host, path} gauge
```

**Upstream Health Metrics**:

```promql
# Upstream availability (0=down, 1=up)
oagw_upstream_available{host, endpoint} gauge

# Connection pool stats
oagw_upstream_connections{host, state} gauge
# state: "idle", "active", "max"
```

### Cardinality Management

**No per-tenant labels**: To avoid metric explosion, tenant_id is NOT included in metric labels. Use aggregation at query time or separate tenant analytics system.

**Host label**: Uses upstream hostname (e.g., `api.openai.com`) instead of UUID for readability.

**Path normalization**: Path from route config (e.g., `/v1/chat/completions`) without dynamic segments or path_suffix. Bounded cardinality per upstream.

**Status class grouping**: Use `status_class` (2xx, 3xx, 4xx, 5xx) instead of individual status codes.

### Histogram Buckets

**Request duration** (milliseconds to seconds):

```
[0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]
```

### Metrics Endpoint

```
GET /metrics

# Example output
oagw_requests_total{host="api.openai.com",path="/v1/chat/completions",method="POST",status_class="2xx"} 1542
oagw_request_duration_seconds_bucket{host="api.openai.com",path="/v1/chat/completions",phase="total",le="0.1"} 1200
oagw_circuit_breaker_state{host="api.openai.com"} 0
```

**Access Control**: Metrics endpoint requires authentication. Only system administrators can access all metrics.

## Audit Logging

OAGW logs all proxy requests for security, compliance, and troubleshooting.

### Log Format

Structured JSON logs sent to stdout, ingested by centralized logging system (e.g., ELK, Loki).

```json
{
  "timestamp": "2026-02-03T11:09:37.431Z",
  "level": "INFO",
  "event": "proxy_request",
  "request_id": "req_abc123",
  "tenant_id": "tenant_xyz",
  "principal_id": "user_456",
  "host": "api.openai.com",
  "path": "/v1/chat/completions",
  "method": "POST",
  "status": 200,
  "duration_ms": 245,
  "request_size": 512,
  "response_size": 2048,
  "error_type": null
}
```

### What is Logged

**Success requests**: Request ID, tenant, host, path, method, status, duration, sizes
**Failed requests**: All above + error_type, error_message
**Config changes**: Upstream/route create/update/delete operations
**Auth failures**: Failed authentication attempts (rate limited to prevent log flooding)
**Circuit breaker events**: State transitions (CLOSED→OPEN, OPEN→HALF_OPEN, etc.)

### What is NOT Logged

**No PII**: Request/response bodies, query parameters, headers (except allowlisted)
**No secrets**: API keys, tokens, credentials
**High-frequency sampling**: Rate-limited to prevent excessive log volume (e.g., sample 1/100 for high-volume routes)

### Log Levels

- `INFO`: Successful requests, normal operations
- `WARN`: Rate limit exceeded, circuit breaker open, retry guidance emitted (`Retry-After`)
- `ERROR`: Upstream failures, timeouts, auth failures
- `DEBUG`: Detailed plugin execution (disabled in production)

## Error Handling

### Error Types

| Error Type           | HTTP | GTS Instance ID                                           | Retriable |
|----------------------|------|-----------------------------------------------------------|-----------|
| RouteError           | 400  | `gts.x.core.errors.err.v1~x.oagw.validation.error.v1`     | No        |
| ValidationError      | 400  | `gts.x.core.errors.err.v1~x.oagw.validation.error.v1`     | No        |
| RouteNotFound        | 404  | `gts.x.core.errors.err.v1~x.oagw.route.not_found.v1`      | No        |
| AuthenticationFailed | 401  | `gts.x.core.errors.err.v1~x.oagw.auth.failed.v1`          | No        |
| PayloadTooLarge      | 413  | `gts.x.core.errors.err.v1~x.oagw.payload.too_large.v1`    | No        |
| RateLimitExceeded    | 429  | `gts.x.core.errors.err.v1~x.oagw.rate_limit.exceeded.v1`  | Yes*      |
| SecretNotFound       | 500  | `gts.x.core.errors.err.v1~x.oagw.secret.not_found.v1`     | No        |
| ProtocolError        | 502  | `gts.x.core.errors.err.v1~x.oagw.protocol.error.v1`       | No        |
| DownstreamError      | 502  | `gts.x.core.errors.err.v1~x.oagw.downstream.error.v1`     | Depends   |
| StreamAborted        | 502  | `gts.x.core.errors.err.v1~x.oagw.stream.aborted.v1`       | No**      |
| LinkUnavailable      | 503  | `gts.x.core.errors.err.v1~x.oagw.link.unavailable.v1`     | Yes       |
| CircuitBreakerOpen   | 503  | `gts.x.core.errors.err.v1~x.oagw.circuit_breaker.open.v1` | Yes       |
| ConnectionTimeout    | 504  | `gts.x.core.errors.err.v1~x.oagw.timeout.connection.v1`   | Yes       |
| RequestTimeout       | 504  | `gts.x.core.errors.err.v1~x.oagw.timeout.request.v1`      | Yes       |
| IdleTimeout          | 504  | `gts.x.core.errors.err.v1~x.oagw.timeout.idle.v1`         | Yes       |
| PluginNotFound       | 503  | `gts.x.core.errors.err.v1~x.oagw.plugin.not_found.v1`     | No        |
| PluginInUse          | 409  | `gts.x.core.errors.err.v1~x.oagw.plugin.in_use.v1`        | No        |


## Review

1. Database schema, indexing and queries
2. [ADR: Rust ABI / Client Libraries](./docs/adr-rust-abi-client-library.md) - HTTP client abstractions, streaming support, plugin development APIs

## Future Developments

1. [Core] Circuit breaker: config and fallback strategies [ADR: Circuit Breaker](./docs/adr-circuit-breaker.md)
2. [Core] Concurrency control [ADR: Concurrency Control](./docs/adr-concurrency-control.md)
3. [Core] Backpressure queueing [ADR: Backpressure](./docs/adr-backpressure-queueing.md) - In-flight limits, queueing strategies, graceful degradation under load
4. [Plugin] Starlark standard library extensions (e.g., HTTP client, caching), with security considerations. Auth plugins may need network I/O.
5. [Security] TLS certificate pinning - Pin specific certificates/public keys for critical upstreams to prevent MITM attacks
6. [Security] mTLS support - Mutual TLS for client certificate authentication with upstream services
7. [Protocol] gRPC support - HTTP/2 multiplexing with content-type detection [ADR: gRPC Support](./docs/adr-grpc-support.md) **Requires prototype**

## Feature Breakdown by Phase

### Phase 0 (p0): MVP - OpenAI Integration Ready

**Goal**: Platform can proxy requests to OpenAI Chat/Completions API with basic security and usability.

**Deliverables**:

#### [ ] F-P0-001: Module Scaffold + SDK Boundary
- Create `oagw-sdk` crate with public models (Upstream/Route request/response DTOs) and error types (Problem `type` identifiers).
- Create `oagw` module crate wired into ModKit module lifecycle + REST registration (OperationBuilder), exposing Management + Proxy routers.
- Define minimal config surface (env/config): database handle, `cred_store` client, `types_registry` client, outbound HTTP client settings.
- Add scenario-driven acceptance checklist mapping `scenarios/case-*.md` to integration tests (start with HTTP + SSE cases).

#### [ ] F-P0-002: DB Schema + SeaORM Entities (Upstream, Route)
- Add migrations for `oagw_upstream` and `oagw_route` tables (tenant-scoped, UUID PKs, `enabled`, tags, JSONB config columns).
- Implement SeaORM entities with `#[derive(Scopable)]` and `SecureConn`-scoped repositories (no raw SQL; SQL in DESIGN is illustrative).
- Enforce constraints/indexes needed for MVP: `UNIQUE (tenant_id, alias)`, indexes for `(tenant_id, enabled)` and route `(upstream_id, priority)`.
- Include minimal query helpers: find upstream by alias, list upstreams, find route by (upstream_id, method, path prefix).

#### [ ] F-P0-003: Types Registry Registration (Schemas + Builtins)
- Register GTS schemas for `gts.x.core.oagw.upstream.v1~`, `gts.x.core.oagw.route.v1~`, and HTTP protocol identifier.
- Register builtin plugin identifiers needed for MVP: `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.noop.v1` and `...~x.core.oagw.apikey.v1`.
- Ensure API layer parses anonymous GTS IDs (`...~{uuid}`) and validates schema/type correctness before DB operations.
- Add OpenAPI schema exposure for upstream/route DTOs (enough to test Management API from generated clients).

#### [ ] F-P0-004: Management API - Upstream CRUD (Minimal)
- Implement `/api/oagw/v1/upstreams` CRUD (POST/GET list/GET by id/PUT/DELETE) using Secure ORM repositories (requires F-P0-002).
- Enforce bearer auth on all endpoints (authentication only; fine-grained permissions in p1) (requires F-P0-001).
- Validate alias format + defaulting rules (single endpoint default alias; reject missing alias for IP/heterogeneous hosts).
- Add minimal list pagination (`$top`/`$skip` only; full OData in p3).

#### [ ] F-P0-005: Management API - Route CRUD (HTTP Only)
- Implement `/api/oagw/v1/routes` CRUD for HTTP match rules: methods allowlist, path prefix, query allowlist, `path_suffix_mode`, priority (requires F-P0-002).
- Enforce upstream ownership link (`route.upstream_id` must belong to same tenant in MVP mode).
- Validate route invariants: non-empty methods, path starts with `/`, priority integer ordering semantics.
- Add minimal list pagination (`$top`/`$skip` only; full OData in p3).

#### [ ] F-P0-006: Proxy Endpoint - Basic Routing (HTTP)
- Implement `{METHOD} /api/oagw/v1/proxy/{alias}[/{path_suffix}][?{query}]` handler (requires F-P0-002).
- Resolve upstream by `alias` (single-tenant, no hierarchy) and match HTTP route by (upstream_id, method, longest path prefix, priority).
- Apply route transformation rules: `match.http.path` + `path_suffix_mode=append`, validate `path_suffix_mode=disabled`.
- Build outbound URL from upstream endpoint + transformed path + allowlisted query params.

#### [ ] F-P0-007: Request/Body Validation Guardrail Set (HTTP)
- Enforce query allowlist (reject unknown query keys) and method validation against route config (requires F-P0-006).
- Enforce body limits: hard cap 100MB; reject early without buffering; support `Transfer-Encoding: chunked` only.
- Validate `Content-Length` (integer, matches observed bytes when present); reject ambiguous TE/CL combinations.
- Add baseline status mapping for these validation failures (400/413) (full Problem coverage in p1).

#### [ ] F-P0-008: Header Transformation + SSRF Baseline
- Strip hop-by-hop headers and replace `Host` with upstream host by default (requires F-P0-006).
- Implement simple configured header transforms via `upstream.headers` (set/add/remove/overwrite) before outbound dispatch.
- Validate well-known headers: reject invalid names/values (CR/LF) and multiple `Host`; normalize/deny unsafe characters.
- Enforce scheme allowlist (`https` for MVP) to prevent accidental SSRF via plaintext upstreams.

#### [ ] F-P0-009: Builtin Auth Plugin - API Key Injection (OpenAI)
- Implement builtin auth plugin resolution for `...auth...~x.core.oagw.apikey.v1` and `...noop...` (requires F-P0-003).
- Integrate `cred_store` lookup for `secret_ref` in upstream auth config; inject into `Authorization: Bearer <key>` (or configured header).
- Ensure secrets never appear in logs/errors; redact auth headers on outbound request logging.
- Add scenario coverage: `scenarios/case-9.1-auth-noop.md`, `scenarios/case-9.2-auth-apikey.md`.

#### [ ] F-P0-010: Rate Limiting (Basic Token Bucket)
- Implement token bucket limiter (per upstream+route scope) with in-memory storage suitable for single-node MVP.
- Support `rate_limit` config on upstream/route (JSONB) with `rate` + `window`; apply route limit first then upstream.
- Return `429` with `Retry-After` seconds when exceeded; do not buffer request bodies before decision.
- Add scenario coverage: `scenarios/case-18.1-rate-limit-token-bucket.md`.

#### [ ] F-P0-011: Streaming Proxy Support (HTTP + SSE)
- Stream request bodies to upstream without buffering (backpressure-safe) where possible (requires F-P0-006).
- Stream responses to client as-is, supporting OpenAI SSE (`text/event-stream`) and large responses.
- On client disconnect, abort upstream request and release any acquired resources (rate limit permits, in-flight counters).
- Add scenario coverage: `scenarios/case-13.1-sse-forwarding.md`, `scenarios/case-13.2-sse-client-disconnect.md`.

#### [ ] F-P0-012: Minimal Error Surface (Gateway vs Upstream)
- Implement baseline gateway errors: 400 validation, 401 auth failed, 404 route/upstream not found, 413 payload too large, 429 rate limit, 502/504 upstream timeouts.
- Passthrough upstream errors (status + headers + body) without modification when response is not a gateway-generated Problem.
- Add `X-OAGW-Error-Source: upstream|gateway` header for errors where applicable (full semantics in p1) (see ADR: Error Source Distinction).
- Add scenario coverage: `scenarios/case-12.1-http-passthrough.md`, `scenarios/case-12.2-upstream-error-passthrough.md`.

**Exclusions** (explicitly out of scope for p0):
- Multi-tenancy hierarchy and configuration sharing/merge semantics
- Custom plugins (Starlark) and plugin CRUD APIs
- Circuit breakers, distributed concurrency limits, backpressure queues
- WebSocket, WebTransport, and gRPC proxying
- CORS handling (preflight) and protocol capability cache (HTTP/2 detection cache)
- Full OData (`$filter`, `$select`, `$orderby`) on list endpoints

---

### Phase 1 (p1): Production-Ready Minimal

**Goal**: Harden MVP for production deployment with monitoring, logging, strict security, and comprehensive error handling (no new product features).

**Deliverables**:

#### [ ] F-P1-001: RFC 9457 Problem Details Everywhere
- Standardize gateway error responses as `application/problem+json` per RFC 9457 with stable GTS `type` identifiers.
- Implement all error types listed in DESIGN error table (even if triggered by later phases) with correct HTTP codes and retriable hints.
- Include `X-OAGW-Error-Source` for all error responses (gateway vs upstream) (ADR: Error Source Distinction).
- Register standard errors in OpenAPI for Management + Proxy operations (ModKit `standard_errors()` patterns).

#### [ ] F-P1-002: AuthN/Z + Tenant Scoping (Management + Proxy)
- Enforce bearer auth on all endpoints using ModKit auth extractors (requires F-P0-001).
- Implement permission checks exactly as specified (Management permissions per resource type; Proxy requires `gts.x.core.oagw.proxy.v1~:invoke`).
- Ensure DB access always uses `SecurityContext` tenant scope; deny-by-default on empty scopes (SECURE-ORM invariant).
- Add scenario coverage: `scenarios/case-1.1-management-bearer-auth.md`, `scenarios/case-1.2-management-permissions.md`, `scenarios/case-5.1-proxy-invoke-permission.md`.

#### [ ] F-P1-003: Secure ORM Repository Hardening (No Raw SQL)
- Convert all lookups/matching queries to Secure ORM query builders (SeaORM + SecureConn), keeping tenant/resource scoping explicit.
- Add repository tests proving: cross-tenant reads/writes are impossible; empty scope returns deny-all results.
- Validate that route matching query respects `enabled`, `priority`, and longest-path semantics without unsafe SQL string building.
- Address design-review finding: treat SQL in DESIGN as illustrative only, not implementation.

#### [ ] F-P1-004: Outbound HTTP Client Reliability (Pooling + Timeouts)
- Use a shared HTTP client with connection pooling, keepalive, and sane defaults (no per-request client construction).
- Implement timeout policy: connection timeout, request timeout, idle timeout mapped to specific error types (504 variants).
- Implement safe cancellation propagation: client disconnect cancels upstream request promptly, avoiding leaked tasks.
- Add scenario coverage: `scenarios/case-10.1-timeout-guard.md`.

#### [ ] F-P1-005: Structured Audit Logging (Proxy + Config Changes)
- Emit structured JSON logs for proxy requests and management CRUD actions with stable fields (tenant_id, principal_id, host, path, status, duration_ms).
- Guarantee “no secrets/PII” invariant: never log headers/bodies; redact sensitive fields; rate-limit auth failure logs.
- Add logging for security events: forbidden upstream access, validation rejects, rate-limit rejects.
- Ensure plugin-generated logs (when added later) pass through a redaction/size-limiting layer by default.

#### [ ] F-P1-006: Metrics + /metrics Endpoint (Auth-Protected)
- Expose Prometheus metrics at `/metrics` with admin-only access.
- Implement core counters/gauges/histograms from DESIGN (requests_total, duration_seconds, errors_total, in_flight, rate_limit metrics).
- Enforce cardinality limits: no tenant labels, normalized path from route config.
- Add scenario coverage for metric presence/sanity (smoke tests; full dashboards later).

#### [ ] F-P1-007: Header/Request Smuggling Defenses (Strict Parsing)
- Reject invalid header names/values and all line terminators (CR/LF plus Unicode separators); disallow obs-fold.
- Reject multiple `Content-Length`, multiple `Host`, and ambiguous CL/TE combinations; ensure hyper/HTTP stack is configured to be strict.
- Strip/override any internal steering headers used by OAGW before forwarding to upstream.
- Add scenario coverage: `scenarios/case-7.2-hop-by-hop-headers-stripped.md`, `scenarios/case-7.4-content-length-validation.md`, `scenarios/case-8.2-transfer-encoding-validation.md`.

#### [ ] F-P1-008: SSRF Guardrails (DNS/IP + Scheme Rules)
- Enforce endpoint allow/deny rules (no link-local/private IPs unless explicitly allowed), and validate resolved IPs per request.
- Define safe semantics for any protocol capability caches: key by upstream endpoint + resolved IP; avoid cross-tenant coupling.
- Harden multi-endpoint host selection to prevent Host-header spoof steering (prefer dedicated internal header, validated and stripped).
- Add scenario coverage for alias + host header behaviors: `scenarios/case-6.3-common-suffix-alias-host-header-required.md`.

#### [ ] F-P1-009: OpenAI “Usable in Prod” E2E Test Suite
- Add integration tests covering: management create upstream/route, proxy request to OpenAI-compatible mock, SSE streaming, rate limiting, error mapping.
- Use scenario markdown as acceptance criteria and keep a one-to-one mapping to test cases (no silent scope drift).
- Add load-smoke: sustained small RPS for 10 minutes, assert no memory growth and stable p95 latency.
- Document operational runbook basics (env vars, DB migration step, health checks).

**Exclusions** (explicitly out of scope for p1):
- New protocols (WebSocket/WebTransport/gRPC), CORS, custom plugins, hierarchical configuration
- Circuit breakers/backpressure queues/distributed rate limits (these are p2+)
- OData `$filter/$select/$orderby` enhancements (p3)

---

### Phase 2 (p2): Scalability & Operational Maturity

**Goal**: Scale under load, degrade gracefully on failures, and add operational controls for reliable day-2 operation.

**Deliverables**:

#### [ ] F-P2-001: Circuit Breaker (Config + Enforcement)
- Implement circuit breaker states and transitions per [ADR: Circuit Breaker](./docs/adr-circuit-breaker.md).
- Add upstream/route config fields for breaker thresholds and fallback behavior; expose in OpenAPI schemas.
- Record breaker metrics (state gauge, transitions counter) and log state changes as audit events.
- Ensure half-open probe gating is atomic in distributed mode (avoid multi-node probe floods).

#### [ ] F-P2-002: Concurrency Control (In-Flight Limits)
- Implement per-scope in-flight request limits per [ADR: Concurrency Control](./docs/adr-concurrency-control.md).
- Support local limiter first; document limits/risks and optionally support a distributed coordinator for strict enforcement.
- Return `503 LinkUnavailable` when saturated; include Retry-After guidance where applicable.
- Add scenario coverage: `scenarios/case-12.6-no-automatic-retries.md` (no retry behaviors) + concurrency cases from ADR.

#### [ ] F-P2-003: Backpressure Queueing (Bounded, Worker-Pool)
- Implement queueing strategy per [ADR: Backpressure](./docs/adr-backpressure-queueing.md) with explicit bounds (queue length, worker count).
- Ensure consumers do not unboundedly `spawn`; use a fixed worker pool and cooperative cancellation.
- Add graceful degradation strategies (reject/queue/degrade) selectable per route.
- Emit queue metrics (depth, dropped, latency) and log overload events.

#### [ ] F-P2-004: Multi-Endpoint Load Balancing + Health
- Implement round-robin distribution across compatible endpoint pools (same scheme/port/protocol) with endpoint-level stats.
- Add endpoint health tracking and temporary ejection on repeated failures; expose `oagw_upstream_available` gauge.
- Coordinate selection with connection pools to avoid slow-endpoint pile-ups; bias away from saturated endpoints.
- Add scenario coverage: `scenarios/case-2.10-load-balancing-round-robin.md`.

#### [ ] F-P2-005: HTTP/2 Negotiation + Safe Capability Cache
- Implement adaptive HTTP/2→HTTP/1.1 fallback with cached capability per upstream endpoint (TTL 1h) as described in DESIGN.
- Define cache key semantics explicitly (endpoint_id + resolved_ip + ALPN result) and invalidation triggers (DNS change, repeated failures).
- Ensure cache is tenant-safe and cannot be used for cross-tenant inference.
- Add scenario coverage: `scenarios/case-12.4-http2-negotiation-fallback.md`.

#### [ ] F-P2-006: Config Caching + Invalidation
- Cache effective upstream/route config in-memory to avoid DB reads on every proxy request (bounded size + TTL).
- Invalidate on management writes (upstream/route update/delete) and on tenant-scoped access changes where applicable.
- Add metrics for cache hit/miss and a protected admin endpoint to inspect cache stats.
- Ensure cache stores only non-secret configuration (secrets still fetched via `cred_store`).

#### [ ] F-P2-007: Graceful Shutdown + Draining
- Implement coordinated shutdown: stop accepting new requests, drain in-flight, and cancel background tasks using cancellation tokens.
- Ensure streaming connections terminate cleanly with bounded timeouts (SSE/WebSocket later phases).
- Add health/readiness endpoints reflecting draining state and dependency health (DB, cred_store, types_registry).
- Add chaos test: SIGTERM during high traffic, verify no deadlocks/leaks.

#### [ ] F-P2-008: Operational Admin Surface (Protected)
- Add admin-only endpoints to inspect circuit breaker state, limiter state, queue depth, and recent error counts.
- Provide safe redaction: never expose secrets, raw request/response bodies, or tenant-specific identifiers unless authorized.
- Emit audit log entries for admin reads of sensitive operational state.
- Document minimal operational playbooks (overload, upstream outage, breaker stuck open).

**Exclusions** (explicitly out of scope for p2):
- Hierarchical configuration sharing/override model
- Starlark custom plugins and plugin CRUD APIs
- WebSocket and WebTransport proxying, plus CORS (p3)
- gRPC proxying and transcoding (p4)

---

### Phase 3 (p3): Advanced Product Features / Enterprise

**Goal**: Add enterprise flexibility: multi-tenant hierarchical configuration, plugin extensibility, additional protocols, and full management query capabilities.

**Deliverables**:

#### [ ] F-P3-001: Tenant Hierarchy Awareness (Core)
- Integrate tenant hierarchy resolution for requests (child→parent→root) for config discovery and authorization decisions.
- Ensure all DB reads are tenant-scoped via Secure ORM, with explicit “visible to tenant” semantics.
- Implement “enabled inheritance”: ancestor disable disables for all descendants, including list/discovery behavior.
- Add scenario coverage: `scenarios/case-5.2-proxy-cannot-access-unshared-upstream.md`, `scenarios/case-5.3-upstream-sharing-ancestor-to-descendant.md`.

#### [ ] F-P3-002: Alias Resolution + Shadowing (Hierarchy)
- Implement alias resolution algorithm walking tenant hierarchy with shadowing (closest tenant wins).
- Enforce alias uniqueness per tenant and correct behavior for non-standard ports and IP endpoints.
- Handle multi-endpoint common-suffix alias selection safely (require validated steering input, not raw Host header).
- Add scenario coverage: `scenarios/case-6.1-alias-shadowing.md`, `scenarios/case-6.4-alias-not-found.md`, `scenarios/case-2.2-alias-nonstandard-port.md`, `scenarios/case-2.3-ip-endpoint-requires-alias.md`.

#### [ ] F-P3-003: Hierarchical Configuration Sharing Modes (Upstream + Route)
- Implement `private|inherit|enforce` sharing modes for auth, rate limits, and plugins as specified in DESIGN.
- Implement merge strategies: auth override gated by permission; rate limits merged by `min(enforced_ancestor, descendant)`; plugin lists append-only under inherit/enforce.
- Add explicit override permissions (`oagw:upstream:bind`, `override_auth`, `override_rate`, `add_plugins`) and enforce them.
- Add scenario coverage: `scenarios/case-9.8-auth-sharing-modes.md`, `scenarios/case-9.9-descendant-override-permissions.md`, `scenarios/case-18.6-rate-limit-hierarchy-min-merge.md`.

#### [ ] F-P3-004: Resource Identification & Binding Model Alignment
- Implement the resource identification/binding semantics per [ADR: Resource Identification and Discovery](./docs/adr-resource-identification.md).
- Resolve the “single upstream table vs binding entity” mismatch: either introduce explicit binding records or formalize the chosen model in persistence/API.
- Ensure auditability: record which tenant/ancestor supplied each effective field (optional debug output for admins).
- Add migration + API changes as needed to support bindings without breaking p0/p1 contracts.

#### [ ] F-P3-005: Rate Limiting Advanced Semantics
- Support additional rate limit shapes per [ADR: Rate Limiting](./docs/adr-rate-limiting.md): cost, capacity, multiple windows, and scope variants (tenant/user/ip).
- Support strategy variants (reject/queue/degrade) and budget modes where specified in scenarios.
- Add stable response headers behavior (enable/disable; Retry-After; optional remaining tokens) without leaking tenant info.
- Add scenario coverage: `scenarios/case-18.3-rate-limit-scope-variants.md`, `scenarios/case-18.4-rate-limit-cost.md`, `scenarios/case-18.5-rate-limit-strategy-variants.md`.

#### [ ] F-P3-006: Plugin Framework (Builtin + Custom)
- Implement plugin chain composition (upstream plugins then route plugins) and execution order: Auth → Guards → Transform(on_request) → Upstream → Transform(on_response/on_error).
- Implement plugin identifier resolution: named builtin IDs vs anonymous UUID-backed custom plugins; enforce plugin_type matches schema.
- Define a consistent plugin lifecycle (immutable custom plugins; deletion only when unreferenced; GC eligibility tracking).
- Add scenario coverage: `scenarios/case-11.6-plugin-ordering-layering.md`, `scenarios/case-11.7-plugin-control-flow.md`.

#### [ ] F-P3-007: Starlark Runtime + Sandbox
- Embed Starlark execution with strict sandbox restrictions (no network/file IO, timeout, memory limits) as specified in DESIGN.
- Provide `ctx` API (request/response/error/config/route/log/time) with safe mutators; ensure short-circuit (`reject/respond`) cleans up permits/resources.
- Add log redaction and message size limits for `ctx.log.*` to prevent secret/PII leakage.
- Add scenario coverage: `scenarios/case-11.8-starlark-sandbox-restrictions.md`, `scenarios/case-10.4-custom-starlark-guard.md`.

#### [ ] F-P3-008: Plugin Persistence + Management API
- Add migrations + SeaORM entity for `oagw_plugin` (tenant-scoped, immutable source, config_schema JSONB, phases, lifecycle fields).
- Implement `/api/oagw/v1/plugins` endpoints (POST/GET list/GET by id/DELETE) and `/source` retrieval with strict permissions.
- Enforce delete semantics: return `409 PluginInUse` with `referenced_by` lists when referenced by upstream/route.
- Add scenario coverage: `scenarios/case-4.1-create-custom-guard-plugin.md`, `scenarios/case-4.2-plugin-immutability.md`, `scenarios/case-4.3-delete-plugin-only-when-unreferenced.md`.

#### [ ] F-P3-009: Plugin Usage Tracking + Garbage Collection Job
- Implement periodic reference scan to update `last_used_at` and set/clear `gc_eligible_at` (bounded work per tick).
- Implement background deletion of plugins past GC TTL with cancellation support and safe rate limiting.
- Expose minimal admin metrics/logs for GC activity (deleted_count, scan_duration).
- Add scenario coverage: `scenarios/case-4.7-plugin-usage-tracking-gc.md`.

#### [ ] F-P3-010: Builtin Plugin Suite (Auth/Guard/Transform)
- Auth builtins: noop, apikey, basic, bearer, oauth2 client credentials (incl basic client auth) with token caching/refresh support and no automatic request replay.
- Guard builtins: timeout enforcement, circuit breaker enforcement (bridging p2), CORS preflight validation.
- Transform builtins: request_id propagation, structured logging, metrics collection hooks (bridging p1).
- Add scenario coverage: `scenarios/case-9.3-auth-basic.md`, `scenarios/case-9.4-auth-bearer.md`, `scenarios/case-9.5-auth-oauth2-client-cred.md`, `scenarios/case-9.6-auth-oauth2-client-cred-basic.md`.

#### [ ] F-P3-011: CORS (Preflight + Policy Enforcement)
- Implement CORS policies configurable per upstream/route (origins/methods/headers/credentials), with local OPTIONS handling (no upstream round-trip).
- Enforce secure defaults (no wildcard-with-credentials), emit clear validation errors (Problem details).
- Ensure CORS evaluation occurs before sending requests upstream; log rejections without leaking request bodies.
- Add scenario coverage: `scenarios/case-10.2-cors-preflight.md`, `scenarios/case-10.3-cors-credentials-wildcard-invalid.md`.

#### [ ] F-P3-012: Protocol Expansion (WebSocket + WebTransport)
- Implement WebSocket proxying (upgrade, bi-directional streaming, auth on handshake, idle timeout, close propagation).
- Implement WebTransport session forwarding (`wt`) with auth at session establishment and bounded idle semantics.
- Add scenario coverage: `scenarios/case-14.1-websocket-upgrade-proxied.md`, `scenarios/case-17.1-webtransport-session-establishment.md`.

#### [ ] F-P3-013: Streaming Lifecycle Semantics (Non-HTTP/1)
- Define consistent lifecycle handling for long-lived streams: rate limit applied on establish, concurrency permits held/released, idle timeout enforcement.
- Implement client disconnect handling across WebSocket and WebTransport sessions with guaranteed cleanup.
- Map streaming aborts to `StreamAborted` with correct `X-OAGW-Error-Source` semantics where possible.
- Add scenario coverage: `scenarios/case-14.4-websocket-idle-timeout.md`, `scenarios/case-17.1-webtransport-session-establishment.md`.

#### [ ] F-P3-014: Full OData Query Support on List Endpoints
- Implement `$filter`, `$select`, `$orderby`, `$top`, `$skip` for upstream/route/plugin list endpoints using ModKit OData helpers.
- Implement field projection via `apply_select` / `page_to_projected_json` (docs/ODATA_SELECT.md), including dot-notation for nested JSON fields where supported.
- Enforce safe filter allowlists (no arbitrary SQL); validate field names and operations; return validation Problems on invalid queries.
- Add scenario coverage: management list query behaviors (extend `scenarios/` as needed).

**Exclusions** (explicitly out of scope for p3):
- TLS pinning and mTLS (p4)
- gRPC proxying/transcoding (p4)
- Starlark stdlib extensions that require network I/O (p4)

---

### Phase 4 (p4): Nice-to-Have / Long Tail

**Goal**: Add advanced security and convenience features, plus long-tail protocol refinements and richer diagnostics.

**Deliverables**:

#### [ ] F-P4-001: TLS Certificate Pinning
- Add optional pin sets per upstream endpoint (SPKI/public key or certificate pins) with rotation-friendly configuration.
- Enforce pin checks on TLS handshake failures with clear `ProtocolError` Problems (no secret leakage).
- Provide safe admin diagnostics (pin mismatch counts, last failure time) without exposing pin material.
- Add scenario coverage (new): pin mismatch and rotation.

#### [ ] F-P4-002: Mutual TLS (mTLS) to Upstreams
- Support client certificate/key material via `cred_store` refs; attach per upstream endpoint.
- Implement secure defaults: minimum TLS version, certificate validation, SNI/ALPN correctness.
- Ensure secrets are never logged; add explicit audit events for mTLS configuration changes.
- Add scenario coverage (new): mTLS handshake success/failure.

#### [ ] F-P4-003: Distributed Tracing (OpenTelemetry)
- Integrate tracing spans across phases (routing/auth/guard/transform/upstream) and propagate trace context (`traceparent`) where applicable.
- Include `trace_id` in RFC 9457 Problem responses (as in DESIGN example) and in structured logs.
- Add sampling controls and cardinality safeguards for span attributes.
- Add scenario coverage (new): trace_id presence and correlation.

#### [ ] F-P4-004: gRPC Proxying + Optional JSON Transcoding
- Implement gRPC proxying (unary + server streaming) with content-type detection and error mapping (ADR: gRPC Support).
- Add optional gRPC JSON transcoding for selected routes (scenario-driven) with explicit schemas and strict validation.
- Ensure error mapping preserves upstream vs gateway source semantics and does not break streaming.
- Add caching of transcoding descriptors with bounded size and invalidation on config update.
- Add scenario coverage: `scenarios/case-15.1-grpc-unary-native-proxy.md`, `scenarios/case-15.2-grpc-server-streaming-proxy.md`, `scenarios/case-15.3-grpc-json-transcoding.md`, `scenarios/case-15.4-grpc-status-mapping-error-source.md`.

#### [ ] F-P4-005: WebTransport (wt) Advanced Refinements
- Improve p3 baseline with session migration/reconnect semantics where supported by client/upstream capabilities.
- Add deep observability for multiplexed streams (per-session counters, queue pressure, abort reasons).
- Validate p95 latency/error budgets for long-lived WT sessions under load.
- Add scenario coverage (new): WT reconnect and sustained-session load behavior.

#### [ ] F-P4-006: Starlark Standard Library Extensions (Carefully Scoped)
- Add safe, vetted extensions (e.g., deterministic caching helpers) without enabling general network/file I/O.
- If HTTP client support is added, gate it behind explicit allowlists and strict quotas; document security model clearly.
- Add tests for sandbox escape attempts and resource exhaustion boundaries.
- Reference future-dev item #4 for scope constraints.

#### [ ] F-P4-007: Advanced Metrics and Diagnostics
- Add per-plugin timing histograms, per-endpoint latency breakdown, and queue/circuit diagnostics dashboards.
- Provide “debug headers” mode for admins (e.g., selected route id, upstream id) with explicit opt-in and stripping.
- Add automated SLO checks (p95 latency budgets, error rate alerts) tied to Prometheus rules.
- Ensure all diagnostic output is safe-by-default for multi-tenant deployments.

#### [ ] F-P4-008: No-Automatic-Retry Invariant
- Keep OAGW behavior strict: no automatic retries in core or plugins.
- Ensure scenario `scenarios/case-12.6-no-automatic-retries.md` remains true across all phases.
- Document client-side retry responsibility and recommended backoff/jitter guidance for callers.

**Exclusions** (explicitly out of scope for p4):
- HTTP/3 (QUIC) support

---

## Implementation Tracking

**Phase Summary**:
- p0: 12 features (0/12 complete)
- p1: 9 features (0/9 complete)
- p2: 8 features (0/8 complete)
- p3: 14 features (0/14 complete)
- p4: 8 features (0/8 complete)

**Total**: 51 features across all phases

| Phase | Feature Count |
|------:|--------------:|
| p0    | 12            |
| p1    | 9             |
| p2    | 8             |
| p3    | 14            |
| p4    | 8             |
| Total | 51            |

---

## Traceability

- **PRD**: [PRD.md](./PRD.md)
- **ADRs**: 
  - [ADR: Resource Identification and Discovery](./docs/adr-resource-identification.md)
  - [ADR: CORS](./docs/adr-cors.md)
  - [ADR: SSRF Protection](./docs/adr-ssrf-protection.md)
  - [ADR: Rate Limiting](./docs/adr-rate-limiting.md)
