# Feature: GTS Core
 
 **Status**: ✅ IMPLEMENTED  
 **Feature Slug**: `gts-core`
 
 ---
 
 ## A. Feature Context
 
 ### 1. Overview
 
 Thin routing layer for GTS unified API - delegates to domain-specific features. Provides no database layer or domain-specific logic, purely routing and common middleware.
 
 **Scope**:
 - GTS API routing (`/gts`, `/gts/{id}`) - routes to domain features
 - Common middleware (auth, tenant context injection)
 - Request validation (structure only, not domain logic)
 - OData metadata endpoint (`/$metadata`)
 
 **Out of Scope**:
 - Database layer - delegated to domain features
 - Domain-specific logic - delegated to domain features
 - Business logic - purely routing
 
 ### 2. Purpose
 
 Provide a unified API gateway for all GTS operations with routing to domain features.
 
 This feature is **routing-only**: it does not implement downstream domain behaviors.
 
 ### 3. Actors
 
 - `fdd-analytics-actor-platform-admin`
 - `fdd-analytics-actor-api-consumer`
 - `fdd-analytics-actor-system-integrator`
 
 ### 4. References
 
 1. Overall Design: [DESIGN.md](../../DESIGN.md)
 2. Features manifest entry: [FEATURES.md](../FEATURES.md)
 
 ### GTS Types
 
 This feature **owns no GTS types** - it is a pure routing layer.

