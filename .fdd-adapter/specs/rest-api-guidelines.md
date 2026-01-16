# REST API Guidelines

**Source**: guidelines/DNA/REST/API.md, guidelines/DNA/REST/VERSIONING.md, guidelines/DNA/REST/QUERYING.md

## Core Principles

- **Consistency over novelty**: One clear way to do things
- **Explicitness**: Always specify types, units, timezones, defaults
- **Evolvability**: Versioned paths, idempotency, forward-compatible schemas
- **Observability**: Every request traceable end-to-end
- **Security first**: HTTPS only, least privilege, safe defaults

## Protocol & Content

- **Media type**: `application/json; charset=utf-8`
- **Errors**: `application/problem+json` (RFC 9457)
- **Encoding**: UTF-8
- **Compression**: gzip/br when `Accept-Encoding` present

## URL Structure

**Pattern**: `/{module}/v{version}/{resource}`

**Examples**:
- `/file-parser/v1/info`
- `/users/v1/users/{id}`
- `/nodes-registry/v1/nodes`

**Rules**:
- Nouns, plural: `/users`, `/tickets`
- Prefer top-level + filters over deep nesting
- Identifiers: UUIDv7 (or ULID)
- Standard fields: `id`, `created_at`, `updated_at`, optional `deleted_at`

## JSON Conventions

- **Naming**: `snake_case` (consistent with backend/DB)
- **Nullability**: Prefer omitting absent fields over `null`
- **Booleans**: Strongly typed, never stringly booleans
- **Money**: Integer minor units + currency code
- **Lists**: Use `items` array with optional `page_info`
- **Single objects**: Return fields directly (no wrapper)

```json
// List response
{
  "items": [ /* objects */ ],
  "page_info": {
    "limit": 25,
    "next_cursor": "...",
    "prev_cursor": null
  }
}

// Single object (no wrapper)
{
  "id": "01J...",
  "title": "Example",
  "created_at": "2025-09-01T20:00:00.000Z"
}
```

## Timestamps

- **Format**: ISO-8601 UTC with `Z`, **always include milliseconds** `.SSS`
- **Example**: `2025-09-01T20:00:00.000Z`
- **Rust**: Use `time::OffsetDateTime` with `#[serde(with = "time::serde::rfc3339")]`

## Pagination (Cursor-Based)

**Query Parameters**:
- `limit` (integer): Number of items (default 25, max 200)
- `cursor` (string, optional): Opaque token from previous response

**Canonical Sort**:
- Must be total and stable: `created_at DESC, id DESC`
- Tiebreaker must be unique (UUIDv7 recommended)

**Response Format**:
```json
{
  "items": [...],
  "page_info": {
    "limit": 25,
    "next_cursor": "eyJ2IjoxLCJrIjpbIjIwMjUiXX0",
    "prev_cursor": null
  }
}
```

## Filtering & Sorting (OData Subset)

**$filter** (OData-style):
- Operators: `eq`, `ne`, `gt`, `ge`, `lt`, `le`, `and`, `or`, `not`, `in`
- Functions: `startswith`, `endswith`, `contains`
- Example: `status in ('open','in_progress') and priority eq 'high'`

**$orderby** (OData-style):
- Format: `field1 desc, field2 asc`
- Must include unique tiebreaker (typically `id`)
- Example: `priority desc, created_at asc, id asc`

**$select** (Field Projection):
- Sparse field selection
- Example: `$select=id,title,status,priority`

**Requirements**:
- Only indexed fields allowed in filters/sorting
- Document allowlisted fields per endpoint
- Max 10 queryable fields per endpoint recommended

## Request Semantics

- **Create**: `POST /tickets` → 201 + `Location` + resource in body
- **Partial update**: `PATCH /tickets/{id}` (JSON Merge Patch)
- **Replace**: `PUT /tickets/{id}` (complete representation)
- **Delete**: `DELETE /tickets/{id}` → 204 (or 200 with `deleted_at` for soft-delete)

## Error Model (RFC 9457 Problem Details)

**Always return Problem Details for 4xx/5xx**:

```json
{
  "type": "https://api.example.com/errors/validation",
  "title": "Invalid request",
  "status": 422,
  "detail": "email is invalid",
  "instance": "https://api.example.com/req/01J...Z",
  "errors": [
    {
      "field": "email",
      "code": "format",
      "message": "must be a valid email"
    }
  ],
  "trace_id": "01J...Z"
}
```

**Status Code Mappings**:
- 400: Bad Request
- 401: Unauthorized
- 403: Forbidden
- 404: Not Found
- 409: Conflict (duplicate resource)
- 412: Precondition Failed (ETag mismatch)
- 422: Unprocessable Entity (validation errors)
- 429: Too Many Requests
- 503: Service Unavailable

## Concurrency & Idempotency

**ETags (Optimistic Locking)**:
- Representations carry `ETag` header (strong or weak)
- Clients send `If-Match` for updates
- On mismatch → 412 Precondition Failed

**Idempotency**:
- Clients send `Idempotency-Key` header on `POST/PATCH/DELETE`
- Server caches **only successful (2xx) responses**
- Error responses (4xx/5xx) NOT cached (allow retries)
- Replays return cached response + `Idempotency-Replayed: true` header

**Retention Tiers**:
- Minimum: 1 hour (network retry protection)
- Important ops: 24h-7d (must document per endpoint)
- Critical ops: Permanent via DB constraints → 409 Conflict after initial creation

## Rate Limiting

**Headers** (IETF RateLimit Draft):
```http
RateLimit-Policy: "default";q=100;w=3600
RateLimit: "default";r=72;t=1800
```

