# Feature: Widget Templates

**Status**: NOT_STARTED  
**Feature Slug**: `feature-widget-templates`

---

## A. Feature Context

### Overview

Widget visualization templates (charts, tables, maps) with JavaScript bundle management. Provides template registration, bundle upload/download, and template rendering lifecycle.

**Purpose**: Enable pluggable, reusable visualization templates with dynamic JavaScript bundle loading.

**Scope**:
- Template GTS type: `template.v1~` (base) + `template.v1~widget.v1~`
- Widget template DB tables
- JavaScript bundle upload/download (`/templates/{id}/bundle`)
- Chart type library (line, bar, pie, scatter, heatmap)
- Template configuration schemas
- Custom template search
- Template rendering lifecycle
- Bundle versioning and caching

**Out of Scope**:
- Values selector templates - handled by feature-values-selector-templates
- Widget instances - handled by feature-widget-items
- Datasource configuration - handled by feature-datasources

### GTS Types

This feature owns:
- **`gts://gts.hypernetix.hyperspot.ax.template.v1~`** - Base template type
- **`gts://gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~`** - Widget template specialization

**Template Type Hierarchy** (via GTS inheritance):
- Base: `gts.hypernetix.hyperspot.ax.template.v1~`
  - Widget: `gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~`
  - Values Selector: `gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.values_selector.v1~`

References from `gts/types/`:
- [base.schema.json](../../../gts/types/template/v1/base.schema.json) - Base template type
- [widget.schema.json](../../../gts/types/template/v1/widget.schema.json) - Widget template specialization
- [values_selector.schema.json](../../../gts/types/template/v1/values_selector.schema.json) - Values selector template

### OpenAPI Endpoints

From `architecture/openapi/v1/api.yaml`:
- `POST /api/analytics/v1/gts` - Register template instance
- `GET /api/analytics/v1/gts` - List/search templates with OData
- `GET /api/analytics/v1/gts/{template-id}` - Get template metadata
- `PATCH /api/analytics/v1/gts/{template-id}` - Update template metadata
- `DELETE /api/analytics/v1/gts/{template-id}` - Soft-delete template
- `POST /api/analytics/v1/templates/{template-id}/bundle` - Upload JavaScript bundle
- `GET /api/analytics/v1/templates/{template-id}/bundle` - Download JavaScript bundle
- `PUT /api/analytics/v1/gts/{template-id}/enablement` - Configure tenant access

### Actors

**Human Actors** (from Overall Design):
- **Developer** - Creates and registers widget templates
- **Admin** - Manages template sharing and enablement
- **End User** - Uses templates indirectly through widgets

**System Actors**:
- **Template Loader** - Downloads and caches template bundles
- **Widget Renderer** - Initializes and renders templates
- **Bundle Storage** - Stores JavaScript assets

**Service Roles** (from OpenAPI):
- `analytics:templates:read` - View templates
- `analytics:templates:write` - Create/update templates
- `analytics:templates:delete` - Delete templates
- `analytics:templates:bundle` - Upload/download bundles

---

## B. Actor Flows

### Two-Step Registration Process

### Step 1: Register Template Instance

```
POST /api/analytics/v1/gts
Type: gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~
Instance: gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~acme.charts._.line_chart.v2
Key fields:
  - name, description
  - config_schema_id → template configuration schema
  - query_returns_schema_id → expected data schema
  - category_id
```

### Step 2: Upload JavaScript Bundle

```
POST /api/analytics/v1/templates/{template-id}/bundle  # Upload JS bundle, returns bundle_url and checksum
```

**Bundle Replacement**:
- Bundles can be uploaded **multiple times** for the same template
- Each upload **replaces** the previous bundle
- UI cache is invalidated on replacement (via ETag/Last-Modified headers)
- Template metadata remains unchanged

---

### Flow 1: Developer Registers Custom Widget Template

**Actor**: Developer  
**Trigger**: Need to create custom visualization  
**Goal**: Register template with bundle for use in dashboards

**Steps**:
1. **Register Instance** - Create GTS template metadata with contract definition
2. **Upload Bundle** - POST JavaScript implementation to `/templates/{id}/bundle`
3. **Verification** - Service validates bundle syntax and security
4. **Storage** - Bundle stored with template ID mapping
5. **UI Loading** - UI fetches bundle via GET `/templates/{id}/bundle.js`
6. **Caching** - UI caches bundle with proper cache headers (ETag, max-age)

**API Interaction**:
```
POST /api/analytics/v1/gts
Type: gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~
Instance: gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~acme.charts._.line_chart.v2
Body: {
  name, description,
  config_schema_id,
  query_returns_schema_id,
  category_id
}
→ Returns: { id, type, registered_at }

POST /api/analytics/v1/templates/{template-id}/bundle
Content-Type: application/javascript
Body: <JavaScript bundle>
→ Returns: { bundle_url, checksum }
```

