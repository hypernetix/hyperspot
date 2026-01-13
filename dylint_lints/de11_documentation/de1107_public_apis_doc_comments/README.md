# DE1107: Public APIs Doc Comments

### What it does

Checks that public items in contract modules have documentation comments.

### Why is this bad?

Contract types are the public API of a module and should be well-documented:
- **API consumers need guidance**: External users don't know implementation details
- **Maintenance**: Future developers need to understand the purpose of types
- **Consistency**: Public APIs should have consistent documentation standards
- **Professionalism**: Well-documented code indicates code quality

### Example

```rust
// ❌ Bad - no doc comment on public struct in contract
// File: src/contract/user.rs
pub struct User {
    pub id: Uuid,
    pub name: String,
}
```

```rust
// ❌ Bad - no doc comment on public enum
pub enum UserRole {
    Admin,
    User,
    Guest,
}
```

Use instead:

```rust
// ✅ Good - documented public struct
// File: src/contract/user.rs

/// A user entity representing a registered user in the system.
///
/// Users can have different roles and permissions.
pub struct User {
    /// Unique identifier for the user.
    pub id: Uuid,
    
    /// Display name of the user.
    pub name: String,
    
    /// User's role determining their permissions.
    pub role: UserRole,
}
```

```rust
// ✅ Good - documented enum with variants
/// Defines the role and permission level of a user.
pub enum UserRole {
    /// Administrator with full system access.
    Admin,
    
    /// Regular user with standard permissions.
    User,
    
    /// Guest with read-only access.
    Guest,
}
```

### Configuration

This lint is configured to **warn** by default.

It checks all public items (structs, enums, functions, traits) in `*/contract/*.rs` files for the presence of doc comments (`///`).

### See Also

- [Rust API Guidelines - Documentation](https://rust-lang.github.io/api-guidelines/documentation.html)
