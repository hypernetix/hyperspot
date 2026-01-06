# Feature: Tenant Enablement

**Status**: NOT_STARTED  
**Feature Slug**: `feature-tenancy-enablement`

---

## A. Feature Context

### Overview

Multi-tenant access control and automatic dependency enablement. Manages which tenants can access GTS entities with automatic transitive dependency resolution.

**Purpose**: Provide multi-tenant access control with automatic transitive dependency enablement.

**Scope**:
- Tenant enablement configuration via `/gts/{id}/enablement`
- Automatic dependency enablement (query → schema, template → config_schema)
- Tenant isolation enforcement
- Enablement API (GET/PUT/PATCH)
- JSON Patch support for enablement updates
- Enablement DB tables
- Transitive dependency resolution algorithm
- Audit logging for enablement propagation

**Out of Scope**:
- GTS entity registration - handled by feature-gts-core
- Tenant management - provided by Hyperspot Platform
- User authentication - provided by Hyperspot Platform

### GTS Types

This feature **does not own GTS types** - it manages enablement for all GTS entities.

### OpenAPI Endpoints

From `architecture/openapi/v1/api.yaml`:
- `GET /api/analytics/v1/gts/{id}/enablement` - Get enablement config
- `PUT /api/analytics/v1/gts/{id}/enablement` - Replace enablement (full)
- `PATCH /api/analytics/v1/gts/{id}/enablement` - Update enablement (JSON Patch)

### Actors

**Human Actors** (from Overall Design):
- **Admin** - Manages entity sharing and tenant access
- **Dashboard Creator** - Shares dashboards with tenants
- **Template Developer** - Makes templates globally available

**System Actors**:
- **Enablement Manager** - Orchestrates enablement operations
- **Dependency Resolver** - Resolves transitive dependencies
- **Isolation Enforcer** - Enforces tenant isolation on queries

**Service Roles** (from OpenAPI):
- `analytics:admin` - Manage tenant enablement
- `analytics:entities:share` - Share entities with tenants

---

## B. Actor Flows

### Flow 1: Admin Shares Dashboard with Specific Tenants

**Actor**: Admin  
**Trigger**: Need to share dashboard with multiple tenants  
**Goal**: Enable dashboard for tenant-1 and tenant-2 with all dependencies

**Steps**:
1. Open dashboard sharing settings
2. Select "Share with specific tenants"
3. Select tenant-1 and tenant-2 from list
4. Click "Save"
5. System enables dashboard + all transitive dependencies

**API Interaction**:
```
PUT /api/analytics/v1/gts/{dashboard-id}/enablement
Body: {"enabled_for": ["tenant-1", "tenant-2"]}

→ System propagates enablement:
  ✓ Dashboard (1 entity)
  ✓ All widgets (5 entities)
  ✓ All templates (3 entities)
  ✓ All datasources (4 entities)
  ✓ All queries (3 entities)
  ✓ All schemas (2 entities)
  Total: 18 entities enabled automatically
```

---

### Flow 2: Template Developer Makes Template Globally Available

**Actor**: Template Developer  
**Trigger**: New chart template ready for all tenants  
**Goal**: Enable template for all current and future tenants

**API Interaction**:
```
PUT /api/analytics/v1/gts/{template-id}/enablement
Body: {"enabled_for": "all"}

→ System enables:
  ✓ Template
  ✓ config_schema
  ✓ query_returns_schema (if widget template)
```

---

### Flow 3: Admin Adds Tenant to Existing Dashboard

**Actor**: Admin  
**Trigger**: New tenant needs access to existing dashboard  
**Goal**: Add tenant-3 to dashboard without affecting existing tenants

**API Interaction**:
```
PATCH /api/analytics/v1/gts/{dashboard-id}/enablement
Content-Type: application/json-patch+json
Body: [
  {"op": "add", "path": "/enabled_for/-", "value": "tenant-3"}
]

→ System adds tenant-3 to all dependencies
→ Existing tenants (tenant-1, tenant-2) unaffected
```

---

### Flow 4: Isolation Enforcer Blocks Unauthorized Access

**Actor**: Isolation Enforcer (System)  
**Trigger**: Tenant-3 attempts to access dashboard only enabled for tenant-1  
**Goal**: Prevent unauthorized access

