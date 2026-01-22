# OAGW Outbound API Gateway Design Document

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

## Context

CyberFabric needs a reliable way to manage outbound API calls to external services.
The Outbound API Gateway (OAGW) provides a centralized layer for routing, authentication, rate limiting, and monitoring of those requests.

OAGW provides:

- Routing: Directs requests to appropriate external services based on predefined rules and configurations.
- Authentication: Manages authentication mechanisms for secure communication with external services.
- Rate Limiting: Controls the rate of outgoing requests to prevent overloading external services.
- Monitoring and Logging: Tracks outbound requests for auditing and performance analysis.

## Architecture

Service Dependencies Map

| Dependency       | Purpose                                     |
|------------------|---------------------------------------------|
| `types_registry` | GTS schema/instance registration            |
| `cred_store`     | Secret material retrieval by UUID reference |
| `api_ingress`    | REST API hosting                            |
| `modkit-db`      | Database persistence                        |
| `modkit-auth`    | Authorization                               |

### Key Concepts

- **Upstream Service**: External services that the OAGW interacts with to fulfill API requests.
- **Route**: A defined path in the OAGW that maps incoming requests to specific upstream services.
- **Plugin**: Modular components that can be applied to requests for additional functionality (e.g., logging, transformation, authentication).

### Out of Scope

- **DNS Resolution**: IP pinning rules, allowed segments matching are out of scope for this document.
- **Plugin Versioning**: Plugin versioning and lifecycle management are out of scope for this document.
- **Response Caching**: OAGW does not cache responses. Caching is client/upstream responsibility.
- **Automatic Retries**: OAGW does not retry failed requests. Retry logic is client responsibility.

### Security Considerations

**Server-Side Request Forgery (SSRF)**:

- DNS: IP pinning rules, allowed segments matching.
- Headers: Well-known headers stripping and validation.
- Request Validation: Path, query parameters validation against route configuration.

**Cross-Origin Resource Sharing (CORS)**:

CORS support is built-in, configured per upstream/route. Preflight OPTIONS requests handled locally (no upstream round-trip).

See [ADR: CORS](./docs/adr-cors.md) for configuration options and security considerations.

**HTTP Version Negotiation**:

OAGW uses adaptive per-host HTTP version detection:

1. **First request**: Attempt HTTP/2 via ALPN during TLS handshake
2. **Success**: Cache "HTTP/2 supported" for this host/IP
3. **Failure**: Fallback to HTTP/1.1, cache "HTTP/1.1 only" for this host/IP
4. **Subsequent requests**: Use cached protocol version

Cache entry TTL: 1 hour. Automatic retry on connection errors.

HTTP/3 (QUIC) support is future work.

**Inbound Authentication & Authorization**

All OAGW API requests require Bearer token authentication.

**Management API** (`/api/oagw/v1/upstreams`, `/api/oagw/v1/routes`, `/api/oagw/v1/plugins`):

| Permission Required                                          | Description                            |
|--------------------------------------------------------------|----------------------------------------|
| `gts.x.core.oagw.upstream.v1~:{create;override;read;delete}` | Create/Override, read, delete upstream |
| `gts.x.core.oagw.route.v1~:{create;override;read;delete}`    | Create/Override, read, delete route    |
| `gts.x.core.oagw.plugin.auth.v1~:{create;read;delete}`       | Create, read, delete auth plugin       |
| `gts.x.core.oagw.plugin.guard.v1~:{create;read;delete}`      | Create, read, delete guard plugin      |
| `gts.x.core.oagw.plugin.transform.v1~:{create;read;delete}`  | Create, read, delete transform plugin  |

**Proxy API** (`/api/oagw/v1/proxy/{alias}/*`):

| Permission Required                | Description                 |
|------------------------------------|-----------------------------|
| `gts.x.core.oagw.proxy.v1~:invoke` | Proxy requests to upstreams |

Authorization checks:

1. Token must have `gts.x.core.oagw.proxy.v1~:invoke` permission
2. Upstream must be owned by token's tenant or shared by ancestor
3. Route must match request method and path

