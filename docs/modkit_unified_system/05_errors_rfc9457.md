# RFC 9457 Problem Errors

ModKit provides a unified error handling system with `Problem` (RFC-9457) for type-safe error propagation.

## Error Architecture Overview

```
DomainError (business logic)
     ↓ From impl
Problem (RFC-9457, implements IntoResponse)
     ↓
ApiResult<T> = Result<T, Problem>  (handler return type)
```

## Core invariants

- **Rule**: Always return RFC 9457 Problem Details for all 4xx/5xx errors via `Problem` (implements `IntoResponse` directly).
- **Rule**: Do not use `ProblemResponse` (doesn’t exist).
- **Rule**: Use `ApiResult<T>` for handler return types.
- **Rule**: Convert domain errors to `Problem` via `From` impls.

## Error types and placement

| Concern | Type/Concept | File (must define) | Notes |
|---------|--------------|--------------------|-------|
| Domain error (business) | `DomainError` | `<module>/src/domain/error.rs` | Pure business errors; no transport details. Variants reflect domain invariants (e.g., `UserNotFound`, `EmailAlreadyExists`, `InvalidEmail`). |
| SDK error (public) | `<ModuleName>Error` | `<module>-sdk/src/errors.rs` | Transport-agnostic surface for consumers. No `serde` derives. Lives in SDK crate. |
| Domain → SDK error conversion | `impl From<DomainError> for <Sdk>Error` | `<module>/src/domain/error.rs` | Module crate imports SDK error and provides `From` impl. |
| REST error mapping | `impl From<DomainError> for Problem` | `<module>/src/api/rest/error.rs` | Centralize RFC-9457 mapping via `From` trait; `Problem` implements `IntoResponse` directly. |

## Domain Error (`<module>/src/domain/error.rs`)

```rust
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum DomainError {
    #[error("User not found: {id}")]
    UserNotFound { id: uuid::Uuid },

    #[error("Email already exists: {email}")]
    EmailAlreadyExists { email: String },

    #[error("Invalid email: {email}")]
    InvalidEmail { email: String },

    #[error("Tenant not found: {id}")]
    TenantNotFound { id: uuid::Uuid },

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Internal error: {0}")]
    Internal(String),
}
```

## SDK Error (`<module>-sdk/src/errors.rs`)

```rust
#[derive(Error, Debug, Clone)]
pub enum UsersInfoError {
    #[error("User not found: {id}")]
    NotFound { id: Uuid },

    #[error("User with email '{email}' already exists")]
    Conflict { email: String },

    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Internal error")]
    Internal,
}

// Convenience constructors
impl UsersInfoError {
    pub fn not_found(id: Uuid) -> Self { Self::NotFound { id } }
    pub fn conflict(email: String) -> Self { Self::Conflict { email } }
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation { message: message.into() }
    }
    pub fn internal() -> Self { Self::Internal }
}
```

## Domain → SDK conversion (`<module>/src/domain/error.rs`)

```rust
impl From<DomainError> for users_info_sdk::UsersInfoError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::UserNotFound { id } => users_info_sdk::UsersInfoError::not_found(id),
            DomainError::EmailAlreadyExists { email } => users_info_sdk::UsersInfoError::conflict(email),
            DomainError::InvalidEmail { email } => users_info_sdk::UsersInfoError::validation(format!("Invalid email: {email}")),
            DomainError::TenantNotFound { .. } => users_info_sdk::UsersInfoError::validation("Tenant not found"),
            DomainError::PermissionDenied => users_info_sdk::UsersInfoError::validation("Permission denied"),
            DomainError::Database(_) | DomainError::Internal(_) => users_info_sdk::UsersInfoError::internal(),
        }
    }
}
```

## REST Problem mapping (`<module>/src/api/rest/error.rs`)