---

### Flow 2: Developer Updates Template Bundle

**Actor**: Developer  
**Trigger**: Bug fix or feature enhancement  
**Goal**: Replace template bundle without changing metadata

**Steps**:
1. Fix bugs or add features to existing template
2. Test changes locally with mock data
3. Bundle updated template assets
4. Upload new bundle version
5. New bundle replaces previous version
6. UI cache invalidated (users get new version)

**API Interaction**:
```
POST /api/analytics/v1/templates/{template-id}/bundle
→ Replaces previous bundle
→ UI cache invalidated via ETag/Last-Modified headers
→ Template metadata remains unchanged
```

---

### Flow 3: Admin Shares Template with Tenants

**Actor**: Admin  
**Trigger**: Template ready for distribution  
**Goal**: Enable specific tenants to use template

**API Interaction**:
```
PUT /api/analytics/v1/gts/{template-id}/enablement
Body: { "enabled_for": ["tenant-1", "tenant-2"] }
  or { "enabled_for": "all" }
→ Specified tenants can now see and use template
```

---

### Flow 4: Widget Renderer Loads and Initializes Template

**Actor**: Widget Renderer (System)  
**Trigger**: Dashboard load with widgets  
**Goal**: Display widget with template visualization

**Steps**:
1. **Widget Initialization** - Parse layout items, read widget settings
2. **Template Bundle Loading** - Download JavaScript bundle (cached)
3. **Data Fetching** - Execute query for widget data
4. **Initial Render** - Call template.render() with data
5. **Update Cycle** - Handle data refresh and config updates
6. **Widget Cleanup** - Call template.destroy() on removal

---

### Flow 5: Developer Edits Template Metadata

**Actor**: Developer  
**Trigger**: Need to update template description or schema refs  
**Goal**: Update template metadata without changing bundle

**API Interaction**:
```
GET /api/analytics/v1/gts/{template-id}
→ Load current metadata

PATCH /api/analytics/v1/gts/{template-id}
Body: JSON Patch operations
→ Can update: name, description, category_id, schema references
```

---

## C. Algorithms

### Template Bundle Structure

Each template is a JavaScript module (ESM) uploaded to `/templates/{template_id}/bundle` that exports:

### init(container, config)

Initialize template instance

- **Parameters:**
  - `container`: DOM element to render into
  - `config`: Template configuration from widget settings
- **Returns:** template instance object

**Purpose:** Initialize template instance with initial configuration

### render(instance, data)

Render/update visualization with data and config

- **Parameters:**
  - `instance`: Object returned by init() (contains config and visualization state)
  - `data`: Query result from datasource
- **Purpose:** Template applies config (colors, axes, legends) and renders data
- **Behavior:** Idempotent - can be called multiple times with different data

### updateConfig(instance, newConfig) *(optional)*

Update configuration without full re-initialization

- **Parameters:**
  - `instance`: Template instance from init()
  - `newConfig`: Updated template settings
- **Purpose:** Change template configuration dynamically
- **Optional:** Not all templates need dynamic config updates

### destroy(instance) *(optional)*

Cleanup before removal

- **Parameters:**
  - `instance`: Template instance to clean up
- **Purpose:** Clean up event listeners, timers, resources
- **Optional:** Only needed if template has event listeners, timers, WebSocket connections

### renderSettings(container, currentConfig, onChange)

Render settings UI for template configuration

- **Parameters:**
  - `container`: DOM element for settings form
  - `currentConfig`: Current template configuration
  - `onChange(newConfig)`: Callback to update config
- **Purpose:** Template must provide UI for all its configuration options
- **Required:** Platform cannot auto-generate settings UI, template owns it

---

### Service Algorithm 1: Widget Rendering Lifecycle

### 1. Widget Initialization

```
Dashboard Load → Parse Layout Items → For Each Widget:
  - Read widget.settings.template.id
  - Read widget.settings.template.config
  - Read widget.settings.datasource
  - Compute absolute dimensions (via Item Preview Rendering algorithm)
  - Create DOM container with computed width/height
```

### 2. Template Bundle Loading

```
GET /api/analytics/v1/templates/{template_id}/bundle
  → Download JavaScript bundle (cached in browser)
  → Import as ESM module
  → Call template.init(container, config)
  → Store returned instance for this widget
```

### 3. Data Fetching

```
Extract datasource configuration
  → Build query URL: /api/analytics/v1/queries/{query_id}
  → Apply OData params from datasource.params
  → Execute query request
  → Receive data response
```

### 4. Initial Render

```
Call template.render(instance, data)
  → Template applies config (colors, axes, legends, etc.) and renders data
  → Updates DOM inside container
  → Widget visible to user
```

