# de0104_no_api_dto_in_contract

## What it does

Checks that structs and enums in contract modules do not use the `api_dto` attribute macro.

## Why is this bad?

Contract models should remain independent of API serialization concerns. The `api_dto` macro is specifically designed for API DTOs (Data Transfer Objects) and should only be used in the API layer, not in contract models.

Using `api_dto` in contract modules violates the separation of concerns between the contract layer (domain models) and the API layer (data transfer objects). This separation ensures:

- **Layer independence**: Contract models can evolve without being tied to API representation
- **Clear boundaries**: API concerns (serialization, validation, OpenAPI schema) stay in the API layer
- **Reusability**: Contract models can be used across different API versions or transport layers

## Known problems

None.

## Example

```rust
// Bad - contract model uses api_dto
mod contract {
    #[modkit_macros::api_dto(request, response)]
    pub struct User {
        pub id: String,
        pub name: String,
    }
}
```

Use instead:

```rust
// Good - contract model without api_dto
mod contract {
    pub struct User {
        pub id: String,
        pub name: String,
    }
}

// Separate DTO in API layer
mod api {
    mod rest {
        #[modkit_macros::api_dto(request, response)]
        pub struct UserDto {
            pub id: String,
            pub name: String,
        }
    }
}
```
