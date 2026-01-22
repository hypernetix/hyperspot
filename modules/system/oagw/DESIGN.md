# OAGW Outbound API Gateway Design Document

## Context

CyberFabric platform requires a robust and flexible solution for managing outbound API requests to various external services.
The Outbound API Gateway (OAGW) is designed to address this need by providing a centralized gateway that handles routing, authentication,
rate limiting, and monitoring of outbound API calls.

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
| `modkit-auth`    | SecurityCtx authorization                   |

### Key Concepts

- **Upstream Service**: External services that the OAGW interacts with to fulfill API requests.
- **Route**: A defined path in the OAGW that maps incoming requests to specific upstream services.
- **Plugin**: Modular components that can be applied to requests for additional functionality (e.g., logging, transformation, authentication).

### Out of Scope

- **Inbound Authentication**: The authentication is ingoing design state.
- **DNS Resolution**: IP pinning rules, allowed segments matching are out of scope for this document.
- **Plugin Versioning**: Plugin versioning and lifecycle management are out of scope for this document.

### Security Considerations

**Server-Side Request Forgery (SSRF)**:

- DNS: IP pinning rules, allowed segments matching.
- Headers: Well-known headers stripping and validation.
- Request Validation: Path, query parameters validation against route configuration.

**Cross-Origin Resource Sharing (CORS)**:

### Routing

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

```go
func handleRequest(req Request) Response {
    // 1. Resolve upstream by alias from URL path
    upstream, ok := resolveUpstreamByAlias(req.tenant, req.alias)
    if !ok {
        return Response{ Status: 404, Code: "UPSTREAM_NOT_FOUND" }
    }

    // 2. Match route by (upstream_id, method, path)
    route, ok := matchRoute(upstream.id, req.method, req.pathSuffix)
    if !ok {
        return Response{ Status: 404, Code: "ROUTE_NOT_FOUND" }
    }

    // 3. Check inbound authentication/authorization
    if !authorizeRequest(req, route) {
        return Response{ Status: 403 }
    }

    // 4. Get tenant-specific configuration
    tenantConfig := getTenantConfig(req.tenant, route.id, upstream.id)

    // 5. Apply configuration layering: Upstream < Route < Tenant
    finalConfig := mergeConfigs(
        upstream.config(),
        route.config(),
        tenantConfig,
    )

    // 6. Build plugin chain based on final configuration
    pluginChain := buildPluginChain(finalConfig.plugins)

    // 7. Prepare outbound request based on final configuration
    outboundReq := prepareRequest(req, finalConfig)

    // 8. Execute plugin chain with outbound request
    return pluginChain.execute(outboundReq)
}
```

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

**Headers Transformation**:

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

Transformation Rules

| Inbound      | Outbound | Rule                                                             |
|--------------|----------|------------------------------------------------------------------|
| Method       | Method   | Passthrough (must be in `match.http.methods`)                    |
| Path suffix  | Path     | Append to `match.http.path` if `path_suffix_mode`: `append`      |
| Query params | Query    | Validate against `match.http.query_allowlist`; reject if unknown |
| Headers      | Headers  | Apply `upstream.headers` transformation rules                    |
| Body         | Body     | Passthrough by default; transformable via plugin chain           |

### Alias Resolution

Upstreams are identified by alias in proxy requests: `{METHOD} /api/oagw/v1/proxy/{alias}/{path}`.

**Alias Auto-Generation Rules**:

| Scenario | Generated Alias | Example |
|----------|----------------|---------|
| Single host | hostname (no port) | `api.openai.com:443` → `api.openai.com` |
| Multiple hosts with common suffix | common domain suffix | `us.vendor.com`, `eu.vendor.com` → `vendor.com` |
| IP addresses or heterogeneous hosts | must be explicit | `10.0.1.1`, `10.0.1.2` → user provides `my-service` |

**Resolution Order** (shadowing):

When resolving alias, OAGW walks tenant hierarchy from descendant to root. Closest match wins.

```
Request from: subsub-tenant
Alias: "vendor.com"

Search order:
1. subsub-tenant upstreams  ← wins if found
2. sub-tenant upstreams
3. root-tenant upstreams
```

**Multi-Endpoint Load Balancing**:

Multiple endpoints in same upstream form a pool. Requests are distributed across endpoints (round-robin). All endpoints must have:
- Same `protocol`
- Same `scheme` (https, wss, etc.)
- Same `port`

For detailed alias resolution and compatibility rules, see [ADR: Resource Identification and Discovery](./docs/adr-resource-identification.md).