**Outbound Authentication** (OAGW → Upstream):

Handled by auth plugins. Token refresh, caching, and retry on 401 managed automatically by builtin auth plugins. No manual token management required.

**Credential Management**:

API keys, OAuth2 credentials, and secrets stored in `cred_store`. Rotation, revocation, and expiration policies managed by `cred_store`, not OAGW.

**Retry Policy**:

OAGW does not retry failed requests. Clients responsible for retry logic. Auth plugins handle token refresh on 401, but do not retry the original request.

## Core Subsystems

### Request Routing

#### Routing Flow

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
- gRPC: 443
- AMQP: 5672

**Non-standard ports** (included in alias): Any port not in standard list.

#### Resolution Algorithm

```
def resolve_upstream_by_alias(tenant_id, alias, req):
    # Walk tenant hierarchy from descendant to root
    hierarchy = get_tenant_hierarchy(tenant_id)  # [child, parent, grandparent, root]
    
    for tid in hierarchy:
        upstream = find_upstream_by_alias(tid, alias)
        
        if upstream is not None:
            # Multiple endpoints with common suffix alias require Host header
            if len(upstream.endpoints) > 1:
                has_common_suffix = any(
                    ep.host != alias and ep.host.endswith("." + alias)
                    for ep in upstream.endpoints
                )
                
                if has_common_suffix and "Host" not in req.headers:
                    return return Response(status=400) # Missing Host header
            
            return upstream
    
    return Response(status=404)  # Not found
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

**Plugins Configuration**:

| Ancestor Sharing | Descendant Specifies | Effective Plugin Chain                  |
|------------------|----------------------|-----------------------------------------|
| `private`        | —                    | Descendant's plugins only               |
| `inherit`        | No                   | Use ancestor's plugins                  |
| `inherit`        | Yes                  | `ancestor.plugins + descendant.plugins` |
| `enforce`        | Any                  | `ancestor.plugins + descendant.plugins` |

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
      "description": "Human-readable routing identifier. Auto-generated if not specified: single host with standard port (80,443,5672) → hostname; single host with non-standard port → hostname:port; multiple hosts with common suffix → common suffix (e.g., us.vendor.com + eu.vendor.com → vendor.com); IP addresses or heterogeneous hosts → explicit alias required."
    },
    "tags": {
      "type": "array",
      "items": {
        "type": "string",
        "pattern": "^[a-z0-9_-]+$"
      },
      "description": "Flat tags for categorization and discovery (e.g., openai, llm)."
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
                "enum": [ "https", "wss", "wt", "amqp", "grpc" ],
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
        "gts.x.core.oagw.protocol.v1~x.core.amqp.v1",
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
      "description": "Protocol-scoped inbound matching rules. Exactly one of {http|grpc|amqp} must be present.",
      "additionalProperties": false,
      "properties": {
        "http": { "$ref": "#/definitions/http_match" },
        "grpc": { "$ref": "#/definitions/grpc_match" },
        "amqp": { "$ref": "#/definitions/amqp_match" }
      },
      "oneOf": [
        { "required": [ "http" ] },
        { "required": [ "grpc" ] },
        { "required": [ "amqp" ] }
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
    "amqp_match": {
      "type": "object",
      "additionalProperties": false,
      "description": "AMQP match rules (used when the upstream protocol is AMQP).",
      "properties": {
        "exchange": {
          "type": "string",
          "minLength": 1,
          "description": "Exchange name to publish to or consume from, depending on your OAGW AMQP mode."
        },
        "routing_key": {
          "type": "string",
          "minLength": 1,
          "description": "Routing key pattern for matching/publishing."
        }
      },
      "required": [ "exchange", "routing_key" ]
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
- [AMQP publish](./examples/5.amqp.positive.md)

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
    tags       TEXT[] DEFAULT '{}',

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

    -- Match rules (JSONB, one of http/grpc/amqp)
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
- `WARN`: Rate limit exceeded, circuit breaker open, retries
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