**Routes to domain handlers**:
 1. `gts://gts.hypernetix.hyperspot.ax.schema.v1~*` → schema-handler
 2. `gts://gts.hypernetix.hyperspot.ax.query.v1~*` → query-handler
 3. `gts://gts.hypernetix.hyperspot.ax.template.v1~*` → widget-template-handler, values-selector-handler
 4. `gts://gts.hypernetix.hyperspot.ax.datasource.v1~*` → datasource-handler
 5. `gts://gts.hypernetix.hyperspot.ax.item.v1~*` → widget-item-handler, group-item-handler
 6. `gts://gts.hypernetix.hyperspot.ax.layout.v1~*` → dashboard-layout-handler, report-layout-handler
 7. `gts://gts.hypernetix.hyperspot.ax.category.v1~*` → category-handler

 ## B. Actor Flows
 
 ### Flow 1: Platform Administrator registers GTS type (routing-only)
 
 - [x] **ID**: fdd-analytics-feature-gts-core-flow-admin-register-type
 
 1. [x] - `ph-1` - Platform administrator sends POST request to `/api/analytics/v1/gts` with JWT token - `inst-send-post-register-type`
 2. [x] - `ph-1` - Request body contains JSON Schema entity with `$schema` and `$id` - `inst-validate-schema-shape`
 3. [x] - `ph-1` - **IF** entity contains `$id`: - `inst-if-has-id`
    1. [x] - `ph-1` - Extract base type from `$id` (prefix before instance part) - `inst-extract-base-type-from-id`
 4. [x] - `ph-1` - **ELSE**: - `inst-else-missing-id`
    1. [x] - `ph-1` - **RETURN** HTTP 400 (type registration requires `$id`) - `inst-return-400-missing-id`
 5. [x] - `ph-1` - Route base type via routing table - `inst-route-base-type`
 6. [x] - `ph-1` - **IF** no match found: - `inst-if-no-match`
    1. [x] - `ph-1` - **RETURN** HTTP 404 (unknown type) - `inst-return-404-unknown-type`
 7. [x] - `ph-1` - **ELSE IF** delegate handler missing: - `inst-else-if-delegate-missing`
    1. [x] - `ph-1` - **RETURN** HTTP 501 (known type, no delegate) - `inst-return-501-no-delegate`
 8. [x] - `ph-1` - **RETURN** delegate routing decision - `inst-return-routing-decision`
 
 ---
 
 ### Flow 2: API Consumer registers instance (routing-only)
 
 - [x] **ID**: fdd-analytics-feature-gts-core-flow-developer-register-instance
 
 1. [x] - `ph-1` - API consumer sends POST request to `/api/analytics/v1/gts` with JWT token - `inst-send-post-register-instance`
 2. [x] - `ph-1` - Request body contains `id` and instance `entity` - `inst-validate-instance-shape`
 3. [x] - `ph-1` - Extract base type from `id` (prefix before last `~`) - `inst-extract-base-type-from-instance-id`
 4. [x] - `ph-1` - Route base type via routing table - `inst-route-base-type`
 5. [x] - `ph-1` - **IF** no match found: - `inst-if-no-match`
    1. [x] - `ph-1` - **RETURN** HTTP 404 (unknown type) - `inst-return-404-unknown-type`
 6. [x] - `ph-1` - **ELSE IF** delegate handler missing: - `inst-else-if-delegate-missing`
    1. [x] - `ph-1` - **RETURN** HTTP 501 (known type, no delegate) - `inst-return-501-no-delegate`
 7. [x] - `ph-1` - **RETURN** delegate routing decision - `inst-return-routing-decision`
 
 ---
 
 ### Flow 3: API Consumer lists entities with OData (routing-only)
 
 - [x] **ID**: fdd-analytics-feature-gts-core-flow-developer-list-entities
 
 1. [x] - `ph-1` - API consumer sends GET request to `/api/analytics/v1/gts` with OData parameters - `inst-send-get-list`
 2. [x] - `ph-1` - Extract base type filter prefix from `$filter` (when present) - `inst-extract-base-type-filter`
 3. [x] - `ph-1` - Route base type via routing table - `inst-route-base-type`
 4. [x] - `ph-1` - **IF** no match found: - `inst-if-no-match`
    1. [x] - `ph-1` - **RETURN** HTTP 404 (unknown type) - `inst-return-404-unknown-type`
 5. [x] - `ph-1` - **ELSE IF** delegate handler missing: - `inst-else-if-delegate-missing`
    1. [x] - `ph-1` - **RETURN** HTTP 501 (known type, no delegate) - `inst-return-501-no-delegate`
 6. [x] - `ph-1` - **RETURN** delegate routing decision - `inst-return-routing-decision`
 
 ---
 
 ### Flow 4: GTS Core routes CRUD operations (routing-only)
 
 - [x] **ID**: fdd-analytics-feature-gts-core-flow-route-crud-operations
 
 1. [x] - `ph-1` - Receive HTTP request for GTS endpoint - `inst-receive-request`
 2. [x] - `ph-1` - **IF** JWT is invalid: - `inst-if-jwt-invalid`
    1. [x] - `ph-1` - **RETURN** HTTP 401 Unauthorized - `inst-return-401`
 3. [x] - `ph-1` - Extract base type according to HTTP method - `inst-extract-base-type`
 4. [x] - `ph-1` - **MATCH** method: - `inst-match-method`
    - [x] - `ph-1` - **CASE** POST: Extract from body (`$id` or `id`) - `inst-case-post-extract`
    - [x] - `ph-1` - **CASE** GET: Extract from path `{id}` - `inst-case-get-extract`
    - [x] - `ph-1` - **CASE** PUT: Extract from path `{id}` - `inst-case-put-extract`
    - [x] - `ph-1` - **CASE** PATCH: Extract from path `{id}` - `inst-case-patch-extract`
    - [x] - `ph-1` - **CASE** DELETE: Extract from path `{id}` - `inst-case-delete-extract`
 5. [x] - `ph-1` - Route base type via routing table - `inst-route-base-type`
 6. [x] - `ph-1` - **IF** no match found: - `inst-if-no-match`
    1. [x] - `ph-1` - **RETURN** HTTP 404 (unknown type) - `inst-return-404-unknown-type`
 7. [x] - `ph-1` - **ELSE IF** delegate handler missing: - `inst-else-if-delegate-missing`
    1. [x] - `ph-1` - **RETURN** HTTP 501 (known type, no delegate) - `inst-return-501-no-delegate`
 8. [x] - `ph-1` - **RETURN** delegate routing decision - `inst-return-routing-decision`
 
 ---
 
 ### Flow 5: Aggregate OData metadata (routing-only)
 
 - [x] **ID**: fdd-analytics-feature-gts-core-flow-aggregate-odata-metadata
 
 1. [x] - `ph-1` - Receive GET request to `/api/analytics/v1/$metadata` - `inst-receive-metadata-request`
 2. [x] - `ph-1` - **IF** metadata provider delegate is missing: - `inst-if-metadata-provider-missing`
    1. [x] - `ph-1` - **RETURN** HTTP 501 (not implemented) - `inst-return-501-metadata`
 3. [x] - `ph-1` - **RETURN** metadata delegate routing decision - `inst-return-metadata-routing-decision`
 
 ---
 
 ## C. Algorithms
 
 ### Algorithm 1: Routing decision
 
 - [x] **ID**: fdd-analytics-feature-gts-core-algo-routing-logic
 
 **Input**: HTTP request
 **Output**: routing decision (404/501 or delegated handler)
 
 1. [x] - `ph-1` - Determine extraction strategy based on HTTP method - `inst-determine-extraction-strategy`
 2. [x] - `ph-1` - Extract base type from request - `inst-extract-base-type`
 3. [x] - `ph-1` - Route base type via routing table - `inst-route-base-type`
 4. [x] - `ph-1` - **IF** no match found: - `inst-if-no-match`
    1. [x] - `ph-1` - **RETURN** HTTP 404 - `inst-return-404`
 5. [x] - `ph-1` - **ELSE IF** delegate handler missing: - `inst-else-if-delegate-missing`
    1. [x] - `ph-1` - **RETURN** HTTP 501 - `inst-return-501`
 6. [x] - `ph-1` - **RETURN** delegated handler - `inst-return-delegated-handler`
 
 ---
 
 ## D. States

