# Feature: Widget Items

**Slug**: `feature-widget-items`
**Status**: ⏳ NOT_STARTED
**Dependencies**: [feature-gts-core](../feature-gts-core/), [feature-widget-templates](../feature-widget-templates/), [feature-datasources](../feature-datasources/)

---

## A. Feature Context

### 1. Feature Overview

**Feature**: Widget Items

**Purpose**: Manages widget item instances for data visualizations. Widget items are the concrete instantiations of widget templates bound to datasources, representing the actual visualization components placed in dashboards and reports.

**Scope**:
- Item GTS type management: `gts.hypernetix.hyperspot.ax.item.v1~` (base) + `gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~`
- Widget item DB tables and CRUD operations
- Widget instance lifecycle (create, update, delete)
- Widget state management (configuration, position, size)
- Widget refresh strategies (real-time, polling, manual)
- Datasource + template binding
- Widget-specific indexing and search
- Widget configuration validation

**References to OVERALL DESIGN**:
- **GTS Types**: 
  - `gts.hypernetix.hyperspot.ax.item.v1~` (base item type)
  - `gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~` (widget-specific item type)
  - `gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~` (referenced from feature-widget-templates)
  - `gts.hypernetix.hyperspot.ax.datasource.v1~` (referenced from feature-datasources)
  
- **OpenAPI Endpoints**:
  - `POST /gts` - Create widget item
  - `GET /gts/{id}` - Retrieve widget item
  - `PUT /gts/{id}` - Update widget item
  - `PATCH /gts/{id}` - Partial update widget item
  - `DELETE /gts/{id}` - Delete widget item
  - `GET /gts` - Search/query widget items with OData

- **Service Roles** (from OpenAPI):
  - Analytics Service - Widget item management
  - GTS Registry Service - Type validation and storage

- **User Roles** (from Overall Design):
  - Platform Administrator - Manages widget item lifecycle
  - Dashboard Designer - Creates and configures widget items
  - Business Analyst - Creates widget items for reports
  - End User - Views widget items in dashboards (read-only)

- **Actors**: 
  - Dashboard Designer - Primary actor for widget item creation and configuration
  - Business Analyst - Creates widget items for ad-hoc analysis
  - End User - Consumes widget visualizations
  - Platform Administrator - Manages widget item access and permissions

---

## B. Actor Flows

### Dashboard Designer Flow

**Goal**: Manage widget items in Widget Library

**UI Flow - Widget Library Management**:
1. Navigate to Widget Library UI
2. View list of existing widget items in library
3. Click "Create New Widget" button
4. Select widget template from template catalog
5. Configure widget settings:
   - Set widget title and description
   - Select datasource from available datasources
   - Map datasource fields to widget template configuration schema
   - Set widget refresh strategy (real-time, polling interval, manual)
   - Configure widget-specific parameters
6. Preview widget with live data
7. Save widget item to library
8. **Later**: Copy widget from library to dashboard (handled by dashboard layout feature)

**UI Flow - Edit Widget in Library**:
1. Open Widget Library UI
2. Search/filter for widget item
3. Click "Edit" on widget item
4. Modify widget configuration
5. Preview changes with live data
6. Save updated widget item

**UI Flow - Delete Widget from Library**:
1. Open Widget Library UI
2. Select widget item to delete
3. System checks if widget is used in any dashboards
4. If widget in use, show warning with dashboard references
5. Confirm deletion
6. Remove widget from library

**API Interactions**:
1. `GET /gts?$filter=startswith(type, 'gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~')` - Load widget library items
2. `GET /gts?$filter=startswith(type, 'gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~')` - Load available widget templates
3. `GET /gts?$filter=startswith(type, 'gts.hypernetix.hyperspot.ax.datasource.v1~')` - Load available datasources
4. `GET /gts/{template_id}` - Retrieve template configuration schema
5. `POST /gts` - Create new widget item in library
6. `GET /gts/{widget_id}` - Retrieve widget item for editing
7. `PATCH /gts/{widget_id}` - Update widget configuration
8. `DELETE /gts/{widget_id}` - Delete widget from library

