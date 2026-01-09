# Feature: Query Execution

**Status**: NOT_STARTED  
**Feature Slug**: `feature-query-execution`

---

## A. Feature Context

### Overview

Query execution engine with OData v4 support using registered plugins. Handles query resolution, JWT generation with tenant context, plugin invocation, response validation, and caching.

**Purpose**: Execute queries against external APIs with automatic tenancy context propagation and OData v4 protocol support.

**Scope**:
- OData v4 execution engine (GET/POST)
- Query metadata endpoint (`/queries/{id}/$metadata`)
- Plugin invocation and orchestration
- JWT generation for external APIs with tenant context
- Query result caching layer
- Multi-datasource orchestration
- `/queries/{id}` and `/queries/{id}/$query` endpoints
- Response validation against schemas
- Circuit breaker and timeout handling

**Out of Scope**:
- Query type registration - handled by feature-query-definitions
- Plugin management - handled by feature-plugins
- Datasource configuration - handled by feature-datasources

### GTS Types

This feature **does not own GTS types** - it executes queries defined by feature-query-definitions.

**Uses types from**:
- `gts://gts.hypernetix.hyperspot.ax.query.v1~*` - Query definitions (from feature-query-definitions)
- `gts://gts.hypernetix.hyperspot.ax.schema.v1~query_returns.v1~*` - Response schemas

References from `gts/types/`:
- Query GTS schema files (owned by feature-query-definitions)

### OpenAPI Endpoints

From `architecture/openapi/v1/api.yaml`:
- `GET /api/analytics/v1/queries/{query-id}` - Execute query with OData GET parameters
- `POST /api/analytics/v1/queries/{query-id}` - Execute query with OData JSON body
- `GET /api/analytics/v1/queries/{query-id}/$metadata` - Get query-specific OData metadata

### Actors

**Human Actors** (from Overall Design):
- **End User** - Executes queries through widgets and dashboards
- **Developer** - Tests queries directly via API

**System Actors**:
- **Query Executor** - Orchestrates query execution flow
- **Plugin Manager** - Invokes registered query plugins
- **JWT Generator** - Creates tokens with tenant context
- **Cache Manager** - Stores and retrieves cached results
- **Response Validator** - Validates responses against schemas

**Service Roles** (from OpenAPI):
- `analytics:queries:execute` - Execute queries
- `analytics:queries:metadata` - View query metadata

---

## B. Actor Flows

### Flow 1: End User Executes Query via Widget

**Actor**: End User  
**Trigger**: Widget loads on dashboard  
**Goal**: Display data visualization with query results

**Steps**:
1. **Resolve Query** - Fetch query definition from GTS Registry (`/gts/{query-id}`)
2. **Build OData Request** - Construct OData query parameters from datasource params + widget overrides
3. **Generate JWT** - Create JWT token with tenancy context (tenant_id, org_id, sub)
4. **Execute Request** - Call external API endpoint with OData parameters and JWT
5. **Validate Response** - Verify response against `returns_schema_id` from query definition
6. **Cache Result** - Store response for performance (with tenant isolation)
7. **Return Data** - Send OData response to widget renderer

**API Interaction**:
```
GET /api/analytics/v1/queries/{query-id}?$filter=...&$orderby=...&$top=50
Authorization: Bearer <user-jwt>

→ Query Executor resolves query definition
→ Generates service JWT with tenant context
→ Invokes plugin with JWT and OData params
→ Plugin calls external API
→ Validates response schema
→ Caches result
→ Returns OData v4 response
```

---

### Flow 2: System Executes Query with JWT Tenancy Context

**Actor**: Query Executor (System)  
**Trigger**: Any query execution request  
**Goal**: Ensure tenant isolation via JWT propagation

**Steps**:
1. Extract SecurityCtx from user request
2. Generate JWT with standard claims:
   - `sub` - User identifier
   - `tenant_id` - Current tenant
   - `org_id` - Organization
   - `iat`, `exp` - Timestamps
   - `scopes` - Access scopes
3. Allow plugin to add custom claims (hook: `before_jwt_sign`)
4. Sign JWT with shared secret
5. Include JWT in `Authorization: Bearer` header
6. Pass to plugin for execution