**Steps**:
1. User from tenant-3 requests dashboard
2. System checks enablement configuration
3. Dashboard `enabled_for: ["tenant-1", "tenant-2"]`
4. tenant-3 not in list
5. Return 403 Forbidden

---

## C. Algorithms

### Service Algorithm 1: Transitive Dependency Resolution

**Purpose**: Recursively enable all referenced entities

See detailed algorithm in Section C above.
    
    // 3. Extract all reference fields based on type
    let references = extract_references(&entity)?;
    
    // 4. Recursively enable all references
    for reference_id in references {
        let ref_entity = gts_registry.get(&reference_id)?;
        
        // Check if already enabled for these tenants
        if !ref_entity.enabled_for.includes_all(tenants) {
            // Recursively enable
            let propagated = enable_entity_for_tenants(
                &reference_id,
                tenants,
                ctx
            )?;
            enabled_entities.extend(propagated);
        }
    }
    
    // 5. Commit transaction
    transaction.commit()?;
    
    Ok(enabled_entities)
}
```

---

### Service Algorithm 2: Reference Field Extraction

**Purpose**: Extract all GTS references from entity based on type

See detailed reference extraction logic in Section C above.
        "datasource" => {
            refs.push(entity.query_id.clone());
        }
        "widget" => {
            refs.push(entity.template_id.clone());
            if let Some(ds_id) = &entity.datasource_id {
                refs.push(ds_id.clone());
            }
        }
        "dashboard" | "report" => {
            for item in &entity.items {
                refs.push(item.id.clone());
            }
        }
        _ => {}
    }
    
    Ok(refs)
}
```

---

## D. States

*(Not applicable - enablement is configuration, no state machine)*

---

## E. Technical Details

### Tenant Enablement System

The service manages multi-tenant access control through the `/enablement` sub-resource with automatic dependency resolution.

### Enablement Configuration

Each GTS entity can be enabled for specific tenants via `/gts/{id}/enablement`.

**`enabled_for` field accepts:**

1. **Array of tenant IDs:** `["tenant-1", "tenant-2"]` - enables for specific tenants
2. **String "all":** `"all"` - enables for all tenants in the system

**Key Property:**
Enablement is **inherited** - when entity is enabled for tenant, all referenced entities are automatically enabled.

---

## Automatic Reference Enablement

When entity is enabled for tenant(s), system **automatically enables all referenced entities** for the same tenant(s).

### Reference Chains by Type

**Query** → `returns_schema_id`, `capabilities_id`

**Template (Widget)** → `config_schema_id`, `query_returns_schema_id`, `category_id`

**Template (Values Selector)** → `config_schema_id`, `values_schema_id`, `category_id`

