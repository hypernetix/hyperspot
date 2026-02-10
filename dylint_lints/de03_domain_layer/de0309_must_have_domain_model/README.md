# DE0309: Must Have Domain Model Attribute

## What it does

Checks that all struct and enum types in domain modules have the `#[domain_model]` attribute.

## Why is this important?

The `#[domain_model]` macro provides **compile-time validation** of Domain-Driven Design (DDD) boundaries. It ensures that domain types don't contain infrastructure dependencies such as:

- HTTP types (`http::StatusCode`, `axum::*`)
- Database types (`sqlx::PgPool`, `sea_orm::*`)
- File system types (`std::fs::*`, `tokio::fs::*`)
- External service clients (`reqwest::*`, `tonic::*`)

By requiring this attribute on all domain types, we guarantee that infrastructure concerns cannot leak into the domain layer.

## Example

### Bad

```rust
// src/domain/user.rs

pub struct User {           // Missing #[domain_model]
    pub id: Uuid,
    pub email: String,
}
```

### Good

```rust
// src/domain/user.rs
use modkit_macros::domain_model;

#[domain_model]
pub struct User {
    pub id: Uuid,
    pub email: String,
}
```

## Configuration

This lint is configured to **deny** by default.

It checks all `struct` and `enum` definitions in files whose path contains `/domain/`.

## TDD Approach

This lint is designed for Test-Driven Development:

1. **Add the lint** - CI will fail for all domain types without the attribute
2. **Fix each violation** - Add `#[domain_model]` to all domain types
3. **CI passes** - All domain types are now validated at compile time

## See Also

- [`#[domain_model]` macro documentation](../../../libs/modkit-macros/src/domain_model.rs)
- [Domain Layer Architecture](../../../docs/modkit_unified_system/02_module_layout_and_sdk_pattern.md)
