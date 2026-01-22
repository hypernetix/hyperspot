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
     │  Route Matching │ ─── Match by route_id from URL path
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
    // 1. Lookup route based on request
    route, ok := matchRoute(req)
    if !ok {
        return Response{ Status: 404 }
    }

    // 2. Get associated upstream and tenant configurations
    upstream := route.upstream()

    // 3. Check inbound authentication/authorization
    if !authorizeRequest(req, route) {
        return Response{ Status: 403 }
    }

    // 4. Get tenant-specific configuration
    tenantConfig := getTenantConifg(req.tenant, route.id, upstream.id)

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
    Inbound:  `POST /api/oagw/v1/proxy/{route_id}/models/gpt-4?version=2`
                                                 └─────┬─────┘ └───┬───┘
                                                  path_suffix    query
```

**Route Config**:

- match.http.path: `/v1/chat/completions`
- match.http.path_suffix_mode: `append`
- match.http.query_allowlist: `[version]`

```
    Outbound: POST https://api.openai.com/v1/chat/completions/models/gpt-4?version=2
                   └──────────┬──────────┘└────────┬────────┘└─────┬─────┘└───┬────┘
                      upstream.host           route.path      path_suffix   allowed query
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

### Plugins System

**Plugin Types**

- `gts.x.core.oagw.plugin.auth.v1~` - Authentication plugin, e.g., BYOK API Key injection. Only upstream level. One per upstream.
- `gts.x.core.oagw.plugin.filter.v1~` - Request/response filtering/transformation plugin. Upstream/Route levels. Multiple per level.

Plugins could be built-in discoverable plugins or custom plugins developed for specific use cases.

**Plugin Naming Convention**

All builtin plugins use the `x.core.oagw.*` prefix (e.g., `~x.core.oagw.logging.v1`). Custom plugins use the `<vendor>.<system>.*` prefix (e.g., `~acme.billing.redact_pii.v1`).

**Plugin Layering**
Plugins can be applied at different levels:

- **Upstream Level**: Plugins that apply to all requests sent to a specific upstream service.
- **Route Level**: Plugins that apply to requests for a specific route.

**Plugin Execution Order**

The order of plugin execution is determined by the layering of configurations, ensuring that tenant-specific plugins can override route and upstream plugins as needed.

```
  Final Plugin Chain Composition (config-resolution time)

  Upstream.plugins    Route.plugins
  [U1, U2]         +  [R1, R2]    =>  [U1, U2, R1, R2]
```

```Starlark
# Example Plugin: Logging
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

## Types Definitions

### Builtin Authentication Plugins

**Base type**: `gts.x.core.oagw.plugin.auth.v1~`

**Supported Types**:

- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.apikey.v1`
- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.basic.v1`
- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.oauth2.client_cred.v1`
- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.oauth2.client_cred_basic.v1`
- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.bearer.v1`

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
      "format": "gts-identifier",
      "examples": [
        "gts.x.core.oagw.upstream.v1~openai.api.rest.v1"
      ],
      "description": "Unique name of the upstream service."
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
          ],
          "description": "Authentication plugin type for the upstream service."
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
      "type": "array",
      "items": {
        "type": "string",
        "format": "gts-identifier"
      },
      "description": "List of plugins applied to this upstream service."
    },
    "rate_limit": {
      "$ref": "#/definitions/rate_limit",
      "description": "Rate limiting configuration for the route."
    }
  },
  "required": [ "id", "server", "protocol" ],
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
      "format": "gts-identifier",
      "description": "Unique name of the route."
    },
    "upstream": {
      "type": "string",
      "format": "gts-identifier",
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
      "type": "array",
      "items": {
        "type": "string",
        "format": "gts-identifier"
      },
      "default": [ ],
      "description": "List of plugins applied to this route."
    },
    "rate_limit": {
      "$ref": "#/definitions/rate_limit",
      "description": "Rate limiting configuration for the route."
    }
  },
  "required": [ "id", "upstream", "match" ],
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

### Filter Plugin

**Base type**: `gts.x.core.oagw.plugin.filter.v1~`

Filter plugins transform requests/responses. Multiple filter plugins per upstream/route, executed in order.

