# DE0301: No Infrastructure in Domain

### What it does

Checks that domain modules do not import infrastructure dependencies.

### Why is this bad?

Domain modules should contain pure business logic and depend only on abstractions (ports), not concrete implementations:
- **Violates Dependency Inversion Principle**: Domain depends on low-level details
- **Harder to test**: Requires infrastructure setup for domain tests
- **Tight coupling**: Changes to infrastructure affect domain logic
- **Prevents portability**: Cannot easily swap infrastructure implementations

### Example

```rust
// ❌ Bad - infrastructure imports in domain
// File: src/domain/users.rs
use crate::infra::storage::UserRepository;  // concrete implementation
use sea_orm::*;  // database framework
use sqlx::*;     // database driver

pub struct UserService {
    repo: UserRepository,  // concrete type
}
```

Use instead:

```rust
// ✅ Good - domain depends on abstractions
// File: src/domain/users.rs
use std::sync::Arc;
use uuid::Uuid;

pub trait UsersRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<User, DomainError>;
}

pub struct UserService {
    repo: Arc<dyn UsersRepository>,  // trait object
}
```

### Configuration

This lint is configured to **deny** by default.

It checks all imports in `*/domain/*.rs` files for references to:
- `sea_orm`, `sqlx` (database frameworks)
- `infra::*` (infrastructure modules)

### See Also

- [DE0308](../de0308_no_http_in_domain) - No HTTP in Domain Layer