### Business Analyst Flow

**Goal**: Create widget items for reports and ad-hoc analysis

**UI Flow**:
1. Navigate to report builder
2. Add widget to report layout
3. Select query/datasource for data
4. Choose visualization type (chart, table, map)
5. Configure widget parameters (filters, grouping, aggregation)
6. Set export format preferences
7. Preview widget with sample data
8. Save widget item to report

**API Interactions**:
1. `GET /gts?$filter=startswith(type, 'gts.hypernetix.hyperspot.ax.datasource.v1~')` - Load datasources
2. `POST /gts` - Create widget item for report
3. `GET /gts/{widget_id}` - Retrieve widget for preview

### End User Flow

**Goal**: View and interact with widget visualizations on dashboards

**UI Flow**:
1. Open dashboard (widgets are copies from Widget Library, managed by dashboard layout feature)
2. View rendered widgets (automatically loaded)
3. Interact with widget controls (filters, drilldowns)
4. Refresh widget data (if manual refresh enabled)
5. Export widget data/visualization

**Note**: End Users do NOT interact with Widget Library directly. They only see widget instances placed on dashboards by Dashboard Designers. Widget Library is a management tool for designers, not end users.

**API Interactions**:
1. `GET /gts?$filter=startswith(type, 'gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~') and layout_id eq '{layout_id}'` - Load widget instances for dashboard (handled by dashboard layout feature)
2. `GET /gts/{widget_id}` - Retrieve widget configuration for rendering

---

## C. Algorithms

### 1. UI Algorithms

**Algorithm: Render Widget Item**

Input: widget_item_id, security_context  
Output: Rendered widget visualization

1. Load widget item from GTS Registry
2. **IF** widget item not found:
   1. **RETURN** error "Widget not found"
3. Validate user has read access to widget item
4. **IF** access denied:
   1. **RETURN** error "Access denied"
5. Load template definition from template_id reference
6. Load datasource definition from datasource_id reference
7. Validate template configuration against template schema
8. Execute datasource query to retrieve data
9. Apply widget refresh strategy configuration
10. Render widget using template with data
11. **RETURN** rendered widget

**Algorithm: Configure Widget Item**

Input: widget_item_id, configuration_updates, security_context  
Output: Updated widget item

1. Load existing widget item
2. Validate user has write access
3. **IF** access denied:
   1. **RETURN** error "Access denied"
4. Load template configuration schema
5. Validate configuration_updates against schema
6. **IF** validation fails:
   1. **RETURN** validation errors
7. Merge configuration updates with existing configuration
8. Update widget item in GTS Registry
9. **RETURN** updated widget item

**Algorithm: Widget Refresh Strategy**

Input: widget_item, refresh_strategy  
Output: Data refresh trigger

1. **IF** refresh_strategy is "real-time":
   1. Establish WebSocket connection to datasource
   2. Listen for data change events
   3. **FOR EACH** data change event:
      1. Re-execute datasource query
      2. Update widget visualization
2. **ELSE IF** refresh_strategy is "polling":
   1. Read polling_interval from configuration
   2. **WHILE** widget is visible:
      1. Wait for polling_interval seconds
      2. Re-execute datasource query
      3. Update widget visualization
3. **ELSE IF** refresh_strategy is "manual":
   1. Wait for user-triggered refresh event
   2. Re-execute datasource query
   3. Update widget visualization
4. **RETURN** success

---

### 2. Service Algorithms

**Algorithm: Create Widget Item**

Input: widget_item_request, security_context  
Output: Created widget item with ID

1. Validate security_context has widget creation permissions
2. **IF** permission denied:
   1. **RETURN** 403 Forbidden
3. Extract template_id and datasource_id from request
4. Verify template_id exists in GTS Registry
5. **IF** template not found:
   1. **RETURN** 400 Bad Request "Template not found"
6. Verify datasource_id exists in GTS Registry
7. **IF** datasource not found:
   1. **RETURN** 400 Bad Request "Datasource not found"
