# modkit-sdk-macros

Procedural macros for `modkit-sdk` OData schema generation.

## `#[derive(ODataSchema)]`

Automatically generates OData schema boilerplate for DTO structs, including:

- Field enum with variants for each struct field
- Schema implementation mapping fields to OData names
- Typed `FieldRef` constructor functions in a snake_case module

### Example

```rust
use modkit_sdk::ODataSchema;

#[derive(ODataSchema)]
struct User {
    id: uuid::Uuid,
    email: String,
    name: String,
    age: i32,
}

// Generated code includes:
// - UserField enum with Id, Email, Name, Age variants
// - UserSchema struct implementing Schema trait
// - user module with id(), email(), name(), age() constructors

// Usage:
use modkit_sdk::odata::{QueryBuilder, FilterExpr};
use modkit_odata::SortDir;

let user_id = uuid::Uuid::new_v4();
let query = QueryBuilder::<UserSchema>::new()
    .filter(user::id().eq(user_id).and(user::age().ge(18)))
    .order_by(user::name(), SortDir::Asc)
    .select(&[&user::id(), &user::email(), &user::name()])
    .page_size(50)
    .build();

// Type safety is enforced at compile time:
// This works - age is i32
let _ = user::age().gt(18);

// This fails - contains() only works on String fields
// let _ = user::age().contains("test");  // Compile error!

// This fails - field constructors are not generic
// let _ = user::age::<String>();  // Compile error!
```

### Custom Field Names

Use `#[odata(name = "...")]` to override the default field name:

```rust
#[derive(ODataSchema)]
struct Product {
    #[odata(name = "product_id")]
    id: uuid::Uuid,
    #[odata(name = "product_name")]
    name: String,
    price: i32,
}

// ProductSchema::field_name(ProductField::Id) returns "product_id"
// product::id() creates a FieldRef with OData name "product_id"
```

### Generated Code Structure

For a struct named `User`, the macro generates:

1. **Field Enum**: `UserField` with a variant for each field
2. **Schema Struct**: `UserSchema` implementing `modkit_sdk::odata::Schema`
3. **Constructor Module**: `user` module with typed constructor functions

```rust
// Generated:
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum UserField {
    Id,
    Email,
    Name,
    Age,
}

pub struct UserSchema;

impl modkit_sdk::odata::Schema for UserSchema {
    type Field = UserField;
    
    fn field_name(field: Self::Field) -> &'static str {
        match field {
            UserField::Id => "id",
            UserField::Email => "email",
            UserField::Name => "name",
            UserField::Age => "age",
        }
    }
}

pub mod user {
    #[must_use]
    pub fn id() -> modkit_sdk::odata::FieldRef<super::UserSchema, uuid::Uuid> {
        modkit_sdk::odata::FieldRef::new(super::UserField::Id)
    }
    
    #[must_use]
    pub fn email() -> modkit_sdk::odata::FieldRef<super::UserSchema, String> {
        modkit_sdk::odata::FieldRef::new(super::UserField::Email)
    }
    
    #[must_use]
    pub fn name() -> modkit_sdk::odata::FieldRef<super::UserSchema, String> {
        modkit_sdk::odata::FieldRef::new(super::UserField::Name)
    }
    
    #[must_use]
    pub fn age() -> modkit_sdk::odata::FieldRef<super::UserSchema, i32> {
        modkit_sdk::odata::FieldRef::new(super::UserField::Age)
    }
}
```

## Requirements

- Only works on structs with named fields
- Does not support enums or tuple structs
- Field types must be compatible with `FieldRef<S, T>`

## Testing

The crate includes comprehensive tests:

- Unit tests in `tests/odata_schema.rs`
- Compile-time tests using `trybuild` in `tests/compile_tests.rs`
- UI tests for both passing and failing cases

Run tests with:

```bash
cargo test --package modkit-sdk-macros
```
