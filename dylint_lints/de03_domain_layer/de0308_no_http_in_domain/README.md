# DE0308: No HTTP in Domain

## What it does

Checks that domain modules do not reference HTTP types or status codes.

## Why is this bad?

Domain modules should be transport-agnostic:
- **HTTP is just one transport**: Domain logic should work with any protocol (gRPC, WebSockets, CLI)
- **Tight coupling**: Domain becomes dependent on web layer
- **Harder to reuse**: Cannot use domain logic in non-HTTP contexts
- **Violates separation of concerns**: HTTP is a delivery detail, not business logic

## Example

```rust
// ❌ Bad - HTTP types in domain
// File: src/domain/error.rs
use http::StatusCode;

pub enum DomainError {
    NotFound(StatusCode),  // HTTP leaking into domain
}
```

```rust
// ❌ Bad - HTTP status in domain function
use axum::http::StatusCode;

pub fn validate_user() -> StatusCode {
    StatusCode::OK  // Domain should not return HTTP types
}
```

Use instead:

```rust
// ✅ Good - domain errors are transport-agnostic
// File: src/domain/error.rs
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("User not found: {id}")]
    UserNotFound { id: Uuid },
    
    #[error("Email '{email}' already exists")]
    EmailAlreadyExists { email: String },
    
    #[error("Validation failed: {field}: {message}")]
    Validation { field: String, message: String },
    
    #[error("Database error: {message}")]
    Database { message: String },
}
```

```rust
// ✅ Good - API layer handles HTTP mapping
// File: src/api/rest/error.rs
use modkit::api::problem::Problem;
use crate::domain::error::DomainError;

impl From<DomainError> for Problem {
    fn from(e: DomainError) -> Self {
        match &e {
            DomainError::UserNotFound { id } => {
                ErrorCode::user_not_found_v1()
                    .with_context(format!("User {id} not found"), "/", None)
            }
            DomainError::Validation { .. } => {
                ErrorCode::validation_error_v1()
                    .with_context(e.to_string(), "/", None)
            }
            _ => ErrorCode::internal_error_v1()
                    .with_context("Internal error", "/", None)
        }
    }
}
```

## Configuration

This lint is configured to **deny** by default.

It checks all imports in `*/domain/*.rs` files for references to `http` crate types.

### See Also

- [DE0301](../de0301_no_infra_in_domain) - No Infrastructure in Domain Layer