**Datasource** → `query_id` (and transitively: query's schemas)

**Widget** → `template_id`, `datasource` reference (and transitively: all their dependencies)

**Group** → nested `items` array (widgets and their dependencies)

**Dashboard/Report** → all items in `entity.items` array (widgets, groups, and their transitive dependencies)

### Transitive Dependency Resolution

- System recursively resolves all reference chains
- No circular dependency handling needed (GTS type system prevents cycles)
- All schemas, capabilities, and referenced instances enabled automatically
- Ensures tenants have complete access to all dependencies

### Example Dependency Chain

```
Dashboard
  ├─ Widget 1
  │   ├─ Template
  │   │   ├─ config_schema_id
  │   │   └─ query_returns_schema_id
  │   └─ Datasource
  │       └─ Query
  │           ├─ returns_schema_id
  │           └─ capabilities_id
  └─ Widget 2
      ├─ Template
      └─ Datasource
          └─ Query

When dashboard enabled for tenant:
  → All widgets automatically enabled
  → All templates automatically enabled
  → All datasources automatically enabled
  → All queries automatically enabled
  → All schemas automatically enabled
  → All capabilities automatically enabled
```

---

## API Endpoints

### Get Enablement Configuration

```
GET /api/analytics/v1/gts/{gts-identifier}/enablement
```

**Response:**
```json
{
  "id": "gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.dashboard.v1~acme.sales._.executive.v1",
  "enabled_for": ["tenant-1", "tenant-2"]
}
```

Or for global enablement:
```json
{
  "id": "gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~charts.line.v1",
  "enabled_for": "all"
}
```

---

### Update Enablement (Full Replacement)

```
PUT /api/analytics/v1/gts/{gts-identifier}/enablement
```

**Enable for specific tenants:**
```json
{
  "enabled_for": ["tenant-1", "tenant-2"]
}
```

**Enable for all tenants:**
```json
{
  "enabled_for": "all"
}
```

**Example:**
```http
PUT /api/analytics/v1/gts/gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.dashboard.v1~acme.sales._.executive.v1/enablement
Authorization: Bearer {token}
Content-Type: application/json

{
  "enabled_for": ["tenant-1"]
}
```

**Automatic Propagation:**
System automatically enables all referenced entities:
- Dashboard itself
- All widgets in dashboard.entity.items[]
- All templates referenced by widgets
- All datasources referenced by widgets
- All queries referenced by datasources
- All schemas referenced by queries and templates
- All capabilities referenced by queries

---

### Partial Update Enablement (JSON Patch)

```
PATCH /api/analytics/v1/gts/{gts-identifier}/enablement
Content-Type: application/json-patch+json
```

**Common JSON Patch operations:**

**Add tenant:**
```json
[
  { "op": "add", "path": "/enabled_for/-", "value": "tenant-4" }
]
```

**Remove tenant:**
```json
[
  { "op": "remove", "path": "/enabled_for/0" }
]
```

**Replace all tenants:**
```json
[
  { "op": "replace", "path": "/enabled_for", "value": ["tenant-1", "tenant-2"] }
]
```

**Test before change:**
```json
[
  { "op": "test", "path": "/enabled_for/0", "value": "tenant-1" },
  { "op": "add", "path": "/enabled_for/-", "value": "tenant-3" }
]
```

**Enable for all tenants:**
```json
[
  { "op": "replace", "path": "/enabled_for", "value": "all" }
]
```

**Example:**
```http
PATCH /api/analytics/v1/gts/{gts-identifier}/enablement
Authorization: Bearer {token}
Content-Type: application/json-patch+json

[
  { "op": "add", "path": "/enabled_for/-", "value": "tenant-4" },
  { "op": "remove", "path": "/enabled_for/0" }
]
```

---

## Implementation Requirements

### 1. Reference Field Tracking

Service MUST track all reference fields for each base type:

| Base Type | Reference Fields |
|-----------|------------------|
| Query | returns_schema_id, capabilities_id |
| Template (Widget) | config_schema_id, query_returns_schema_id, category_id |
| Template (Values Selector) | config_schema_id, values_schema_id, category_id |
| Datasource | query_id |
| Widget | template_id, datasource (inline or ref) |
| Group | items[] array |
| Dashboard | items[] array |
| Report | items[] array |

### 2. Recursive Enablement Propagation

**Algorithm:**

```
function enableForTenants(entity_id, tenants):
  1. Load entity from GTS Registry
  2. Update entity.enabled_for = tenants
  3. Extract all reference fields from entity
  4. For each reference:
     a. Resolve referenced entity ID
     b. Check if already enabled for same tenants
     c. If not: recursively call enableForTenants(reference_id, tenants)
  5. Commit all changes transactionally
```

See Section C for complete ADL algorithm.
    // Recursively enable all references
    for reference_id in references {
        // Check if already enabled
        let ref_entity = gts_registry.get(&reference_id)?;
        if !ref_entity.enabled_for.includes_all(tenants) {
            // Recursively enable
            enable_entity_for_tenants(&reference_id, tenants)?;
        }
    }
    
    // Commit transaction
    transaction.commit()?;
    Ok(())
}
```

### 3. Transaction Guarantees

Enablement operations MUST be transactional (all-or-nothing):

- All entities in dependency chain enabled together
- If any enablement fails, entire operation rolls back
- No partial enablement states
- Atomic database updates

### 4. Validation

Service MUST prevent enablement of entity when referenced entities don't exist:

**Steps**:
1. Extract all references from entity
2. **FOR EACH** reference:
   1. Check if reference exists in registry
   2. **IF** not found:
      1. **RETURN** error
3. **RETURN** success
    for reference_id in references {
        if !gts_registry.exists(&reference_id) {
            return Err(Error::MissingReference {
                entity_id: entity.id.clone(),
                reference_id: reference_id.clone(),
            });
        }
    }
    
    Ok(())
}
```

