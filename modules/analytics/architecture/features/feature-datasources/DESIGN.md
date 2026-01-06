# Feature: Datasources

**Status**: NOT_STARTED  
**Feature Slug**: `feature-datasources`

---

## A. Feature Context

### Overview

Datasource instances with query + parameter binding. Connects query definitions with runtime parameters and UI controls, providing reusable data configurations.

**Purpose**: Provide reusable datasource configurations that bind queries with default parameters and UI control settings.

**Scope**:
- Datasource GTS type: `datasource.v1~`
- Datasource DB tables
- Query + parameters binding
- Values selector integration for parameter inputs
- Runtime parameter injection
- Datasource reusability and presets
- Custom datasource search
- Render options for UI controls
- Parameter merging and priority resolution

**Out of Scope**:
- Query execution - handled by feature-query-execution
- Query type registration - handled by feature-query-definitions
- Values selector templates - handled by feature-values-selector-templates
- Widget items - handled by feature-widget-items

### GTS Types

This feature **owns** the datasource GTS type:

**GTS Type Identifier**: `gts://gts.hypernetix.hyperspot.ax.datasource.v1~`

References from `gts/types/`:
- (GTS schema file to be created)

### OpenAPI Endpoints

From `architecture/openapi/v1/api.yaml`:
- `GET /api/analytics/v1/gts` - List/search datasources (OData)
- `POST /api/analytics/v1/gts` - Create datasource instance
- `GET /api/analytics/v1/gts/{datasource-id}` - Get datasource
- `PUT /api/analytics/v1/gts/{datasource-id}` - Update datasource
- `DELETE /api/analytics/v1/gts/{datasource-id}` - Delete datasource
- `PUT /api/analytics/v1/gts/{datasource-id}/enablement` - Share datasource

### Actors

**Human Actors** (from Overall Design):
- **Admin** - Creates and manages datasource presets
- **Widget Designer** - Selects datasources for widgets
- **End User** - Interacts with datasource UI controls

**System Actors**:
- **Datasource Manager** - Manages datasource lifecycle
- **Parameter Merger** - Merges datasource/widget/user parameters
- **UI Renderer** - Renders datasource controls based on render_options

**Service Roles** (from OpenAPI):
- `analytics:datasources:read` - View datasources
- `analytics:datasources:write` - Create/edit datasources
- `analytics:datasources:delete` - Delete datasources

---

## B. Actor Flows

### Flow 1: Admin Creates Datasource Preset

**Actor**: Admin  
**Trigger**: Need reusable datasource configuration  
**Goal**: Create datasource preset for widget library

**Steps**:
1. Select query from available queries
2. Configure default OData parameters ($filter, $orderby, $top, etc.)
3. Configure render_options (which UI controls to show)
4. Configure filter fields with values selector templates
5. Save as datasource preset
6. Optionally share with specific tenants

**API Interaction**:
```
POST /api/analytics/v1/gts
Type: gts.hypernetix.hyperspot.ax.datasource.v1~
Instance: gts.hypernetix.hyperspot.ax.datasource.v1~acme.monitoring.metrics.cpu_usage.v1

Body:
{
  "query_id": "gts.hypernetix.hyperspot.ax.query.v1~monitoring.system._.cpu.v1",
  "params": {
    "$filter": "server_type eq 'production'",
    "$orderby": "timestamp desc",
    "$top": 100
  },
  "render_options": {
    "filters": {"enabled": true},
    "time_range": {"enabled": true, "default": "last_24h"},
    "sorting": {"enabled": true}
  }
}

PUT /api/analytics/v1/gts/{datasource-id}/enablement
Body: {"enabled_for": ["tenant-1", "tenant-2"]}
```

---

### Flow 2: Widget Designer Selects Datasource

**Actor**: Widget Designer  
**Trigger**: Configuring widget data source  
**Goal**: Select and customize datasource for widget

**Steps**:
1. Browse available datasource presets
2. Select datasource that matches query needs
3. Optionally override default parameters
4. Preview data with current configuration
5. Save widget with datasource configuration

