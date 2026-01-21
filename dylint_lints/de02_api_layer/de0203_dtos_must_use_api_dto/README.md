# DE0203: DTOs Must Use api_dto Macro

### What it does

Checks that all DTO types (structs/enums ending with `Dto`) in the API layer use the `#[modkit_macros::api_dto(...)]` macro instead of manually adding serde derives.

### Why is this bad?

The `api_dto` macro ensures consistent serialization behavior across all DTOs by automatically adding:
- `Serialize` and `Deserialize` derives (based on `request`/`response` arguments)
- `ToSchema` derive for OpenAPI documentation
- `#[serde(rename_all = "snake_case")]` for consistent field naming

Manually adding these derives:
- **Inconsistent**: Different DTOs may have different configurations
- **Error-prone**: Easy to forget ToSchema or snake_case renaming
- **Maintenance burden**: Changes to DTO standards require updating every DTO
- **Missing features**: May not include all required derives and attributes

### Example

```rust
// ❌ Bad - DTO with manual derives
// File: src/api/rest/dto.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
    pub name: String,
}
```

```rust
// ❌ Bad - DTO with manual derives and ToSchema
// File: src/api/rest/dto.rs
use serde::{Serialize, Deserialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub struct UserDto {
    pub id: String,
    pub name: String,
}
```

Use instead:

```rust
// ✅ Good - DTO using api_dto macro for request and response
// File: src/api/rest/dto.rs
#[modkit_macros::api_dto(request, response)]
pub struct UserDto {
    pub id: String,
    pub name: String,
}

// ✅ Good - DTO for request only
#[modkit_macros::api_dto(request)]
pub struct CreateUserReq {
    pub name: String,
    pub email: String,
}

// ✅ Good - DTO for response only
#[modkit_macros::api_dto(response)]
pub struct UserResponseDto {
    pub id: String,
    pub name: String,
}
```

### Configuration

This lint is configured to **deny** by default.

It checks all types with names ending in `Dto` (case-insensitive) in `*/api/rest/*.rs` files.

### See Also

- [DE0201](../de0201_dtos_only_in_api_rest) - DTOs Only in API Rest Folder
- [DE0204](../de0204_dtos_must_have_toschema_derive) - DTOs Must Have ToSchema Derive
