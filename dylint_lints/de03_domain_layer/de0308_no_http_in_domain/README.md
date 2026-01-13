# DE0308: No HTTP in Domain

### What it does

Checks that domain modules do not reference HTTP types or status codes.

### Why is this bad?

Domain modules should be transport-agnostic:
- **HTTP is just one transport**: Domain logic should work with any protocol (gRPC, WebSockets, CLI)
- **Tight coupling**: Domain becomes dependent on web layer
- **Harder to reuse**: Cannot use domain logic in non-HTTP contexts
- **Violates separation of concerns**: HTTP is a delivery detail, not business logic

### Example

```rust
// ❌ Bad - HTTP types in domain
// File: src/domain/validation.rs
use http::StatusCode;

pub fn check_result() -> StatusCode {
    StatusCode::OK  // HTTP-specific
}
```

```rust
// ❌ Bad - HTTP status in domain error
use http::StatusCode;

pub enum DomainError {
    NotFound(StatusCode),  // HTTP leaking into domain
}
```

Use instead:

```rust
// ✅ Good - domain errors converted in API layer
// File: src/domain/validation.rs
pub enum DomainResult {
    Success,
    NotFound,
    InvalidData,
}

// File: src/api/rest/handlers.rs
impl From<DomainResult> for StatusCode {
    fn from(result: DomainResult) -> Self {
        match result {
            DomainResult::Success => StatusCode::OK,
            DomainResult::NotFound => StatusCode::NOT_FOUND,
            DomainResult::InvalidData => StatusCode::BAD_REQUEST,
        }
    }
}
```

### Configuration

This lint is configured to **deny** by default.

It checks all imports in `*/domain/*.rs` files for references to `http` crate types.

### See Also

- [DE0301](../de0301_no_infra_in_domain) - No Infrastructure in Domain Layer
