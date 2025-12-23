# DE0204: DTOs Must Have ToSchema Derive

### What it does

Checks that all DTO types (structs/enums ending with `Dto`) in the API layer derive `ToSchema` from utoipa for OpenAPI documentation.

### Why is this bad?

DTOs are the API's public contract and should be documented in OpenAPI specs. A DTO without `ToSchema`:
- **Missing API documentation**: Won't appear in Swagger/OpenAPI docs
- **Incomplete API contract**: Clients can't discover the schema
- **Likely a mistake**: Forgot to add derive or expose in docs
- **Inconsistent**: Other DTOs have ToSchema, this one should too

OpenAPI documentation is essential for:
- API discoverability
- Client code generation
- Integration testing
- API versioning tracking

### Example

```rust
// ❌ Bad - DTO without ToSchema
// File: src/api/rest/dto.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
    pub name: String,
}
```

Use instead:

```rust
// ✅ Good - DTO with ToSchema
// File: src/api/rest/dto.rs
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserDto {
    pub id: String,
    pub name: String,
}

// ✅ Also good - with documentation
#[derive(Serialize, Deserialize, ToSchema)]
#[schema(example = json!({"id": "123", "name": "John"}))]
pub struct ProductDto {
    /// Unique product identifier
    pub id: String,
    /// Product display name
    pub name: String,
}
```

### Configuration

This lint is configured to **deny** by default.

It checks all types with names ending in `Dto` (case-insensitive) in `*/api/rest/*.rs` files.

### See Also

- [DE0201](../de0201_dtos_only_in_api_rest) - DTOs Only in API Rest Folder
- [DE0203](../de0203_dtos_must_have_serde_derives) - DTOs Must Have Serde Derives
