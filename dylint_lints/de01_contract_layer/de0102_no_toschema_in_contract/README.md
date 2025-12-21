# DE0102: No ToSchema in Contract

### What it does

Checks that structs and enums in contract modules do not derive `ToSchema` from utoipa (OpenAPI schema generation).

### Why is this bad?

Contract models should remain independent of API documentation concerns. OpenAPI schema generation is a presentation layer responsibility and should not be mixed with domain models. Use DTOs (Data Transfer Objects) in the API layer for schema generation instead.

This separation provides:
- **Clear separation of concerns**: Domain logic vs. API documentation
- **Flexibility**: API schemas can differ from internal models
- **Protection**: Contract models stay stable when API documentation changes
- **API versioning**: Different versions can have different schemas

### Example

```rust
// ❌ Bad - contract model derives ToSchema
mod contract {
    use utoipa::ToSchema;
    
    #[derive(ToSchema)]
    pub struct User { 
        pub id: String 
    }
}
```

Use instead:

```rust
// ✅ Good - contract model without ToSchema
mod contract {
    pub struct User { 
        pub id: String 
    }
}

// Separate DTO in API layer with ToSchema
mod api {
    use utoipa::ToSchema;
    
    #[derive(ToSchema)]
    pub struct UserDto { 
        pub id: String 
    }
}
```

### Configuration

This lint is configured to **deny** by default.

### See Also

- [DE0101](../de0101_no_serde_in_contract) - No Serde in Contract
- [DE0103](../de0103_no_http_types_in_contract) - No HTTP Types in Contract