*(Not applicable - GTS Core is stateless router)*

---

## E. Technical Details

### API Endpoints

**Note**: All endpoints delegate to domain features. GTS Core only routes.

### Register GTS Type or Instance

```
POST /api/analytics/v1/gts
```

**Request Fields:**
- **`id`** (optional for types, required for instances) - GTS identifier
- **`entity`** (required) - JSON Schema (for types) or instance data (for instances)

**Response Fields:**
- **`id`** (read-only) - GTS identifier of registered entity
- **`type`** (read-only) - GTS type identifier, automatically derived:
  - For type registration: equals `id` (extracted from `$id`)
  - For instance registration: extracted from `id` (left part before last `~`)
- **`entity`** - The registered entity content
- **Metadata:** `registered_at`, `updated_at`, `deleted_at`, `tenant`, `registered_by`, `updated_by`, `deleted_by`

**Registration Logic:**

1. **Type Registration** (no `id` in request):
   - `entity` MUST contain a valid JSON Schema with `$schema` and `$id` fields
   - `$id` contains the GTS type identifier (ends with `~`)
   - Response `id` and `type` are extracted from `$id`

2. **Instance Registration** (`id` in request):
   - `id` determines instance identifier
   - Supports named identifiers: `gts.vendor.pkg.ns.type.v1~vendor.pkg.ns.instance.v1`
   - Supports anonymous identifiers with UUID: `gts.vendor.pkg.ns.type.v1~550e8400-e29b-41d4-a716-446655440000`
   - Response `type` is derived from `id` (left part before last `~`)
   - `entity` content must conform to the derived type schema
   - Can contain any valid instance data (not a schema)

**Validation Rules:**
- If `id` is NOT provided and `entity` lacks `$schema`: **ERROR** (expected type registration)
- If `id` is provided: instance registration regardless of `entity` content
- `entity` must always conform to the type schema

**Example 1: Register Type**
- Type ID: `gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.custom_query.v1~`
- Provides JSON Schema with `$schema` and `$id` fields in `entity`