### 5. Update Cycle

**Data Refresh:**
```
Re-execute query (manual refresh, auto-refresh timer, filter change)
  → Receive new data
  → Call template.render(instance, newData)
  → Template updates DOM (smooth transitions)
```

**Config Update:**
```
User changes template settings (colors, axes, etc.)
  → Call template.updateConfig(instance, newConfig)
  → Template re-renders with new configuration
```

### 6. Widget Cleanup

```
Widget removed from dashboard or layout changed
  → Call template.destroy(instance)
  → Remove DOM container
  → Clear instance reference
```

---

### Service Algorithm 2: Template API Contract

### init(container, config)
- Initialize template instance with initial configuration
- Returns opaque instance object (stores template state)

### render(instance, data)
- Render/update visualization with new data using stored config
- Applies config settings (colors, axes, legends) and renders data
- Idempotent - can be called multiple times with different data

### updateConfig(instance, newConfig)
- Update template configuration without full re-initialization
- Optional - not all templates need dynamic config updates

### destroy(instance)
- Cleanup resources before widget removal
- Optional - only needed if template has event listeners, timers, WebSocket connections

### renderSettings(container, currentConfig, onChange)
- Render UI for template configuration in settings dialog
- Required - template must provide settings UI, platform cannot auto-generate
- Template owns its configuration UI (color pickers, chart options, validators)

---

## D. States

*(Not applicable - templates are stateless JavaScript modules)*

---

## E. Technical Details

### Template Asset Requirements

- Template must export standard interface with required methods
- Must support rendering with config and data
- Must provide config editor for user customization
- Must validate config against schema
- Must support resource cleanup

**Key Points**:
- **Self-contained bundle** - All dependencies must be bundled in the JavaScript asset
- **Schema references** - Both config schema and datasource schema must exist before registration
- **Config schema** - Defines structure of template configuration
- **Datasource schema** - Defines expected data format from datasource
- **Dynamic loading** - Templates loaded on-demand in browser
- **Version management** - Multiple versions can coexist via GTS identifiers
- **Security** - Checksum verification, sandboxed execution

### Error Handling

**Bundle Load Failure:** 404 or network error → Show error placeholder with retry action

**Data Fetch Failure:** Query error → Call template.render(instance, null) or show error state

**Render Exception:** Catch exception → Display error overlay, keep container intact

---

### Caching Strategy

- **Template Bundles:** Cached by browser (ETag + Last-Modified headers)
- **Query Data:** Cached per query+params+tenant (configurable TTL)
- **Bundle Versioning:** Template ID includes version → cache invalidation on update

**Cache Hit Flow:**
1. Check browser cache for bundle (ETag match)
2. If valid → use cached bundle
3. If invalid → download new bundle, update cache

**Cache Miss Flow:**
1. Download bundle from `/templates/{id}/bundle`
2. Store in browser cache with ETag
3. Import and initialize template

---

### Access Control

**SecurityCtx Enforcement**:
- All template operations require authenticated user
- Template ownership validated via `created_by` field
- Tenant isolation enforced on all queries
- Bundle upload restricted to template owner or admin

**Permission Checks**:
- Template registration: Requires `analytics:templates:write`
- Bundle upload: Requires `analytics:templates:bundle` + ownership verification
- Template enablement: Requires `analytics:admin`

---

### Database Operations

**Tables**:
- `widget_templates` - Template metadata
- `template_bundles` - JavaScript assets with checksums
- `template_enablement` - Tenant access control

**Indexes**:
- `idx_templates_tenant_type` - Fast template listing per tenant
- `idx_templates_category` - Category-based browsing
- `idx_bundles_template` - Bundle lookup by template ID

**Queries**:
```sql
-- List templates for tenant
SELECT * FROM widget_templates
WHERE tenant_id = $1 AND deleted_at IS NULL
ORDER BY name;

-- Get template with bundle info
SELECT t.*, b.bundle_url, b.checksum
FROM widget_templates t
LEFT JOIN template_bundles b ON t.id = b.template_id
WHERE t.id = $1 AND t.tenant_id = $2;
```

---

## F. Validation & Implementation

### Testing Scenarios

**Unit Tests**:
- Template metadata CRUD operations
- Bundle upload/download logic
- Checksum validation
- Cache header generation
- Permission checks

**Integration Tests**:
- End-to-end template registration flow
- Bundle replacement and cache invalidation
- Template enablement across tenants
- Widget rendering with template loading

**Browser Tests**:
- Template bundle loading and caching
- Template initialization and rendering
- Config updates without re-initialization
- Error handling for missing bundles
- Memory leak detection on template destroy