### Plugins System

**Plugin Types**

- `gts.x.core.oagw.plugin.auth.v1~*` - Authentication plugin for credential injection. Only upstream level. One per upstream.
- `gts.x.core.oagw.plugin.guard.v1~*` - Validation and policy enforcement plugin. Can reject requests. Upstream/Route levels. Multiple per level.
- `gts.x.core.oagw.plugin.transform.v1~*` - Request/response transformation plugin. Upstream/Route levels. Multiple per level.

Plugins can be built-in or custom Starlark scripts.

**Plugin Naming Convention**

All builtin plugins use the `x.core.oagw.*` prefix (e.g., `~x.core.oagw.logging.v1`). Custom plugins use the `<vendor>.<system>.*` prefix (e.g., `~acme.billing.redact_pii.v1`).

**Plugin Layering**
Plugins can be applied at different levels:

- **Upstream Level**: Plugins that apply to all requests sent to a specific upstream service.
- **Route Level**: Plugins that apply to requests for a specific route.

**Plugin Execution Order**

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

```starlark
# Example Transform Plugin: Logging
# Type: gts.x.core.oagw.plugin.transform.v1~acme.custom.logging.v1

def on_request(ctx):
    ctx.log.info("request", {
        "method": ctx.request.method,
        "path": ctx.request.path,
        "tenant_id": ctx.request.tenant_id,
        "request_id": ctx.request.headers.get("X-Request-ID"),
    })
    return ctx.next()

def on_response(ctx):
    ctx.log.info("response", {
        "status": ctx.response.status,
        "latency_ms": ctx.time.elapsed_ms(),
    })
    return ctx.next()

def on_error(ctx):
    ctx.log.error("error", {
        "status": ctx.error.status,
        "code": ctx.error.code,
        "message": ctx.error.message,
        "upstream": ctx.error.upstream,
    })
    return ctx.next()
```

```starlark
# Example Guard Plugin: Request Validator
# Type: gts.x.core.oagw.plugin.guard.v1~acme.security.request_validator.v1

def on_request(ctx):
    # Guards can only implement on_request phase
    for h in ctx.config.get("required_headers", []):
        if not ctx.request.headers.get(h):
            return ctx.reject(400, "MISSING_HEADER", "Required header: " + h)
    
    if len(ctx.request.body) > ctx.config.get("max_body_size", 1048576):
        return ctx.reject(413, "BODY_TOO_LARGE", "Body exceeds limit")
    
    return ctx.next()
```

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
- **Rate Limits** (`rate_limit.sharing`): Rate limiting rules (rate, window, capacity, scope)
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

