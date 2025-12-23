# DE0203: DTOs Must Have Serde Derives

### What it does

Checks that all DTO types (structs/enums ending with `Dto`) in the API layer have both `Serialize` and `Deserialize` derives from serde.

### Why is this bad?

DTOs (Data Transfer Objects) are specifically designed for API serialization/deserialization. A DTO without serde derives:
- **Cannot be serialized**: Won't work with JSON, MessagePack, or other formats
- **Incomplete API contract**: May be meant for API but missing required traits
- **Likely a mistake**: Forgot to add derives or misnamed type
- **Inconsistent**: Other DTOs have derives, this one should too

### Example

```rust
// ❌ Bad - DTO without serde derives
// File: src/api/rest/dto.rs
pub struct UserDto {
    pub id: String,
    pub name: String,
}
```

```rust
// ❌ Bad - DTO with only Serialize (missing Deserialize)
// File: src/api/rest/dto.rs
use serde::Serialize;

#[derive(Serialize)]
pub struct UserDto {
    pub id: String,
    pub name: String,
}
```

Use instead:

```rust
// ✅ Good - DTO with both Serialize and Deserialize
// File: src/api/rest/dto.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct UserDto {
    pub id: String,
    pub name: String,
}

// ✅ Also good - with additional derives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductDto {
    pub id: String,
    pub price: f64,
}
```

### Configuration

This lint is configured to **deny** by default.

It checks all types with names ending in `Dto` (case-insensitive) in `*/api/rest/*.rs` files.

### See Also

- [DE0201](../de0201_dtos_only_in_api_rest) - DTOs Only in API Rest Folder
- [DE0204](../de0204_dtos_must_have_toschema_derive) - DTOs Must Have ToSchema Derive