**Example 2: Register Named Instance**
- Instance ID: `gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.sales.v1`
- Type (derived): `gts.hypernetix.hyperspot.ax.query.v1~`
- Key fields: `category`, `returns_schema_id`, `capabilities_id`

**Example 3: Register Anonymous Instance (with UUID)**
- Instance ID: `gts.hypernetix.hyperspot.ax.datasource.v1~550e8400-e29b-41d4-a716-446655440000`
- Type (derived): `gts.hypernetix.hyperspot.ax.datasource.v1~`
- References: `query_id` points to query instance

---

### List and Search GTS Entities

```
GET /api/analytics/v1/gts
```

Powerful search and listing endpoint with filtering, full-text search, and property-based queries.

**OData Query Parameters:**

- **`$filter`** - Filter expression using OData syntax
  - GTS identifier filters: `startswith(id, 'gts.hypernetix.hyperspot.ax.query.v1~')`
  - GTS segment filters: `gts_vendor eq 'acme'`, `gts_package eq 'analytics'`, `gts_namespace eq '_'`, `gts_type eq 'query'`
  - Entity property filters: `entity/api_endpoint eq 'https://api.acme.com/analytics/sales'`
  - Metadata filters: `tenant eq '550e8400-e29b-41d4-a716-446655440000'`
  - Date range filters: `registered_at ge 2024-01-01T00:00:00Z and registered_at le 2024-01-31T23:59:59Z`
  - Full-text search: `contains(entity/name, 'monitoring')` or `search.ismatch('monitoring metrics')`
  - Logical operators: `and`, `or`, `not`
  - Comparison operators: `eq`, `ne`, `gt`, `ge`, `lt`, `le`
  - String functions: `contains`, `startswith`, `endswith`
- **`$select`** - Field projection (comma-separated)
  - Example: `$select=id,type,entity/name,registered_at`
  - Supports dot notation for nested fields: `entity/api_endpoint`, `entity/params/filters`
- **`$orderby`** - Sort expression
  - Example: `$orderby=registered_at desc`, `$orderby=entity/name asc,registered_at desc`
- **`$top`** - Page size (default: 50, max: 200)
- **`$skiptoken`** - Pagination cursor from previous response (opaque string)
- **`$count`** - Include total count (`true`/`false`, default: `false`)

**Custom Parameters:**
- **`allow_deleted`** - Include soft-deleted entities (`true`/`false`, default: `false`)

**Notes:**
- GTS segment filters (`gts_vendor`, `gts_package`, `gts_namespace`, `gts_type`) apply to the rightmost chained segment
- Entity properties accessed via `/` notation: `entity/name`, `entity/api_endpoint`
- Full OData v4 filter syntax supported

**Query Optimization:**

The service validates filter expressions against available indexes and supported operations for each base type. If a query cannot be executed efficiently (e.g., missing required indexes, unsupported filter combinations), the service returns an error instead of performing a full table scan.

- Service maintains knowledge of supported queries for each base type and its extensions
- Each base type has predefined indexed fields and supported filter operations
- Unsupported or inefficient queries return HTTP 400 Bad Request with details:

```json
{
  "type": "https://example.com/problems/unsupported-query",
  "title": "Unsupported Query Operation",
  "status": 400,
  "detail": "Filter on 'entity/custom_field' is not supported for type 'gts.hypernetix.hyperspot.ax.query.v1~'. Available indexed fields: [id, type, tenant, registered_at, entity/api_endpoint, entity/name]"
}
```

This ensures consistent query performance and prevents resource exhaustion from inefficient operations.

**Response Format:** `@odata.context`, `@odata.count`, `@odata.nextLink`, `items[]`

**Pagination Flow:**

```http
# First page
GET /api/analytics/v1/gts?$filter=startswith(id, 'gts.hypernetix.hyperspot.ax.query.v1~')&$top=50&$count=true

# Next page (use @odata.nextLink from response or $skiptoken)
GET /api/analytics/v1/gts?$filter=startswith(id, 'gts.hypernetix.hyperspot.ax.query.v1~')&$top=50&$skiptoken=eyJpZCI6Imd0cy5oeXBlcm5ldGl4LmhR...
```

