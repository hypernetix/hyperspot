# OData: $filter, $orderby, $select, and Pagination

ModKit provides OData query support with type-safe filtering, ordering, field selection, and cursor-based pagination. See `docs/ODATA_SELECT.md` and `docs/ODATA_MACRO_MIGRATION.md` for detailed guides.

## Core invariants

- **Rule**: Use `modkit_odata_macros::ODataFilterable` for DTO filtering.
- **Rule**: Use `OperationBuilderODataExt` helpers instead of manual `.query_param(...)`.
- **Rule**: Use `apply_select()` for field projection in handlers.
- **Rule**: Return `Page<T>` from domain services.
- **Rule**: Use `page_to_projected_json()` for JSON responses with $select.

## OData macro migration

### Before (old)

```rust
use modkit_db_macros::ODataFilterable;
```

### After (current)

```rust
use modkit_odata_macros::ODataFilterable;
```

## DTO with OData filtering

### Define filterable DTO

```rust
use chrono::{DateTime, Utc};
use modkit_odata_macros::ODataFilterable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// REST DTO for user representation with OData filtering
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, ODataFilterable)]
pub struct UserDto {
    #[odata(filter(kind = "Uuid"))]
    pub id: Uuid,
    #[odata(filter(kind = "Uuid"))]
    pub tenant_id: Uuid,
    #[odata(filter(kind = "String"))]
    pub email: String,
    pub display_name: String,
    #[odata(filter(kind = "DateTimeUtc"))]
    pub created_at: DateTime<Utc>,
    #[odata(filter(kind = "DateTimeUtc"))]
    pub updated_at: DateTime<Utc>,
}
```

### Filter field kinds

| Kind | Type | Example |
|------|------|---------|
| `String` | `String` | `email eq 'test@example.com'` |
| `Uuid` | `uuid::Uuid` | `id eq 550e8400-e29b-41d4-a716-446655440000` |
| `DateTimeUtc` | `chrono::DateTime<Utc>` | `created_at gt 2024-01-01T00:00:00Z` |
| `I32` | `i32` | `age gt 18` |
| `I64` | `i64` | `count ge 100` |
| `Bool` | `bool` | `is_active eq true` |

## OperationBuilder with OData

### OData-enabled list endpoint

```rust
use modkit::api::operation_builder::{OperationBuilderODataExt};

OperationBuilder::get("/users-info/v1/users")
    .operation_id("users_info.list_users")
    .require_auth(&Resource::Users, &Action::Read)
    .handler(handlers::list_users)
    .json_response_with_schema::<modkit_odata::Page<dto::UserDto>>(
        openapi,
        StatusCode::OK,
        "Paginated list of users",
    )
    .with_odata_filter::<dto::UserDtoFilterField>() // not .query_param("$filter", ...)
    .with_odata_select() // not .query_param("$select", ...)
    .with_odata_orderby::<dto::UserDtoFilterField>() // not .query_param("$orderby", ...)
    .standard_errors(openapi)
    .register(router, openapi);
```

## Handler with OData

### Basic OData handler

```rust
use modkit::api::prelude::*;
use modkit::api::odata::OData;
use modkit_auth::axum_ext::Authz;

pub async fn list_users(
    Authz(ctx): Authz,
    Extension(svc): Extension<Arc<Service>>,
    OData(query): OData,
) -> ApiResult<JsonPage<serde_json::Value>> {
    let page: modkit_odata::Page<user_info_sdk::User> =
        svc.users.list_users_page(&ctx, &query).await?;
    let page = page.map_items(UserDto::from);
    Ok(Json(page_to_projected_json(&page, query.selected_fields())))
}
```

### Domain service with OData

```rust
impl UserService {
    pub async fn list_users_page(
        &self,
        ctx: &SecurityCtx,
        query: &ODataQuery,
    ) -> Result<Page<User>, DomainError> {
        let secure_conn = self.db.sea_secure();
        
        // Build base query with OData filter
        let mut db_query = secure_conn.find::<user::Entity>(ctx)?;
        
        // Apply $filter
        if let Some(filter) = query.filter() {
            db_query = db_query.filter(filter.to_sea_condition::<user::Entity>());
        }
        
        // Apply $orderby
        if let Some(order) = query.orderby() {
            db_query = db_query.order_by(order.to_sea_order::<user::Entity>());
        }
        
        // Apply pagination
        let limit = query.limit().unwrap_or(50);
        let cursor = query.cursor();
        
        let page = if let Some(cursor) = cursor {
            db_query.find_after_cursor(cursor, Some(limit), None).await?
        } else {
            db_query.paginate(secure_conn.conn(), limit).await?
        };
        
        Ok(page)
    }
}
```

## Field projection ($select)

### apply_select helper

```rust
use modkit::api::select::apply_select;

pub async fn list_users(
    Authz(ctx): Authz,
    Extension(svc): Extension<Arc<Service>>,
    OData(query): OData,
) -> ApiResult<JsonPage<serde_json::Value>> {
    let page: Page<User> = svc.list_users_page(&ctx, &query).await?;
    let page = page.map_items(|user| UserDto::from(user));
    
    // Apply field projection
    let page = if let Some(fields) = query.selected_fields() {
        page.map_items(|dto| apply_select(&dto, fields))
    } else {
        page
    };
    
    Ok(Json(page_to_projected_json(&page, query.selected_fields())))
}
```