```rust
use modkit::api::problem::{Problem, ProblemType};
use crate::domain::error::DomainError;

impl From<DomainError> for Problem {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::UserNotFound { id } => Problem::builder()
                .type_url(ProblemType::NotFound)
                .title("User not found")
                .detail(format!("User with id {} not found", id))
                .build(),

            DomainError::EmailAlreadyExists { email } => Problem::builder()
                .type_url(ProblemType::Conflict)
                .title("Email already exists")
                .detail(format!("User with email {} already exists", email))
                .build(),

            DomainError::InvalidEmail { email } => Problem::builder()
                .type_url(ProblemType::BadRequest)
                .title("Invalid email")
                .detail(format!("Invalid email format: {}", email))
                .build(),

            DomainError::TenantNotFound { .. } => Problem::builder()
                .type_url(ProblemType::NotFound)
                .title("Tenant not found")
                .detail("Tenant not found")
                .build(),

            DomainError::PermissionDenied => Problem::builder()
                .type_url(ProblemType::Forbidden)
                .title("Permission denied")
                .detail("Permission denied")
                .build(),

            DomainError::Database(err) => Problem::builder()
                .type_url(ProblemType::InternalServerError)
                .title("Database error")
                .detail(format!("Database error: {}", err))
                .build(),

            DomainError::Internal(msg) => Problem::builder()
                .type_url(ProblemType::InternalServerError)
                .title("Internal error")
                .detail(msg)
                .build(),
        }
    }
}
```

## Handler error propagation

```rust
use modkit::api::prelude::*;
use crate::domain::error::DomainError;

pub async fn get_user(
    Authz(ctx): Authz,
    Extension(svc): Extension<Arc<Service>>,
    Path(id): Path<Uuid>,
) -> ApiResult<JsonBody<UserDto>> {
    // DomainError auto-converts to Problem via From impl
    let user = svc.get_user(&ctx, id).await?;
    Ok(Json(UserDto::from(user)))
}
```

## OperationBuilder error registration

```rust
OperationBuilder::get("/users-info/v1/users/{id}")
    .operation_id("users_info.get_user")
    .require_auth(&Resource::Users, &Action::Read)
    .handler(handlers::get_user)
    .json_response_with_schema::<UserDto>(openapi, StatusCode::OK, "User")
    .error_404(openapi)
    .error_403(openapi)
    .error_500(openapi)
    .register(router, openapi);
```

## ProblemType constants

```rust
use modkit::api::problem::ProblemType;

// Common types
ProblemType::BadRequest
ProblemType::Unauthorized
ProblemType::Forbidden
ProblemType::NotFound
ProblemType::Conflict
ProblemType::UnprocessableEntity
ProblemType::TooManyRequests
ProblemType::InternalServerError
```

## Custom Problem details

```rust
impl From<DomainError> for Problem {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::UserNotFound { id } => Problem::builder()
                .type_url(ProblemType::NotFound)
                .title("User not found")
                .detail(format!("User with id {} not found", id))
                .instance(format!("/users-info/v1/users/{}", id))
                .extensions(serde_json::json!({
                    "user_id": id,
                    "resource": "user"
                }))
                .build(),
            // ... other variants
        }
    }
}
```

## Testing errors

```rust
#[tokio::test]
async fn test_get_user_not_found() {
    let app = test_app().await;
    let response = app
        .get(&format!("/users-info/v1/users/{}", uuid::Uuid::new_v4()))
        .await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);
    let problem: Problem = response.json().await;
    assert_eq!(problem.type_url, "https://httpstatuses.io/404");
    assert_eq!(problem.title, "User not found");
}
```

## Quick checklist

- [ ] Define `DomainError` in `domain/error.rs` with `thiserror::Error`.
- [ ] Define SDK error in `<module>-sdk/src/errors.rs` (transport-agnostic).
- [ ] Implement `From<DomainError> for <Sdk>Error` in module crate.
- [ ] Implement `From<DomainError> for Problem` in `api/rest/error.rs`.
- [ ] Use `ApiResult<T>` in handlers and `?` for error propagation.
- [ ] Register relevant errors in OperationBuilder (`.error_*` or `.standard_errors()`).
- [ ] Do not use `ProblemResponse` (doesn’t exist).