8. Load template configuration schema
9. Validate widget configuration against schema
10. **IF** validation fails:
    1. **RETURN** 400 Bad Request with validation errors
11. Generate unique widget item ID
12. Set widget item type to `gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~`
13. Store widget item in database with tenant isolation
14. Index widget item for search
15. **RETURN** 201 Created with widget item

**Algorithm: Update Widget Item**

Input: widget_item_id, update_request, security_context  
Output: Updated widget item

1. Load widget item from database by ID and tenant
2. **IF** widget item not found:
   1. **RETURN** 404 Not Found
3. Validate security_context has write access
4. **IF** permission denied:
   1. **RETURN** 403 Forbidden
5. **IF** update includes template_id change:
   1. Verify new template exists
   2. Validate configuration against new template schema
6. **IF** update includes datasource_id change:
   1. Verify new datasource exists
   2. Validate datasource compatibility with template
7. Apply updates to widget item
8. Update widget item in database
9. Reindex widget item for search
10. **RETURN** 200 OK with updated widget item

**Algorithm: Delete Widget Item**

Input: widget_item_id, security_context  
Output: Deletion confirmation

1. Load widget item from database by ID and tenant
2. **IF** widget item not found:
   1. **RETURN** 404 Not Found
3. Validate security_context has delete access
4. **IF** permission denied:
   1. **RETURN** 403 Forbidden
5. Check if widget item is referenced by any layouts
6. **IF** widget item is in use:
   1. **RETURN** 409 Conflict "Widget is referenced by layouts"
7. Delete widget item from database
8. Remove widget item from search index
9. **RETURN** 204 No Content

**Algorithm: Search Widget Items**

Input: odata_query, security_context  
Output: Paginated list of widget items

1. Parse OData query parameters ($filter, $orderby, $top, $skip, $select)
2. Validate security_context has read access
3. Build database query with tenant filter
4. Apply OData filters to query
5. Apply ordering
6. Apply pagination ($top, $skip)
7. Apply field projection ($select)
8. Execute query against database
9. Load related entities (template, datasource) if requested
10. **RETURN** paginated result set with count

---

## D. States

### 1. State Machines (Optional)

**Widget Item Lifecycle States**:

States:
- **DRAFT** - Widget created but not yet published
- **ACTIVE** - Widget published and available for use
- **ARCHIVED** - Widget archived but not deleted
- **ERROR** - Widget configuration error or datasource unavailable

Transitions:
- DRAFT → ACTIVE: Publish widget (validation passes)
- ACTIVE → DRAFT: Unpublish widget for editing
- ACTIVE → ARCHIVED: Archive widget (remove from active use)
- ARCHIVED → ACTIVE: Restore archived widget
- ANY → ERROR: Configuration validation fails or datasource error

---

## E. Technical Details

### 1. High-Level DB Schema

**Table: widget_items**
```
widget_items (inherits from gts_entities)
├── id (UUID, PK) - Unique widget item identifier
├── tenant_id (UUID, FK) - Tenant isolation
├── type (TEXT) - Always "gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~"
├── template_id (UUID, FK) - Reference to widget template
├── datasource_id (UUID, FK) - Reference to datasource
├── configuration (JSONB) - Widget-specific configuration
├── refresh_strategy (ENUM) - real-time, polling, manual
├── polling_interval (INTEGER) - Seconds between refresh (if polling)
├── state (ENUM) - DRAFT, ACTIVE, ARCHIVED, ERROR
├── position_x (INTEGER) - Optional grid position
├── position_y (INTEGER) - Optional grid position
├── width (INTEGER) - Optional grid width
├── height (INTEGER) - Optional grid height
├── created_at (TIMESTAMP)
├── updated_at (TIMESTAMP)
├── created_by (UUID, FK users)
├── updated_by (UUID, FK users)
└── metadata (JSONB) - Additional metadata

Indexes:
- idx_widget_items_tenant_type (tenant_id, type)
- idx_widget_items_template (template_id)
- idx_widget_items_datasource (datasource_id)
- idx_widget_items_state (state)
- idx_widget_items_configuration (configuration) - GIN index
```