### page_to_projected_json

```rust
use modkit::api::select::page_to_projected_json;

pub async fn list_users(
    Authz(ctx): Authz,
    Extension(svc): Extension<Arc<Service>>,
    OData(query): OData,
) -> ApiResult<JsonPage<serde_json::Value>> {
    let page: Page<User> = svc.list_users_page(&ctx, &query).await?;
    let page = page.map_items(UserDto::from);
    Ok(Json(page_to_projected_json(&page, query.selected_fields())))
}
```

## Cursor-based pagination

### Page structure

```rust
use modkit_odata::Page;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_next: bool,
    pub total: Option<u64>,
}
```

### Cursor handling

```rust
// In domain service
impl UserService {
    pub async fn list_users_page(
        &self,
        ctx: &SecurityCtx,
        query: &ODataQuery,
    ) -> Result<Page<User>, DomainError> {
        let secure_conn = self.db.sea_secure();
        let mut db_query = secure_conn.find::<user::Entity>(ctx)?;
        
        // Apply filter and order
        if let Some(filter) = query.filter() {
            db_query = db_query.filter(filter.to_sea_condition::<user::Entity>());
        }
        if let Some(order) = query.orderby() {
            db_query = db_query.order_by(order.to_sea_order::<user::Entity>());
        }
        
        // Apply cursor pagination
        let limit = query.limit().unwrap_or(50);
        let page = if let Some(cursor) = query.cursor() {
            db_query.find_after_cursor(cursor, Some(limit), None).await?
        } else {
            db_query.paginate(secure_conn.conn(), limit).await?
        };
        
        Ok(page)
    }
}
```

## Common OData queries

### Filter examples

```bash
# String equality
$filter=email eq 'test@example.com'

# String contains (requires custom implementation)
$filter=contains(email, 'test')

# UUID comparison
$filter=id eq 550e8400-e29b-41d4-a716-446655440000

# DateTime comparison
$filter=created_at gt 2024-01-01T00:00:00Z

# Logical operators
$filter=email eq 'test@example.com' and created_at gt 2024-01-01T00:00:00Z
$filter=age gt 18 or age lt 65
```

### Order examples

```bash
# Single field
$orderby=email

# Multiple fields
$orderby=created_at desc,email

# With direction
$orderby=created_at asc
```

### Select examples

```bash
# Single field
$select=id

# Multiple fields
$select=id,email,created_at

# Nested fields (if supported)
$select=id,name/display_name
```

### Combined examples

```bash
# Full query
/users-info/v1/users?$filter=email eq 'test@example.com'&$orderby=created_at desc&$select=id,email,created_at&$top=20

# With cursor
/users-info/v1/users?$cursor=eyJpZCI6IjU1MGU4NDAwLWUyOWItNDFkNC1hNzE2LTQ0NjY1NTQ0MDAwMCJ9&$top=20
```

## Error handling

### OData errors

```rust
use modkit::api::odata::ODataError;

impl From<ODataError> for Problem {
    fn from(err: ODataError) -> Self {
        match err {
            ODataError::InvalidFilter(msg) => Problem::builder()
                .type_url(ProblemType::BadRequest)
                .title("Invalid filter")
                .detail(format!("Invalid $filter: {}", msg))
                .build(),
            ODataError::InvalidOrderBy(msg) => Problem::builder()
                .type_url(ProblemType::BadRequest)
                .title("Invalid orderby")
                .detail(format!("Invalid $orderby: {}", msg))
                .build(),
            ODataError::InvalidSelect(msg) => Problem::builder()
                .type_url(ProblemType::BadRequest)
                .title("Invalid select")
                .detail(format!("Invalid $select: {}", msg))
                .build(),
        }
    }
}
```

## Testing OData

### Test filter parsing

```rust
#[tokio::test]
async fn test_odata_filter_parsing() {
    let query = ODataQuery::from_str("?$filter=email eq 'test@example.com'").unwrap();
    assert!(query.filter().is_some());
    
    let filter = query.filter().unwrap();
    // Test filter conversion to SeaORM condition
    let condition = filter.to_sea_condition::<user::Entity>();
    // Verify condition
}
```

### Test field projection

```rust
#[tokio::test]
async fn test_field_projection() {
    let dto = UserDto {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        display_name: "Test User".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    let fields = SelectedFields::from_str("id,email").unwrap();
    let projected = apply_select(&dto, &fields);
    
    let expected = json!({
        "id": dto.id,
        "email": dto.email
    });
    
    assert_eq!(projected, expected);
}
```

## Quick checklist

- [ ] Add `#[derive(ODataFilterable)]` on DTOs with `#[odata(filter(kind = "..."))]`.
- [ ] Import `modkit_odata_macros::ODataFilterable`.
- [ ] Use `OperationBuilderODataExt` helpers (`.with_odata_*()`).
- [ ] Use `OData(query)` extractor in handlers.
- [ ] Return `Page<T>` from domain services.
- [ ] Use `page_to_projected_json()` for responses with $select.
- [ ] Add `.standard_errors()` for OData error handling.