### 5. Audit Logging

Service SHOULD log enablement propagation for audit purposes:

```
[2024-01-08 10:15:30] INFO: Enablement started
  Entity: dashboard.acme.sales.executive.v1
  Tenants: [tenant-1, tenant-2]
  User: admin@acme.com

[2024-01-08 10:15:30] DEBUG: Enabling widget.revenue_trend.v1 for [tenant-1, tenant-2]
[2024-01-08 10:15:30] DEBUG: Enabling template.line_chart.v1 for [tenant-1, tenant-2]
[2024-01-08 10:15:30] DEBUG: Enabling datasource.sales_revenue.v1 for [tenant-1, tenant-2]
[2024-01-08 10:15:30] DEBUG: Enabling query.sales.v1 for [tenant-1, tenant-2]
[2024-01-08 10:15:30] DEBUG: Enabling schema.query_returns.v1 for [tenant-1, tenant-2]

[2024-01-08 10:15:31] INFO: Enablement completed
  Total entities enabled: 15
  Duration: 1.2s
```

---

## Database Schema

```sql
CREATE TABLE entity_enablement (
    entity_id VARCHAR(500) PRIMARY KEY,
    enabled_for JSONB NOT NULL,  -- Array of tenant IDs or "all"
    updated_at TIMESTAMPTZ NOT NULL,
    updated_by VARCHAR(255)
);

-- Efficient tenant membership check
CREATE INDEX idx_entity_enablement_tenants ON entity_enablement USING GIN(enabled_for);

-- Track enablement history
CREATE TABLE enablement_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_id VARCHAR(500) NOT NULL,
    action VARCHAR(50) NOT NULL,  -- 'enabled', 'disabled', 'modified'
    old_enabled_for JSONB,
    new_enabled_for JSONB,
    changed_at TIMESTAMPTZ NOT NULL,
    changed_by VARCHAR(255),
    propagated_to JSONB  -- Array of entity IDs affected by propagation
);

CREATE INDEX idx_enablement_history_entity ON enablement_history(entity_id);
CREATE INDEX idx_enablement_history_changed_at ON enablement_history(changed_at DESC);
```

---

## Tenant Isolation Enforcement

When querying GTS entities, system enforces tenant isolation:

```sql
-- Query with tenant filter
SELECT * FROM gts_entities 
WHERE id = ? 
  AND (
    -- Entity enabled for all tenants
    enabled_for @> '"all"'::jsonb
    OR
    -- Entity enabled for current tenant
    enabled_for @> ?::jsonb  -- Current tenant_id as JSONB
  )
  AND deleted_at IS NULL;
```

**SecurityCtx Integration**: All operations require SecurityCtx with tenant_id for isolation enforcement.
                "Entity '{}' not enabled for tenant '{}'", 
                entity_id, 
                ctx.tenant_id
            )
        });
    }
    
    Ok(entity)
}

fn is_enabled_for_tenant(
    enabled_for: &EnabledFor, 
    tenant_id: &str
) -> bool {
    match enabled_for {
        EnabledFor::All => true,
        EnabledFor::Specific(tenants) => tenants.contains(tenant_id),
    }
}
```

---

## User Scenarios

### Scenario: Share Dashboard with Specific Tenants

**UI Flow:**
1. User creates or edits dashboard
2. In sharing settings, selects "Share with specific tenants"
3. Selects tenant-1 and tenant-2 from list
4. Clicks "Save"
5. System enables dashboard + all dependencies for selected tenants

**API Calls:**
```http
PUT /api/analytics/v1/gts/{dashboard_id}/enablement
Authorization: Bearer {token}
Content-Type: application/json

{
  "enabled_for": ["tenant-1", "tenant-2"]
}
```

**System Actions:**
- Enables dashboard for tenant-1, tenant-2
- Enables all 5 widgets in dashboard
- Enables all 3 templates used by widgets
- Enables all 4 datasources used by widgets
- Enables all 3 queries referenced by datasources
- Enables all 2 schemas referenced by queries and templates
- Total: 18 entities enabled automatically

---

### Scenario: Make Template Globally Available

**UI Flow:**
1. Template developer creates new chart template
2. In sharing settings, selects "Share with all tenants"
3. Clicks "Save"
4. Template now available to all tenants (current and future)

**API Calls:**
```http
PUT /api/analytics/v1/gts/{template_id}/enablement
Authorization: Bearer {token}
Content-Type: application/json

