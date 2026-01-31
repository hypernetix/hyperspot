# Unified Error System - Intention Document

This document defines the architecture, rules, and implementation plan for a unified error handling system across the Hyperspot project. All REST API errors MUST have a GTS identifier, trace ID, and be registered in the types registry.

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Industry Research](#industry-research)
3. [Current State Analysis](#current-state-analysis)
4. [Requirements](#requirements)
5. [Implementation Rules](#implementation-rules)
6. [Migration Plan](#migration-plan)
7. [Examples](#examples)
8. [HTTP Headers](#http-headers)
9. [Security Considerations](#security-considerations)
10. [Appendix A: Error Category Definitions](#appendix-a-error-category-definitions)
11. [Appendix B: Retryable Error Guidelines](#appendix-b-retryable-error-guidelines)
12. [GTS System Errors Catalog](#gts-system-errors-catalog)
13. [References](#references)

---

## Executive Summary

### Problem Statement

The current error handling across Hyperspot modules is inconsistent:
- Some modules use `declare_errors!` macro with JSON catalogs
- Others define errors inline without GTS codes
- Errors are not registered in the types registry
- No standardized metadata/context fields
- Inconsistent trace ID propagation

### Goals

1. **Every REST API error** MUST have a valid GTS identifier
2. **Every error response** MUST include a trace ID for debugging
3. **All error types** MUST be registered in the types registry
4. **Consistent error schema** across all modules following RFC 9457
5. **Machine-readable error codes** for programmatic handling
6. **Error stack propagation** for nested service calls

### Non-Goals

- Changing internal domain error types (only API-facing errors)
- Modifying logging infrastructure
- Changing HTTP status code semantics

---

## Industry Research

### How Major Tech Companies Handle Errors

| Company | Standard | Key Features |
|---------|----------|--------------|
| **Google** | AIP-193 + gRPC Status | `reason` (UPPER_SNAKE_CASE), `domain` (service name), `metadata` (key-value context), `ErrorInfo`, `LocalizedMessage`, `Help` links |
| **Amazon AWS** | Custom JSON | `errorType`, `httpStatus`, `requestId`, `message`, custom error object serialization |
| **Twitter/X** | JSON with `errors` array | `code` (CAPS_CASE), `message`, `parameter`, `details`, `value` |
| **RFC 9457** | Problem Details | `type` (URI), `title`, `status`, `detail`, `instance`, extension members |

### Key Patterns Identified

1. **Machine-readable identifiers** - Unique codes for programmatic handling (Google's `reason`, Twitter's `code`)
2. **Domain/namespace separation** - Identify error source (Google's `domain`, our GTS vendor.package)
3. **Structured metadata** - Key-value context without parsing messages (Google's `ErrorInfo.metadata`)
4. **Trace/Request IDs** - Correlation for debugging (AWS `requestId`, our `trace_id`)
5. **Documentation links** - URIs to error documentation (RFC 9457 `type`, Google's `Help`)
6. **Error categorization** - System vs User-defined, Retryable vs Non-retryable

### Google AIP-193 ErrorInfo Structure (Reference)

```json
{
  "error": {
    "code": 429,
    "message": "The zone 'us-east1-a' does not have enough resources...",
    "status": "RESOURCE_EXHAUSTED",
    "details": [
      {
        "@type": "type.googleapis.com/google.rpc.ErrorInfo",
        "reason": "RESOURCE_AVAILABILITY",
        "domain": "compute.googleapis.com",
        "metadata": {
          "zone": "us-east1-a",
          "vmType": "e2-medium"
        }
      }
    ]
  }
}
```

---

## Current State Analysis

### Existing Infrastructure

1. **`Problem` struct** (`libs/modkit-errors/src/problem.rs`) - RFC 9457 compliant
   - Fields: `type_url`, `title`, `status`, `detail`, `instance`, `code`, `trace_id`, `errors`
   - Implements `IntoResponse` for Axum with automatic trace ID enrichment

2. **`struct_to_gts_schema` macro** (`modules/types-registry/gts-rust/gts-macros/src/lib.rs`)
   - Attribute macro for explicit struct definitions with GTS IDs visible in code
   - Generates constants: `GTS_SCHEMA_ID`, `GTS_SCHEMA_JSON`, etc.
   - **This pattern will be adapted for error definitions**

3. **`declare_errors!` macro** (`libs/modkit-errors-macro/src/lib.rs`) - **TO BE DEPRECATED**
   - Generates type-safe error enums from JSON catalogs
   - Problem: GTS IDs hidden in external JSON files, not visible to LLMs
   - Used by: `users_info` example, `modkit-odata`

### Gaps Identified

| Gap | Current State | Required State |
|-----|---------------|----------------|
| **Hidden GTS IDs** | JSON-based `declare_errors!` | Explicit `#[gts_error]` structs with visible GTS IDs |
| **Inconsistent definitions** | Some modules use inline errors | All modules use `#[gts_error]` structs |
| **No types registry** | Errors not registered | All errors registered as GTS types |
| **Missing metadata** | No structured context | Struct fields → `metadata` map |
| **Partial trace IDs** | Only in `IntoResponse` | Mandatory in all error responses |
| **No error categories** | Flat error codes | System/Operational/Client categories |
| **No error stack** | Single error | Upstream error chain propagation |

### Current Module Migration Status

| Module | Location | Status | Issues |
|--------|----------|--------|--------|
| **file_parser** | `modules/file_parser/` | 🔴 Not migrated | No GTS IDs, inline `Problem::new()` |
| **types-registry** | `modules/system/types-registry/` | 🟡 Partial | Has codes but not GTS format (e.g., `TYPES_REGISTRY_NOT_FOUND`) |
| **nodes_registry** | `modules/system/nodes_registry/` | 🔴 Not migrated | Non-GTS codes (e.g., `NODES_NOT_FOUND`) |
| **api_gateway** | `modules/system/api_gateway/` | 🔴 Not migrated | No Problem struct, simple error codes |
| **tenant_resolver** | `modules/system/tenant_resolver/` | 🔴 Not migrated | No GTS IDs |
| **simple-user-settings** | `modules/simple-user-settings/` | 🟡 Partial | Uses `declare_errors!` with old chain format |
| **modkit-auth** | `libs/modkit-auth/` | 🔴 Not migrated | No GTS IDs, custom `IntoResponse` |
| **modkit-odata** | `libs/modkit-odata/` | 🟡 Partial | Uses `declare_errors!` with old chain format |
| **users_info** (example) | `examples/modkit/users_info/` | 🟡 Partial | Uses `declare_errors!` with old chain format |

#### Legend
- 🟢 **Migrated** - Uses `#[gts_error]` structs with proper base types
- 🟡 **Partial** - Has GTS IDs but uses deprecated `declare_errors!` or old format
- 🔴 **Not migrated** - No GTS IDs or non-GTS format codes

#### GTS ID Format Note

**Old chain format** (used by `declare_errors!`):
```
gts.hx.core.errors.err.v1~hx.module.error_name.v1
```

**New format** (with `base` field):
```
gts.hx.module.namespace.error_name.v1~
```
The `base` field in `#[gts_error]` replaces the chain suffix, making inheritance explicit in code.

---

## Requirements

### R1: Error Schema Requirements

Every error returned from REST API MUST conform to the unified schema:

```rust
pub struct Problem {
    // =========================================================================
    // REQUIRED FIELDS - Core error identification
    // =========================================================================
    
    /// URI reference identifying the problem type (GTS-based)
    /// Example: "https://docs.hyperspot.com/errors/types_registry_entity_not_found_v1"
    #[serde(rename = "type")]
    pub type_url: String,
    
    /// Human-readable summary of the problem type (static per error type)
    pub title: String,
    
    /// GTS error code (machine-readable identifier)
    pub code: String,
    
    /// Trace ID for request correlation
    pub trace_id: String,
    
    // =========================================================================
    // CONTEXTUAL FIELDS - Optional based on error context
    // =========================================================================
    
    /// HTTP status code. Optional because:
    /// - gRPC errors use different status codes
    /// - Internal/domain errors may not have HTTP context
    /// - Event-driven systems have no request/response
    /// When serializing for HTTP response, this is populated from error definition.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    
    /// Human-readable explanation specific to this occurrence.
    /// Optional - title may be sufficient for simple errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    
    /// Error category for client handling.
    /// Optional - can be inferred from base type hierarchy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<ErrorCategory>,
    
    /// Whether client can retry this request.
    /// Optional - can be inferred from base type; not relevant for all contexts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    
    /// URI identifying this specific occurrence (auto-populated from request path)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    
    /// Structured metadata (key-value context)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    
    /// Validation errors for 4xx responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ValidationViolation>>,
    
    /// Upstream error chain (when error_stack enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upstream_errors: Option<Vec<UpstreamError>>,
}

pub enum ErrorCategory {
    /// Platform-level execution errors (panics, OOM, internal bugs)
    System,
    /// Infrastructure/operational issues (DB unavailable, service down, rate limits)
    Operational,
    /// Client input/request errors (validation, not found, unauthorized)
    Client,
}
```

#### Field Requirements Summary

| Field | Required | Reason |
|-------|----------|--------|
| `type_url` | ✅ Yes | RFC 9457 requires problem type URI |
| `title` | ✅ Yes | Human-readable error name (static per type) |
| `code` | ✅ Yes | GTS identifier for machine processing |
| `trace_id` | ✅ Yes | Request correlation for debugging |
| `status` | ❌ No | Only relevant for HTTP; gRPC/events/domain errors don't need it |
| `detail` | ❌ No | Instance-specific message; title may suffice |
| `category` | ❌ No | Inherited from base type hierarchy |
| `retryable` | ❌ No | Inherited from base type; not relevant for all contexts |
| `instance` | ❌ No | Auto-populated from request path when applicable |
| `metadata` | ❌ No | Additional context key-value pairs |
| `errors` | ❌ No | Validation violations for 4xx errors |
| `upstream_errors` | ❌ No | Error chain from upstream services |

**Note:** When converting to HTTP response, `status` is populated from the error definition. When the error is used in non-HTTP contexts (gRPC, domain logic, events), `status` remains `None`.

### R2: GTS Identifier and Base Type Requirements

Every error MUST have a valid GTS identifier and specify its base type using the `base` field.

#### GTS ID Format

```
gts.<vendor>.<package>.<namespace>.<type>.v<version>~
```

**Examples:**
- `gts.hx.types_registry.entity.not_found.v1~` - Types registry not found error
- `gts.hx.file_parser.validation.invalid_format.v1~` - File parser validation error
- `gts.hx.system.runtime.timeout.v1~` - System timeout error

#### Base Type Chaining

Errors form an inheritance chain using the `base` field. The `base` field references another error struct type (not a string literal), similar to how `struct_to_gts_schema` works.

The root base error type is:

```rust
/// Root error type for all GTS errors.
#[gts_error(
    gts_id = "gts.hx.core.errors.err.v1~",
    base = true,  // Marks this as a root base type (no parent)
    title = "Error",
    // Optional attributes with defaults for inheritance:
    status = 500,        // Default HTTP status (optional, inherited by children)
    category = "System", // Default category (optional, inherited by children)
    retryable = false    // Default retryable (optional, inherited by children)
)]
pub struct BaseError;
```

All module-specific errors MUST reference their base type as a struct (extending the appropriate logical/system error):

```rust
use modkit_errors::system::logical::NotFoundError;

/// Entity was not found in the types registry.
/// Note: status, category, retryable are inherited from NotFoundError if not specified.
#[gts_error(
    gts_id = "gts.hx.types_registry.entity.not_found.v1~",
    base = NotFoundError,  // Extends logical.not_found, not BaseError
    title = "Entity Not Found",
    // These are optional - inherited from base if omitted:
    // status = 404,      // Inherited from NotFoundError
    // category = "Client", // Inherited from NotFoundError  
    // retryable = false,   // Inherited from NotFoundError
)]
pub struct EntityNotFoundError {
    pub gts_id: String,
}
```

#### Multi-Level Inheritance

Errors can form deeper inheritance chains:

```rust
/// Logical not found error (platform-level).
#[gts_error(
    gts_id = "gts.hx.system.logical.not_found.v1~",
    base = BaseError,  // Platform errors extend BaseError
    status = 404,
    title = "Not Found",
    category = "Client",
    retryable = false
)]
pub struct NotFoundError;

/// Types registry entity not found (module-level).
#[gts_error(
    gts_id = "gts.hx.types_registry.entity.not_found.v1~",
    base = NotFoundError,  // Module errors extend logical errors
    status = 404,
    title = "Entity Not Found",
    category = "Client",
    retryable = false
)]
pub struct EntityNotFoundError {
    pub gts_id: String,
}
```

#### Inheritance Hierarchy

```
BaseError (gts.hx.core.errors.err.v1~)                              [base = true]
├── Transport Errors (hx.system.transport.*)                        [base = BaseError]
├── Runtime Errors (hx.system.runtime.*)                            [base = BaseError]
│   └── hx.types_registry.system.internal.v1~                       [base = runtime.internal]
├── HTTP Errors (hx.system.http.*)                                  [base = BaseError]
│   └── hx.auth.token_expired.v1~                                   [base = http.unauthorized]
├── gRPC Errors (hx.system.grpc.*)                                  [base = BaseError]
└── Logical Errors (hx.system.logical.*)                            [base = BaseError]
    ├── logical.not_found.v1~                                       [base = BaseError]
    │   ├── hx.types_registry.entity.not_found.v1~                  [base = logical.not_found]
    │   └── hx.db.entity.not_found.v1~                              [base = logical.not_found]
    ├── logical.already_exists.v1~                                  [base = BaseError]
    │   └── hx.types_registry.entity.already_exists.v1~             [base = logical.already_exists]
    └── logical.validation_failed.v1~                               [base = BaseError]
        └── hx.file_parser.validation.invalid_format.v1~            [base = logical.validation_failed]
```

### R3: Trace ID Requirements

- Every error response MUST include a non-empty `trace_id`
- Trace ID MUST be propagated from incoming request headers or generated
- Trace ID MUST be logged with the error for correlation

### R4: Types Registry Integration

- All error types MUST be registered in the types registry at startup
- Error schema MUST extend base error type: `gts.hx.core.errors.err.v1`
- Registration MUST include: code, title, status, category, retryable flag

### R5: Standardized System Error Types

See [GTS System Errors Catalog](#gts-system-errors-catalog) for the complete list of all system errors.

### R6: Error Stack Propagation

When a function calls another service/function that returns an error:
- The error SHOULD be wrapped with context
- `upstream_errors` array SHOULD contain the chain of errors
- Each upstream error includes: `code`, `title`, `detail`, `source`
- Propagation controlled by `error_stack` configuration flag

---

## GTS Error Code Format

### Structure

```
gts.<vendor>.<package>.<namespace>.<type>.<version>~<chain>
```

### Components

| Component | Description | Example |
|-----------|-------------|---------|
| `gts` | Global Type System prefix | `gts` |
| `vendor` | Organization identifier | `hx` (Hyperspot) |
| `package` | Module/service name | `types_registry`, `file_parser` |
| `namespace` | Error category | `entity`, `validation`, `runtime` |
| `type` | Specific error type | `not_found`, `invalid_format` |
| `version` | Schema version | `v1`, `v1.0` |
| `~chain` | Base type chain | `~hx.core.errors.err.v1` |

### Validation Rules

1. MUST start with `gts.`
2. All segments MUST be lowercase alphanumeric or underscore
3. No empty segments allowed
4. Final segment MUST be version format (`vN` or `vN.M`)
5. MUST chain from base error type

### Short Name Derivation

Short accessor names derived from final GTX segment:
```
gts.hx.core.errors.err.v1~hx.types_registry.entity.not_found.v1
                         └─────────────────────────────────────┘
                                          ↓
                         types_registry_entity_not_found_v1()
```

---

## Types Registry Integration

### Error Type Schema

```json
{
  "gts_id": "gts.hx.types_registry.entity.not_found.v1~",
  "kind": "error",
  "schema": {
    "type": "object",
    "extends": "gts.hx.system.logical.not_found.v1~",
    "properties": {
      "status": { "const": 404 },
      "title": { "const": "Entity Not Found" },
      "category": { "const": "Client" },
      "retryable": { "const": false }
    }
  },
  "metadata": {
    "module": "types-registry",
    "documentation_url": "https://docs.hyperspot.com/errors/types_registry_entity_not_found_v1"
  }
}
```

### Registration Flow

1. Module defines errors as explicit structs with `#[gts_error]` attribute in `src/errors.rs`
2. Each struct generates `ERROR_DEF` constant with GTS metadata
3. Module implements `all_error_definitions()` function returning all error definitions
4. At module startup (`on_ready`), errors are registered with types registry:
   ```rust
   impl Module for MyModule {
       async fn on_ready(&self, ctx: &ModuleContext) -> Result<()> {
           // Register all error types with types registry
           let errors = crate::errors::all_error_definitions();
           ctx.types_registry().register_errors(errors).await?;
           Ok(())
       }
   }
   ```
5. Types registry validates GTS format and schema compliance
6. Errors become discoverable via types registry API

### API for Error Discovery

```http
GET /api/types-registry/v1/types?kind=error&module=types-registry

Response:
{
  "items": [
    {
      "gts_id": "gts.hx.core.errors.err.v1~hx.types_registry.entity.not_found.v1",
      "status": 404,
      "title": "Entity Not Found",
      "category": "Client",
      "retryable": false,
      "documentation_url": "..."
    }
  ]
}
```

---

## Implementation Rules

### Rule 1: All Errors MUST Use Explicit Struct Definitions with `#[gts_error]`

Errors are defined as explicit structs with the `#[gts_error]` attribute macro. This approach:
- Makes GTS IDs visible directly in code (better for LLM code generation)
- Provides compile-time validation
- Generates `Problem` conversion automatically
- Enables types registry integration

```rust
// ✅ CORRECT - Explicit struct definition with visible GTS ID and base type reference
use modkit_errors::gts_error;

// First, import the appropriate base error types from system errors
use modkit_errors::system::logical::{NotFoundError, AlreadyExistsError, ValidationFailedError};

/// Entity was not found in the types registry.
#[gts_error(
    gts_id = "gts.hx.types_registry.entity.not_found.v1~",
    base = NotFoundError,  // Extends logical.not_found, not BaseError
    status = 404,
    title = "Entity Not Found",
    category = "Client",
    retryable = false
)]
pub struct EntityNotFoundError {
    /// The GTS ID that was not found
    pub gts_id: String,
}

/// Entity already exists in the registry.
#[gts_error(
    gts_id = "gts.hx.types_registry.entity.already_exists.v1~",
    base = AlreadyExistsError,  // Extends logical.already_exists
    status = 409,
    title = "Entity Already Exists",
    category = "Client",
    retryable = false
)]
pub struct EntityAlreadyExistsError {
    /// The GTS ID that already exists
    pub gts_id: String,
}

/// Invalid GTS ID format.
#[gts_error(
    gts_id = "gts.hx.types_registry.validation.invalid_gts_id.v1~",
    base = ValidationFailedError,  // Extends logical.validation_failed
    status = 400,
    title = "Invalid GTS ID",
    category = "Client",
    retryable = false
)]
pub struct InvalidGtsIdError {
    /// Description of the validation error
    pub message: String,
}

// ❌ WRONG - Inline error definitions without GTS ID
impl From<DomainError> for Problem {
    fn from(e: DomainError) -> Self {
        Problem::new(StatusCode::NOT_FOUND, "Not Found", "...")
            .with_code("SOME_CODE") // Not GTS format!
    }
}

// ❌ WRONG - JSON-based declaration (GTS ID hidden in external file)
declare_errors! {
    path = "gts/errors.json",
    namespace = "errors",
    vis = "pub"
}
```

### Rule 2: Error Struct Field Conventions

Error structs contain context fields that become `metadata` in the Problem response:

```rust
use modkit_errors::system::logical::ValidationFailedError as LogicalValidationFailed;

/// Request validation failed (module-specific).
#[gts_error(
    gts_id = "gts.hx.types_registry.validation.failed.v1~",
    base = LogicalValidationFailed,  // Extends logical.validation_failed
    status = 422,
    title = "Validation Failed",
    category = "Client",
    retryable = false
)]
pub struct ValidationFailedError {
    /// Human-readable error message
    pub message: String,
    /// Field that failed validation (optional)
    #[gts_error(skip_metadata)]  // Don't include in metadata
    pub violations: Vec<ValidationViolation>,
}
```

### Rule 3: Error Mapping Pattern

```rust
use crate::errors::{EntityNotFoundError, EntityAlreadyExistsError, InvalidGtsIdError};

pub fn domain_error_to_problem(e: &DomainError) -> Problem {
    match e {
        DomainError::NotFound { id } => {
            // into_problem() auto-populates trace_id from current tracing span
            EntityNotFoundError { gts_id: id.clone() }
                .into_problem()
                .with_detail(format!("Entity with GTS ID '{}' was not found", id))
        }
        DomainError::AlreadyExists { id } => {
            EntityAlreadyExistsError { gts_id: id.clone() }
                .into_problem()
                .with_detail(format!("Entity with GTS ID '{}' already exists", id))
        }
        DomainError::InvalidGtsId(msg) => {
            InvalidGtsIdError { message: msg.clone() }
                .into_problem()
        }
        // ...
    }
}
```

### Rule 4: Trace ID Auto-Population

Trace ID is **mandatory** in all error responses but **optional** in `into_problem()`. If not provided, it's auto-populated from the current tracing span.

```rust
// Default: trace_id auto-populated from current tracing span
let problem = EntityNotFoundError { gts_id: id.clone() }
    .into_problem()  // trace_id extracted from tracing::Span::current()
    .with_detail("Entity not found");

// Override: explicitly pass trace_id if needed
let problem = EntityNotFoundError { gts_id: id.clone() }
    .into_problem()
    .with_trace_id(&custom_trace_id)  // Override auto-populated value
    .with_detail("Entity not found");
```

### Rule 5: Instance Auto-Population

The `instance` field is **optional** and **auto-populated** from the request path in the Axum `IntoResponse` implementation. You don't need to pass it manually.

```rust
// In Axum IntoResponse implementation
impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        // Auto-populate instance from request URI if not set
        let problem = if self.instance.is_none() {
            // Instance is populated from request extensions or current URI
            if let Some(uri) = REQUEST_URI.try_with(|u| u.clone()).ok() {
                self.with_instance(uri.path())
            } else {
                self
            }
        } else {
            self
        };
        // ... rest of response building
    }
}
```

If you need to override the instance manually, use `with_instance()`:

```rust
let problem = EntityNotFoundError { gts_id: id.clone() }
    .into_problem()
    .with_instance("/custom/path");  // Optional override
```

### Rule 6: Error Registration at Startup

Error definitions are **auto-collected** at compile time using the `#[gts_error]` macro with `linkme`. No manual registration list needed.

#### How `linkme` Works

The [`linkme`](https://docs.rs/linkme) crate provides **distributed slices** - a way to collect static items from across the crate into a single slice at link time.

**Step 1: Define the collection point** (in `src/errors.rs`):
```rust
use linkme::distributed_slice;

/// This is the "collection point" - an empty slice that will be filled at link time.
/// The linker will gather all items marked with #[distributed_slice(ERROR_DEFINITIONS)]
/// and place them contiguously in this slice.
#[distributed_slice]
pub static ERROR_DEFINITIONS: [&'static GtsErrorDefStatic];
```

**Step 2: Each `#[gts_error]` macro generates a contributor**:
```rust
// When you write:
#[gts_error(
    gts_id = "gts.hx.types_registry.entity.not_found.v1~",
    base = NotFoundError,  // Extends logical.not_found
    status = 404,
    ...
)]
pub struct EntityNotFoundError { ... }

// The macro generates (among other things):
impl EntityNotFoundError {
    pub const ERROR_DEF: GtsErrorDefStatic = GtsErrorDefStatic { ... };
}

// AND this linkme contributor:
#[linkme::distributed_slice(crate::errors::ERROR_DEFINITIONS)]
#[linkme(crate = linkme)]
static __ENTITY_NOT_FOUND_ERROR_DEF: &'static GtsErrorDefStatic = &EntityNotFoundError::ERROR_DEF;
```

**Step 3: At link time, the linker collects all contributors**:
```
┌─────────────────────────────────────────────────────────────────┐
│                    Link-Time Collection                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  EntityNotFoundError::ERROR_DEF ──────┐                         │
│  EntityAlreadyExistsError::ERROR_DEF ─┼──► ERROR_DEFINITIONS[]  │
│  InvalidGtsIdError::ERROR_DEF ────────┤    (contiguous slice)   │
│  ValidationFailedError::ERROR_DEF ────┤                         │
│  ServiceNotReadyError::ERROR_DEF ─────┤                         │
│  InternalError::ERROR_DEF ────────────┘                         │
│                                                                  │
│  The linker places all #[distributed_slice(ERROR_DEFINITIONS)]  │
│  items into a contiguous memory section, making them accessible │
│  as a single &[&'static GtsErrorDefStatic] slice at runtime.    │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Step 4: Access at runtime**:
```rust
pub fn all_error_definitions() -> &'static [&'static GtsErrorDefStatic] {
    &ERROR_DEFINITIONS  // Returns slice with ALL error defs, auto-collected
}
```

#### Why This Works

- **Linker magic**: `linkme` uses platform-specific linker sections (`.init_array` on Linux, `__DATA,__mod_init_func` on macOS) to collect items
- **Zero runtime cost**: Collection happens at link time, not runtime
- **Compile-time guarantee**: If you use `#[gts_error]`, it's automatically in the slice
- **No manual list**: Impossible to forget an error - the macro handles registration

#### Module Startup

```rust
impl Module for MyModule {
    async fn on_ready(&self, ctx: &ModuleContext) -> Result<()> {
        // All errors are auto-collected - no way to forget one!
        let errors = crate::errors::all_error_definitions();
        ctx.types_registry().register_errors(errors).await?;
        Ok(())
    }
}
```

### Rule 7: Metadata for Dynamic Context

```rust
// ✅ CORRECT - Use metadata for dynamic values
ValidationFailedError { message: "Validation failed".to_string(), violations: vec![] }
    .into_problem()  // trace_id auto-populated from tracing span
    .with_metadata([
        ("field", "email"),
        ("constraint", "must be valid email format"),
        ("provided_value", "[REDACTED]"), // Never expose sensitive data
    ])

// ❌ WRONG - Parsing dynamic values from detail string
ValidationFailedError { 
    message: "Field 'email' failed constraint 'must be valid email'".to_string(),
    violations: vec![] 
}
    .into_problem()
```

---

## Migration Plan

### Phase 1: Create `#[gts_error]` Macro

1. Create new proc-macro crate `libs/modkit-errors-macro-v2/` with `#[gts_error]` attribute macro
2. Implement macro to generate:
   - `GTS_ID`, `STATUS`, `TITLE`, `CATEGORY`, `RETRYABLE` constants
   - `ERROR_DEF` static for registration
   - `into_problem()` method
   - `to_metadata()` for struct fields → HashMap conversion
   - `Display` and `Error` trait implementations
3. Add compile-time GTS format validation
4. Add field attribute `#[gts_error(skip_metadata)]` and `#[gts_error(as_errors)]`

### Phase 2: Extend Problem Struct

1. Extend `Problem` struct with new fields:
   - `category: ErrorCategory`
   - `retryable: bool`
   - `metadata: Option<HashMap<String, String>>`
   - `upstream_errors: Option<Vec<UpstreamError>>`
   - Make `trace_id` non-optional (String instead of Option<String>)
2. Add builder methods: `with_category()`, `with_retryable()`, `with_metadata()`
3. Update `IntoResponse` implementation

### Phase 3: Types Registry Integration

1. Define `GtsErrorDef` trait and `GtsErrorDefStatic` struct
2. Add error registration API to types registry
3. Implement `register_errors()` method in module context
4. Add error discovery endpoint: `GET /api/types-registry/v1/types?kind=error`

### Phase 4: Module Migration

Migrate modules to explicit struct-based errors:
1. `types-registry` - Create `src/errors.rs` with `#[gts_error]` structs
2. `file_parser` - Create `src/errors.rs` with `#[gts_error]` structs
3. `nodes_registry` - Create `src/errors.rs` with `#[gts_error]` structs
4. `api_gateway` - Create `src/errors.rs` with `#[gts_error]` structs

### Phase 5: System Errors Library

1. Create `libs/modkit-errors/src/system_errors.rs` with common errors:
   - `InternalServerError`, `ServiceUnavailableError`, `GatewayTimeoutError`
   - `UnauthorizedError`, `ForbiddenError`, `NotFoundError`
   - `BadRequestError`, `ValidationFailedError`, `ConflictError`
   - `RateLimitedError`
2. Export from `modkit::errors::system` for all modules to use
3. Register system errors at platform startup

### Phase 6: Deprecate `declare_errors!`

1. Mark `declare_errors!` macro as deprecated
2. Update `GTS_ERRORS.md` documentation
3. Add migration guide from JSON-based to struct-based errors
4. Remove JSON error catalogs from migrated modules

---

## Examples

### Example 1: Complete Error Response

```json
{
  "type": "https://docs.hyperspot.com/errors/types_registry_entity_not_found_v1",
  "title": "Entity Not Found",
  "status": 404,
  "detail": "Entity with GTS ID 'gts.hx.example.test.v1' was not found in the registry",
  "instance": "/api/types-registry/v1/types/gts.hx.example.test.v1",
  "code": "gts.hx.core.errors.err.v1~hx.types_registry.entity.not_found.v1",
  "trace_id": "abc123-def456-ghi789",
  "category": "Client",
  "retryable": false,
  "metadata": {
    "gts_id": "gts.hx.example.test.v1",
    "searched_at": "2026-01-19T13:30:00Z"
  }
}
```

### Example 2: Validation Error with Multiple Violations

```json
{
  "type": "https://docs.hyperspot.com/errors/validation_failed_v1",
  "title": "Validation Failed",
  "status": 422,
  "detail": "Request validation failed with 2 errors",
  "instance": "/api/users/v1/users",
  "code": "gts.hx.core.errors.err.v1~hx.system.validation_failed.v1",
  "trace_id": "req-12345",
  "category": "Client",
  "retryable": false,
  "errors": [
    {
      "field": "email",
      "message": "Must be a valid email address",
      "code": "INVALID_EMAIL_FORMAT"
    },
    {
      "field": "age",
      "message": "Must be at least 18",
      "code": "MIN_VALUE_VIOLATION"
    }
  ]
}
```

### Example 3: Error with Upstream Chain

```json
{
  "type": "https://docs.hyperspot.com/errors/upstream_error_v1",
  "title": "Upstream Service Error",
  "status": 502,
  "detail": "Failed to fetch user data from identity service",
  "instance": "/api/users/v1/users/123/profile",
  "code": "gts.hx.core.errors.err.v1~hx.users.upstream.identity_failed.v1",
  "trace_id": "trace-abc-123",
  "category": "System",
  "retryable": true,
  "upstream_errors": [
    {
      "code": "gts.hx.core.errors.err.v1~hx.identity.user.not_found.v1",
      "title": "User Not Found",
      "detail": "User 123 not found in identity service",
      "source": "identity-service"
    }
  ]
}
```

### Example 4: Complete Error Module Definition

```rust
//! Error definitions for the Types Registry module.
//!
//! All errors are defined as explicit structs with `#[gts_error]` attribute.
//! This makes GTS IDs visible in code for better LLM code generation.

use modkit_errors::{gts_error, ValidationViolation};
use modkit_errors::system::logical::{NotFoundError, AlreadyExistsError, ValidationFailedError as LogicalValidationFailed, OperationFailedError};
use modkit_errors::system::runtime::{UnavailableError, InternalError as RuntimeInternalError};

// =============================================================================
// Client Errors (4xx) - Extend logical errors
// =============================================================================

/// Entity was not found in the types registry.
#[gts_error(
    gts_id = "gts.hx.types_registry.entity.not_found.v1~",
    base = NotFoundError,  // Extends gts.hx.system.logical.not_found.v1~
    status = 404,
    title = "Entity Not Found",
    category = "Client",
    retryable = false
)]
pub struct EntityNotFoundError {
    /// The GTS ID that was not found
    pub gts_id: String,
}

/// Entity already exists in the registry.
#[gts_error(
    gts_id = "gts.hx.types_registry.entity.already_exists.v1~",
    base = AlreadyExistsError,  // Extends gts.hx.system.logical.already_exists.v1~
    status = 409,
    title = "Entity Already Exists",
    category = "Client",
    retryable = false
)]
pub struct EntityAlreadyExistsError {
    /// The GTS ID that already exists
    pub gts_id: String,
}

/// Invalid GTS ID format provided.
#[gts_error(
    gts_id = "gts.hx.types_registry.validation.invalid_gts_id.v1~",
    base = LogicalValidationFailed,  // Extends gts.hx.system.logical.validation_failed.v1~
    status = 400,
    title = "Invalid GTS ID",
    category = "Client",
    retryable = false
)]
pub struct InvalidGtsIdError {
    /// Description of what's wrong with the GTS ID
    pub message: String,
}

/// Request validation failed.
#[gts_error(
    gts_id = "gts.hx.types_registry.validation.failed.v1~",
    base = LogicalValidationFailed,  // Extends gts.hx.system.logical.validation_failed.v1~
    status = 422,
    title = "Validation Failed",
    category = "Client",
    retryable = false
)]
pub struct ValidationFailedError {
    /// Human-readable summary
    pub message: String,
    /// Individual validation violations
    #[gts_error(as_errors)]  // Maps to Problem.errors field
    pub violations: Vec<ValidationViolation>,
}

// =============================================================================
// Operational Errors (5xx - Retryable) - Extend runtime/logical errors
// =============================================================================

/// Service is not ready to accept requests.
#[gts_error(
    gts_id = "gts.hx.types_registry.operational.not_ready.v1~",
    base = UnavailableError,  // Extends gts.hx.system.runtime.unavailable.v1~
    status = 503,
    title = "Service Not Ready",
    category = "Operational",
    retryable = true
)]
pub struct ServiceNotReadyError;

/// Registry activation failed due to validation errors.
#[gts_error(
    gts_id = "gts.hx.types_registry.operational.activation_failed.v1~",
    base = OperationFailedError,  // Extends gts.hx.system.logical.operation_failed.v1~
    status = 500,
    title = "Registry Activation Failed",
    category = "Operational",
    retryable = false
)]
pub struct ActivationFailedError {
    /// Number of validation errors
    pub error_count: usize,
    /// Summary of errors
    pub summary: String,
}

// =============================================================================
// System Errors (5xx - Internal) - Extend runtime errors
// =============================================================================

/// Internal server error.
#[gts_error(
    gts_id = "gts.hx.types_registry.system.internal.v1~",
    base = RuntimeInternalError,  // Extends gts.hx.system.runtime.internal.v1~
    status = 500,
    title = "Internal Server Error",
    category = "System",
    retryable = true
)]
pub struct InternalError;

// =============================================================================
// Error Auto-Collection (via linkme)
// =============================================================================

use linkme::distributed_slice;

/// Auto-collected error definitions - no manual list needed!
/// Each #[gts_error] struct automatically registers itself here.
#[distributed_slice]
pub static ERROR_DEFINITIONS: [&'static GtsErrorDefStatic];

/// Get all error definitions (auto-collected at compile time).
pub fn all_error_definitions() -> &'static [&'static GtsErrorDefStatic] {
    &ERROR_DEFINITIONS
}

// Note: The manual list below is NO LONGER NEEDED - shown for reference only.
// Each #[gts_error] macro generates a #[distributed_slice] entry automatically.
//
// OLD APPROACH (error-prone, easy to forget):
// pub fn all_error_definitions() -> Vec<&'static dyn GtsErrorDef> {
//     vec![
//         &EntityNotFoundError::ERROR_DEF,
//         &EntityAlreadyExistsError::ERROR_DEF,
//         ...
//     ]
// }
```

### Example 5: Complete Usage in Handler

```rust
// In src/errors.rs - see Example 4 for full error definitions

// In src/api/rest/error.rs
use crate::errors::*;
use crate::domain::error::DomainError;

impl DomainError {
    /// Convert domain error to Problem response.
    ///
    /// Each error struct contains its GTS ID in the `#[gts_error]` attribute,
    /// making it visible to LLMs and developers reading the code.
    ///
    /// Note: `trace_id` and `instance` are auto-populated.
    pub fn to_problem(&self) -> Problem {
        match self {
            DomainError::NotFound(id) => {
                // GTS ID visible: gts.hx.types_registry.entity.not_found.v1~
                EntityNotFoundError { gts_id: id.clone() }
                    .into_problem()
                    .with_detail(format!("Entity with GTS ID '{}' was not found", id))
            }
            DomainError::AlreadyExists(id) => {
                // GTS ID visible: gts.hx.types_registry.entity.already_exists.v1~
                EntityAlreadyExistsError { gts_id: id.clone() }
                    .into_problem()
                    .with_detail(format!("Entity with GTS ID '{}' already exists", id))
            }
            DomainError::InvalidGtsId(msg) => {
                // GTS ID visible: gts.hx.types_registry.validation.invalid_gts_id.v1~
                InvalidGtsIdError { message: msg.clone() }
                    .into_problem()
            }
            DomainError::ValidationFailed { message, violations } => {
                // GTS ID visible: gts.hx.types_registry.validation.failed.v1~
                ValidationFailedError {
                    message: message.clone(),
                    violations: violations.clone(),
                }
                    .into_problem()
            }
            DomainError::NotInReadyMode => {
                // GTS ID visible: gts.hx.types_registry.operational.not_ready.v1~
                ServiceNotReadyError
                    .into_problem()
                    .with_detail("The types registry is not yet ready to accept requests")
            }
            DomainError::ReadyCommitFailed(errors) => {
                // GTS ID visible: gts.hx.types_registry.operational.activation_failed.v1~
                ActivationFailedError {
                    error_count: errors.len(),
                    summary: errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; "),
                }
                    .into_problem()
            }
            DomainError::Internal(e) => {
                tracing::error!(error = ?e, "Internal error in types_registry");
                // GTS ID visible: gts.hx.types_registry.system.internal.v1~
                InternalError
                    .into_problem()
                    .with_detail("An internal error occurred")
            }
        }
    }
}

// In handler - trace_id and instance are auto-populated
async fn get_type(
    State(state): State<AppState>,
    Path(gts_id): Path<String>,
) -> Result<Json<TypeDefinition>, Problem> {
    state.service
        .get_type(&gts_id)
        .await
        .map(Json)
        .map_err(|e| e.to_problem())
}
```

### Example 6: Generated Code from `#[gts_error]` Macro

The `#[gts_error]` attribute macro generates the following for each error struct:

```rust
// Original:
#[gts_error(
    gts_id = "gts.hx.types_registry.entity.not_found.v1~",
    base = NotFoundError,  // Reference to logical.not_found parent struct
    status = 404,
    title = "Entity Not Found",
    category = "Client",
    retryable = false
)]
pub struct EntityNotFoundError {
    pub gts_id: String,
}

// Generated:
pub struct EntityNotFoundError {
    pub gts_id: String,
}

impl EntityNotFoundError {
    /// GTS error code identifier
    pub const GTS_ID: &'static str = "gts.hx.types_registry.entity.not_found.v1~";
    
    /// Base error type (resolved from NotFoundError::GTS_ID at compile time)
    pub const BASE: &'static str = NotFoundError::GTS_ID;  // "gts.hx.system.logical.not_found.v1~"
    
    /// HTTP status code
    pub const STATUS: u16 = 404;
    
    /// Human-readable title
    pub const TITLE: &'static str = "Entity Not Found";
    
    /// Error category
    pub const CATEGORY: ErrorCategory = ErrorCategory::Client;
    
    /// Whether this error is retryable
    pub const RETRYABLE: bool = false;
    
    /// Static error definition for registration
    pub const ERROR_DEF: GtsErrorDefStatic = GtsErrorDefStatic {
        gts_id: Self::GTS_ID,
        base: Self::BASE,
        status: Self::STATUS,
        title: Self::TITLE,
        category: Self::CATEGORY,
        retryable: Self::RETRYABLE,
        type_url: "https://docs.hyperspot.com/errors/hx_types_registry_entity_not_found_v1",
    };
    
    /// Convert to Problem response.
    /// 
    /// `trace_id` and `instance` are auto-populated from the current tracing span
    /// and request path respectively.
    pub fn into_problem(self) -> Problem {
        // Auto-populate trace_id from current tracing span
        let trace_id = tracing::Span::current()
            .id()
            .map(|id| id.into_u64().to_string());
        
        let mut problem = Problem::new(StatusCode::from_u16(Self::STATUS).unwrap(), Self::TITLE, "")
            .with_code(Self::GTS_ID)
            .with_base(Self::BASE)
            .with_type(Self::ERROR_DEF.type_url)
            .with_category(Self::CATEGORY)
            .with_retryable(Self::RETRYABLE)
            .with_metadata(self.to_metadata());
        
        if let Some(tid) = trace_id {
            problem = problem.with_trace_id(tid);
        }
        
        problem
    }
    
    /// Convert struct fields to metadata map
    fn to_metadata(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("gts_id".to_string(), self.gts_id.clone());
        map
    }
}

impl std::error::Error for EntityNotFoundError {}

impl std::fmt::Display for EntityNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: gts_id={}", Self::TITLE, self.gts_id)
    }
}
```

---

## HTTP Headers

### Request Headers

| Header | Description | Required |
|--------|-------------|----------|
| `X-Trace-Id` | Client-provided trace ID | Optional |
| `X-Request-Id` | Request correlation ID | Optional |

### Response Headers

| Header | Description | When |
|--------|-------------|------|
| `X-Trace-Id` | Trace ID for debugging | Always on errors |
| `X-Error-Code` | GTS error code | Always on errors |
| `Content-Type` | `application/problem+json` | Always on errors |
| `Retry-After` | Seconds to wait | On 429, 503 |

---

## Security Considerations

### Sensitive Data Handling

1. **Never expose** internal implementation details in error messages
2. **Never include** credentials, tokens, or PII in metadata
3. **Sanitize** user input before including in error details
4. **Log full details** server-side, return sanitized version to client

### Error Detail Levels

| Audience | Detail Level |
|----------|--------------|
| External clients | Sanitized, no internal details |
| Internal services | Full details with upstream chain |
| Debugging (with scope) | Full stack trace in logs |

### Role-Based Error Details

```rust
// Check user scope for detailed errors
if user.has_scope("debugging") {
    problem.with_upstream_errors(full_chain)
} else {
    problem // Sanitized version
}
```

---

## Appendix A: Error Category Definitions

| Category | Description | Examples |
|----------|-------------|----------|
| **System** | Platform-level execution errors | Runtime panics, OOM, internal bugs |
| **Operational** | Infrastructure/operational issues | DB unavailable, service down, rate limits |
| **Client** | Client input/request errors | Validation, not found, unauthorized, business rule violations |

---

## Appendix B: Retryable Error Guidelines

| Condition | Retryable | Notes |
|-----------|-----------|-------|
| 429 Too Many Requests | Yes | Respect `Retry-After` header |
| 500 Internal Server Error | Yes | With exponential backoff |
| 502 Bad Gateway | Yes | Upstream may recover |
| 503 Service Unavailable | Yes | Respect `Retry-After` header |
| 504 Gateway Timeout | Yes | Upstream may be slow |
| 400 Bad Request | No | Client must fix input |
| 401 Unauthorized | No | Client must authenticate |
| 403 Forbidden | No | Permission issue |
| 404 Not Found | No | Resource doesn't exist |
| 409 Conflict | No | State conflict |
| 422 Validation Failed | No | Client must fix input |

---

## GTS System Errors Catalog

This section defines all GTS error codes used across the Hyperspot system. All errors specify their base type using the `base` field.

### Base Error Type

```
gts.hx.core.errors.err.v1~
```

All **platform system errors** (Transport, Runtime, HTTP, gRPC, Logical) MUST specify this as their `base` type. Module-specific errors extend the appropriate platform error as their base.

---

### 1. Platform System Errors (`hx.system.*`)

Core platform-level errors available to all modules. Defined in `libs/modkit-errors/src/system_errors.rs`.

Platform errors are organized by **layer**:

```
┌─────────────────────────────────────────────────────────────────┐
│                    Error Layer Hierarchy                         │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────┐  Transport layer (network, connection)         │
│  │  Transport  │  → TCP/TLS failures, DNS, connection refused   │
│  └──────┬──────┘                                                 │
│         │                                                        │
│  ┌──────▼──────┐  Protocol layer (HTTP, gRPC)                   │
│  │  HTTP/gRPC  │  → Status codes, headers, protocol errors      │
│  └──────┬──────┘                                                 │
│         │                                                        │
│  ┌──────▼──────┐  Runtime layer (environment, resources)        │
│  │   Runtime   │  → OOM, panic, timeout, rate limits            │
│  └──────┬──────┘                                                 │
│         │                                                        │
│  ┌──────▼──────┐  Application layer (per-module business logic) │
│  │   Logical   │  → Validation, not found, domain errors        │
│  └─────────────┘                                                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

#### 1.1 Transport Errors (`hx.system.transport.*`)

Low-level network and connection errors.

| GTS ID | Base | Status | Title | Category | Retryable | Description |
|--------|------|--------|-------|----------|-----------|-------------|
| `gts.hx.system.transport.connection_refused.v1~` | `gts.hx.core.errors.err.v1~` | 502 | Connection Refused | Operational | Yes | Remote host refused connection |
| `gts.hx.system.transport.connection_reset.v1~` | `gts.hx.core.errors.err.v1~` | 502 | Connection Reset | Operational | Yes | Connection was reset by peer |
| `gts.hx.system.transport.connection_timeout.v1~` | `gts.hx.core.errors.err.v1~` | 504 | Connection Timeout | Operational | Yes | Connection attempt timed out |
| `gts.hx.system.transport.dns_failed.v1~` | `gts.hx.core.errors.err.v1~` | 502 | DNS Resolution Failed | Operational | Yes | Failed to resolve hostname |
| `gts.hx.system.transport.tls_handshake_failed.v1~` | `gts.hx.core.errors.err.v1~` | 502 | TLS Handshake Failed | Operational | No | TLS/SSL handshake failed |
| `gts.hx.system.transport.tls_certificate_invalid.v1~` | `gts.hx.core.errors.err.v1~` | 502 | Invalid Certificate | Operational | No | TLS certificate validation failed |
| `gts.hx.system.transport.network_unreachable.v1~` | `gts.hx.core.errors.err.v1~` | 502 | Network Unreachable | Operational | Yes | Network is unreachable |

#### 1.2 Runtime Errors (`hx.system.runtime.*`)

Runtime environment and resource errors.

| GTS ID | Base | Status | Title | Category | Retryable | Description |
|--------|------|--------|-------|----------|-----------|-------------|
| `gts.hx.system.runtime.panic.v1~` | `gts.hx.core.errors.err.v1~` | 500 | Service Panic | System | Yes | Service panicked during request processing |
| `gts.hx.system.runtime.oom.v1~` | `gts.hx.core.errors.err.v1~` | 503 | Out of Memory | System | Yes | Service ran out of memory |
| `gts.hx.system.runtime.timeout.v1~` | `gts.hx.core.errors.err.v1~` | 504 | Request Timeout | Operational | Yes | Request processing timed out |
| `gts.hx.system.runtime.rate_limited.v1~` | `gts.hx.core.errors.err.v1~` | 429 | Too Many Requests | Operational | Yes | Rate limit exceeded |
| `gts.hx.system.runtime.circuit_open.v1~` | `gts.hx.core.errors.err.v1~` | 503 | Circuit Breaker Open | Operational | Yes | Circuit breaker is open due to failures |
| `gts.hx.system.runtime.resource_exhausted.v1~` | `gts.hx.core.errors.err.v1~` | 503 | Resource Exhausted | Operational | Yes | System resources exhausted (CPU, connections) |
| `gts.hx.system.runtime.internal.v1~` | `gts.hx.core.errors.err.v1~` | 500 | Internal Server Error | System | Yes | Unexpected internal error occurred |
| `gts.hx.system.runtime.unhandled.v1~` | `gts.hx.core.errors.err.v1~` | 500 | Unhandled Error | System | Yes | Error was not handled by application code |
| `gts.hx.system.runtime.unavailable.v1~` | `gts.hx.core.errors.err.v1~` | 503 | Service Unavailable | Operational | Yes | Service temporarily unavailable |

#### 1.3 HTTP Errors (`hx.system.http.*`)

HTTP protocol-level errors.

| GTS ID | Base | Status | Title | Category | Retryable | Description |
|--------|------|--------|-------|----------|-----------|-------------|
| `gts.hx.system.http.bad_request.v1~` | `gts.hx.core.errors.err.v1~` | 400 | Bad Request | Client | No | HTTP request is malformed |
| `gts.hx.system.http.unauthorized.v1~` | `gts.hx.core.errors.err.v1~` | 401 | Unauthorized | Client | No | Authentication required |
| `gts.hx.system.http.forbidden.v1~` | `gts.hx.core.errors.err.v1~` | 403 | Forbidden | Client | No | Access denied |
| `gts.hx.system.http.not_found.v1~` | `gts.hx.core.errors.err.v1~` | 404 | Not Found | Client | No | HTTP resource not found |
| `gts.hx.system.http.method_not_allowed.v1~` | `gts.hx.core.errors.err.v1~` | 405 | Method Not Allowed | Client | No | HTTP method not allowed |
| `gts.hx.system.http.not_acceptable.v1~` | `gts.hx.core.errors.err.v1~` | 406 | Not Acceptable | Client | No | Cannot produce acceptable response |
| `gts.hx.system.http.conflict.v1~` | `gts.hx.core.errors.err.v1~` | 409 | Conflict | Client | No | Request conflicts with current state |
| `gts.hx.system.http.gone.v1~` | `gts.hx.core.errors.err.v1~` | 410 | Gone | Client | No | Resource no longer available |
| `gts.hx.system.http.payload_too_large.v1~` | `gts.hx.core.errors.err.v1~` | 413 | Payload Too Large | Client | No | Request body exceeds limit |
| `gts.hx.system.http.unsupported_media_type.v1~` | `gts.hx.core.errors.err.v1~` | 415 | Unsupported Media Type | Client | No | Content-Type not supported |
| `gts.hx.system.http.unprocessable_entity.v1~` | `gts.hx.core.errors.err.v1~` | 422 | Unprocessable Entity | Client | No | Request body validation failed |
| `gts.hx.system.http.upstream_error.v1~` | `gts.hx.core.errors.err.v1~` | 502 | Bad Gateway | Operational | Yes | Upstream HTTP service returned error |
| `gts.hx.system.http.upstream_timeout.v1~` | `gts.hx.core.errors.err.v1~` | 504 | Gateway Timeout | Operational | Yes | Upstream HTTP service timed out |

#### 1.4 gRPC Errors (`hx.system.grpc.*`)

gRPC protocol-level errors.

| GTS ID | Base | Status | Title | Category | Retryable | Description |
|--------|------|--------|-------|----------|-----------|-------------|
| `gts.hx.system.grpc.cancelled.v1~` | `gts.hx.core.errors.err.v1~` | 499 | Request Cancelled | Client | No | gRPC request was cancelled |
| `gts.hx.system.grpc.invalid_argument.v1~` | `gts.hx.core.errors.err.v1~` | 400 | Invalid Argument | Client | No | Invalid gRPC request argument |
| `gts.hx.system.grpc.deadline_exceeded.v1~` | `gts.hx.core.errors.err.v1~` | 504 | Deadline Exceeded | Operational | Yes | gRPC deadline exceeded |
| `gts.hx.system.grpc.not_found.v1~` | `gts.hx.core.errors.err.v1~` | 404 | Not Found | Client | No | gRPC resource not found |
| `gts.hx.system.grpc.already_exists.v1~` | `gts.hx.core.errors.err.v1~` | 409 | Already Exists | Client | No | gRPC resource already exists |
| `gts.hx.system.grpc.permission_denied.v1~` | `gts.hx.core.errors.err.v1~` | 403 | Permission Denied | Client | No | gRPC permission denied |
| `gts.hx.system.grpc.resource_exhausted.v1~` | `gts.hx.core.errors.err.v1~` | 429 | Resource Exhausted | Operational | Yes | gRPC resource exhausted |
| `gts.hx.system.grpc.failed_precondition.v1~` | `gts.hx.core.errors.err.v1~` | 400 | Failed Precondition | Client | No | gRPC precondition failed |
| `gts.hx.system.grpc.aborted.v1~` | `gts.hx.core.errors.err.v1~` | 409 | Aborted | Client | Yes | gRPC operation aborted |
| `gts.hx.system.grpc.unimplemented.v1~` | `gts.hx.core.errors.err.v1~` | 501 | Unimplemented | System | No | gRPC method not implemented |
| `gts.hx.system.grpc.internal.v1~` | `gts.hx.core.errors.err.v1~` | 500 | Internal Error | System | Yes | gRPC internal error |
| `gts.hx.system.grpc.unavailable.v1~` | `gts.hx.core.errors.err.v1~` | 503 | Unavailable | Operational | Yes | gRPC service unavailable |
| `gts.hx.system.grpc.data_loss.v1~` | `gts.hx.core.errors.err.v1~` | 500 | Data Loss | System | No | Unrecoverable data loss |
| `gts.hx.system.grpc.unauthenticated.v1~` | `gts.hx.core.errors.err.v1~` | 401 | Unauthenticated | Client | No | gRPC authentication required |

#### 1.5 Logical Errors (`hx.system.logical.*`)

Application-level logical errors (generic, per-module errors extend these).

| GTS ID | Base | Status | Title | Category | Retryable | Description |
|--------|------|--------|-------|----------|-----------|-------------|
| `gts.hx.system.logical.validation_failed.v1~` | `gts.hx.core.errors.err.v1~` | 422 | Validation Failed | Client | No | Business logic validation failed |
| `gts.hx.system.logical.not_found.v1~` | `gts.hx.core.errors.err.v1~` | 404 | Not Found | Client | No | Business entity not found |
| `gts.hx.system.logical.already_exists.v1~` | `gts.hx.core.errors.err.v1~` | 409 | Already Exists | Client | No | Business entity already exists |
| `gts.hx.system.logical.precondition_failed.v1~` | `gts.hx.core.errors.err.v1~` | 412 | Precondition Failed | Client | No | Business precondition not met |
| `gts.hx.system.logical.state_conflict.v1~` | `gts.hx.core.errors.err.v1~` | 409 | State Conflict | Client | No | Business state conflict |
| `gts.hx.system.logical.operation_failed.v1~` | `gts.hx.core.errors.err.v1~` | 500 | Operation Failed | System | Yes | Business operation failed |

> **Note:** Module-specific errors (e.g., `hx.types_registry.*`, `hx.file_parser.*`) should extend these logical errors as their base type.

---

### 2. Authentication & Authorization Errors (`hx.auth.*`)

| GTS ID | Base | Status | Title | Category | Retryable | Description |
|--------|------|--------|-------|----------|-----------|-------------|
| `gts.hx.auth.token_expired.v1~` | `gts.hx.system.http.unauthorized.v1~` | 401 | Token Expired | Client | No | Authentication token has expired |
| `gts.hx.auth.token_invalid.v1~` | `gts.hx.system.http.unauthorized.v1~` | 401 | Invalid Token | Client | No | Authentication token is invalid |
| `gts.hx.auth.token_missing.v1~` | `gts.hx.system.http.unauthorized.v1~` | 401 | Missing Token | Client | No | Authentication token not provided |
| `gts.hx.auth.insufficient_scope.v1~` | `gts.hx.system.http.forbidden.v1~` | 403 | Insufficient Scope | Client | No | Token lacks required scope |
| `gts.hx.auth.tenant_mismatch.v1~` | `gts.hx.system.http.forbidden.v1~` | 403 | Tenant Mismatch | Client | No | Resource belongs to different tenant |

---

### 3. Types Registry Errors (`hx.types_registry.*`)

| GTS ID | Base | Status | Title | Category | Retryable | Description |
|--------|------|--------|-------|----------|-----------|-------------|
| `gts.hx.types_registry.entity.not_found.v1~` | `gts.hx.system.logical.not_found.v1~` | 404 | Entity Not Found | Client | No | Type definition not found |
| `gts.hx.types_registry.entity.already_exists.v1~` | `gts.hx.system.logical.already_exists.v1~` | 409 | Entity Already Exists | Client | No | Type definition already exists |
| `gts.hx.types_registry.validation.invalid_gts_id.v1~` | `gts.hx.system.logical.validation_failed.v1~` | 400 | Invalid GTS ID | Client | No | GTS ID format is invalid |
| `gts.hx.types_registry.validation.failed.v1~` | `gts.hx.system.logical.validation_failed.v1~` | 422 | Validation Failed | Client | No | Schema validation failed |
| `gts.hx.types_registry.operational.not_ready.v1~` | `gts.hx.system.runtime.unavailable.v1~` | 503 | Service Not Ready | Operational | Yes | Registry not ready |
| `gts.hx.types_registry.operational.activation_failed.v1~` | `gts.hx.system.logical.operation_failed.v1~` | 500 | Activation Failed | Operational | No | Registry activation failed |
| `gts.hx.types_registry.system.internal.v1~` | `gts.hx.system.runtime.internal.v1~` | 500 | Internal Error | System | Yes | Internal registry error |

---

### 4. File Parser Errors (`hx.file_parser.*`)

| GTS ID | Base | Status | Title | Category | Retryable | Description |
|--------|------|--------|-------|----------|-----------|-------------|
| `gts.hx.file_parser.validation.invalid_format.v1~` | `gts.hx.system.logical.validation_failed.v1~` | 400 | Invalid Format | Client | No | File format is invalid |
| `gts.hx.file_parser.validation.unsupported_type.v1~` | `gts.hx.system.http.unsupported_media_type.v1~` | 415 | Unsupported Type | Client | No | File type not supported |
| `gts.hx.file_parser.validation.size_exceeded.v1~` | `gts.hx.system.http.payload_too_large.v1~` | 413 | Size Exceeded | Client | No | File size exceeds limit |
| `gts.hx.file_parser.parsing.failed.v1~` | `gts.hx.system.logical.validation_failed.v1~` | 422 | Parsing Failed | Client | No | Failed to parse file content |
| `gts.hx.file_parser.system.internal.v1~` | `gts.hx.system.runtime.internal.v1~` | 500 | Internal Error | System | Yes | Internal parser error |

---

### 5. Database Errors (`hx.db.*`)

| GTS ID | Base | Status | Title | Category | Retryable | Description |
|--------|------|--------|-------|----------|-----------|-------------|
| `gts.hx.db.connection.failed.v1~` | `gts.hx.system.runtime.unavailable.v1~` | 503 | Connection Failed | Operational | Yes | Database connection failed |
| `gts.hx.db.connection.timeout.v1~` | `gts.hx.system.runtime.timeout.v1~` | 504 | Connection Timeout | Operational | Yes | Database connection timed out |
| `gts.hx.db.query.failed.v1~` | `gts.hx.system.runtime.internal.v1~` | 500 | Query Failed | System | Yes | Database query failed |
| `gts.hx.db.transaction.failed.v1~` | `gts.hx.system.runtime.internal.v1~` | 500 | Transaction Failed | System | Yes | Database transaction failed |
| `gts.hx.db.constraint.violation.v1~` | `gts.hx.system.logical.state_conflict.v1~` | 409 | Constraint Violation | Client | No | Database constraint violated |
| `gts.hx.db.entity.not_found.v1~` | `gts.hx.system.logical.not_found.v1~` | 404 | Entity Not Found | Client | No | Database entity not found |

---

### 6. Nodes Registry Errors (`hx.nodes_registry.*`)

| GTS ID | Base | Status | Title | Category | Retryable | Description |
|--------|------|--------|-------|----------|-----------|-------------|
| `gts.hx.nodes_registry.node.not_found.v1~` | `gts.hx.system.logical.not_found.v1~` | 404 | Node Not Found | Client | No | Node not found in registry |
| `gts.hx.nodes_registry.sysinfo.collection_failed.v1~` | `gts.hx.system.runtime.internal.v1~` | 500 | System Info Collection Failed | System | Yes | Failed to collect system information |
| `gts.hx.nodes_registry.syscap.collection_failed.v1~` | `gts.hx.system.runtime.internal.v1~` | 500 | System Capabilities Collection Failed | System | Yes | Failed to collect system capabilities |
| `gts.hx.nodes_registry.validation.invalid_input.v1~` | `gts.hx.system.logical.validation_failed.v1~` | 400 | Invalid Input | Client | No | Input validation failed |
| `gts.hx.nodes_registry.system.internal.v1~` | `gts.hx.system.runtime.internal.v1~` | 500 | Internal Error | System | Yes | Internal nodes registry error |

---

### 7. OData Errors (`hx.odata.*`)

| GTS ID | Base | Status | Title | Category | Retryable | Description |
|--------|------|--------|-------|----------|-----------|-------------|
| `gts.hx.odata.query.invalid_filter.v1~` | `gts.hx.system.logical.validation_failed.v1~` | 422 | Invalid Filter | Client | No | OData $filter expression is invalid |
| `gts.hx.odata.query.invalid_orderby.v1~` | `gts.hx.system.logical.validation_failed.v1~` | 422 | Invalid OrderBy | Client | No | OData $orderby expression is invalid |
| `gts.hx.odata.query.invalid_cursor.v1~` | `gts.hx.system.logical.validation_failed.v1~` | 422 | Invalid Cursor | Client | No | Pagination cursor is invalid |
| `gts.hx.odata.system.internal.v1~` | `gts.hx.system.runtime.internal.v1~` | 500 | Internal OData Error | System | Yes | Internal OData processing error |

---

### 8. API Gateway Errors (`hx.api_gateway.*`)

| GTS ID | Base | Status | Title | Category | Retryable | Description |
|--------|------|--------|-------|----------|-----------|-------------|
| `gts.hx.api_gateway.request.bad_request.v1~` | `gts.hx.system.http.bad_request.v1~` | 400 | Bad Request | Client | No | Malformed request |
| `gts.hx.api_gateway.auth.unauthorized.v1~` | `gts.hx.system.http.unauthorized.v1~` | 401 | Unauthorized | Client | No | Authentication required |
| `gts.hx.api_gateway.auth.forbidden.v1~` | `gts.hx.system.http.forbidden.v1~` | 403 | Forbidden | Client | No | Access denied |
| `gts.hx.api_gateway.routing.not_found.v1~` | `gts.hx.system.http.not_found.v1~` | 404 | Not Found | Client | No | Route not found |
| `gts.hx.api_gateway.state.conflict.v1~` | `gts.hx.system.http.conflict.v1~` | 409 | Conflict | Client | No | Request conflicts with current state |
| `gts.hx.api_gateway.rate.limited.v1~` | `gts.hx.system.runtime.rate_limited.v1~` | 429 | Too Many Requests | Operational | Yes | Rate limit exceeded |
| `gts.hx.api_gateway.system.internal.v1~` | `gts.hx.system.runtime.internal.v1~` | 500 | Internal Error | System | Yes | Internal gateway error |

---

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

## References

- [RFC 9457 - Problem Details for HTTP APIs](https://www.rfc-editor.org/rfc/rfc9457.html)
- [Google AIP-193 - Errors](https://google.aip.dev/193)
- [AWS Error Handling Patterns](https://aws.amazon.com/blogs/compute/error-handling-patterns-in-amazon-api-gateway-and-aws-lambda/)
- [Hyperspot GTS_ERRORS.md](../docs/GTS_ERRORS.md)
- [Hyperspot STATUS_CODES.md](../guidelines/DNA/REST/STATUS_CODES.md)