**API Interaction**:
```
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.datasource.v1~')
GET /api/analytics/v1/gts/{datasource-id}
GET /api/analytics/v1/queries/{query-id}?... (preview)
```

---

### Flow 3: End User Interacts with Datasource Controls

**Actor**: End User  
**Trigger**: Viewing widget on dashboard  
**Goal**: Filter/sort/paginate data

**Steps**:
1. Widget renders with datasource default parameters
2. User sees UI controls based on render_options
3. User adjusts filters (e.g., select server type)
4. User changes time range (e.g., last 7 days)
5. User sorts by column
6. Widget re-executes query with merged parameters

**Parameter Merging**:
```
Datasource default: {$filter: "status eq 'active'", $top: 50}
Widget override: {$top: 100}
User input: {$filter: "status eq 'active' and region eq 'EMEA'"}

Final params: {$filter: "...region eq 'EMEA'", $top: 100}
```

---

### Flow 4: System Merges Parameters at Runtime

**Actor**: Parameter Merger (System)  
**Trigger**: Widget query execution  
**Goal**: Combine all parameter sources

**Steps**:
1. Load datasource configuration
2. Load widget parameter overrides
3. Load user runtime inputs from UI
4. Apply parameter priority (user > widget > datasource > query)
5. Build final OData query string
6. Execute query with merged parameters

---

## C. Algorithms

### Service Algorithm 1: Parameter Merging

**Purpose**: Merge parameters from multiple sources with priority

**Priority Order** (highest to lowest):
1. User runtime inputs (from UI controls)
2. Widget-specific overrides
3. Datasource default params
4. Query default values

**Steps**:
1. Start with empty result
2. Apply datasource defaults
3. Apply widget overrides (higher priority)
4. Apply user inputs (highest priority)
5. **RETURN** merged parameters
    // Apply user runtime inputs (highest priority)
    final_params.merge(user_inputs);
    
    final_params
}
```

---

### UI Algorithm 1: Render Options Processing

**Purpose**: Render appropriate UI controls based on capabilities and render_options

**Steps**:
1. Load query capabilities
2. Initialize empty controls list
3. **IF** filters supported AND enabled:
   1. **FOR EACH** filter field:
      1. Create filter control
      2. Add to controls list
4. **IF** time range supported AND enabled:
   1. Create time range control
   2. Add to controls list
5. **IF** sorting supported AND enabled:
   1. Create sort control
   2. Add to controls list
6. **RETURN** controls list
        }
    }
    
    // Time range
    if should_render_control(&capabilities.filter, &datasource.render_options.time_range) {
        controls.push(render_time_range_control());
    }
    
    // Sorting, pagination, etc.
    // ...
    
    controls
}
```

---

## D. States

*(Not applicable - datasources are stateless configurations)*

---

## E. Technical Details

### Query and Datasource Architecture

**Conceptual Separation**:
- **Query** - Base component with data retrieval logic and API contract
- **Datasource** - Configuration built on Query with default parameters

**User Workflow**:
1. System admin registers Query definitions
2. Admin creates Datasource configs (Query + default params)
3. User selects Datasource from list when creating widget
4. User customizes parameters if needed
5. Widget config includes final datasource configuration

---

## Datasource Registration

### Register Datasource

```
POST /api/analytics/v1/gts
Type: gts.hypernetix.hyperspot.ax.datasource.v1~
Instance: gts.hypernetix.hyperspot.ax.datasource.v1~acme.monitoring.metrics.cpu_usage.v1
Key fields:
  - query_id → links to query instance
  - params → OData parameters ($filter, $orderby, $top, $select, $count)
  - render_options → UI controls (filters, time, sorting, pagination)
```

**Example Datasource:**

