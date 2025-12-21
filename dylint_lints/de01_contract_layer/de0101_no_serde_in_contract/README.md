# DE0101: No Serde in Contract

### What it does

Checks that structs and enums in contract modules do not derive `Serialize` or `Deserialize` from serde.

### Why is this bad?

Contract models should remain independent of serialization concerns. They represent pure domain logic and should not be coupled to any specific serialization format or library. Use DTOs (Data Transfer Objects) in the API layer for serialization instead.

This separation provides:
- **Clear separation of concerns**: Domain logic vs. API representation
- **Flexibility**: Different API endpoints can use different serialization strategies
- **Protection**: Contract models stay stable when API format changes

### Example

```rust
// ❌ Bad - contract model derives serde traits
mod contract {
    use serde::Serialize;
    
    #[derive(Serialize)]
    pub struct User { 
        pub id: String 
    }
}
```

Use instead:

```rust
// ✅ Good - contract model without serde
mod contract {
    pub struct User { 
        pub id: String 
    }
}

// Separate DTO in API layer
mod api {
    use serde::Serialize;
    
    #[derive(Serialize)]
    pub struct UserDto { 
        pub id: String 
    }
}
```

### Configuration

This lint is configured to **deny** by default.

### See Also

- [DE0102](../de0102_no_toschema_in_contract) - No ToSchema in Contract
- [DE0103](../de0103_no_http_types_in_contract) - No HTTP Types in Contract
