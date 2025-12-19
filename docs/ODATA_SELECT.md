# OData Field Projection via $select

## Overview

The `$select` OData query option enables sparse field selection, allowing clients to request only the fields they need from API responses. This reduces bandwidth and improves performance by returning only the requested fields instead of full resource representations.

## Format

```
$select=field1,field2,field3
```

Field names are case-insensitive and whitespace is trimmed. Multiple fields are separated by commas.

### Dot Notation for Nested Fields

Use dot notation to select specific nested fields:

```
$select=access_control.read,access_control.write
```

This includes only the `read` and `write` fields within `access_control`, filtering out other nested fields like `delete`.

## Examples

### Basic Usage

Request only `id` and `name` fields:
```
GET /api/users?$select=id,name
```

Response:
```json
{
  "items": [
    {"id": "123", "name": "John"},
    {"id": "456", "name": "Jane"}
  ],
  "page_info": { ... }
}
```

### With Other OData Options

Combine `$select` with `$filter` and `$orderby`:
```
GET /api/users?$filter=email eq 'john@example.com'&$orderby=created_at desc&$select=id,email,created_at
```

### Single Resource

Get specific fields for a single user:
```
GET /api/users/123?$select=id,email,display_name
```

Response:
```json
{
  "id": "123",
  "email": "john@example.com",
  "display_name": "John Doe"
}
```

### Nested Field Selection

Select entire nested object:
```
GET /api/users?$select=id,access_control
```

Response:
```json
{
  "items": [
    {
      "id": "123",
      "access_control": {
        "read": true,
        "write": false,
        "delete": false
      }
    }
  ]
}
```

Select specific nested fields using dot notation:
```
GET /api/users?$select=id,access_control.read,access_control.write
```

Response:
```json
{
  "items": [
    {
      "id": "123",
      "access_control": {
        "read": true,
        "write": false
      }
    }
  ]
}
```

### Deeply Nested Selection

Select specific fields from deeply nested objects:
```
GET /api/users?$select=id,user.profile.name,user.profile.email
```

Response:
```json
{
  "items": [
    {
      "id": "123",
      "user": {
        "profile": {
          "name": "John Doe",
          "email": "john@example.com"
        }
      }
    }
  ]
}
```

## Implementation

### Quick Start: Using Generalized Helpers

The simplest approach is to use the provided helper functions that automatically handle `$select` projection:

#### For Single Resources

```rust
use modkit::api::odata::OData;
use modkit::api::select::apply_select;
use modkit::api::prelude::*;

pub async fn get_user(
    OData(query): OData,
    // ... other extractors
) -> Result<JsonBody<serde_json::Value>> {
    let user = fetch_user().await?;
    let projected = apply_select(&user, query.selected_fields());
    Ok(Json(projected))
}
```

#### For Paginated Responses (Recommended)

```rust
use modkit::api::odata::OData;
use modkit::api::select::page_to_projected_json;
use modkit::api::prelude::*;

pub async fn list_users(
    OData(query): OData,
    // ... other extractors
) -> Result<JsonPage<serde_json::Value>> {
    let page = fetch_page().await?;
    let projected_page = page_to_projected_json(&page, query.selected_fields())?;
    Ok(Json(projected_page))
}
```

The `page_to_projected_json` function automatically:
- Serializes each item in the page
- Applies `$select` projection to each item
- Preserves page metadata (`page_info`)
- Returns a `Page<Value>` ready for JSON serialization

### Advanced: Manual Projection

For custom projection logic, use `project_json` directly:

```rust
use modkit::api::select::project_json;
use std::collections::HashSet;

let fields_set: HashSet<String> = query
    .selected_fields()
    .map(|fields| fields.iter().map(|f| f.to_lowercase()).collect())
    .unwrap_or_default();

let projected = project_json(&value, &fields_set);
```

## API Reference

### ODataQuery Methods

```rust
// Check if field selection is present
pub fn has_select(&self) -> bool

// Get selected fields as a slice
pub fn selected_fields(&self) -> Option<&[String]>

// Set selected fields
pub fn with_select(mut self, fields: Vec<String>) -> Self
```