**OData Response Fields:**
- `@odata.context` - Metadata context URL
- `@odata.count` - Total count (when `$count=true`)
- `@odata.nextLink` - URL for next page (`null` when no more results)
- `$skiptoken` is opaque - do not parse or modify

---

### OData Metadata

```
GET /api/analytics/v1/$metadata
Accept: application/json
Returns: OData JSON CSDL with Capabilities vocabulary annotations
```

Service exposes full OData metadata in JSON CSDL format (OData v4.01) with capability annotations (FilterRestrictions, SortRestrictions, SearchRestrictions, SelectSupport, TopSupported, SkipSupported).

Spec: [OData JSON CSDL v4.01](https://docs.oasis-open.org/odata/odata-csdl-json/v4.01/odata-csdl-json-v4.01.html) | [Capabilities Vocabulary](https://github.com/oasis-tcs/odata-vocabularies/blob/master/vocabularies/Org.OData.Capabilities.V1.md)

---

### Get GTS Item (routing-only)

```
GET /api/analytics/v1/gts/{gts-identifier}
Returns: Routing decision; if handler missing → 501; if unknown type → 404; delegate response is out of scope.
```

---

### Update GTS Item (Full Replacement, routing-only)

```
PUT /api/analytics/v1/gts/{gts-identifier}
Body: { "entity": { ... } }  # Full entity replacement
```

**Note:** No persistence or validation in this feature; delegate required. If no handler → 501.

---

### Partially Update GTS Item (routing-only)

```
PATCH /api/analytics/v1/gts/{gts-identifier}
Content-Type: application/json-patch+json
Body: JSON Patch operations (RFC 6902) on /entity/* paths
```

No JSON Patch processing in this feature; if handler missing → 501; unknown type → 404.

**Error: Attempting to Update Read-Only Entity**

```http
PUT /api/analytics/v1/gts/gts.hypernetix.hyperspot.ax.query.v1~
Authorization: Bearer {token}
```

Response:
```http
403 Forbidden
Content-Type: application/problem+json

{
  "type": "https://example.com/problems/read-only-entity",
  "title": "Read-Only Entity",
  "status": 403,
  "detail": "Entity 'gts.hypernetix.hyperspot.ax.query.v1~' is read-only. It was provisioned through configuration files and cannot be modified via the API."
}
```

---

### Delete GTS Item (routing-only)

```
DELETE /api/analytics/v1/gts/{gts-identifier}
Soft-delete (sets deleted_at timestamp)
Returns: 204 No Content only if downstream handler exists and succeeds (delegate). If handler missing → 501.
```

---

## F. Requirements
 
 ### Routing layer routes GTS operations
 
 - [x] **ID**: fdd-analytics-feature-gts-core-req-routing
 **Status**: ✅ IMPLEMENTED
 **Description**: The system SHALL implement a routing-only layer for unified GTS endpoints and delegate handling to downstream domain features. Unknown type patterns MUST return HTTP 404. Known type patterns with missing delegate MUST return HTTP 501.
 **References**:
 - [API Endpoints](#api-endpoints)
 **Implements**:
 - `fdd-analytics-feature-gts-core-flow-route-crud-operations`
 - `fdd-analytics-feature-gts-core-algo-routing-logic`
 - Routing-only REST handlers in `modules/analytics/analytics/src/api/rest/gts_core/`
 **Phases**:
 - [x] `ph-1`: Route requests by base type and return 404/501 when routing cannot be resolved
 **Testing Scenarios (FDL)**:
 - [x] **ID**: fdd-analytics-feature-gts-core-test-routing-table-lookup
   1. [x] - `ph-1` - Provide a known base type identifier and route it - `inst-route-known-type`
   2. [x] - `ph-1` - Verify a routing target is selected - `inst-verify-target-selected`
   3. [x] - `ph-1` - Provide an unknown base type identifier and route it - `inst-route-unknown-type`
   4. [x] - `ph-1` - Verify response status is HTTP 404 - `inst-verify-404`
   5. [x] - `ph-1` - Provide a known base type with missing delegate and route it - `inst-route-missing-delegate`
   6. [x] - `ph-1` - Verify response status is HTTP 501 - `inst-verify-501`
 **Acceptance Criteria**:
 - Unknown base type patterns return HTTP 404
 - Known base type patterns without a registered delegate return HTTP 501
 
 ---
 
 ### Platform middleware integration is used
 
 - [x] **ID**: fdd-analytics-feature-gts-core-req-middleware
 **Status**: ✅ IMPLEMENTED
 **Description**: The system SHALL integrate GTS endpoints via ModKit REST integration patterns (RestfulModule + OperationBuilder) and rely on platform middleware for authentication and SecurityCtx injection.
 **References**:
 - [API Endpoints](#api-endpoints)
 **Implements**:
 - `fdd-analytics-feature-gts-core-flow-route-crud-operations`
 - REST module registration in `modules/analytics/analytics/src/module.rs`
 **Phases**:
 - [x] `ph-1`: Register routes via OperationBuilder and extend the passed router
 **Testing Scenarios (FDL)**:
 - [x] **ID**: fdd-analytics-feature-gts-core-test-operations-registered
   1. [x] - `ph-1` - Register routes through RestfulModule integration - `inst-register-restful-module`
   2. [x] - `ph-1` - Verify router is extended (not replaced) - `inst-verify-router-extended`
   3. [x] - `ph-1` - Verify endpoints are present in OpenAPI registry - `inst-verify-openapi-registration`
 **Acceptance Criteria**:
 - All GTS endpoints are registered via OperationBuilder
 - Router is extended and returned unchanged except for added routes
 
 ---
 
 ### Tolerant reader semantics are enforced at API boundary
 
 - [x] **ID**: fdd-analytics-feature-gts-core-req-tolerant-reader
 **Status**: ✅ IMPLEMENTED
 **Description**: The system SHALL implement tolerant handling of server-managed fields and prevent clients from overriding read-only metadata fields.
 **References**:
 - [API Endpoints](#api-endpoints)
 **Implements**:
 - `fdd-analytics-feature-gts-core-flow-admin-register-type`
 - `fdd-analytics-feature-gts-core-flow-developer-register-instance`
 - Request validation and response shaping in `modules/analytics/analytics/src/api/rest/gts_core/`
 **Phases**:
 - [x] `ph-1`: Reject or ignore client-provided read-only fields consistently
 **Testing Scenarios (FDL)**:
 - [x] **ID**: fdd-analytics-feature-gts-core-test-client-cannot-override-fields
   1. [x] - `ph-1` - Send request including read-only metadata fields - `inst-send-request-with-readonly-fields`
   2. [x] - `ph-1` - Verify read-only fields are ignored or rejected - `inst-verify-readonly-fields-not-applied`
 **Acceptance Criteria**:
 - Client cannot set id, type, tenant, or timestamp metadata fields
 - Read-only fields are consistently ignored or rejected

---

## Dependencies

- **Depends On**: (none - foundational)
- **Blocks**: All domain type features

---

## References

- Overall Design: `architecture/DESIGN.md` Section 2 (Actors), Section 3 (System Capabilities)
- GTS Specification: `gts/README.md` (GTS identifier format)
- OpenAPI Spec: `architecture/openapi/v1/api.yaml` (unified /gts endpoints)
- Feature Manifest: `architecture/features/FEATURES.md` (feature-gts-core entry)
- OData v4.01 Spec: https://www.odata.org/documentation/
- RFC 7807 Problem Details: https://tools.ietf.org/html/rfc7807
- Tolerant Reader Pattern: https://martinfowler.com/bliki/TolerantReader.html
