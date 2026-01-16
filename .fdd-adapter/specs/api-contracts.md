# API Contract Specification

**Source**: docs/api/api.json, libs/modkit/src/api/, README.md

## Technology

**OpenAPI 3.1.0** (auto-generated from Rust code)

## Location

**Generated Spec**: `docs/api/api.json`  
**Source Code**: `modules/{module}/src/api/rest/routes.rs`

## Format

REST APIs are defined using Axum routing with utoipa macros for automatic OpenAPI generation.

## Endpoint Pattern

```
/{module}/v{version}/{resource}
```

**Examples**:
- `/file-parser/v1/info`
- `/file-parser/v1/parse`
- `/nodes-registry/v1/nodes`

## API Convention

**Versioning**: `/v{major}` in URL path  
**Module Prefix**: Always prefixed with module name  
**Methods**: GET, POST, PUT, PATCH, DELETE  
**Content-Type**: `application/json`  
**Error Format**: RFC 7807 Problem Details (`application/problem+json`)

## Documentation

**Auto-generated**: OpenAPI spec generated from Rust code annotations  
**Access**: `http://127.0.0.1:8087/docs` (when server running)  
**Format**: Swagger UI + ReDoc

## Example Route Definition

```rust
use axum::{routing::get, Router};
use utoipa::OpenApi;

#[utoipa::path(
    get,
    path = "/file-parser/v1/info",
    tag = "File Parser",
    responses(
        (status = 200, description = "Parser information", body = FileParserInfoDto),
        (status = 400, description = "Bad Request", body = Problem),
    )
)]
async fn get_parser_info() -> Json<FileParserInfoDto> {
    // Implementation
}

pub fn routes() -> Router {
    Router::new()
        .route("/file-parser/v1/info", get(get_parser_info))
}
```

## Validation

**Generate Spec**: Run server, OpenAPI auto-generated at startup  
**Check Spec**: `curl http://127.0.0.1:8087/api-docs/openapi.json`  
**Expected**: Valid OpenAPI 3.1.0 JSON

## Traceability

Feature DESIGN.md files must reference API endpoints using the pattern:
- Format: `@API.GET:/{module}/v{version}/{resource}`
- Example: `@API.GET:/file-parser/v1/info`

---

## Validation Checklist

### MUST Requirements (Mandatory)

- [ ] **Technology specified** (OpenAPI 3.1.0)
- [ ] **Auto-generation configured** (utoipa macros)
- [ ] **Location documented** (docs/api/, source code)
- [ ] **Endpoint pattern defined** (/{module}/v{version}/{resource})
- [ ] **Versioning strategy documented** (/v{major})
- [ ] **Module prefix enforced**
- [ ] **Error format specified** (RFC 7807 Problem Details)
- [ ] **Content-Type declared** (application/json)
- [ ] **Traceability format defined** for DESIGN.md

### SHOULD Requirements (Strongly Recommended)

- [ ] Documentation endpoint accessible (/docs)
- [ ] All handlers have utoipa::path annotations
- [ ] Response types documented with examples
- [ ] Error responses included in specs
- [ ] Tags used for logical grouping

### MAY Requirements (Optional)

- [ ] Additional API documentation formats
- [ ] Custom OpenAPI extensions
- [ ] SDK generation configured

## Compliance Criteria

**Pass**: All MUST requirements met (9/9) + OpenAPI validates  
**Fail**: Any MUST requirement missing or invalid OpenAPI

### Agent Instructions

When implementing API endpoints:
1. ✅ **ALWAYS include version in path** (/v1, /v2, etc.)
2. ✅ **ALWAYS prefix with module name** (/{module}/v{version}/...)
3. ✅ **ALWAYS annotate with utoipa::path**
4. ✅ **ALWAYS document responses** (200, 4xx, 5xx)
5. ✅ **ALWAYS use Problem Details for errors**
6. ✅ **ALWAYS use application/json** content type
7. ✅ **ALWAYS follow REST conventions** (GET, POST, PUT, PATCH, DELETE)
8. ✅ **ALWAYS validate OpenAPI generation** (check /api-docs/openapi.json)
9. ❌ **NEVER create unversioned endpoints**
10. ❌ **NEVER skip utoipa annotations**
11. ❌ **NEVER use custom error formats** (always RFC 7807)
12. ❌ **NEVER hardcode module prefix** (use from config)

### API Implementation Checklist

Before deploying API:
- [ ] All endpoints versioned
- [ ] Module prefix present
- [ ] utoipa::path annotations complete
- [ ] Response types have ToSchema
- [ ] Error handling uses Problem Details
- [ ] OpenAPI spec validates
- [ ] Documentation accessible at /docs
- [ ] Referenced in DESIGN.md with @API notation