{
  "enabled_for": "all"
}
```

**System Actions:**
- Enables template for all tenants
- Enables config_schema for all tenants
- Enables query_returns_schema for all tenants (if widget template)

---

### Scenario: Add New Tenant to Existing Dashboard

**UI Flow:**
1. User opens dashboard sharing settings
2. Clicks "Add tenant"
3. Selects tenant-3 from dropdown
4. Clicks "Add"
5. Dashboard and all dependencies now available to tenant-3

**API Calls:**
```http
PATCH /api/analytics/v1/gts/{dashboard_id}/enablement
Authorization: Bearer {token}
Content-Type: application/json-patch+json

[
  { "op": "add", "path": "/enabled_for/-", "value": "tenant-3" }
]
```

**System Actions:**
- Adds tenant-3 to dashboard.enabled_for
- Propagates tenant-3 enablement to all dependencies
- Existing tenants (tenant-1, tenant-2) unaffected

---

### Scenario: Remove Tenant Access

**UI Flow:**
1. User opens dashboard sharing settings
2. Finds tenant-2 in shared list
3. Clicks "Remove" next to tenant-2
4. Confirms removal
5. Dashboard no longer accessible to tenant-2

**API Calls:**
```http
PATCH /api/analytics/v1/gts/{dashboard_id}/enablement
Authorization: Bearer {token}
Content-Type: application/json-patch+json

