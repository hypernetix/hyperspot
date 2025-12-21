# DE0103: No HTTP Types in Contract

### What it does

Checks that contract modules do not use HTTP-specific types such as `StatusCode`, `HeaderMap`, `Response`, `Request`, etc.

### Why is this bad?

Contract models represent pure domain logic and should be transport-agnostic. Using HTTP-specific types couples your domain layer to a specific transport protocol (HTTP), making it harder to:
- Reuse domain logic with different transports (gRPC, WebSocket, message queues)
- Test domain logic in isolation
- Maintain clean architecture boundaries

HTTP types belong in the API layer, not the contract layer.

### Detected Types

The lint detects usage of common HTTP types from popular Rust web frameworks:
- `axum`: StatusCode, HeaderMap, Response, Request, Body
- `hyper`: StatusCode, HeaderMap, Response, Request, Body  
- `http`: StatusCode, HeaderMap, Response, Request
- And other HTTP-related types

### Example

```rust
// ❌ Bad - contract uses HTTP types
mod contract {
    use axum::http::StatusCode;
    
    pub struct UserService {
        pub status: StatusCode,
    }
    
    pub fn create_user() -> (StatusCode, String) {
        (StatusCode::OK, "user created".to_string())
    }
}
```

Use instead:

```rust
// ✅ Good - contract uses domain types
mod contract {
    pub enum UserCreationResult {
        Success(String),
        AlreadyExists,
        ValidationError(String),
    }
    
    pub fn create_user() -> UserCreationResult {
        UserCreationResult::Success("user-id-123".to_string())
    }
}

// Map to HTTP in API layer
mod api {
    use axum::http::StatusCode;
    use crate::contract;
    
    pub async fn create_user_handler() -> (StatusCode, String) {
        match contract::create_user() {
            UserCreationResult::Success(id) => (StatusCode::CREATED, id),
            UserCreationResult::AlreadyExists => (StatusCode::CONFLICT, "User exists".into()),
            UserCreationResult::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
        }
    }
}
```

### Configuration

This lint is configured to **deny** by default.

### See Also

- [DE0101](../de0101_no_serde_in_contract) - No Serde in Contract
- [DE0102](../de0102_no_toschema_in_contract) - No ToSchema in Contract