---

### Flow 3: Cache Manager Handles Result Caching

**Actor**: Cache Manager (System)  
**Trigger**: Query execution completes  
**Goal**: Cache results for performance

**Steps**:
1. Build cache key: `query_id` + OData params + `tenant_id`
2. Check cache for existing entry
3. If hit and not expired → return cached data
4. If miss or expired:
   - Execute query via plugin
   - Validate response
   - Store in cache with TTL (default: 5 min)
   - Return data

---

### Flow 4: Developer Executes Query with POST Body

**Actor**: Developer  
**Trigger**: Testing complex query with many parameters  
**Goal**: Execute query using JSON body instead of URL params

**API Interaction**:
```
POST /api/analytics/v1/queries/{query-id}
Authorization: Bearer <jwt>
Content-Type: application/json

Body: {
  "$filter": "status eq 'active' and revenue gt 1000",
  "$orderby": "created_at desc",
  "$top": 50,
  "$skip": 0,
  "$select": "id,name,revenue",
  "$count": true
}

→ Same execution flow as GET
→ OData params extracted from JSON body
```

---

### Flow 5: Widget Renderer Loads Query Metadata

**Actor**: Widget Renderer (System)  
**Trigger**: Need to understand query schema and capabilities  
**Goal**: Display correct UI controls and field names

**API Interaction**:
```
GET /api/analytics/v1/queries/{query-id}/$metadata
Accept: application/json

→ Returns OData JSON CSDL with:
  - Entity type definition from returns_schema_id
  - Capabilities annotations from capabilities_id
  - Supported operations
```

---

## C. Algorithms

### Service Algorithm 1: Query Execution Flow

**Purpose**: Execute query with plugin orchestration and tenant isolation

**Input**: Query ID, OData parameters, SecurityCtx  
**Output**: OData v4 response with data

**Steps**:

1. Resolve query definition from GTS registry
2. Build cache key from query_id, params, tenant_id
3. **TRY** get cached result:
   1. **IF** cache hit:
      1. **RETURN** cached response
4. Generate JWT with tenant context:
   1. Create JWT claims from SecurityCtx
   2. Call plugin.before_jwt_sign (allow custom claims)
   3. Sign JWT token
5. Execute via plugin:
   1. Call plugin.execute(query_id, params, jwt_token)
   2. Wait for response
6. Validate response against returns_schema_id
7. **IF** validation passes:
   1. Store in cache with TTL
   2. **RETURN** OData response
8. **ELSE**:
   1. **RETURN** validation error

---

### Service Algorithm 2: JWT Generation with Tenant Context

**Purpose**: Generate JWT tokens with automatic tenant isolation

**Standard JWT Claims** (always included):
- `sub`: User ID from SecurityCtx
- `tenant_id`: Current tenant (automatic isolation)
- `org_id`: Organization
- `iat`: Issued at timestamp
- `exp`: Expiration (default: 5 minutes)
- `scopes`: Access scopes

**Plugin Hook** (optional):
Plugins can add custom claims or modify expiration, but CANNOT remove standard claims

---

### Service Algorithm 3: Cache Key Generation

**Purpose**: Generate unique cache keys with tenant isolation

**Algorithm**:
1. Concatenate: "query" + query_id + serialized_params + tenant_id
2. Hash the result (SHA256)
3. **RETURN** hash as cache key

**Cache Key Components**:
- `query_id` - Query identifier
- `hash(OData params)` - Normalized hash of filter, orderby, top, skip, select
- `tenant_id` - Tenant isolation

**Example**: `query:sales.v1:a3f5bc2e:tenant-123`

---

## D. States

*(Not applicable - query execution is stateless)*

---

## E. Technical Details

### Query Response Format

OData v4 format:
```json
{
  "@odata.context": "https://api.example.com/analytics/v1/$metadata#Sales",
  "@odata.count": 1234,
  "@odata.nextLink": "https://api.example.com/analytics/v1/queries/{id}?$skiptoken=...",
  "value": [
    {"id": "123", "name": "Product A", "revenue": 5000}
  ]
}
```