```json
{
  "id": "gts.hypernetix.hyperspot.ax.datasource.v1~acme.monitoring.metrics.cpu_usage.v1",
  "type": "gts.hypernetix.hyperspot.ax.datasource.v1~",
  "entity": {
    "name": "CPU Usage Metrics",
    "description": "Real-time CPU usage data with configurable time range",
    "query_id": "gts.hypernetix.hyperspot.ax.query.v1~monitoring.system._.cpu.v1",
    "params": {
      "$filter": "server_type eq 'production'",
      "$orderby": "timestamp desc",
      "$top": 100,
      "$select": "timestamp,cpu_percent,server_name"
    },
    "render_options": {
      "filters": {
        "enabled": true,
        "fields": [
          {
            "name": "server_type",
            "label": "Server Type",
            "type": "values_selector",
            "template_id": "gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.values_selector.v1~ui.dropdown.v1",
            "query_id": "gts.hypernetix.hyperspot.ax.query.v1~monitoring._.server_types.v1"
          }
        ]
      },
      "time_range": {
        "enabled": true,
        "default": "last_24h",
        "custom_range_enabled": true
      },
      "sorting": {
        "enabled": true,
        "fields": ["timestamp", "cpu_percent", "server_name"]
      },
      "pagination": {
        "enabled": true,
        "page_size_options": [25, 50, 100, 200],
        "default_page_size": 50
      },
      "search": {
        "enabled": false
      },
      "grouping": {
        "enabled": false
      }
    }
  }
}
```

---

## Datasource Configuration Fields

### query_id (required)
- **Type:** GTS identifier reference
- **Description:** Links to the Query definition
- **Example:** `gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.sales.v1`

### params (optional)
- **Type:** Object containing OData query parameters
- **Description:** Default query parameters that can be overridden by widgets
- **Fields:**
  - `$filter` - OData filter expression
  - `$orderby` - Sort expression
  - `$top` - Page size
  - `$skip` - Offset
  - `$select` - Field projection
  - `$expand` - Navigation property expansion
  - `$search` - Full-text search
  - `$count` - Include total count

### render_options (optional)
- **Type:** Object defining UI controls for parameters
- **Description:** Configures which UI controls are shown to users for adjusting query parameters

---

## Render Options Structure

Render options define what UI controls are exposed to users for adjusting query parameters.

### filters
- **enabled** - Show filter controls
- **fields** - Array of filterable fields with their UI controls
  - **name** - Field name
  - **label** - Display label
  - **type** - Control type (values_selector, date_range, text_input, etc.)
  - **template_id** - Values selector template for dropdowns/pickers
  - **query_id** - Query to fetch filter values (for values selectors)

### time_range
- **enabled** - Show time range selector
- **default** - Default time range (last_24h, last_7d, last_30d, custom)
- **custom_range_enabled** - Allow custom date range selection

### sorting
- **enabled** - Show sorting controls
- **fields** - Array of sortable field names
- **multi_column** - Allow multi-column sorting

### pagination
- **enabled** - Show pagination controls
- **page_size_options** - Available page sizes
- **default_page_size** - Initial page size

### search
- **enabled** - Show full-text search box
- **placeholder** - Search input placeholder

### grouping
- **enabled** - Show grouping controls
- **fields** - Array of groupable field names
- **aggregations** - Available aggregation functions (SUM, AVG, COUNT, MIN, MAX)

---

## OData Capabilities + Render Options Integration

The platform combines **OData metadata capabilities** (from query.capabilities_id) with **datasource.render_options** to render appropriate UI controls.

**Two-layer system:**

### 1. OData Capabilities
Define what query *technically supports*:
- FilterFunctions
- SortRestrictions
- SearchRestrictions
- SelectSupport
- ExpandRestrictions
- TopSupported
- SkipSupported

### 2. Render Options
Define what UI *should show to user*:
- Which filters are exposed
- Sort options
- Pagination config
- Time range controls
- Search visibility

### UI Rendering Logic

```
For each render option:
  1. Check if query capabilities support the feature
  2. Check if datasource render_options enable the feature
  3. If BOTH true → render UI control
  4. If capabilities missing → disable/hide control
  5. If render_options disabled → hide control
```

