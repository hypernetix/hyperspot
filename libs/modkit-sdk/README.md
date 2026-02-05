# modkit-sdk

Security Context Scoping and Typed OData Query Builder for Clients

## Overview

This crate provides two main features:

1. **Security Context Scoping**: A lightweight, zero-allocation wrapper that binds a `SecurityContext` to any client type
2. **Typed OData Query Builder**: A generic, reusable query builder that produces type-safe OData queries with automatic filter hashing

## Security Context Scoping

### Example

```rust
use modkit_sdk::WithSecurityContext;
use modkit_security::SecurityContext;

let client = MyClient::new();
let ctx = SecurityContext::root();

// Bind the security context to the client
let secured = client.security_ctx(&ctx);

// Access the client and context
let client_ref = secured.client();
let ctx_ref = secured.ctx();
```

## Typed OData Query Builder

The typed OData query builder provides a type-safe way to construct OData queries without manually building `ODataQuery` instances. It ensures compile-time type checking for field references and operations.

### Features

- **Type-safe field references**: Field types are checked at compile time
- **Schema trait**: Define field enums and their string mappings
- **Typed filter constructors**: Comparison and string operations with type safety
- **Fluent API**: Chain methods to build complex queries
- **Automatic filter hashing**: Stable, deterministic hashing for cursor pagination
- **No proc macros**: Pure Rust implementation without code generation

### Quick Start

#### 1. Define Your Schema

```rust
use modkit_sdk::odata::{Schema, FieldRef};

// Define field enum
#[derive(Copy, Clone, Eq, PartialEq)]
enum UserField {
    Id,
    Name,
    Email,
    Age,
}

// Define schema struct
struct UserSchema;

// Implement Schema trait
impl Schema for UserSchema {
    type Field = UserField;

    fn field_name(field: Self::Field) -> &'static str {
        match field {
            UserField::Id => "id",
            UserField::Name => "name",
            UserField::Email => "email",
            UserField::Age => "age",
        }
    }
}
```

#### 2. Create Typed Field References

```rust
// Define typed field constants
const ID: FieldRef<UserSchema, uuid::Uuid> = FieldRef::new(UserField::Id);
const NAME: FieldRef<UserSchema, String> = FieldRef::new(UserField::Name);
const EMAIL: FieldRef<UserSchema, String> = FieldRef::new(UserField::Email);
const AGE: FieldRef<UserSchema, i32> = FieldRef::new(UserField::Age);
```

#### 3. Build Queries

```rust
use modkit_sdk::odata::{QueryBuilder, FilterExpr};
use modkit_odata::SortDir;

// Simple equality filter
let user_id = uuid::Uuid::new_v4();
let query = QueryBuilder::<UserSchema>::new()
    .filter(ID.eq(user_id))
    .build();

// Complex filter with AND/OR
let query = QueryBuilder::<UserSchema>::new()
    .filter(
        AGE.ge(18)
            .and(AGE.le(65))
            .and(NAME.contains("smith"))
    )
    .order_by(NAME, SortDir::Asc)
    .page_size(50)
    .build();

// Full query with all features
let query = QueryBuilder::<UserSchema>::new()
    .filter(ID.eq(user_id).and(AGE.gt(18)))
    .order_by(NAME, SortDir::Asc)
    .order_by(AGE, SortDir::Desc)
    .select([NAME, EMAIL])
    .page_size(25)
    .build();
```

### Supported Operations

#### Comparison Operators (All Field Types)

- `eq(value)` - Equality: `field eq value`
- `ne(value)` - Not equal: `field ne value`
- `gt(value)` - Greater than: `field gt value`
- `ge(value)` - Greater or equal: `field ge value`
- `lt(value)` - Less than: `field lt value`
- `le(value)` - Less or equal: `field le value`

#### String Operations (String Fields Only)

- `contains(value)` - Contains: `contains(field, 'value')`
- `startswith(value)` - Starts with: `startswith(field, 'value')`
- `endswith(value)` - Ends with: `endswith(field, 'value')`

#### Logical Combinators

- `and(expr)` - Logical AND: `expr1 and expr2`
- `or(expr)` - Logical OR: `expr1 or expr2`
- `not()` - Logical NOT: `not expr`

#### Query Builder Methods

- `filter(expr)` - Set the filter expression
- `order_by(field, dir)` - Add an order-by clause (can be called multiple times)
- `select(fields)` - Set field projection (pass `&[&field1, &field2, ...]`)
- `page_size(limit)` - Set the page size limit
- `build()` - Build the final `ODataQuery` with computed filter hash

### Type Safety

The query builder enforces type safety at compile time:

```rust
// ✅ Correct: String field with string operations
let query = QueryBuilder::<UserSchema>::new()
    .filter(NAME.contains("john"))
    .build();

// ❌ Compile error: contains() only available for String fields
let query = QueryBuilder::<UserSchema>::new()
    .filter(AGE.contains("test"))  // Won't compile!
    .build();

// ✅ Correct: Comparison operations work on all types
let query = QueryBuilder::<UserSchema>::new()
    .filter(AGE.gt(18))
    .build();
```

### Filter Hash Stability

The query builder automatically computes a stable, deterministic hash for filter expressions using the same algorithm as `modkit_odata::pagination::short_filter_hash`. This ensures cursor pagination consistency:

```rust
let user_id = uuid::Uuid::new_v4();

let query1 = QueryBuilder::<UserSchema>::new()
    .filter(ID.eq(user_id))
    .build();

let query2 = QueryBuilder::<UserSchema>::new()
    .filter(ID.eq(user_id))
    .build();

// Same filter produces same hash
assert_eq!(query1.filter_hash, query2.filter_hash);
```

### Supported Value Types

The following Rust types can be used in filter expressions:

- `bool`
- `uuid::Uuid`
- `String` and `&str`
- `i32`, `i64`, `u32`, `u64`

Additional types can be supported by implementing the `IntoODataValue` trait.

### Examples

See `examples/typed_odata_query.rs` for examples demonstrating:

- Simple equality filters
- String operations (contains, startswith, endswith)
- Complex filters with AND/OR/NOT
- Ordering and field selection
- Page size limits
- Full queries with all features
- Filter hash stability

Run the example:

```bash
cargo run --package modkit-sdk --example typed_odata_query
```

### Design Constraints

- **No proc macros**: Pure Rust implementation without code generation
- **Small footprint**: Minimal API surface with no large facades
- **No DB concepts**: Does not expose database or SeaORM concepts
- **AST-based**: Produces `modkit_odata::ast::Expr` for maximum flexibility
- **Stable hashing**: Deterministic filter hashing for cursor pagination

### Testing

The query builder includes comprehensive unit tests verifying:

- Field name mapping works correctly
- Building queries sets order/limit/filter properly
- Filter hash is stable for identical filters
- All comparison and string operations work
- Logical combinators (AND/OR/NOT) function correctly
- Field selection handles heterogeneous field types

Run tests:

```bash
cargo test --package modkit-sdk --lib odata
```

## License

See workspace license.
