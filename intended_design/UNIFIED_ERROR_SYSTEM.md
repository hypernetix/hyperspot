# Unified Error System

This document defines the unified error handling system for CyberFabric. All REST API errors MUST have a GTS type identifier, trace ID, and be registered in the types registry.

## Table of Contents

1. [Requirements](#requirements)
2. [GTS Error Identifier Format](#gts-error-identifier-format)
3. [Problem Response Schema](#problem-response-schema)
4. [Error Definition](#error-definition)
5. [Implementation Rules](#implementation-rules)
6. [Inheritance Hierarchy](#inheritance-hierarchy)
7. [GTS System Errors Catalog](#gts-system-errors-catalog)
8. [HTTP Headers](#http-headers)
9. [References](#references)

---

## Requirements

### Functional

1. **Every REST API error** MUST have a valid GTS type identifier
2. **Every error response** MUST include a trace ID for debugging
3. **All error types** MUST be registered in the types registry
4. **Consistent error schema** across all modules following RFC 9457
5. **Machine-readable error codes** for programmatic handling

### Non-Functional

#### Security

1. **NEVER expose error chains** — reveals internal system architecture
2. **NEVER expose full W3C traceparent** — only expose trace-id (32 hex chars); span-id reveals internal call hierarchy, trace-flags reveal sampling strategy
3. **NEVER include** credentials, tokens, PII, SQL errors, stack traces, or internal hostnames in metadata
4. **Always sanitize** user input before including in `metadata`
5. **Always log full details** server-side with `trace_id`, return only sanitized version to client

#### Error Detail Levels

| Audience | Access | Detail Level |
|----------|--------|--------------|
| External clients | API response | `type`, `title`, `status`, `trace_id`, `metadata` (sanitized) |
| Internal services | API response | Same as external — use `trace_id` for correlation |
| Developers/QA | Observability tools | Full details via `trace_id` (logs, traces, error chains) |

### Non-Goals

- Changing internal domain error types (only API-facing errors)
- Modifying logging infrastructure
- Changing HTTP status code semantics

---

## GTS Error Identifier Format

### Base Error Schema

The root error type for all CyberFabric errors:

```
gts.cf.core.errors.err.v1~
```

This schema defines the core Problem fields: `type`, `title`, `status`, `trace_id`, `metadata`.

Every error is a **2-segment chain**: `base ~ specific_error`.

**System error** — generic platform-level:

```
gts.cf.core.errors.err.v1~cf.system.logical.not_found.v1~
|__ base error schema __|  |__ system error (status=404) __|
```

**Module error** — domain-specific:

```
gts.cf.core.errors.err.v1~cf.types_registry.entity.not_found.v1~
|__ base error schema __|  |__ module error (status=404, adds gts_id) __|
```

The `base` field in `#[gts_error]` determines which GTS schema segment is prepended to form the chain — it does not inherit `status`. Each error defines its own `status` explicitly. The GTS chain always remains 2 segments. More segments are possible in rare cases but not the default.

### Segment Structure

Each segment follows the canonical GTS format:

```
<vendor>.<package>.<namespace>.<type>.v<MAJOR>[.<MINOR>]
```

| Component | Description | Example |
|-----------|-------------|---------|
| `vendor` | Organization identifier | `cf` |
| `package` | Module/service name | `types_registry`, `system` |
| `namespace` | Error category | `entity`, `logical`, `transport` |
| `type` | Specific error type | `not_found`, `timeout` |
| `version` | Schema version | `v1`, `v1.2` |

### Key Principle: Schemas Define Fields

Each segment in the chain can introduce new fields that appear in the Problem `metadata`:

| Segment | New Fields in `metadata` |
|---------|--------------------------|
| `gts.cf.core.errors.err.v1~` | _(defines core Problem shape)_ |
| `cf.system.runtime.rate_limited.v1~` | `retry_after` |
| `cf.system.logical.validation_failed.v1~` | `errors` (array of violations) |
| `cf.types_registry.entity.not_found.v1~` | `gts_id` |

---

## Problem Response Schema

Every error returned from REST API MUST conform to this RFC 9457 compliant schema:

```rust
pub struct Problem {
    /// GTS error type URI (RFC 9457 `type`)
    /// Full chained GTS type identifier with `gts://` prefix.
    /// Example: "gts://gts.cf.core.errors.err.v1~cf.types_registry.entity.not_found.v1~"
    #[serde(rename = "type")]
    pub r#type: String,

    /// Human-readable summary of the problem type (RFC 9457 `title`)
    /// Static per error type — not a dynamic message.
    pub title: String,

    /// HTTP status code (RFC 9457 `status`)
    /// Non-optional: every error response has a definitive HTTP status.
    /// Previously `Option<StatusCode>` was considered for non-HTTP contexts (gRPC, events),
    /// but in practice all errors flow through HTTP and the optionality only added
    /// `.unwrap()` noise without real benefit.
    /// Uses `http::StatusCode` for type safety. Serializes as u16.
    #[serde(
        serialize_with = "serialize_status_code",
        deserialize_with = "deserialize_status_code"
    )]
    pub status: StatusCode,

    /// Trace ID for request correlation (32 hex chars, trace-id portion only).
    /// Auto-populated from current tracing span in HTTP context.
    /// Prefer `None` over empty string when unavailable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trace_id: Option<String>,

    /// Segment-specific extension data (RFC 9457 extension members).
    /// Contains all fields defined by schemas in the GTS type chain.
    /// Values can be strings, numbers, arrays, objects, etc.
    /// This single field replaces the previous separate `errors` and `metadata` fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}
```

### Field Summary

| Field | Required | Description |
|-------|----------|-------------|
| `type` | Yes | GTS chained type URI (`gts://...~`) |
| `title` | Yes | Static human-readable error name |
| `status` | Yes | HTTP status code (uses `http::StatusCode` for type safety) |
| `trace_id` | No | 32 hex chars, auto-populated in HTTP context |
| `metadata` | No | All segment-specific data as key-value pairs |

**Why `detail` was removed:** Developers commonly leak sensitive data (SQL errors, stack traces, hostnames) through it. Use `title` for static description, `metadata` for structured sanitized context, and server-side logging with `trace_id` for technical details.

### JSON Response Examples

**Entity Not Found** (module error with `gts_id` in metadata):

```json
{
  "type": "gts://gts.cf.core.errors.err.v1~cf.types_registry.entity.not_found.v1~",
  "title": "Entity Not Found",
  "status": 404,
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "metadata": {
    "gts_id": "gts.cf.example.test.v1~"
  }
}
```

**Validation Failed** (module error with `errors` array in metadata):

```json
{
  "type": "gts://gts.cf.core.errors.err.v1~cf.types_registry.validation.failed.v1~",
  "title": "Validation Failed",
  "status": 422,
  "trace_id": "7d9e8f5a4c3b2a1098765432abcdef01",
  "metadata": {
    "errors": [
      { "field": "email", "message": "Must be a valid email address", "code": "INVALID_EMAIL_FORMAT" },
      { "field": "age", "message": "Must be at least 18", "code": "MIN_VALUE_VIOLATION" }
    ]
  }
}
```

**Rate Limited** (2-segment chain, system segment adds `retry_after`):

```json
{
  "type": "gts://gts.cf.core.errors.err.v1~cf.system.runtime.rate_limited.v1~",
  "title": "Too Many Requests",
  "status": 429,
  "trace_id": "a1b2c3d4e5f6789012345678abcdef90",
  "metadata": {
    "retry_after": 30
  }
}
```

---

## Error Definition

### The `#[gts_error]` Macro

All errors are defined as explicit structs with the `#[gts_error]` attribute. This makes GTS IDs visible in code, provides compile-time validation, and generates `Problem` conversion.

```rust
use modkit_errors::gts_error;

/// Entity was not found in the types registry.
#[gts_error(
    type = "cf.types_registry.entity.not_found.v1",
    base = BaseError,
    status = 404,
    title = "Entity Not Found",
)]
pub struct EntityNotFoundError {
    /// Added to metadata as "gts_id"
    pub gts_id: String,
}
```

The macro generates:

- `GTS_ID`, `STATUS`, `TITLE` constants
- `ERROR_DEF` constant for types registry registration
- `into_problem()` — builds metadata from struct fields and converts to `Problem` with auto-populated trace_id
- `Display` and `Error` trait implementations

#### Generated Code Example

For the `EntityNotFoundError` above, the macro expands roughly to:

```rust
impl EntityNotFoundError {
    /// Full 2-segment GTS chain.
    pub const GTS_ID: &'static str =
        "gts://gts.cf.core.errors.err.v1~cf.types_registry.entity.not_found.v1~";
    pub const STATUS: StatusCode = StatusCode::NOT_FOUND;
    pub const TITLE: &'static str = "Entity Not Found";

    /// Static definition for types registry registration.
    pub const ERROR_DEF: ErrorDefinition = ErrorDefinition {
        gts_id: Self::GTS_ID,
        status: Self::STATUS,
        title: Self::TITLE,
    };

    /// Converts the error struct into a Problem response.
    /// Metadata is built inline from struct fields — no separate `to_metadata()` needed.
    pub fn into_problem(self) -> Problem {
        let mut metadata = HashMap::new();
        metadata.insert("gts_id".to_owned(), serde_json::Value::String(self.gts_id));

        Problem {
            r#type: Self::GTS_ID.to_owned(),
            title: Self::TITLE.to_owned(),
            status: Self::STATUS,
            trace_id: trace_id_from_current_span(),
            metadata: if metadata.is_empty() { None } else { Some(metadata) },
        }
    }
}

impl std::fmt::Display for EntityNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: gts_id={}", Self::TITLE, self.gts_id)
    }
}

impl std::error::Error for EntityNotFoundError {}
```

#### `trace_id_from_current_span()`

This helper is provided by `modkit-errors` and extracts the W3C trace-id (32 hex chars) from the current OpenTelemetry span context:

```rust
/// Extracts the trace-id (32 hex chars) from the current span's OTel context.
/// Returns `None` if no active span or no valid trace-id is present.
pub fn trace_id_from_current_span() -> Option<String> {
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    let ctx = tracing::Span::current().context();
    let span_ref = ctx.span();
    let trace_id = span_ref.span_context().trace_id();
    if trace_id == opentelemetry::trace::TraceId::INVALID {
        None
    } else {
        Some(trace_id.to_string())
    }
}
```

> **Note:** This returns the OTel trace-id (32 hex chars), NOT `tracing::Span::id()` (which is a local span ID). The 32-char trace-id is what gets exposed to clients and used for cross-service correlation.

For a unit struct (no fields), `metadata` is `None`:

```rust
/// #[gts_error(type = "cf.system.runtime.internal.v1", base = BaseError, status = 500, title = "Internal Server Error")]
/// pub struct InternalError;
impl InternalError {
    pub fn into_problem(self) -> Problem {
        Problem {
            r#type: Self::GTS_ID.to_owned(),
            title: Self::TITLE.to_owned(),
            status: Self::STATUS,
            trace_id: trace_id_from_current_span(),
            metadata: None,
        }
    }
}
```

### Base Type Reference

`BaseError` is defined in `modkit-errors` and represents the root GTS error schema (`gts.cf.core.errors.err.v1~`). It is not an error you instantiate directly — it exists only to anchor the 2-segment GTS chain. All errors use `base = BaseError` to keep the chain at exactly 2 segments. Each error defines its own `status` explicitly.

```rust
// Defined in modkit-errors — NOT instantiated directly.
// Provides the root GTS schema segment for all error chains.
#[gts_error(
    type = "gts.cf.core.errors.err.v1",
    status = 500,
    title = "Error",
)]
pub struct BaseError;
```

All concrete errors extend `BaseError`:

```rust
// Platform error (defined in modkit-errors)
#[gts_error(
    type = "cf.system.logical.not_found.v1",
    base = BaseError,
    status = 404,
    title = "Not Found",
)]
pub struct NotFoundError;

// Module error (also extends BaseError directly — keeps GTS chain at 2 segments)
#[gts_error(
    type = "cf.types_registry.entity.not_found.v1",
    base = BaseError,
    status = 404,
    title = "Entity Not Found",
)]
pub struct EntityNotFoundError {
    pub gts_id: String,
}
```

The GTS chain built at compile time is always 2 segments:

```
gts://gts.cf.core.errors.err.v1~cf.types_registry.entity.not_found.v1~
```

### Field Attributes

| Attribute | Effect |
|-----------|--------|
| _(none)_ | Field included in `metadata` |
| `#[gts_error(skip_metadata)]` | Field excluded from `metadata` |
| `#[gts_error(as_errors)]` | Field serialized as `metadata.errors` (for validation violations) |

### Error Mapping Pattern

```rust
use crate::errors::*;

impl DomainError {
    pub fn to_problem(&self) -> Problem {
        match self {
            DomainError::NotFound(id) =>
                EntityNotFoundError { gts_id: id.clone() }.into_problem(),
            DomainError::AlreadyExists(id) =>
                EntityAlreadyExistsError { gts_id: id.clone() }.into_problem(),
            DomainError::Internal(e) => {
                tracing::error!(error = ?e, "Internal error");
                InternalError.into_problem()
            }
        }
    }
}
```

---

## Implementation Rules

### Rule 1: All Errors MUST Use `#[gts_error]` Structs

No inline errors, no JSON catalogs. GTS IDs must be visible in code.

```rust
// CORRECT
#[gts_error(
    type = "cf.types_registry.entity.not_found.v1",
    base = BaseError,
    status = 404,
    title = "Entity Not Found",
)]
pub struct EntityNotFoundError { pub gts_id: String }

// WRONG — no GTS ID
Problem::new(StatusCode::NOT_FOUND, "Not Found")

// WRONG — GTS ID hidden in JSON file
declare_errors! { path = "gts/errors.json", ... }
```

### Rule 2: Trace ID is Optional

- `trace_id` is `Option<String>` — may be `None` at some code points
- Auto-populated from tracing span in HTTP response handlers
- Only expose trace-id portion (32 hex chars), not full W3C traceparent
- Prefer `None` over empty string `""`

### Rule 3: Error Registration

All error types MUST be registered in the types registry at startup:

```rust
impl Module for MyModule {
    async fn on_ready(&self, ctx: &ModuleContext) -> Result<()> {
        ctx.types_registry().register_errors(
            crate::errors::all_error_definitions()
        ).await?;
        Ok(())
    }
}
```

### Rule 4: Metadata Comes From Struct Fields

All metadata is populated through struct fields — there is no `.with_metadata()` builder. If you need a value in metadata, declare it as a field on the error struct.

```rust
// CORRECT — metadata populated via struct fields
EntityNotFoundError { gts_id: id.clone() }.into_problem()
// produces: { "metadata": { "gts_id": "gts.cf.example.test.v1~" } }

// CORRECT — multiple metadata fields
#[gts_error(
    type = "cf.types_registry.operational.activation_failed.v1",
    base = BaseError,
    status = 500,
    title = "Activation Failed",
)]
pub struct ActivationFailedError {
    pub error_count: u32,
    pub summary: String,
}

ActivationFailedError { error_count: 3, summary: "3 schemas failed validation".into() }
    .into_problem()
// produces: { "metadata": { "error_count": 3, "summary": "3 schemas failed validation" } }

// CORRECT — unit struct, no metadata
InternalError.into_problem()
// produces: { "metadata": null }

// CORRECT — validation errors use #[gts_error(as_errors)]
#[gts_error(
    type = "cf.types_registry.validation.failed.v1",
    base = BaseError,
    status = 422,
    title = "Validation Failed",
)]
pub struct ValidationFailedError {
    #[gts_error(as_errors)]
    pub violations: Vec<ValidationViolation>,
}

ValidationFailedError {
    violations: vec![
        ValidationViolation { field: "email".into(), message: "Invalid format".into(), code: "INVALID_EMAIL".into() },
    ],
}.into_problem()
// produces: { "metadata": { "errors": [{ "field": "email", "message": "Invalid format", "code": "INVALID_EMAIL" }] } }

// CORRECT — skip internal fields with #[gts_error(skip_metadata)]
#[gts_error(
    type = "cf.file_parser.parsing.failed.v1",
    base = BaseError,
    status = 422,
    title = "Parse Error",
)]
pub struct ParseError {
    pub file_name: String,
    #[gts_error(skip_metadata)]
    pub raw_cause: String,  // logged server-side, not in response
}

// WRONG — no .with_metadata() builder exists
EntityNotFoundError { gts_id: id.clone() }
    .into_problem()
    .with_metadata("extra", json!("value"))  // COMPILE ERROR

// WRONG — sensitive data as a struct field
pub struct BadError {
    pub sql: String,  // NEVER — will leak to client in metadata
}
```

### Rule 5: Error Context and Observability

- Log full error details server-side with `trace_id`
- NEVER expose error chains, stack traces, or internal service names in API responses
- Client-facing errors contain only: `type`, `title`, `status`, `trace_id`, `metadata` (sanitized)

---

## Inheritance Hierarchy

All errors are 2-segment chains: `gts.cf.core.errors.err.v1~<error_type>~`. The `base` field in `#[gts_error]` is for Rust-level status inheritance only.

```
gts.cf.core.errors.err.v1~                                         [root]
|
|-- PLATFORM ERRORS (cf.system.*)
|   |-- cf.system.transport.*       connection_refused, connection_reset,
|   |                               connection_timeout, dns_failed, tls_*, network_unreachable
|   |-- cf.system.runtime.*         panic, oom, timeout, rate_limited, circuit_open,
|   |                               resource_exhausted, internal, unhandled, unavailable
|   |-- cf.system.http.*            bad_request, unauthorized, forbidden, not_found,
|   |                               method_not_allowed, not_acceptable, conflict, gone,
|   |                               payload_too_large, unsupported_media_type,
|   |                               unprocessable_entity, upstream_error, upstream_timeout
|   |-- cf.system.grpc.*            cancelled, invalid_argument, deadline_exceeded,
|   |                               not_found, already_exists, permission_denied,
|   |                               resource_exhausted, failed_precondition, aborted,
|   |                               unimplemented, internal, unavailable, data_loss,
|   |                               unauthenticated
|   +-- cf.system.logical.*         not_found, already_exists, validation_failed,
|                                   precondition_failed, state_conflict, operation_failed
|
+-- MODULE ERRORS
    |-- cf.auth.*                   token.expired, token.invalid, token.missing,
    |                               issuer.mismatch, audience.mismatch,
    |                               jwks.fetch_failed, scope.insufficient,
    |                               tenant.mismatch, system.internal
    |-- cf.types_registry.*         entity.not_found, entity.already_exists,
    |                               validation.invalid_gts_id, validation.failed,
    |                               operational.not_ready, operational.activation_failed,
    |                               system.internal
    |-- cf.file_parser.*            file.not_found, validation.invalid_format,
    |                               validation.unsupported_type, validation.no_parser,
    |                               parsing.failed, validation.invalid_url,
    |                               validation.invalid_request, transport.download_failed,
    |                               system.io_error, system.internal
    |-- cf.db.*                     connection.failed, connection.timeout, query.failed,
    |                               transaction.failed, constraint.violation, entity.not_found
    |-- cf.nodes_registry.*         node.not_found, sysinfo.collection_failed,
    |                               syscap.collection_failed, validation.invalid_input,
    |                               system.internal
    |-- cf.odata.*                  query.invalid_filter, query.invalid_orderby,
    |                               query.invalid_cursor, system.internal
    |-- cf.api_gateway.*            request.bad_request, auth.unauthorized, auth.forbidden,
    |                               routing.not_found, state.conflict, rate.limited,
    |                               system.internal
    |-- cf.tenant_resolver.*        registry.unavailable, plugin.not_found,
    |                               plugin.invalid_instance, plugin.client_not_found,
    |                               tenant.not_found, tenant.access_denied,
    |                               auth.unauthorized, system.internal
    +-- cf.simple_user_settings.*   settings.not_found, settings.validation,
                                    system.internal_database
```

---

## GTS System Errors Catalog

All types below show the full chained GTS type identifier used in the Problem `type` field (`gts://` prefix stripped for brevity). All identifiers end with `~` since they are schemas.

### 1. Transport Errors

| Full GTS Type | Status | Title |
|---------------|--------|-------|
| `gts.cf.core.errors.err.v1~cf.system.transport.connection_refused.v1~` | 502 | Connection Refused |
| `gts.cf.core.errors.err.v1~cf.system.transport.connection_reset.v1~` | 502 | Connection Reset |
| `gts.cf.core.errors.err.v1~cf.system.transport.connection_timeout.v1~` | 504 | Connection Timeout |
| `gts.cf.core.errors.err.v1~cf.system.transport.dns_failed.v1~` | 502 | DNS Resolution Failed |
| `gts.cf.core.errors.err.v1~cf.system.transport.tls_handshake_failed.v1~` | 502 | TLS Handshake Failed |
| `gts.cf.core.errors.err.v1~cf.system.transport.tls_certificate_invalid.v1~` | 502 | Invalid Certificate |
| `gts.cf.core.errors.err.v1~cf.system.transport.network_unreachable.v1~` | 502 | Network Unreachable |

### 2. Runtime Errors

| Full GTS Type | Status | Title | Metadata Fields |
|---------------|--------|-------|-----------------|
| `gts.cf.core.errors.err.v1~cf.system.runtime.panic.v1~` | 500 | Service Panic | |
| `gts.cf.core.errors.err.v1~cf.system.runtime.oom.v1~` | 503 | Out of Memory | |
| `gts.cf.core.errors.err.v1~cf.system.runtime.timeout.v1~` | 504 | Request Timeout | |
| `gts.cf.core.errors.err.v1~cf.system.runtime.rate_limited.v1~` | 429 | Too Many Requests | `retry_after` |
| `gts.cf.core.errors.err.v1~cf.system.runtime.circuit_open.v1~` | 503 | Circuit Breaker Open | |
| `gts.cf.core.errors.err.v1~cf.system.runtime.resource_exhausted.v1~` | 503 | Resource Exhausted | |
| `gts.cf.core.errors.err.v1~cf.system.runtime.internal.v1~` | 500 | Internal Server Error | |
| `gts.cf.core.errors.err.v1~cf.system.runtime.unhandled.v1~` | 500 | Unhandled Error | |
| `gts.cf.core.errors.err.v1~cf.system.runtime.unavailable.v1~` | 503 | Service Unavailable | |

### 3. HTTP Errors

| Full GTS Type | Status | Title |
|---------------|--------|-------|
| `gts.cf.core.errors.err.v1~cf.system.http.bad_request.v1~` | 400 | Bad Request |
| `gts.cf.core.errors.err.v1~cf.system.http.unauthorized.v1~` | 401 | Unauthorized |
| `gts.cf.core.errors.err.v1~cf.system.http.forbidden.v1~` | 403 | Forbidden |
| `gts.cf.core.errors.err.v1~cf.system.http.not_found.v1~` | 404 | Not Found |
| `gts.cf.core.errors.err.v1~cf.system.http.method_not_allowed.v1~` | 405 | Method Not Allowed |
| `gts.cf.core.errors.err.v1~cf.system.http.not_acceptable.v1~` | 406 | Not Acceptable |
| `gts.cf.core.errors.err.v1~cf.system.http.conflict.v1~` | 409 | Conflict |
| `gts.cf.core.errors.err.v1~cf.system.http.gone.v1~` | 410 | Gone |
| `gts.cf.core.errors.err.v1~cf.system.http.payload_too_large.v1~` | 413 | Payload Too Large |
| `gts.cf.core.errors.err.v1~cf.system.http.unsupported_media_type.v1~` | 415 | Unsupported Media Type |
| `gts.cf.core.errors.err.v1~cf.system.http.unprocessable_entity.v1~` | 422 | Unprocessable Entity |
| `gts.cf.core.errors.err.v1~cf.system.http.upstream_error.v1~` | 502 | Bad Gateway |
| `gts.cf.core.errors.err.v1~cf.system.http.upstream_timeout.v1~` | 504 | Gateway Timeout |

### 4. gRPC Errors

> **Note:** gRPC uses its own status codes, mapped to HTTP for consistency in the unified system.

| Full GTS Type | Status | Title |
|---------------|--------|-------|
| `gts.cf.core.errors.err.v1~cf.system.grpc.cancelled.v1~` | 499 | Request Cancelled |
| `gts.cf.core.errors.err.v1~cf.system.grpc.invalid_argument.v1~` | 400 | Invalid Argument |
| `gts.cf.core.errors.err.v1~cf.system.grpc.deadline_exceeded.v1~` | 504 | Deadline Exceeded |
| `gts.cf.core.errors.err.v1~cf.system.grpc.not_found.v1~` | 404 | Not Found |
| `gts.cf.core.errors.err.v1~cf.system.grpc.already_exists.v1~` | 409 | Already Exists |
| `gts.cf.core.errors.err.v1~cf.system.grpc.permission_denied.v1~` | 403 | Permission Denied |
| `gts.cf.core.errors.err.v1~cf.system.grpc.resource_exhausted.v1~` | 429 | Resource Exhausted |
| `gts.cf.core.errors.err.v1~cf.system.grpc.failed_precondition.v1~` | 400 | Failed Precondition |
| `gts.cf.core.errors.err.v1~cf.system.grpc.aborted.v1~` | 409 | Aborted |
| `gts.cf.core.errors.err.v1~cf.system.grpc.unimplemented.v1~` | 501 | Unimplemented |
| `gts.cf.core.errors.err.v1~cf.system.grpc.internal.v1~` | 500 | Internal Error |
| `gts.cf.core.errors.err.v1~cf.system.grpc.unavailable.v1~` | 503 | Unavailable |
| `gts.cf.core.errors.err.v1~cf.system.grpc.data_loss.v1~` | 500 | Data Loss |
| `gts.cf.core.errors.err.v1~cf.system.grpc.unauthenticated.v1~` | 401 | Unauthenticated |

### 5. Logical Errors

| Full GTS Type | Status | Title | Metadata Fields |
|---------------|--------|-------|-----------------|
| `gts.cf.core.errors.err.v1~cf.system.logical.validation_failed.v1~` | 422 | Validation Failed | `errors` |
| `gts.cf.core.errors.err.v1~cf.system.logical.not_found.v1~` | 404 | Not Found | |
| `gts.cf.core.errors.err.v1~cf.system.logical.already_exists.v1~` | 409 | Already Exists | |
| `gts.cf.core.errors.err.v1~cf.system.logical.precondition_failed.v1~` | 412 | Precondition Failed | |
| `gts.cf.core.errors.err.v1~cf.system.logical.state_conflict.v1~` | 409 | State Conflict | |
| `gts.cf.core.errors.err.v1~cf.system.logical.operation_failed.v1~` | 500 | Operation Failed | |

### 6. Auth Errors

| Full GTS Type | Status | Title |
|---------------|--------|-------|
| `gts.cf.core.errors.err.v1~cf.auth.token.expired.v1~` | 401 | Token Expired |
| `gts.cf.core.errors.err.v1~cf.auth.token.invalid.v1~` | 401 | Invalid Token |
| `gts.cf.core.errors.err.v1~cf.auth.token.missing.v1~` | 401 | Missing Token |
| `gts.cf.core.errors.err.v1~cf.auth.issuer.mismatch.v1~` | 401 | Issuer Mismatch |
| `gts.cf.core.errors.err.v1~cf.auth.audience.mismatch.v1~` | 401 | Audience Mismatch |
| `gts.cf.core.errors.err.v1~cf.auth.jwks.fetch_failed.v1~` | 500 | JWKS Fetch Failed |
| `gts.cf.core.errors.err.v1~cf.auth.scope.insufficient.v1~` | 403 | Insufficient Scope |
| `gts.cf.core.errors.err.v1~cf.auth.tenant.mismatch.v1~` | 403 | Tenant Mismatch |
| `gts.cf.core.errors.err.v1~cf.auth.system.internal.v1~` | 500 | Internal Error |

### 7. Types Registry Errors

| Full GTS Type | Status | Title | Metadata Fields |
|---------------|--------|-------|------------------|
| `gts.cf.core.errors.err.v1~cf.types_registry.entity.not_found.v1~` | 404 | Entity Not Found | `gts_id` |
| `gts.cf.core.errors.err.v1~cf.types_registry.entity.already_exists.v1~` | 409 | Entity Already Exists | `gts_id` |
| `gts.cf.core.errors.err.v1~cf.types_registry.validation.invalid_gts_id.v1~` | 400 | Invalid GTS ID | `message` |
| `gts.cf.core.errors.err.v1~cf.types_registry.validation.failed.v1~` | 422 | Validation Failed | `message`, `errors` |
| `gts.cf.core.errors.err.v1~cf.types_registry.operational.not_ready.v1~` | 503 | Service Not Ready | |
| `gts.cf.core.errors.err.v1~cf.types_registry.operational.activation_failed.v1~` | 500 | Activation Failed | `error_count`, `summary` |
| `gts.cf.core.errors.err.v1~cf.types_registry.system.internal.v1~` | 500 | Internal Error | |

### 8. File Parser Errors

| Full GTS Type | Status | Title |
|---------------|--------|-------|
| `gts.cf.core.errors.err.v1~cf.file_parser.file.not_found.v1~` | 404 | File Not Found |
| `gts.cf.core.errors.err.v1~cf.file_parser.validation.invalid_format.v1~` | 422 | Invalid File Format |
| `gts.cf.core.errors.err.v1~cf.file_parser.validation.unsupported_type.v1~` | 400 | Unsupported File Type |
| `gts.cf.core.errors.err.v1~cf.file_parser.validation.no_parser.v1~` | 415 | No Parser Available |
| `gts.cf.core.errors.err.v1~cf.file_parser.parsing.failed.v1~` | 422 | Parse Error |
| `gts.cf.core.errors.err.v1~cf.file_parser.validation.invalid_url.v1~` | 400 | Invalid URL |
| `gts.cf.core.errors.err.v1~cf.file_parser.validation.invalid_request.v1~` | 400 | Invalid Request |
| `gts.cf.core.errors.err.v1~cf.file_parser.transport.download_failed.v1~` | 502 | Download Failed |
| `gts.cf.core.errors.err.v1~cf.file_parser.system.io_error.v1~` | 500 | IO Error |
| `gts.cf.core.errors.err.v1~cf.file_parser.system.internal.v1~` | 500 | Internal Error |

### 9. Database Errors

| Full GTS Type | Status | Title |
|---------------|--------|-------|
| `gts.cf.core.errors.err.v1~cf.db.connection.failed.v1~` | 503 | Connection Failed |
| `gts.cf.core.errors.err.v1~cf.db.connection.timeout.v1~` | 504 | Connection Timeout |
| `gts.cf.core.errors.err.v1~cf.db.query.failed.v1~` | 500 | Query Failed |
| `gts.cf.core.errors.err.v1~cf.db.transaction.failed.v1~` | 500 | Transaction Failed |
| `gts.cf.core.errors.err.v1~cf.db.constraint.violation.v1~` | 409 | Constraint Violation |
| `gts.cf.core.errors.err.v1~cf.db.entity.not_found.v1~` | 404 | Entity Not Found |

### 10. Nodes Registry Errors

| Full GTS Type | Status | Title |
|---------------|--------|-------|
| `gts.cf.core.errors.err.v1~cf.nodes_registry.node.not_found.v1~` | 404 | Node Not Found |
| `gts.cf.core.errors.err.v1~cf.nodes_registry.sysinfo.collection_failed.v1~` | 500 | System Info Collection Failed |
| `gts.cf.core.errors.err.v1~cf.nodes_registry.syscap.collection_failed.v1~` | 500 | System Capabilities Collection Failed |
| `gts.cf.core.errors.err.v1~cf.nodes_registry.validation.invalid_input.v1~` | 400 | Invalid Input |
| `gts.cf.core.errors.err.v1~cf.nodes_registry.system.internal.v1~` | 500 | Internal Error |

### 11. OData Errors

| Full GTS Type | Status | Title |
|---------------|--------|-------|
| `gts.cf.core.errors.err.v1~cf.odata.query.invalid_filter.v1~` | 422 | Invalid Filter |
| `gts.cf.core.errors.err.v1~cf.odata.query.invalid_orderby.v1~` | 422 | Invalid OrderBy |
| `gts.cf.core.errors.err.v1~cf.odata.query.invalid_cursor.v1~` | 422 | Invalid Cursor |
| `gts.cf.core.errors.err.v1~cf.odata.system.internal.v1~` | 500 | Internal OData Error |

### 12. Tenant Resolver Errors

| Full GTS Type | Status | Title |
|---------------|--------|-------|
| `gts.cf.core.errors.err.v1~cf.tenant_resolver.registry.unavailable.v1~` | 503 | Types Registry Unavailable |
| `gts.cf.core.errors.err.v1~cf.tenant_resolver.plugin.not_found.v1~` | 404 | Plugin Not Found |
| `gts.cf.core.errors.err.v1~cf.tenant_resolver.plugin.invalid_instance.v1~` | 500 | Invalid Plugin Instance |
| `gts.cf.core.errors.err.v1~cf.tenant_resolver.plugin.client_not_found.v1~` | 500 | Plugin Client Not Found |
| `gts.cf.core.errors.err.v1~cf.tenant_resolver.tenant.not_found.v1~` | 404 | Tenant Not Found |
| `gts.cf.core.errors.err.v1~cf.tenant_resolver.tenant.access_denied.v1~` | 403 | Access Denied |
| `gts.cf.core.errors.err.v1~cf.tenant_resolver.auth.unauthorized.v1~` | 401 | Unauthorized |
| `gts.cf.core.errors.err.v1~cf.tenant_resolver.system.internal.v1~` | 500 | Internal Error |

### 13. API Gateway Errors

| Full GTS Type | Status | Title |
|---------------|--------|-------|
| `gts.cf.core.errors.err.v1~cf.api_gateway.request.bad_request.v1~` | 400 | Bad Request |
| `gts.cf.core.errors.err.v1~cf.api_gateway.auth.unauthorized.v1~` | 401 | Unauthorized |
| `gts.cf.core.errors.err.v1~cf.api_gateway.auth.forbidden.v1~` | 403 | Forbidden |
| `gts.cf.core.errors.err.v1~cf.api_gateway.routing.not_found.v1~` | 404 | Not Found |
| `gts.cf.core.errors.err.v1~cf.api_gateway.state.conflict.v1~` | 409 | Conflict |
| `gts.cf.core.errors.err.v1~cf.api_gateway.rate.limited.v1~` | 429 | Too Many Requests |
| `gts.cf.core.errors.err.v1~cf.api_gateway.system.internal.v1~` | 500 | Internal Error |

### 14. Simple User Settings Errors

| Full GTS Type | Status | Title |
|---------------|--------|-------|
| `gts.cf.core.errors.err.v1~cf.simple_user_settings.settings.not_found.v1~` | 404 | Settings Not Found |
| `gts.cf.core.errors.err.v1~cf.simple_user_settings.settings.validation.v1~` | 422 | Validation Error |
| `gts.cf.core.errors.err.v1~cf.simple_user_settings.system.internal_database.v1~` | 500 | Internal Database Error |

### Error Code Quick Reference

| Status | Transport | Runtime | HTTP | gRPC | Logical |
|--------|-----------|---------|------|------|---------|
| **400** | - | - | `http.bad_request` | `grpc.invalid_argument`, `grpc.failed_precondition` | - |
| **401** | - | - | `http.unauthorized` | `grpc.unauthenticated` | - |
| **403** | - | - | `http.forbidden` | `grpc.permission_denied` | - |
| **404** | - | - | `http.not_found` | `grpc.not_found` | `logical.not_found` |
| **405** | - | - | `http.method_not_allowed` | - | - |
| **406** | - | - | `http.not_acceptable` | - | - |
| **409** | - | - | `http.conflict` | `grpc.already_exists`, `grpc.aborted` | `logical.already_exists`, `logical.state_conflict` |
| **410** | - | - | `http.gone` | - | - |
| **412** | - | - | - | - | `logical.precondition_failed` |
| **413** | - | - | `http.payload_too_large` | - | - |
| **415** | - | - | `http.unsupported_media_type` | - | - |
| **422** | - | - | `http.unprocessable_entity` | - | `logical.validation_failed` |
| **429** | - | `runtime.rate_limited` | - | `grpc.resource_exhausted` | - |
| **499** | - | - | - | `grpc.cancelled` | - |
| **500** | - | `runtime.internal`, `runtime.panic`, `runtime.unhandled` | - | `grpc.internal`, `grpc.data_loss` | `logical.operation_failed` |
| **501** | - | - | - | `grpc.unimplemented` | - |
| **502** | `transport.connection_refused`, `transport.connection_reset`, `transport.dns_failed`, `transport.tls_*`, `transport.network_unreachable` | - | `http.upstream_error` | - | - |
| **503** | - | `runtime.oom`, `runtime.circuit_open`, `runtime.resource_exhausted`, `runtime.unavailable` | - | `grpc.unavailable` | - |
| **504** | `transport.connection_timeout` | `runtime.timeout` | `http.upstream_timeout` | `grpc.deadline_exceeded` | - |

---

## HTTP Headers

### Request Headers

| Header | Required | Description |
|--------|----------|-------------|
| `X-Trace-Id` | No | Client-provided trace ID |
| `X-Request-Id` | No | Request correlation ID |

### Response Headers (on errors)

| Header | When | Description |
|--------|------|-------------|
| `X-Trace-Id` | Always | Trace ID (32 hex chars) |
| `X-Error-Code` | Always | GTS error type |
| `Content-Type` | Always | `application/problem+json` |
| `Retry-After` | 429, 503 | Seconds to wait |

---

## References

- [GTS Specification](https://github.com/GlobalTypeSystem/gts-spec)
- [RFC 9457 — Problem Details for HTTP APIs](https://www.rfc-editor.org/rfc/rfc9457.html)
- [Google AIP-193 — Errors](https://google.aip.dev/193)
- [CyberFabric GTS_ERRORS.md](../docs/GTS_ERRORS.md)
- [CyberFabric STATUS_CODES.md](../guidelines/DNA/REST/STATUS_CODES.md)