**Example:**
- Query supports `$filter` on all fields (capabilities)
- Datasource only enables filters on `server_type` and `region` (render_options)
- Result: UI shows only server_type and region filter dropdowns

---

## Datasource Reusability

Datasources are **reusable configurations** that can be:

1. **Referenced by multiple widgets** - Many widgets can use the same datasource
2. **Saved as presets** - Create datasource instances for common queries
3. **Inline in widgets** - Embed datasource config directly in widget
4. **Shared across tenants** - Enable datasource for specific tenants via `/enablement`

### Datasource Preset Creation

**UI Flow:**
1. Configure datasource (query_id, params, render_options)
2. Click "Save as Preset"
3. Enter preset metadata (name, description, category)
4. Configure sharing:
   - Choose visibility: private, specific tenants, or all tenants
   - Select tenants if sharing with specific tenants
5. Save preset as datasource instance in GTS registry
6. Preset available for reuse in multiple widgets

**API Calls:**
```
POST /api/analytics/v1/gts  # Create datasource instance (preset)
  # Type: gts.hypernetix.hyperspot.ax.datasource.v1~
  # Instance contains query_id, params, render_options configuration
PUT /api/analytics/v1/gts/{datasource_preset_id}/enablement  # Share datasource preset with tenants (optional)
  # Body: { "enabled_for": ["tenant-1", "tenant-2"] } or { "enabled_for": "all" }
  # Allows specified tenants or all tenants to use this datasource preset
```

---

## Runtime Parameter Injection

When a widget executes a query:

1. **Load datasource config** - Fetch from preset or inline config
2. **Merge parameters** - Combine datasource defaults + widget overrides
3. **Apply user inputs** - Incorporate filter/sort/pagination changes from UI
4. **Build final query** - Construct complete OData query
5. **Execute query** - Call `/queries/{id}` with final parameters

**Parameter Priority (highest to lowest):**
1. User runtime inputs (from UI controls)
2. Widget-specific overrides
3. Datasource default params
4. Query default values

**Example**:
- Datasource default: `{$filter: "status eq 'active'", $top: 50}`
- Widget override: `{$top: 100}`
- User input: `{$filter: "status eq 'active' and region eq 'EMEA'"}`
- Final merged: `{$filter: "status eq 'active' and region eq 'EMEA'", $top: 100}`
}
```

---

## Widget Settings UI Integration

When user opens widget settings dialog, the UI renders datasource configuration controls.

### Datasource Configuration Section

**1. Query Selection** - Searchable dropdown of available queries

**2. OData Parameters** - Manual parameter configuration
- $filter expression editor
- $orderby field selection
- $top, $skip numeric inputs
- $select field picker
- $expand navigation property selector
- $search text input

**3. Render Options** - UI control configuration
- Enable/disable filters, sorting, pagination, time range, search, grouping
- Configure specific filter fields with values selector templates
- Set default values and options

### UI Controls Rendering

Based on render_options, the platform generates appropriate UI controls:

- **filters** → Dropdown/multi-select/date pickers (using values selector templates)
- **time_range** → Quick range buttons + custom range picker
- **sorting** → Column headers with sort indicators
- **pagination** → Page size selector + page navigation
- **search** → Full-text search input box
- **grouping** → Group by field selector + aggregation function picker

---

## Database Schema

```sql
CREATE TABLE datasources (
    id VARCHAR(500) PRIMARY KEY,
    type VARCHAR(500) NOT NULL,
    tenant VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    query_id VARCHAR(500) NOT NULL,
    params JSONB,
    render_options JSONB,
    registered_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    registered_by VARCHAR(255),
    updated_by VARCHAR(255),
    deleted_by VARCHAR(255),
    FOREIGN KEY (query_id) REFERENCES queries(id)
);