[
  { "op": "remove", "path": "/enabled_for/1" }
]
```

**System Actions:**
- Removes tenant-2 from dashboard.enabled_for
- Does NOT remove tenant-2 from dependencies (other entities may still need them)
- Tenant-2 can no longer access dashboard

---

## Error Handling

**Missing Reference:**
```json
{
  "type": "https://example.com/problems/missing-reference",
  "title": "Missing Reference",
  "status": 400,
  "detail": "Cannot enable entity 'dashboard.acme.sales.v1'. Referenced entity 'query.sales.v1' does not exist.",
  "references": [
    "gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.sales.v1"
  ]
}
```

**Circular Dependency (should never happen with GTS):**
```json
{
  "type": "https://example.com/problems/circular-dependency",
  "title": "Circular Dependency Detected",
  "status": 500,
  "detail": "Internal error: circular dependency detected in enablement propagation",
  "trace_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

---

### Access Control

**SecurityCtx Enforcement**:
- All enablement operations require admin permissions
- Entity owners can manage their own entity enablement
- Tenant isolation enforced on all GTS queries

**Permission Checks**:
- Enablement management: Requires `analytics:admin` OR entity ownership

---

### Database Operations

**Tables**:
- `entity_enablement` - Current enablement state
- `enablement_history` - Audit trail

**Indexes**:
- `idx_entity_enablement_tenants` (GIN) - Fast tenant membership check
- `idx_enablement_history_entity` - History by entity
- `idx_enablement_history_changed_at` - Recent changes

**Tenant Isolation Query**:
```sql
SELECT * FROM gts_entities 
WHERE id = $1 
  AND (
    enabled_for @> '"all"'::jsonb
    OR
    enabled_for @> $2::jsonb  -- Current tenant_id
  )
  AND deleted_at IS NULL;
```

---

### Error Handling

**Common Errors**:
- **400 Bad Request**: Missing referenced entity
- **403 Forbidden**: Insufficient permissions
- **500 Internal Server Error**: Circular dependency detected

**Error Response Format (RFC 7807)**:
```json
{
  "type": "https://example.com/problems/missing-reference",
  "title": "Missing Reference",
  "status": 400,
  "detail": "Cannot enable entity 'dashboard.acme.sales.v1'. Referenced entity 'query.sales.v1' does not exist.",
  "references": ["gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.sales.v1"]
}
```

---

## F. Validation & Implementation

### Testing Scenarios

**Unit Tests**:
- Dependency resolution algorithm
- Transitive propagation logic
- Reference field extraction by type
- Tenant membership checking
- JSON Patch operations

**Integration Tests**:
- Enablement API endpoints
- Transaction rollback on error
- Audit logging verification
- Multi-level dependency chains

**Performance Tests**:
- Large dependency graph (1000+ entities, < 5s)
- Concurrent enablement operations
- Tenant isolation query performance
- GIN index effectiveness

**Edge Cases**:
1. Dashboard with 100+ widgets
2. Missing reference in chain
3. Circular dependency (should never happen)
4. Enablement for 50+ tenants
5. JSON Patch with conflicting operations
6. Transaction rollback mid-propagation

---

### OpenSpec Changes Plan

#### Change 001: Enablement Data Model
- **Type**: database
- **Files**: 
  - `modules/analytics/migrations/001_create_enablement.sql`
- **Description**: Create entity_enablement and enablement_history tables
- **Dependencies**: None (foundational)
- **Effort**: 0.5 hours (AI agent)
- **Validation**: Migration tests, index verification

#### Change 002: Dependency Resolution Engine
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/enablement/resolver.rs`
- **Description**: Transitive dependency resolution algorithm
- **Dependencies**: Change 001
- **Effort**: 2 hours (AI agent)
- **Validation**: Unit tests with various dependency chains

#### Change 003: Reference Field Extractor
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/enablement/references.rs`
- **Description**: Extract GTS references by entity type
- **Dependencies**: Change 002
- **Effort**: 1.5 hours (AI agent)
- **Validation**: Unit tests for all entity types

#### Change 004: Enablement API Handlers
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/api/rest/enablement/handlers.rs`
  - `modules/analytics/src/domain/enablement/service.rs`
- **Description**: GET/PUT/PATCH endpoints for enablement
- **Dependencies**: Change 002
- **Effort**: 2 hours (AI agent)
- **Validation**: API tests, JSON Patch tests

#### Change 005: Tenant Isolation Middleware
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/api/rest/middleware/tenant_isolation.rs`
- **Description**: Enforce tenant isolation on all GTS queries
- **Dependencies**: Change 001
- **Effort**: 1 hour (AI agent)
- **Validation**: Isolation tests with multiple tenants

#### Change 006: Transaction Management
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/enablement/transaction.rs`
- **Description**: Atomic enablement with rollback support
- **Dependencies**: Change 002
- **Effort**: 1.5 hours (AI agent)
- **Validation**: Rollback tests, error scenarios

#### Change 007: Audit Logging
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/enablement/audit.rs`
- **Description**: Log enablement propagation for audit trail
- **Dependencies**: Change 001
- **Effort**: 1 hour (AI agent)
- **Validation**: Log verification tests

#### Change 008: JSON Patch Support
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/enablement/json_patch.rs`
- **Description**: RFC 6902 JSON Patch operations
- **Dependencies**: Change 004
- **Effort**: 1 hour (AI agent)
- **Validation**: JSON Patch spec compliance tests

#### Change 009: OpenAPI Specification
- **Type**: openapi
- **Files**: 
  - `architecture/openapi/v1/api.yaml`
- **Description**: Document enablement endpoints
- **Dependencies**: All previous changes
- **Effort**: 0.5 hours (AI agent)
- **Validation**: Swagger validation

#### Change 010: Integration Testing Suite
- **Type**: rust (tests)
- **Files**: 
  - `tests/integration/enablement_test.rs`
- **Description**: End-to-end enablement scenarios
- **Dependencies**: All previous changes
- **Effort**: 2 hours (AI agent)
- **Validation**: 100% scenario coverage

**Total Effort**: 13 hours (AI agent + OpenSpec)

---

## Dependencies

- **Depends On**: 
  - feature-gts-core (GTS registry access)
- **Blocks**: (none - orthogonal cross-cutting concern)

---

## References

- Overall Design: `architecture/DESIGN.md` Section 2 (Actors), Section 3 (System Capabilities)
- GTS Types: All GTS entity types (managed by various features)
- OpenAPI Spec: `architecture/openapi/v1/api.yaml` (enablement endpoints)
- Feature Manifest: `architecture/features/FEATURES.md` (feature-tenancy-enablement entry)