**Security Tests**:
- Bundle syntax validation
- XSS prevention in template code
- Sandboxed execution verification
- Unauthorized bundle upload prevention

**Performance Tests**:
- Bundle download speed (< 100ms for cached)
- Template initialization time (< 50ms)
- Render performance with large datasets
- Cache hit rate (> 90%)

**Edge Cases**:
1. Upload bundle before template registration
2. Delete template with active widgets
3. Upload corrupted JavaScript bundle
4. Template init() throws exception
5. Concurrent bundle uploads to same template
6. Template references non-existent schema

---

### OpenSpec Changes Plan

#### Change 001: GTS Template Type Definition
- **Type**: gts
- **Files**: 
  - [base.schema.json](../../../gts/types/template/v1/base.schema.json)
  - [widget.schema.json](../../../gts/types/template/v1/widget.schema.json)
- **Description**: Define GTS schema for widget template with config and data schema references
- **Dependencies**: None (foundational)
- **Effort**: 0.5 hours (AI agent)
- **Validation**: JSON Schema validation, sample instances

#### Change 002: Database Schema
- **Type**: database
- **Files**: 
  - `modules/analytics/migrations/001_create_widget_templates.sql`
- **Description**: Create tables: widget_templates, template_bundles, template_enablement
- **Dependencies**: Change 001
- **Effort**: 1 hour (AI agent)
- **Validation**: Migration tests, constraint validation

#### Change 003: Template Metadata CRUD
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/api/rest/widget_templates/handlers.rs`
  - `modules/analytics/src/domain/widget_templates/repository.rs`
- **Description**: Implement CRUD operations for template metadata via GTS API
- **Dependencies**: Change 002
- **Effort**: 2 hours (AI agent)
- **Validation**: Unit tests, integration tests

#### Change 004: Bundle Upload/Download API
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/api/rest/widget_templates/bundle_handler.rs`
  - `modules/analytics/src/infra/storage/template_bundles.rs`
- **Description**: Implement POST/GET endpoints for JavaScript bundle management
- **Dependencies**: Change 003
- **Effort**: 3 hours (AI agent)
- **Validation**: Integration tests, file storage tests

#### Change 005: Bundle Validation & Security
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/widget_templates/validator.rs`
- **Description**: Validate JavaScript syntax, checksum verification, security checks
- **Dependencies**: Change 004
- **Effort**: 2 hours (AI agent)
- **Validation**: Security tests, malformed bundle tests

#### Change 006: Caching Headers
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/widget_templates/cache.rs`
- **Description**: Generate ETag, Last-Modified, Cache-Control headers
- **Dependencies**: Change 004
- **Effort**: 1 hour (AI agent)
- **Validation**: HTTP caching tests

#### Change 007: Template Enablement API
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/widget_templates/enablement.rs`
- **Description**: Manage tenant access to templates
- **Dependencies**: Change 003
- **Effort**: 1.5 hours (AI agent)
- **Validation**: Multi-tenant access tests

#### Change 008: OpenAPI Specification
- **Type**: openapi
- **Files**: 
  - `architecture/openapi/v1/api.yaml`
- **Description**: Document all template and bundle endpoints
- **Dependencies**: All previous changes
- **Effort**: 0.5 hours (AI agent)
- **Validation**: Swagger validation

#### Change 009: Integration Testing Suite
- **Type**: rust (tests)
- **Files**: 
  - `tests/integration/widget_templates_test.rs`
- **Description**: End-to-end tests for template lifecycle
- **Dependencies**: All previous changes
- **Effort**: 2 hours (AI agent)
- **Validation**: 100% scenario coverage

#### Change 010: Documentation & Examples
- **Type**: documentation
- **Files**: 
  - `modules/analytics/docs/widget_templates.md`
  - `examples/template_hello_world.js`
- **Description**: Developer guide and example template
- **Dependencies**: All previous changes
- **Effort**: 1.5 hours (AI agent)
- **Validation**: Example template runs successfully

**Total Effort**: 15 hours (AI agent + OpenSpec)

---

## Dependencies

- **Depends On**: 
  - feature-gts-core (routing and GTS registry)
  - feature-schema-template-config (configuration schemas)
- **Blocks**: 
  - feature-widget-items (widgets need templates)

---

## References

- Overall Design: `architecture/DESIGN.md` Section 2 (Actors), Section 3 (System Capabilities)
- GTS Types: [base.schema.json](../../../gts/types/template/v1/base.schema.json), [widget.schema.json](../../../gts/types/template/v1/widget.schema.json), [values_selector.schema.json](../../../gts/types/template/v1/values_selector.schema.json)
- OpenAPI Spec: `architecture/openapi/v1/api.yaml` (template endpoints)
- Feature Manifest: `architecture/features/FEATURES.md` (feature-widget-templates entry)
