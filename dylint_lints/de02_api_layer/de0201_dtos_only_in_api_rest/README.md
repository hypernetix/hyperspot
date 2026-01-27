# DE0201: DTOs Only in API Rest Folder

### What it does

Checks that types with DTO suffixes (e.g., `UserDto`, `ProductDto`) are only defined in `*/api/rest/*.rs` files.

### Why is this bad?

DTOs (Data Transfer Objects) are specifically designed for API communication and should be colocated with the API layer code. Defining DTOs outside the API folder can lead to:
- **Confusion**: Unclear which types are for API vs. internal use
- **Coupling**: Non-API code depending on API-specific structures
- **Organization**: Scattered API concerns across the codebase

### Example

```rust
// ❌ Bad - DTO defined in domain folder
// File: src/domain/user.rs
pub struct UserDto {
    pub id: String,
    pub name: String,
}
```

Use instead:

```rust
// ✅ Good - DTO defined in api/rest folder
// File: src/api/rest/dto.rs
pub struct UserDto {
    pub id: String,
    pub name: String,
}

// Domain types stay in domain folder
// File: src/domain/user.rs
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,  // May have fields DTOs don't expose
}
```

### Configuration

This lint is configured to **deny** by default.

Types matching the pattern `*Dto` (case-insensitive) must be in files matching `*/api/rest/*.rs`.