On 429 also include `Retry-After` (seconds).

## Asynchronous Operations

For long tasks return **202 Accepted** + `Location: /jobs/{job_id}`

**Job Resource**:
```json
{
  "id": "01J...",
  "status": "queued|running|succeeded|failed|canceled",
  "percent": 35,
  "result": {},
  "error": {},
  "created_at": "...",
  "updated_at": "..."
}
```

Clients poll `GET /jobs/{id}` or subscribe via SSE if available.

## Versioning

**Strategy**: Path-based versioning (`/v1`, `/v2`)

**Breaking Changes Require New Version**:
- Removing fields
- Changing field types/semantics
- Making optional fields required
- Changing URL structure

**Non-Breaking Changes**:
- Adding optional fields
- Adding new endpoints
- New enum values (with graceful degradation)
- Relaxing validation

**Deprecation Headers**:
```http
Deprecation: true
Sunset: Sat, 31 Dec 2025 23:59:59 GMT
Link: <https://docs.api.example.com/migration/v1-to-v2>; rel="deprecation"
```

## Security

- **HTTPS only**, HSTS enabled
- **Auth**: OAuth2/OIDC Bearer tokens in `Authorization: Bearer <token>`
- **No secrets in URLs**
- **CORS**: Allow-list explicit origins

## Observability

**Tracing**:
- Accept/propagate `traceparent` (W3C)
- Emit `trace_id` header on all responses

**Structured Logs** (per request):
- `trace_id`, `request_id`, `user_id`, `path`, `status`, `duration_ms`, `bytes`

## OpenAPI Documentation

**Requirements**:
- OpenAPI 3.1 as source of truth
- Auto-generate from Rust code (utoipa)
- Document all endpoints with examples
- Include error responses (Problem Details)
- Specify rate limits, auth requirements
- Document allowed OData fields per endpoint

## Client Compatibility

Clients **MUST**:
- Ignore unknown fields in responses
- Handle new enum values gracefully
- Not rely on field order
- Treat missing optional fields as absent/default

## Performance & DoS Protection

- Enforce max payload sizes (e.g., 1MB JSON)
- Handler timeout ≤ 30s (use async jobs for longer)
- Strict input validation with precise 422s
- Deny N+1 by default

## Reference

Complete guidelines: `guidelines/DNA/REST/API.md`, `guidelines/DNA/REST/VERSIONING.md`, `guidelines/DNA/REST/QUERYING.md`

---

## Validation Checklist

### MUST Requirements (Mandatory)

- [ ] **URL structure follows pattern** (/{module}/v{version}/{resource})
- [ ] **JSON conventions applied** (snake_case, omit null, typed booleans)
- [ ] **Timestamps ISO-8601 UTC** with milliseconds (.SSS)
- [ ] **Cursor pagination used** (opaque cursors, stable sort)
- [ ] **OData subset implemented** ($filter, $orderby, $select)
- [ ] **Problem Details for errors** (RFC 9457 always)
- [ ] **Idempotency-Key supported** for POST/PATCH/DELETE
- [ ] **ETags for concurrency** (If-Match headers)
- [ ] **Versioning path-based** (/v1, /v2)
- [ ] **HTTPS enforced** (HSTS enabled)
- [ ] **Observability headers** (traceparent, trace_id)

### SHOULD Requirements (Strongly Recommended)

- [ ] Rate limiting with IETF headers
- [ ] Async operations (202 + Location)
- [ ] Deprecation headers for old versions
- [ ] Field projection ($select)
- [ ] Compression (gzip/br)
- [ ] CORS allow-list configured
- [ ] Security headers present

### MAY Requirements (Optional)

- [ ] Webhooks for events
- [ ] Batch operations
- [ ] Custom OpenAPI extensions
- [ ] SDK generation

## Compliance Criteria

**Pass**: All MUST requirements met (11/11) + validates against OpenAPI  
**Fail**: Any MUST requirement missing or non-compliant responses

### Agent Instructions

When implementing REST APIs:
1. ✅ **ALWAYS use /{module}/v{version}/** pattern
2. ✅ **ALWAYS use snake_case** for JSON fields
3. ✅ **ALWAYS include milliseconds** in timestamps (.SSS)
4. ✅ **ALWAYS use cursor pagination** (not offset/page)
5. ✅ **ALWAYS implement OData** for filtering/sorting
6. ✅ **ALWAYS use Problem Details** for errors (RFC 9457)
7. ✅ **ALWAYS support Idempotency-Key** for mutations
8. ✅ **ALWAYS use ETags** for concurrency control
9. ✅ **ALWAYS version endpoints** (/v1, /v2, never unversioned)
10. ✅ **ALWAYS use HTTPS** (no HTTP in production)
11. ✅ **ALWAYS emit trace_id** in responses
12. ❌ **NEVER break backward compatibility** within version
13. ❌ **NEVER use custom error format** (always RFC 9457)
14. ❌ **NEVER skip versioning**
15. ❌ **NEVER expose secrets** in URLs or responses

### REST API Implementation Checklist

Before deploying endpoint:
- [ ] URL follows /{module}/v{version}/{resource}
- [ ] JSON uses snake_case
- [ ] Timestamps have milliseconds
- [ ] Pagination uses cursors
- [ ] Filtering/sorting uses OData
- [ ] Errors use Problem Details
- [ ] Idempotency-Key handled
- [ ] ETags implemented
- [ ] Version in path
- [ ] HTTPS only
- [ ] Trace headers propagated
- [ ] OpenAPI docs generated
- [ ] Rate limits documented
- [ ] Auth requirements specified
- [ ] Client compatibility verified