**Relationships**:
- widget_items.template_id → widget_templates.id (many-to-one)
- widget_items.datasource_id → datasources.id (many-to-one)
- layout_items.widget_id → widget_items.id (one-to-many)

---

### 2. Database Operations

**Query Patterns**:

1. **List widget items by tenant**:
```sql
SELECT * FROM widget_items 
WHERE tenant_id = $1 AND type = 'gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~' 
ORDER BY updated_at DESC
LIMIT $2 OFFSET $3
```

2. **Load widget item with template and datasource**:
```sql
SELECT w.*, t.entity as template, d.entity as datasource
FROM widget_items w
JOIN widget_templates t ON w.template_id = t.id
JOIN datasources d ON w.datasource_id = d.id
WHERE w.id = $1 AND w.tenant_id = $2
```

3. **Search widget items by configuration**:
```sql
SELECT * FROM widget_items
WHERE tenant_id = $1 
  AND configuration @> $2::jsonb
ORDER BY created_at DESC
```

4. **Find widgets by template**:
```sql
SELECT * FROM widget_items
WHERE tenant_id = $1 AND template_id = $2
```

---

### 3. Access Control

**SecurityCtx Usage**:
- All widget item operations require authenticated user with tenant context
- Tenant isolation enforced via tenant_id foreign key
- Widget items are tenant-scoped - no cross-tenant access
- Permission checks:
  - `analytics:widget_create` - Create widget items
  - `analytics:widget_read` - Read widget items
  - `analytics:widget_update` - Update widget items
  - `analytics:widget_delete` - Delete widget items

**Row-Level Security**:
```sql
CREATE POLICY widget_items_tenant_isolation ON widget_items
FOR ALL
USING (tenant_id = current_setting('app.tenant_id')::uuid);
```

---

### 4. Error Handling

**Error Scenarios**:

1. **Template not found**:
   - Return 400 Bad Request
   - Message: "Template {template_id} not found"

2. **Datasource not found**:
   - Return 400 Bad Request
   - Message: "Datasource {datasource_id} not found"

3. **Configuration validation error**:
   - Return 400 Bad Request
   - Message: Validation error details from schema

4. **Widget in use (delete attempt)**:
   - Return 409 Conflict
   - Message: "Widget is referenced by {count} layouts"

5. **Access denied**:
   - Return 403 Forbidden
   - Message: "Insufficient permissions"