CREATE INDEX idx_datasources_tenant ON datasources(tenant);
CREATE INDEX idx_datasources_query_id ON datasources(query_id);
CREATE INDEX idx_datasources_deleted_at ON datasources(deleted_at) WHERE deleted_at IS NULL;
```

---

## User Scenarios

### Scenario: Create Datasource Preset

**UI Flow:**
1. Configure datasource (query_id, params, render_options)
2. Click "Save as Preset"
3. Enter preset metadata (name, description, category)
4. Configure sharing:
   - Choose visibility: private, specific tenants, or all tenants
   - Select tenants if sharing with specific tenants
5. Save preset as datasource instance in GTS registry
6. Preset available for reuse in multiple widgets

**API Calls:**
```
POST /api/analytics/v1/gts  # Create datasource instance (preset)
  # Type: gts.hypernetix.hyperspot.ax.datasource.v1~
  # Instance contains query_id, params, render_options configuration
PUT /api/analytics/v1/gts/{datasource_preset_id}/enablement  # Share datasource preset with tenants (optional)
  # Body: { "enabled_for": ["tenant-1", "tenant-2"] } or { "enabled_for": "all" }
  # Allows specified tenants or all tenants to use this datasource preset
```

### Scenario: Delete Datasource Preset

**UI Flow:**
1. Navigate to Datasource library
2. Select datasource preset to delete
3. Click delete button
4. Confirm deletion - warn if datasource is used in widgets
5. Datasource preset soft-deleted

**API Calls:**
```
DELETE /api/analytics/v1/gts/{datasource_preset_id}
  # Soft-delete datasource preset (sets deleted_at timestamp)
  # Returns: 204 No Content
```

---

### Access Control

**SecurityCtx Enforcement**:
- All datasource operations require authenticated user
- Tenant isolation enforced on all queries
- Datasource ownership via `registered_by` field

**Permission Checks**:
- Datasource creation: Requires `analytics:datasources:write`
- Datasource sharing: Requires `analytics:admin` + ownership verification

---

### Database Operations

**Tables**:
- `datasources` - Datasource configurations with tenant isolation

**Indexes**:
- `idx_datasources_tenant` - Fast tenant lookup
- `idx_datasources_query_id` - Datasource by query
- `idx_datasources_deleted_at` - Soft-delete filtering

**Queries**:
```sql
-- List tenant datasources
SELECT * FROM datasources 
WHERE tenant = $1 AND deleted_at IS NULL;

-- Get datasource with query metadata
SELECT d.*, q.name as query_name, q.capabilities_id
FROM datasources d
JOIN queries q ON d.query_id = q.id
WHERE d.id = $1;
```

---

### Error Handling

**Common Errors**:
- **404 Not Found**: Datasource or query not found
- **400 Bad Request**: Invalid OData parameters
- **403 Forbidden**: Insufficient permissions
- **422 Unprocessable Entity**: Invalid render_options schema
- **409 Conflict**: Datasource ID already exists

**Error Response Format (RFC 7807)**:
```json
{
  "type": "https://example.com/problems/invalid-odata-params",
  "title": "Invalid OData Parameters",
  "status": 400,
  "detail": "$filter expression contains unsupported operator 'like'",
  "instance": "/api/analytics/v1/gts/datasource-123"
}
```

---

## F. Validation & Implementation

### Testing Scenarios

**Unit Tests**:
- Parameter merging with priority resolution
- Render options validation
- OData parameter validation
- Filter field configuration

**Integration Tests**:
- Datasource CRUD operations
- Query execution with merged parameters
- Tenant enablement
- Soft-delete behavior

**UI Tests**:
- Render options control generation
- Values selector integration
- Time range picker
- Filter UI rendering

**Performance Tests**:
- Datasource lookup time (< 50ms)
- Parameter merging overhead (< 5ms)
- Large datasource list pagination

**Edge Cases**:
1. Datasource with invalid query_id reference
2. Conflicting parameter overrides
3. Render options with unsupported capabilities
4. Datasource used in 100+ widgets
5. Circular datasource references

---

### OpenSpec Changes Plan

#### Change 001: GTS Datasource Schema
- **Type**: gts
- **Files**: 
  - `modules/analytics/gts/types/datasource/v1/base.schema.json`
- **Description**: Define GTS schema for datasource with query_id, params, render_options
- **Dependencies**: None (foundational)
- **Effort**: 1 hour (AI agent)
- **Validation**: JSON Schema validation, sample instances

#### Change 002: Database Schema
- **Type**: database
- **Files**: 
  - `modules/analytics/migrations/001_create_datasources.sql`
- **Description**: Create datasources table with tenant isolation, indexes
- **Dependencies**: Change 001
- **Effort**: 0.5 hours (AI agent)
- **Validation**: Migration tests, constraint validation

#### Change 003: Datasource CRUD API
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/api/rest/datasources/handlers.rs`
  - `modules/analytics/src/domain/datasources/repository.rs`