**Schema**:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "OAGW Filter Plugin",
  "type": "object",
  "properties": {
    "id": {
      "type": "string",
      "format": "gts-identifier",
      "examples": [
        "gts.x.core.oagw.plugin.filter.v1~x.core.oagw.logging.v1",
        "gts.x.core.oagw.plugin.filter.v1~acme.billing.redact_pii.v1"
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

**Builtin Filter Plugins**:

| Plugin ID                                                         | Phase                    | Description                        |
|-------------------------------------------------------------------|--------------------------|------------------------------------|
| `gts.x.core.oagw.plugin.filter.v1~x.core.oagw.logging.v1`         | request, response, error | Request/response logging           |
| `gts.x.core.oagw.plugin.filter.v1~x.core.oagw.metrics.v1`         | request, response        | Prometheus metrics                 |
| `gts.x.core.oagw.plugin.filter.v1~x.core.oagw.timeout.v1`         | request                  | Request timeout enforcement        |
| `gts.x.core.oagw.plugin.filter.v1~x.core.oagw.retry.v1`           | response, error          | Retry on failure                   |
| `gts.x.core.oagw.plugin.filter.v1~x.core.oagw.circuit_breaker.v1` | request, error           | Circuit breaker pattern            |
| `gts.x.core.oagw.plugin.filter.v1~x.core.oagw.cors.v1`            | request, response        | CORS handling                      |
| `gts.x.core.oagw.plugin.filter.v1~x.core.oagw.request_id.v1`      | request, response        | X-Request-ID injection/propagation |

**Starlark Filter Plugin Context API**:

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

**Example: Custom Filter Plugin Definition**:

```json
{
  "id": "gts.x.core.oagw.plugin.filter.v1~acme.security.request_validator.v1",
  "type": "gts.x.core.oagw.plugin.type.v1~x.core.oagw.starlark.v1",
  "phase": [ "on_request" ],
  "config_schema": {
    "type": "object",
    "properties": {
      "max_body_size": { "type": "integer", "default": 1048576 },
      "required_headers": { "type": "array", "items": { "type": "string" } }
    }
  },
  "source_ref": "/api/oagw/v1/plugins/gts.x.core.oagw.plugin.filter.v1~acme.security.request_validator.v1/source"
}
```

**Plugin Source** (fetched via `GET {source_ref}`):

```starlark
def on_request(ctx):
    for h in ctx.config.get("required_headers", []):
        if not ctx.request.headers.get(h):
            return ctx.reject(400, "MISSING_HEADER", "Required header: " + h)
    
    if len(ctx.request.body) > ctx.config.get("max_body_size", 1048576):
        return ctx.reject(413, "BODY_TOO_LARGE", "Body exceeds limit")
    
    return ctx.next()

def on_response(ctx):
    return ctx.next()

def on_error(ctx):
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

`{METHOD} /api/oagw/v1/proxy/{route_id}[/{path_suffix}][?{query_parameters}]`

### API Call Examples

- [Plain HTTP Request/Response](./examples/1.plain_http.md)
- [Server-Sent Events (SSE)](./examples/2.sse.md)
- [Streaming WebSockets](./examples/3.websocket.md)
- [Streaming gRPC](./examples/4.grpc.md)

## Error Handling

| Error Type      | HTTP | GTS Instance ID                                       | Retriable |
|-----------------|------|-------------------------------------------------------|-----------|
| RouteError      | 400  | `gts.x.core.errors.err.v1~x.oagw.validation.error.v1` | No        |
| ValidationError | 400  | `gts.x.core.errors.err.v1~x.oagw.validation.error.v1` | No        |
| RouteNotFound   | 404  | `gts.x.core.errors.err.v1~x.oagw.route.not_found.v1`  | No        |

| AuthenticationFailed | 401 | `gts.x.core.errors.err.v1~x.oagw.auth.failed.v1`          | No |

| PayloadTooLarge | 413 | `gts.x.core.errors.err.v1~x.oagw.payload.too_large.v1`    | No |
| RateLimitExceeded | 429 | `gts.x.core.errors.err.v1~x.oagw.rate_limit.exceeded.v1`  | Yes*      |

| SecretNotFound | 500 | `gts.x.core.errors.err.v1~x.oagw.secret.not_found.v1`     | No |
| ProtocolError | 502 | `gts.x.core.errors.err.v1~x.oagw.protocol.error.v1`       | No |
| DownstreamError | 502 | `gts.x.core.errors.err.v1~x.oagw.downstream.error.v1`     | Depends |
| StreamAborted | 502 | `gts.x.core.errors.err.v1~x.oagw.stream.aborted.v1`       | No**      |
| LinkUnavailable | 503 | `gts.x.core.errors.err.v1~x.oagw.link.unavailable.v1`     | Yes |
| CircuitBreakerOpen | 503 | `gts.x.core.errors.err.v1~x.oagw.circuit_breaker.open.v1` | Yes |
| ConnectionTimeout | 504 | `gts.x.core.errors.err.v1~x.oagw.timeout.connection.v1`   | Yes |
| RequestTimeout | 504 | `gts.x.core.errors.err.v1~x.oagw.timeout.request.v1`      | Yes |
| IdleTimeout | 504 | `gts.x.core.errors.err.v1~x.oagw.timeout.idle.v1`         | Yes |
| PluginNotFound | 503 | `gts.x.core.errors.err.v1~x.oagw.plugin.not_found.v1`     | No |

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
3. Per-Tenant Upstream and Route Overrides
4. Retry and Circuit Breaker Strategies Clarification