6. **Tenant isolation violation**:
   - Return 404 Not Found (don't reveal existence)

**Fallback Logic**:
- If datasource unavailable: Show last cached data with warning
- If template missing: Show placeholder widget with error state
- If configuration invalid: Show configuration error in widget

---

## F. Validation & Implementation

### 1. Testing Scenarios

**Unit Tests**:
- Widget item CRUD operations
- Configuration validation against template schema
- Template and datasource reference validation
- Tenant isolation enforcement
- Permission checks
- State transitions
- Search and filtering

**Integration Tests**:
- End-to-end widget creation flow
- Widget rendering with template and datasource
- Widget refresh strategies (real-time, polling, manual)
- Widget item deletion with layout references
- OData query operations on widget items
- Multi-tenant widget isolation

**Edge Cases**:
- Widget with invalid template reference
- Widget with invalid datasource reference
- Widget with malformed configuration
- Concurrent updates to same widget
- Widget deletion while in use by layouts
- Cross-tenant access attempts

---

### 2. OpenSpec Changes Plan

**Total Changes**: 5
**Estimated Effort**: 12 hours (with AI agent)

---

### Change 001: Widget Item DB Schema & Types

**Status**: ⏳ NOT_STARTED

**Scope**: Implement widget item database tables, GTS type definitions, and base entity model

**Tasks**:
- [ ] Create `widget_items` table with proper schema
- [ ] Define GTS types: `gts.hypernetix.hyperspot.ax.item.v1~`, `gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~`
- [ ] Implement entity model with validation
- [ ] Create database indexes for performance
- [ ] Set up row-level security policies

**Files**:
- Backend: `modules/analytics/src/domain/widget_items/mod.rs`
- Backend: `modules/analytics/src/domain/widget_items/entity.rs`
- Backend: `modules/analytics/src/domain/widget_items/schema.rs`
- Migrations: `modules/analytics/migrations/XXX_create_widget_items.sql`

**Dependencies**: None (foundational)

**Effort**: 3 hours (AI agent)

---

### Change 002: Widget Item CRUD Operations

**Status**: ⏳ NOT_STARTED

**Scope**: Implement create, read, update, delete operations for widget items via GTS API

**Tasks**:
- [ ] Implement widget item creation with validation
- [ ] Implement widget item retrieval (single and list)
- [ ] Implement widget item update (full and partial)
- [ ] Implement widget item deletion with reference checks
- [ ] Add template and datasource reference validation
- [ ] Implement configuration schema validation

**Files**:
- Backend: `modules/analytics/src/domain/widget_items/repository.rs`
- Backend: `modules/analytics/src/domain/widget_items/service.rs`
- Backend: `modules/analytics/src/api/gts_registry.rs`
- Tests: `modules/analytics/tests/widget_items_crud.rs`

**Dependencies**: Change 001

**Effort**: 4 hours (AI agent)

---

### Change 003: Widget Item Search & Filtering

**Status**: ⏳ NOT_STARTED

**Scope**: Implement OData v4 query capabilities for widget items

**Tasks**:
- [ ] Implement OData $filter support
- [ ] Implement OData $orderby support
- [ ] Implement OData $top and $skip pagination
- [ ] Implement OData $select field projection
- [ ] Add full-text search on widget metadata
- [ ] Optimize search queries with indexes

**Files**:
- Backend: `modules/analytics/src/domain/widget_items/query.rs`
- Backend: `modules/analytics/src/odata/widget_items_handler.rs`
- Tests: `modules/analytics/tests/widget_items_search.rs`

**Dependencies**: Change 002

**Effort**: 2 hours (AI agent)

---

### Change 004: Widget Refresh Strategies

**Status**: ⏳ NOT_STARTED

**Scope**: Implement widget refresh strategies (real-time, polling, manual)

**Tasks**:
- [ ] Implement refresh strategy configuration
- [ ] Add polling interval management
- [ ] Create refresh scheduler for polling widgets
- [ ] Add WebSocket support for real-time updates
- [ ] Implement manual refresh API endpoint
- [ ] Add refresh rate limiting

**Files**:
- Backend: `modules/analytics/src/domain/widget_items/refresh.rs`
- Backend: `modules/analytics/src/websocket/widget_updates.rs`
- Tests: `modules/analytics/tests/widget_refresh.rs`

**Dependencies**: Change 002

**Effort**: 2 hours (AI agent)

---

### Change 005: Integration Tests & Documentation

**Status**: ⏳ NOT_STARTED

**Scope**: Comprehensive integration tests and API documentation

**Tasks**:
- [ ] End-to-end widget lifecycle tests
- [ ] Multi-tenant isolation tests
- [ ] Widget rendering integration tests
- [ ] Performance tests for large widget collections
- [ ] OpenAPI specification updates
- [ ] API usage examples

**Files**:
- Tests: `modules/analytics/tests/integration/widget_items_e2e.rs`
- Tests: `modules/analytics/tests/integration/widget_items_performance.rs`
- Docs: `modules/analytics/architecture/openapi/v1/api.yaml`
- Docs: `modules/analytics/architecture/features/feature-widget-items/EXAMPLES.md`

**Dependencies**: Change 001, 002, 003, 004

**Effort**: 1 hour (AI agent)

---

**Status Legend**:
- ⏳ **NOT_STARTED** - Change not yet started
- 🔄 **IN_PROGRESS** - Change currently being implemented
- ✅ **COMPLETED** - Change implemented and archived