### Field Projection Utilities

#### `project_json`

Projects a JSON value to only include selected fields.

```rust
pub fn project_json(value: &Value, selected_fields: &HashSet<String>) -> Value
```

- **Arguments:**
  - `value`: The JSON value to project
  - `selected_fields`: Set of field names to include (case-insensitive)

- **Returns:** A new JSON value containing only the selected fields

- **Behavior:**
  - For objects: Includes only specified fields
  - For arrays: Projects each element
  - For other types: Returns unchanged

#### `apply_select`

Helper function to apply field projection to a serializable value.

```rust
pub fn apply_select<T: serde::Serialize>(
    value: &T,
    selected_fields: Option<&[String]>,
) -> Value
```

- **Arguments:**
  - `value`: The value to project
  - `selected_fields`: Optional slice of field names to include

- **Returns:** The projected JSON value, or the original value if no fields are selected

#### `SelectProjectable` Trait

Implement on your DTO types for custom projection logic:

```rust
pub trait SelectProjectable: serde::Serialize {
    fn project_select(&self, selected_fields: &[String]) -> Value;
}
```

## Validation & Constraints

The `$select` parameter is validated with the following constraints:

| Constraint | Value | Error |
|-----------|-------|-------|
| Maximum length | 2048 characters | `$select too long` |
| Maximum fields | 100 fields | `$select contains too many fields` |
| Empty check | Must contain at least one field | `$select must contain at least one field` |
| Duplicates | Field names must be unique | `duplicate field in $select: {field}` |

## Error Handling

Invalid `$select` parameters return HTTP 400 Bad Request with RFC 9457 Problem Details:

```json
{
  "type": "https://example.com/problems/bad-request",
  "title": "Bad Request",
  "status": 400,
  "detail": "$select too long"
}
```

## Performance Considerations

1. **Bandwidth Reduction**: Only requested fields are included in responses
2. **Serialization Overhead**: Field projection adds minimal JSON serialization overhead
3. **Database Queries**: The database still fetches all fields; projection happens at the application layer
4. **Caching**: Consider caching common field selections

## Best Practices

1. **Always Check for Selection**: Use `query.selected_fields()` to determine if projection is needed
2. **Preserve Metadata**: Keep pagination metadata (`page_info`) even when projecting list items
3. **Case Handling**: Field names are case-insensitive; normalize to lowercase internally
4. **Error Messages**: Provide clear error messages for invalid field selections
5. **Documentation**: Document which fields are available for selection in your OpenAPI specs

## Dot Notation Behavior

When using dot notation for nested field selection:

1. **Entire Parent Selection**: If you select `access_control` without dot notation, the entire nested object is included with all its fields.

2. **Specific Nested Fields**: If you select `access_control.read` and `access_control.write`, only those specific fields are included in the nested object.

3. **Deep Nesting**: Dot notation works at any depth: `user.profile.name`, `user.profile.settings.notifications`, etc.

4. **Case Insensitivity**: Both parent and nested field names are case-insensitive: `AccessControl.Read` is equivalent to `access_control.read`.

5. **Array Projection**: When projecting arrays, the dot notation is applied to each element in the array.

6. **Mixed Selection**: You can mix top-level and nested selections: `$select=id,access_control,profile.bio` will include the entire `access_control` object and only the `bio` field from `profile`.

## Limitations

- Field projection happens at the application layer, not the database layer
- Nested object projection includes the entire nested object if the parent field is selected
- Computed or derived fields cannot be selectively excluded
- The `$select` parameter does not affect database query performance
- Dot notation requires exact field path matching (e.g., `access_control.read` won't match `access_control.permissions.read`)

## Future Enhancements

Potential improvements for future versions:

- Database-level field selection (SELECT specific columns)
- Nested field projection (e.g., `$select=user/name,user/email`)
- Field aliasing support
- Automatic OpenAPI documentation of selectable fields