### JWT-Based Tenancy Context Propagation

**Standard**: All query executions automatically include tenant context via JWT tokens. This behavior is **always enforced** and cannot be disabled.

### Automatic JWT Generation

When Analytics Service executes any query to external APIs, it **always** generates a JWT token containing:
- `sub` - User identifier from SecurityCtx
- `tenant_id` - Current tenant identifier
- `org_id` - Organization identifier  
- `iat`, `exp` - Timestamps
- `scopes` - Access scopes

The JWT is **always** placed in the `Authorization: Bearer` header for all query executions.

### Plugin Influence

Query plugins **can** influence JWT generation:

1. **Add custom claims** - Plugin can inject additional claims before signing
2. **Modify expiration** - Adjust token lifetime based on query type
3. **Add scopes** - Include plugin-specific scopes

**Plugin Hook**: before_jwt_sign(ctx, claims)
```

**Important**: 
- ✅ Plugins **must** receive and process JWT with tenancy context from Analytics Service
- ✅ Plugins **can** use any auth mechanism for their own requests to external APIs
- ❌ Plugins **cannot** remove standard claims from received JWT (`tenant_id`, `org_id`, `sub`)

**Plugin Flexibility**:
Once the plugin receives SecurityCtx and JWT, it has full control over how it communicates with external systems:
- Can extract tenant_id and use different auth (API Key, OAuth, mTLS)
- Can transform requests to proprietary protocols
- Can implement custom retry/fallback logic
- **Must** respect tenant isolation based on received context

### For External API Providers

Your API **must**:

1. **Validate JWT signature** - Use shared signing key or public key
2. **Extract tenant_id** - Read from JWT claims
3. **Filter data by tenant** - Return only tenant-scoped data
4. **Return 403** if tenant context is invalid or missing

**Validation Steps**:
1. Extract Bearer token from Authorization header
2. Validate JWT signature
3. Decode JWT claims
4. Extract tenant_id from claims
5. Verify tenant context is valid
6. Filter data by tenant_id

### Security Guarantees

- **Cryptographic integrity** - JWT signature prevents tampering
- **Cannot forge tenant_id** - Claims are signed together
- **Automatic expiration** - Tokens are short-lived (default: 5 minutes)
- **Audit trail** - All claims logged with every request

---

### OData Query Options Reference

### Filtering (`$filter`)

**Comparison**: `eq`, `ne`, `gt`, `ge`, `lt`, `le`

**Logical**: `and`, `or`, `not`

**String functions**: `contains()`, `startswith()`, `endswith()`, `length()`, `tolower()`, `toupper()`

**Date functions**: `year()`, `month()`, `day()`, `hour()`, `minute()`, `second()`

**Collections**: `in` (value in list), `any()`, `all()`

**Examples:**
```odata
$filter=status eq 'active'
$filter=revenue gt 1000 and region eq 'EU'
$filter=created_at ge 2024-01-01T00:00:00Z
$filter=contains(name, 'server')
$filter=region in ('EU','US','APAC')
```

### Sorting (`$orderby`)

```odata
$orderby=created_at desc
$orderby=region asc,revenue desc
$orderby=tolower(name) asc
```

### Pagination

- `$top=50` - page size (limit)
- `$skip=100` - offset
- Use `@odata.nextLink` from response for cursor-based pagination

### Field Selection (`$select`)

```odata
$select=id,name,revenue
$select=*
```

### Expand Navigation Properties (`$expand`)

```odata
$expand=customer
$expand=customer($select=id,name)
$expand=customer($filter=type eq 'enterprise')
```

### Full-Text Search (`$search`)

```odata
$search="server error"
$search=cpu OR memory
```

### Count (`$count=true`)

Includes total count in `@odata.count` field.

---

### Query Metadata Endpoint

```
GET /api/analytics/v1/queries/{gts-identifier}/$metadata
Accept: application/json
Returns: OData JSON CSDL with query-specific schema and capabilities
```

Returns query-specific metadata in OData JSON CSDL format including:
- Entity type definition from `returns_schema_id`
- Capabilities annotations from `capabilities_id`
- Supported query operations and restrictions

---

### Integration Scenarios

### 1. Native Contract Implementation (No plugin needed)
- Service already implements Analytics contract
- Direct registration via API call
- Example: Custom analytics service built with Analytics contract from start

### 2. Plugin-based Wrapper (Plugin wraps existing API)
- Service has API but doesn't implement Analytics contract
- Write plugin that implements contract and calls your API
- Plugin embedded in platform, registered as local datasource
- Example: Legacy monitoring system with custom API format

### 3. Adapter-based Integration (Using 3rd-party contract)
- Service implements known 3rd-party contract (e.g., Prometheus, Elasticsearch)
- Use existing adapter plugin or write custom one
- Adapter converts 3rd-party format to native contract
- Optionally combine with plugin for complex auth/logic
- Example: Prometheus exporter with adapter plugin

---

### Access Control

**SecurityCtx Enforcement**:
- All query executions require authenticated user
- Tenant ID extracted from SecurityCtx
- JWT automatically includes tenant context
- Plugin invocations include SecurityCtx

**Permission Checks**:
- Query execution: Requires `analytics:queries:execute`
- Tenant enablement: Verify query enabled for ctx.tenant_id

---

### Database Operations

This feature is **read-only** - no database writes.

**Query Lookups**:
- Query definitions fetched from GTS Registry via `/gts/{query-id}`
- Response schemas fetched for validation
- Capabilities metadata for OData support

---

### Caching Strategy

- **Cache Key**: `query_id` + OData parameters + `tenant_id`
- **TTL**: Configurable per query (default: 5 minutes)
- **Invalidation**: Time-based expiration
- **Tenant Isolation**: Cache keys include tenant_id
- **Storage**: Redis or in-memory cache

**Cache Headers**:
- `Cache-Control: max-age=300`
- `ETag` - Entity tag
- `Last-Modified` - Timestamp

---

### Multi-Datasource Orchestration

When a single widget/dashboard requires data from multiple queries:

1. **Parallel Execution**: Execute queries in parallel when independent
2. **Sequential Execution**: Execute queries in order when dependent
3. **Result Merging**: Combine results based on join/merge strategy
4. **Error Handling**: Partial failure handling with fallback data

**Example Scenario:**
Dashboard with 3 widgets, each with different query:
- Widget A: Sales metrics query
- Widget B: Customer demographics query
- Widget C: Product performance query

All three queries execute in parallel with separate JWT tokens, cached independently, and rendered when all complete.

---

### Performance Optimization

- **Connection Pooling**: Reuse HTTP connections to external APIs
- **Parallel Queries**: Execute independent queries simultaneously
- **Result Streaming**: Stream large result sets
- **Query Timeouts**: Cancel long-running queries (default: 30s)
- **Circuit Breaker**: Protect against failing external services

---

### Error Handling

**Common Errors:**

- **404 Not Found**: Query definition not found in GTS Registry
- **400 Bad Request**: Invalid OData parameters
- **401 Unauthorized**: Missing or invalid JWT token
- **403 Forbidden**: Tenant not enabled for query
- **422 Unprocessable Entity**: Response doesn't match schema
- **504 Gateway Timeout**: External API timeout
- **503 Service Unavailable**: Plugin unavailable or circuit breaker open

**Error Response Format (RFC 7807):**
```json
{
  "type": "https://example.com/problems/query-timeout",
  "title": "Query Execution Timeout",
  "status": 504,
  "detail": "Query 'acme.analytics._.sales.v1' exceeded maximum execution time of 30 seconds",
  "instance": "/api/analytics/v1/queries/gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.sales.v1",
  "trace_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

## F. Validation & Implementation

### Testing Scenarios

**Unit Tests**:
- OData parameter parsing and validation
- JWT generation with standard claims
- Cache key generation with tenant isolation
- Response schema validation
- Error response formatting (RFC 7807)

**Integration Tests**:
- End-to-end query execution flow
- Plugin invocation with JWT
- Cache hit/miss scenarios
- Multi-datasource orchestration
- Timeout and circuit breaker

**Performance Tests**:
- Query execution latency (< 100ms cached, < 1s uncached)
- Cache hit rate (> 80%)
- Concurrent query execution (100+ simultaneous)
- Large result set streaming

**Security Tests**:
- JWT signature validation
- Tenant isolation verification
- Unauthorized query execution prevention
- JWT tampering detection

**Edge Cases**:
1. Query definition not found
2. Invalid OData parameters
3. Response doesn't match schema
4. External API timeout
5. Plugin unavailable
6. Cache service down

---

### OpenSpec Changes Plan

#### Change 001: OData v4 Engine
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/query_execution/odata_engine.rs`
- **Description**: Implement OData v4 parameter parsing (GET/POST)
- **Dependencies**: None (foundational)
- **Effort**: 3 hours (AI agent)
- **Validation**: Unit tests for all OData operators

#### Change 002: JWT Generation Service
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/query_execution/jwt_generator.rs`
- **Description**: Generate JWT with automatic tenant context
- **Dependencies**: Change 001
- **Effort**: 1.5 hours (AI agent)
- **Validation**: JWT validation tests, claim verification

#### Change 003: Plugin Orchestration
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/query_execution/plugin_orchestrator.rs`
- **Description**: Invoke plugins with SecurityCtx and JWT
- **Dependencies**: Change 002
- **Effort**: 2 hours (AI agent)
- **Validation**: Integration tests with mock plugins

#### Change 004: Response Validation
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/query_execution/response_validator.rs`
- **Description**: Validate responses against returns_schema_id
- **Dependencies**: Change 003
- **Effort**: 2 hours (AI agent)
- **Validation**: Schema validation tests

#### Change 005: Cache Layer
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/query_execution/cache_manager.rs`
- **Description**: Result caching with tenant isolation
- **Dependencies**: Change 004
- **Effort**: 2 hours (AI agent)
- **Validation**: Cache hit/miss tests, TTL tests

#### Change 006: Query Metadata Endpoint
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/api/rest/query_execution/metadata_handler.rs`
- **Description**: OData JSON CSDL generation
- **Dependencies**: Change 001
- **Effort**: 1.5 hours (AI agent)
- **Validation**: OData CSDL validator

#### Change 007: Circuit Breaker
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/query_execution/circuit_breaker.rs`
- **Description**: Protect against failing external services
- **Dependencies**: Change 003
- **Effort**: 1 hour (AI agent)
- **Validation**: Failure scenario tests

#### Change 008: Multi-Query Orchestration
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/query_execution/multi_query.rs`
- **Description**: Parallel execution of independent queries
- **Dependencies**: Change 005
- **Effort**: 2 hours (AI agent)
- **Validation**: Concurrent execution tests

#### Change 009: OpenAPI Specification
- **Type**: openapi
- **Files**: 
  - `architecture/openapi/v1/api.yaml`
- **Description**: Document query execution endpoints
- **Dependencies**: All previous changes
- **Effort**: 0.5 hours (AI agent)
- **Validation**: Swagger validation

#### Change 010: Integration Testing Suite
- **Type**: rust (tests)
- **Files**: 
  - `tests/integration/query_execution_test.rs`
- **Description**: End-to-end query execution tests
- **Dependencies**: All previous changes
- **Effort**: 7 hours
- **Validation**: 100% flow coverage

**Total Effort**: 17 hours (AI agent + OpenSpec)

---

## Dependencies

- **Depends On**: 
  - feature-query-definitions (query metadata)
  - feature-plugins (query execution plugins)
  - feature-gts-core (GTS registry access)
- **Blocks**: 
  - feature-datasources (datasources use queries)
  - feature-widget-items (widgets execute queries)

---

## References

- Overall Design: `architecture/DESIGN.md` Section 2 (Actors), Section 3 (System Capabilities)
- GTS Types: Query GTS schema files (owned by feature-query-definitions)
- OpenAPI Spec: `architecture/openapi/v1/api.yaml` (query execution endpoints)
- Feature Manifest: `architecture/features/FEATURES.md` (feature-query-execution entry)