- **Description**: Implement datasource CRUD via GTS API
- **Dependencies**: Change 002
- **Effort**: 2 hours (AI agent)
- **Validation**: API tests, integration tests

#### Change 004: Parameter Merging Service
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/datasources/parameter_merger.rs`
- **Description**: Implement parameter priority resolution and merging
- **Dependencies**: Change 001
- **Effort**: 1.5 hours (AI agent)
- **Validation**: Unit tests with all priority combinations

#### Change 005: Render Options Validator
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/datasources/render_options_validator.rs`
- **Description**: Validate render_options against query capabilities
- **Dependencies**: Change 001, feature-query-execution
- **Effort**: 1 hour (AI agent)
- **Validation**: Validation tests with various configurations

#### Change 006: UI Controls Renderer
- **Type**: typescript
- **Files**: 
  - `ui/src/features/datasources/ControlsRenderer.tsx`
- **Description**: Generate UI controls from render_options
- **Dependencies**: Change 001
- **Effort**: 3 hours (AI agent)
- **Validation**: UI tests, visual regression tests

#### Change 007: Values Selector Integration
- **Type**: typescript
- **Files**: 
  - `ui/src/features/datasources/ValuesSelectorIntegration.tsx`
- **Description**: Integrate values selector templates for filter fields
- **Dependencies**: Change 006, feature-values-selector-templates
- **Effort**: 2 hours (AI agent)
- **Validation**: Integration tests with mock templates

#### Change 008: Datasource Preset Manager
- **Type**: react
- **Files**: 
  - `ui/src/features/datasources/PresetManager.tsx`
- **Description**: UI for creating and managing datasource presets
- **Dependencies**: Change 003
- **Effort**: 2 hours (AI agent)
- **Validation**: E2E tests

#### Change 009: OpenAPI Specification
- **Type**: openapi
- **Files**: 
  - `architecture/openapi/v1/api.yaml`
- **Description**: Document datasource endpoints
- **Dependencies**: All previous changes
- **Effort**: 0.5 hours (AI agent)
- **Validation**: Swagger validation

#### Change 010: Integration Testing Suite
- **Type**: rust + typescript (tests)
- **Files**: 
  - `tests/integration/datasources_test.rs`
  - `ui/tests/datasources.test.tsx`
- **Description**: End-to-end datasource lifecycle tests
- **Dependencies**: All previous changes
- **Effort**: 2 hours (AI agent)
- **Validation**: 100% scenario coverage

**Total Effort**: 15 hours (AI agent + OpenSpec)

---

## Dependencies

- **Depends On**: 
  - feature-gts-core (GTS registry)
  - feature-query-execution (query execution)
  - feature-values-selector-templates (UI control templates)
- **Blocks**: 
  - feature-widget-items (widgets use datasources)

---

## References

- Overall Design: `architecture/DESIGN.md` Section 2 (Actors), Section 3 (System Capabilities)
- GTS Types: Datasource schema (to be created in gts/types/datasource/v1/)
- OpenAPI Spec: `architecture/openapi/v1/api.yaml` (datasource endpoints)
- Feature Manifest: `architecture/features/FEATURES.md` (feature-datasources entry)