```go
func resolveEffectiveConfig(tenantID, upstreamID string) EffectiveConfig {
    // 1. Walk tenant hierarchy from descendant to root
    hierarchy := getTenantHierarchy(tenantID) // [child, parent, grandparent, root]
    
    // 2. Collect bindings for this upstream across hierarchy
    bindings := []Binding{}
    for _, tid := range hierarchy {
        if b := findBinding(tid, upstreamID); b != nil {
            bindings = append(bindings, b)
        }
    }
    
    // 3. Merge from root to child (root is base, child overrides)
    result := EffectiveConfig{}
    for i := len(bindings) - 1; i >= 0; i-- {
        b := bindings[i]
        isOwn := (i == 0)
        
        // Auth - check sharing mode
        if b.Auth != nil && b.Auth.Sharing != "private" {
            if isOwn && b.Auth.SecretRef != "" {
                result.Auth = b.Auth // descendant overrides
            } else if result.Auth == nil || b.Auth.Sharing == "enforce" {
                result.Auth = b.Auth // ancestor's auth applies
            }
        }
        
        // Rate limit - merge with min() strategy
        result.RateLimit = mergeRateLimit(result.RateLimit, b.RateLimit, isOwn)
        
        // Plugins - concatenate chains
        result.Plugins = mergePlugins(result.Plugins, b.Plugins, isOwn)
    }
    
    return result
}

func mergeRateLimit(ancestor, descendant *RateLimitConfig, isOwn bool) *RateLimitConfig {
    if ancestor == nil {
        return descendant
    }
    if descendant == nil {
        if ancestor.Sharing == "private" && !isOwn {
            return nil
        }
        return ancestor
    }
    
    // Both exist - take stricter (minimum rate)
    if ancestor.Sharing == "enforce" || ancestor.Sharing == "inherit" {
        return &RateLimitConfig{
            Rate:   min(ancestor.Rate, descendant.Rate),
            Window: ancestor.Window,
        }
    }
    return descendant
}

func mergePlugins(ancestor, descendant *PluginsConfig, isOwn bool) []string {
    result := []string{}
    
    // Add ancestor plugins if shared
    if ancestor != nil && ancestor.Sharing != "private" {
        result = append(result, ancestor.Items...)
    }
    
    // Add descendant plugins
    if descendant != nil {
        result = append(result, descendant.Items...)
    }
    
    return result
}
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
    "rate": 10000,
    "window": "minute"
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
    "rate": 100,
    "window": "minute"
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
    "rate": 100,
    "window": "minute",
    "note": "min(partner.enforce:10000, customer:100) = 100"
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

## Types Definitions

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
    "alias": {
      "type": "string",
      "pattern": "^[a-z0-9]([a-z0-9.-]*[a-z0-9])?$",
      "description": "Human-readable routing identifier. Auto-generated if not specified: single host → hostname; multiple hosts with common suffix → common suffix (e.g., us.vendor.com + eu.vendor.com → vendor.com); IP addresses or heterogeneous hosts → explicit alias required."
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
            "type": "string",
            "format": "gts-identifier"
          },
          "description": "List of plugins applied to this upstream service."
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
        "rate": {
          "type": "integer",
          "minimum": 1,
          "description": "Number of requests allowed per window."
        },
        "window": {
          "type": "string",
          "enum": [ "second", "minute", "hour", "day" ],
          "default": "minute",
          "description": "Time window for the rate."
        },
        "capacity": {
          "type": "integer",
          "minimum": 1,
          "description": "Maximum burst size (bucket capacity). Defaults to 'rate' if not specified."
        },
        "cost": {
          "type": "integer",
          "minimum": 1,
          "default": 1,
          "description": "Tokens consumed per request. Useful for weighted endpoints."
        },
        "scope": {
          "type": "string",
          "enum": [ "global", "tenant", "user", "ip" ],
          "default": "tenant",
          "description": "Scope for rate limit counters."
        },
        "strategy": {
          "type": "string",
          "enum": [ "reject", "queue", "degrade" ],
          "default": "reject",
          "description": "Behavior when limit exceeded."
        }
      },
      "required": [ "rate" ]
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
        "rate": {
          "type": "integer",
          "minimum": 1,
          "description": "Number of requests allowed per window."
        },
        "window": {
          "type": "string",
          "enum": [ "second", "minute", "hour", "day" ],
          "default": "minute",
          "description": "Time window for the rate."
        },
        "capacity": {
          "type": "integer",
          "minimum": 1,
          "description": "Maximum burst size (bucket capacity). Defaults to 'rate' if not specified."
        },
        "cost": {
          "type": "integer",
          "minimum": 1,
          "default": 1,
          "description": "Tokens consumed per request. Useful for weighted endpoints."
        },
        "scope": {
          "type": "string",
          "enum": [ "global", "tenant", "user", "ip" ],
          "default": "tenant",
          "description": "Scope for rate limit counters."
        },
        "strategy": {
          "type": "string",
          "enum": [ "reject", "queue", "degrade" ],
          "default": "reject",
          "description": "Behavior when limit exceeded."
        }
      },
      "required": [ "rate" ]
    }
  }
}
```

### Auth Plugin

**Base type**: `gts.x.core.oagw.plugin.auth.v1~`

Auth plugins handle outbound authentication to upstream services. Only one auth plugin per upstream.

**Schema**:

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

| Plugin ID                                                        | Description                 |
|------------------------------------------------------------------|-----------------------------|
| `gts.x.core.oagw.plugin.guard.v1~x.core.oagw.timeout.v1`         | Request timeout enforcement |
| `gts.x.core.oagw.plugin.guard.v1~x.core.oagw.circuit_breaker.v1` | Circuit breaker pattern     |
| `gts.x.core.oagw.plugin.guard.v1~x.core.oagw.cors.v1`            | CORS preflight validation   |

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
| `gts.x.core.oagw.plugin.transform.v1~x.core.oagw.retry.v1`      | response, error          | Retry on failure                   |
| `gts.x.core.oagw.plugin.transform.v1~x.core.oagw.request_id.v1` | request, response        | X-Request-ID injection/propagation |

**Starlark Plugin Context API**:

```starlark
# ctx.request (on_request phase)
ctx.request.method              # str: "GET", "POST", etc.
ctx.request.path                # str: "/v1/chat/completions"
ctx.request.query               # dict: {"version": "2"}
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
  "id": "gts.x.core.oagw.plugin.guard.v1~acme.security.request_validator.v1",
  "type": "gts.x.core.oagw.plugin.type.v1~x.core.oagw.starlark.v1",
  "config_schema": {
    "type": "object",
    "properties": {
      "max_body_size": { "type": "integer", "default": 1048576 },
      "required_headers": { "type": "array", "items": { "type": "string" } }
    }
  },
  "source_ref": "/api/oagw/v1/plugins/gts.x.core.oagw.plugin.guard.v1~acme.security.request_validator.v1/source"
}
```

**Plugin Source** (fetched via `GET {source_ref}`):

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
  "id": "gts.x.core.oagw.plugin.transform.v1~acme.billing.redact_pii.v1",
  "type": "gts.x.core.oagw.plugin.type.v1~x.core.oagw.starlark.v1",
  "phase": [ "on_response" ],
  "config_schema": {
    "type": "object",
    "properties": {
      "fields": { "type": "array", "items": { "type": "string" } }
    }
  },
  "source_ref": "/api/oagw/v1/plugins/gts.x.core.oagw.plugin.transform.v1~acme.billing.redact_pii.v1/source"
}
```

**Plugin Source** (fetched via `GET {source_ref}`):

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

## API Endpoints

### Upstream Endpoints

`POST /api/oagw/v1/upstreams`
`GET /api/oagw/v1/upstreams/{upstream_id}`
`PUT /api/oagw/v1/upstreams/{upstream_id}`
`DELETE /api/oagw/v1/upstreams/{upstream_id}`

### Route Endpoints

`POST /api/oagw/v1/routes`
`GET /api/oagw/v1/routes/{route_id}`
`PUT /api/oagw/v1/routes/{route_id}`
`DELETE /api/oagw/v1/routes/{route_id}`

### Plugin Endpoints

`POST /api/oagw/v1/plugins`
`GET /api/oagw/v1/plugins/{plugin_id}`
`PUT /api/oagw/v1/plugins/{plugin_id}`
`DELETE /api/oagw/v1/plugins/{plugin_id}`
`GET /api/oagw/v1/plugins/{plugin_id}/source`
`PUT /api/oagw/v1/plugins/{plugin_id}/source`

### Proxy Endpoint

`{METHOD} /api/oagw/v1/proxy/{alias}[/{path_suffix}][?{query_parameters}]`

Where:

- `{alias}` - Upstream alias (e.g., `api.openai.com` or `my-internal-service`)
- `{path_suffix}` - Path to match against route's `match.http.path` pattern
- `{query_parameters}` - Query params validated against route's `match.http.query_allowlist`

### API Call Examples

- [Plain HTTP Request/Response](./examples/1.plain_http.md)
- [Server-Sent Events (SSE)](./examples/2.sse.md)
- [Streaming WebSockets](./examples/3.websocket.md)
- [Streaming gRPC](./examples/4.grpc.md)

## Error Handling

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

## Use Cases

## To Be Covered

1. Configuration REST | Static Type Registry | Database persistence
2. Protocol Negotiation (HTTP/1.1, HTTP/2, HTTP/3, gRPC)
3. Authentication
4. Plugin Versioning and Lifecycle Management
5. Cache Management
6. TLS certificate pinning
7. mTLS support
8. Rust ABI / Client Libraries for Requests and Plugin Development
9. Audit logging
10. Metrics

## Feedback To Be Covered

1. How to distinguish upstream error/body vs OAGW error/body? [ADR](./docs/adr-error-source-distinction.md)
2. Rate limiting strategies and algorithms
3. ~~Validation and Mutation Plugins~~
4. Query plugin transformation in plugin
5. Body validation rules
6. Per-Tenant Upstream and Route Overrides [ADR](./docs/adr-resource-identification.md)
7. Retry and Circuit Breaker Strategies Clarification
8. In-flight Limits and Backpressure Handling
9. Rate Limit as a plugin
10. Full custom overrides
11. Enable/disable upstreams on a tenant level
12. Question of uniqueness of alias
13. Use anonymous gts for upstream and route definitions

TODO: PRD create

root - is responsible

upstream (route) - {auth, alias, limits}
route (link)     - {path, query, limits}

POST /upstream

POST /route

GET /oagw/v1/api.openai.com/v1/completions

matching rules

api.openai.com - upstream.alias
/v1/completions - for route matching

api.openai.com:443

api.openai.com:8443

rules

1. tenant defines for itself
2. partner for itself and children |
3. parent for itself and children with shared creds |

HTTP api.openai.com:443 - OK
{
}

HTTP my.caw.com:443 - NO OK
{
}
