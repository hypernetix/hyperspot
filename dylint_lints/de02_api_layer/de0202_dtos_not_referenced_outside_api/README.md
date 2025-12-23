# DE0202: DTOs Not Referenced Outside API

### What it does

Checks that DTO types defined in the API layer are not referenced in domain, contract, or infrastructure layers.

### Why is this bad?

DTOs are API-specific representations designed for external communication. When non-API layers depend on DTOs, it creates:
- **Wrong dependencies**: Domain logic should not know about API representations
- **Tight coupling**: Changes to API formats affect domain logic
- **Poor architecture**: Violates clean architecture principles
- **Testing difficulties**: Harder to test domain logic in isolation

The data flow should be: Contract → Domain → API (with DTOs), never the reverse.

### Example

```rust
// ❌ Bad - domain layer uses DTO
// File: src/domain/user_service.rs
use crate::api::rest::UserDto;

pub fn process_user(dto: UserDto) {  // Wrong layer dependency
    // domain logic
}
```

Use instead:

```rust
// ✅ Good - domain uses contract types, API converts
// File: src/contract/user.rs
pub struct User {
    pub id: String,
    pub name: String,
}

// File: src/domain/user_service.rs
use crate::contract::User;

pub fn process_user(user: User) {  // ✅ Uses contract type
    // domain logic
}

// File: src/api/rest/handlers.rs
use crate::contract::User;
use crate::api::rest::UserDto;

pub async fn create_user(dto: UserDto) -> Result<()> {
    // Convert DTO to contract type
    let user = User {
        id: dto.id,
        name: dto.name,
    };
    
    // Pass contract type to domain
    domain::process_user(user)
}
```

### Configuration

This lint is configured to **deny** by default.

It checks that paths in domain, contract, and infra layers do not reference types ending with `Dto`.

### See Also

- [DE0201](../de0201_dtos_only_in_api_rest) - DTOs Only in API Rest Folder
- [DE0203](../de0203_dtos_must_have_serde_derives) - DTOs Must Have Serde Derives
