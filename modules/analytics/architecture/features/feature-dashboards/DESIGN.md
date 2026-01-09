# Feature: Dashboards

**Status**: NOT_STARTED  
**Feature Slug**: `feature-dashboards`

---

## A. Feature Context

### Overview

Dashboard UI management and business logic - grid layout, drag-and-drop, templates, version history. Handles all dashboard-related user scenarios and widget configuration workflows.

**Purpose**: Provide complete dashboard UI/UX layer with widget configuration, drag-and-drop, and business logic.

**Scope**:
- Dashboard CRUD operations (business logic layer)
- Grid-based responsive layouts
- Drag-and-drop widget positioning
- Dashboard templates
- Version history
- Dashboard-specific business logic (NOT layout storage)
- Widget settings UI rendering
- User scenarios and workflows
- Widget/group management workflows
- Widget preset creation and management
- Group creation and nesting

**Out of Scope**:
- Dashboard layout storage - handled by feature-dashboard-layouts
- Widget templates - handled by feature-widget-templates
- Datasource configuration - handled by feature-datasources
- Query execution - handled by feature-query-execution
- Report generation - handled by feature-reporting

### GTS Types

This feature **uses but does not own** GTS types:

**Uses types from**:
- `gts://gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.dashboard.v1~*` - Dashboard layout instances
- `gts://gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~*` - Widget items
- `gts://gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.group.v1~*` - Group items
- `gts://gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~*` - Widget templates
- `gts://gts.hypernetix.hyperspot.ax.query.v1~*` - Query definitions
- `gts://gts.hypernetix.hyperspot.ax.datasource.v1~*` - Datasource configurations

References from `gts/types/`:
- Dashboard layout schemas (owned by feature-dashboard-layouts)
- Item schemas (owned by feature-widget-items, feature-group-items)

### OpenAPI Endpoints

From `architecture/openapi/v1/api.yaml`:
- `GET /api/analytics/v1/gts` - List/search dashboards and items with OData
- `POST /api/analytics/v1/gts` - Create dashboard instance
- `GET /api/analytics/v1/gts/{dashboard-id}` - Get dashboard with layout
- `PATCH /api/analytics/v1/gts/{dashboard-id}` - Update dashboard (add/remove/move widgets)
- `DELETE /api/analytics/v1/gts/{dashboard-id}` - Soft-delete dashboard
- `PUT /api/analytics/v1/gts/{dashboard-id}/enablement` - Share dashboard with tenants
- `GET /api/analytics/v1/templates/{template-id}/bundle` - Load template bundles
- `GET /api/analytics/v1/queries/{query-id}` - Preview widget data

### Actors

**Human Actors** (from Overall Design):
- **Dashboard Creator** - Creates and configures dashboards
- **Widget Designer** - Adds and configures widgets
- **End User** - Views and interacts with dashboards
- **Admin** - Manages dashboard sharing and permissions

**System Actors**:
- **Dashboard Manager** - Orchestrates dashboard CRUD operations
- **Widget Configurator** - Handles widget settings and validation
- **Grid Layout Engine** - Manages responsive grid positioning
- **Template Loader** - Loads widget template bundles

**Service Roles** (from OpenAPI):
- `analytics:dashboards:read` - View dashboards
- `analytics:dashboards:write` - Create/edit dashboards
- `analytics:dashboards:delete` - Delete dashboards
- `analytics:widgets:configure` - Configure widget settings

---

## B. Actor Flows

### Flow 1: Dashboard Creator Creates New Dashboard

When user opens widget settings dialog, the UI renders two types of configuration:
1. **Platform-managed settings** - standard widget properties (rendered by platform)
2. **Template-specific settings** - custom configuration (rendered by template bundle)

### Platform-Managed Settings

#### 1. Item Properties
Name, description, icon, size (width %, height preset)

#### 2. Template Selection
Searchable dropdown filtered by query compatibility

#### 3. Datasource Configuration
- Query selection
- OData parameters ($filter, $orderby, $top, $skip, $select, $expand, $search)
- Render options (filters, sorting, pagination, time range, search, grouping)

### OData Capabilities + Render Options Integration

The platform combines **OData metadata capabilities** (from query.capabilities_id) with **datasource.render_options** to render appropriate UI controls.

**Two-layer system:**

**1. OData Capabilities** - Define what query *technically supports*:
- FilterFunctions, SortRestrictions, SearchRestrictions, SelectSupport, ExpandRestrictions, TopSupported, SkipSupported

**2. Render Options** - Define what UI *should show to user*:
- Which filters are exposed, sort options, pagination config, time range controls, search visibility

**UI Rendering Logic:**
```
For each render option:
  1. Check if query capabilities support the feature
  2. Check if datasource render_options enable the feature
  3. If BOTH true → render UI control
  4. If capabilities missing → disable/hide control
  5. If render_options disabled → hide control
```

**Actor**: Dashboard Creator  
**Trigger**: Need new dashboard for analytics  
**Goal**: Create empty dashboard ready for widgets

**Steps**:
1. Navigate to Dashboards → Create New
2. Enter dashboard metadata (name, description, icon)
3. Select category
4. Configure auto-refresh settings
5. Configure sharing (optional): private, specific tenants, or all
6. Save dashboard

**API Interaction**:
```
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.dashboard.v1~')
POST /api/analytics/v1/gts
Type: gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.dashboard.v1~
Instance: gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.dashboard.v1~acme.sales._.executive_dashboard.v1

PUT /api/analytics/v1/gts/{dashboard-id}/enablement (optional)
Body: { "enabled_for": ["tenant-1", "tenant-2"] } or { "enabled_for": "all" }
```

---

### Flow 2: Widget Designer Adds Widget to Dashboard

**Actor**: Widget Designer  
**Trigger**: Dashboard needs data visualization  
**Goal**: Add configured widget to dashboard

**Steps**:
1. Open existing dashboard (edit permissions)
2. Click "Add Widget"
3. Choose: Select preset OR Create custom
4. If preset: Browse and select widget instance by category
5. If custom:
   - Select query
   - Configure datasource (OData params, render_options)
   - Select compatible template
   - Configure template settings (colors, axes, legends)
   - Configure item properties (name, size, icon)
6. Preview widget with live data
7. Add to dashboard grid

**API Interaction**:
```
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~')
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query.v1~')
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~')
GET /api/analytics/v1/templates/{template-id}/bundle
GET /api/analytics/v1/queries/{query-id}?$filter=...&$top=50 (preview)

PATCH /api/analytics/v1/gts/{dashboard-id}
JSON Patch: Add widget to dashboard/entity/items
```

---

### Flow 3: End User Views and Interacts with Dashboard

**Actor**: End User  
**Trigger**: Need to monitor business metrics  
**Goal**: View widgets with live data and interact with controls

**Steps**:
1. Login to Analytics Portal
2. Navigate to Dashboards list
3. Search/browse for dashboard
4. Click to open
5. View widgets with auto-refresh
6. Interact with widget controls (filters, search, sorting, pagination)
7. Drill down on data points
8. Export data if needed

**API Interaction**:
```
GET /api/analytics/v1/gts?$filter=...&$select=...
GET /api/analytics/v1/gts/{dashboard-id}
GET /api/analytics/v1/templates/{template-id}/bundle.js
GET /api/analytics/v1/queries/{query-id}?$filter=...&$orderby=...&$top=...
```

---

### Flow 4: Widget Designer Edits Widget Settings

**Actor**: Widget Designer  
**Trigger**: Widget needs configuration changes  
**Goal**: Update widget datasource or template settings

**Steps**:
1. Open dashboard (edit permissions)
2. Click widget settings/gear icon
3. Edit platform-managed settings OR template-specific settings
4. Changes apply immediately (live preview)
5. Save automatically

**API Interaction**:
```
GET /api/analytics/v1/gts/{dashboard-id}
PATCH /api/analytics/v1/gts/{dashboard-id}
JSON Patch: Update dashboard/entity/items[n]/settings
```

---

### Flow 5: Dashboard Creator Creates Widget Preset

**Actor**: Dashboard Creator  
**Trigger**: Reusable widget configuration needed  
**Goal**: Save widget as preset for library

**Steps**:
1. Configure widget fully (datasource, template, properties)
2. Click "Save as Preset"
3. Enter preset metadata (name, description, category)
4. Configure sharing: private, specific tenants, or all
5. Save to GTS registry
6. Preset appears in widget library

**API Interaction**:
```
POST /api/analytics/v1/gts
Type: gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~
Instance: Full widget configuration

PUT /api/analytics/v1/gts/{widget-preset-id}/enablement
Body: { "enabled_for": ["tenant-1"] } or { "enabled_for": "all" }
```

---

## C. Algorithms

### UI Algorithm 1: Widget Settings Dialog Rendering

**Purpose**: Render appropriate settings UI based on capabilities and render_options

### Settings Dialog Structure

```
Widget Settings Dialog
├─ Tab: General Settings (Platform UI)
│  ├─ Item Properties (name, description, icon, size)
│  ├─ Template Selection (compatibility-filtered dropdown)
│  └─ Datasource Configuration
│     ├─ Query selection
│     ├─ OData parameters
│     └─ Render options (filtered by capabilities)
└─ Tab: Template Settings (Template Bundle UI)
   └─ [Custom UI rendered by template.renderSettings()]
```

### Key Differences

| Aspect | Platform Settings | Template Settings |
|--------|------------------|-------------------|
| **Rendered by** | Platform UI (standard controls) | Template bundle (custom code) |
| **Configuration** | Item properties, datasource, template selection | Template-specific options (colors, axes, legends) |
| **Schema source** | base.schema.json, widget.schema.json | template.config_schema_id |
| **UI generation** | Automatic from known schemas | Manual via renderSettings() |
| **Validation** | Platform validates against schemas | Template validates in onChange() |
| **Changes trigger** | May require template reload | Calls template.updateConfig() |

---

### Service Algorithm 1: Widget Configuration Validation

**Purpose**: Validate widget configuration against schemas and capabilities

**Steps**:

1. Load query from GTS registry
2. Verify query enabled for tenant
3. Load template from GTS registry
4. Validate schema compatibility (query returns vs template expects)
5. Validate OData params against query capabilities
6. **RETURN** validation result
    let capabilities = gts_registry.get(&query.capabilities_id)?;
    validate_odata_params(&widget.datasource.params, &capabilities)?;
    
    // 4. Validate template config against schema
    let config_schema = gts_registry.get(&template.config_schema_id)?;
    validate_json_schema(&widget.template.config, &config_schema)?;
    
    Ok(())
}
```

---

### Service Algorithm 2: Grid Layout Auto-Adjustment

**Purpose**: Automatically adjust widget positions when adding/removing items

**Input**: Dashboard layout, new widget position  
**Output**: Adjusted layout with no overlaps

**Steps**:
1. Calculate new widget position on grid
2. Detect overlaps with existing widgets
3. Shift overlapping widgets down/right
4. Recursively adjust displaced widgets
5. Return final layout

---

## D. States

*(Not applicable - dashboard state is stored in layout, no FSM needed)*

---

## E. Technical Details

### Widget Settings UI Rendering

When user opens widget settings dialog, UI renders:
1. **Platform-managed settings** - Standard widget properties (platform renders)
2. **Template-specific settings** - Custom configuration (template.renderSettings())

**Platform-Managed Settings**:
- Item properties (name, description, icon, size)
- Template selection (compatibility-filtered dropdown)
- Datasource configuration (query, OData params, render_options)

**OData Capabilities + Render Options Integration**:

Two-layer system:
1. **OData Capabilities** - What query technically supports
2. **Render Options** - What UI should show to user

**UI Rendering Logic**:
```
For each render option:
  1. Check if query capabilities support the feature
  2. Check if datasource render_options enable the feature
  3. If BOTH true → render UI control
  4. If capabilities missing → disable/hide control
  5. If render_options disabled → hide control
```

---

### Access Control

**SecurityCtx Enforcement**:
- All dashboard operations require authenticated user
- Tenant isolation enforced on all queries
- Dashboard ownership via `created_by` field
- Widget configuration requires edit permissions

**Permission Checks**:
- Dashboard creation: Requires `analytics:dashboards:write`
- Widget configuration: Requires `analytics:widgets:configure` + ownership verification
- Dashboard sharing: Requires `analytics:admin`

---

### Database Operations

This feature operates on GTS registry - no direct database access.

**Operations via GTS API**:
- Dashboard CRUD via `/gts` endpoints
- Widget configuration via PATCH operations
- Preset creation via POST operations

---

### Error Handling

**Common Errors**:
- **404 Not Found**: Dashboard or widget preset not found
- **400 Bad Request**: Invalid widget configuration
- **403 Forbidden**: Insufficient permissions
- **422 Unprocessable Entity**: Schema validation failure
- **409 Conflict**: Widget position overlap

**Error Response Format (RFC 7807)**:
```json
{
  "type": "https://example.com/problems/widget-config-invalid",
  "title": "Widget Configuration Invalid",
  "status": 422,
  "detail": "Template 'line_chart.v1' incompatible with query schema",
  "instance": "/api/analytics/v1/gts/dashboard-123"
}
```

---

## F. Validation & Implementation

### User Scenarios

*(17 comprehensive user scenarios preserved below)*

### Scenario 1: Create New Dashboard

**UI Flow:**
1. Navigate to Dashboards → Create New
2. Enter dashboard metadata (name, description, icon)
3. Select category
4. Choose layout template or start blank *(future release)*
5. Configure auto-refresh settings
6. Configure sharing (optional):
   - Choose visibility: private, specific tenants, or all tenants
   - Select tenants if sharing with specific tenants
7. Save dashboard

**API Calls:**
```
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.dashboard.v1~')
POST /api/analytics/v1/gts  # Create dashboard instance
  # Type: gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.dashboard.v1~
  # Instance ID: gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.dashboard.v1~acme.sales._.executive_dashboard.v1
PUT /api/analytics/v1/gts/{dashboard_id}/enablement  # Share dashboard with tenants (optional)
  # Body: { "enabled_for": ["tenant-1", "tenant-2"] } - specific tenants
  # Or:   { "enabled_for": "all" } - all tenants
  # NOTE: System automatically enables all referenced entities (widgets, templates, datasources, queries, schemas)
```

---

### Scenario 2: Add Widget to Dashboard

**UI Flow:**
1. Open existing dashboard (user has edit permissions)
2. Click "Add Widget"
3. Choose starting point:
   - **Select preset** (widget instance with pre-configured settings) - pre-fills all configuration
   - **Create custom** (configure from scratch)
4. If preset: browse and select widget instance by category
5. Configure datasource (select query_id, set OData params, optionally configure render_options for UI controls)
6. Select template that fits the data (system suggests compatible templates based on query schema)
7. Configure template settings (data mapping, chart title, colors, axes, legend, tooltips)
8. Iterate steps 5-7 as needed (refine datasource and template configuration)
9. Configure item properties (name, description, icon, size: width %, height preset)
10. Preview widget with live data
11. Add widget to dashboard and set position on grid (defined by dashboard layout)

Note: Configuration is iterative - user can adjust datasource and template settings cyclically until satisfied. Preset pre-fills all fields but allows modifications.

**API Calls:**
```
# Option A: Select widget preset
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~')&$select=...
  # Browse available widget instances (presets) with pre-configured settings

# Option B: Create custom widget
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query.v1~')&$select=...
  # Browse available queries
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.datasource.v1~')&$select=...
  # Browse available datasource presets (optional - can create inline or use preset)
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~')&$select=...
  # Browse widget template instances (metadata) - system suggests compatible templates
GET /api/analytics/v1/templates/{template_id}/bundle
  # Download widget template JavaScript bundle (implementation)
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.values_selector.v1~')&$select=...
  # Browse values selector templates for render_options UI controls
GET /api/analytics/v1/templates/{values_selector_template_id}/bundle
  # Download values selector template JavaScript bundle

# Schemas for validation and compatibility checking
GET /api/analytics/v1/gts/{query_returns_schema_id}
  # Get query returns schema - defines query result structure
GET /api/analytics/v1/gts/{template_config_schema_id}
  # Get template config schema - defines valid template configuration
GET /api/analytics/v1/gts/{values_schema_id}
  # Get values schema - defines filter values structure

# Add widget to dashboard (both options)
PATCH /api/analytics/v1/gts/{dashboard_id}  # Add widget item inline to dashboard/entity/items
  # Widget type: gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~
GET /api/analytics/v1/queries/{query_id}?$filter=...&$orderby=...&$top=...  # Preview data
```

---

### Scenario 3: Move Widget Position

**UI Flow:**
1. Open dashboard (user has edit permissions)
2. Drag widget to new position on grid
3. Drop widget in new location
4. Grid automatically adjusts other widgets if needed
5. Save dashboard layout

**API Calls:**
```
PATCH /api/analytics/v1/gts/{dashboard_id}  # Update dashboard/entity/items[n]/grid_position
  # JSON Patch operation updating grid_position for specific item
```

---

### Scenario 4: Edit Widget Settings

**UI Flow:**
1. Open dashboard
2. Click widget settings/gear icon
3. Edit settings exposed by render_options:
   - Adjust filters (if enabled in datasource render_options)
   - Change sorting (if enabled)
   - Modify pagination (if enabled)
   - Update grouping/aggregation (if enabled)
   - Adjust time range (if enabled)
4. Changes apply immediately (live preview)
5. Settings saved automatically

**API Calls:**
```
PATCH /api/analytics/v1/gts/{dashboard_id}  # Update widget datasource params
  # JSON Patch operation updating dashboard/entity/items[n]/settings/datasource/params
```

---

### Scenario 5: Advanced Widget Editor

**UI Flow:**
1. Open dashboard (user has edit permissions)
2. Right-click widget → "Advanced Edit"
3. Edit datasource configuration:
   - Change query_id
   - Modify OData params ($filter, $orderby, etc.)
   - Configure render_options (filters, sorting, pagination, grouping, time, search)
4. Edit template configuration:
   - Change template_id
   - Modify template config (colors, axes, legends, etc.)
5. Edit item properties (name, description, icon, size: width %, height preset)
6. Preview changes with live data
7. Save or discard changes

**API Calls:**
```
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query.v1~')&$select=...
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.datasource.v1~')&$select=...
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~')&$select=...
GET /api/analytics/v1/templates/{template_id}/bundle
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.values_selector.v1~')&$select=...
GET /api/analytics/v1/templates/{values_selector_template_id}/bundle
GET /api/analytics/v1/gts/{query_returns_schema_id}
GET /api/analytics/v1/gts/{template_config_schema_id}
GET /api/analytics/v1/gts/{values_schema_id}
PATCH /api/analytics/v1/gts/{dashboard_id}  # Update widget settings
GET /api/analytics/v1/queries/{query_id}?...  # Preview data with new params
```

---

### Scenario 6: Add Widget to Group

**UI Flow:**
1. Open dashboard with existing group
2. Drag widget into group container
3. Drop widget inside group
4. Widget becomes child of group
5. Save dashboard

**API Calls:**
```
PATCH /api/analytics/v1/gts/{dashboard_id}  # Move widget into group
  # JSON Patch: remove widget from dashboard/entity/items
  # Add widget to group's settings/items array
```

---

### Scenario 7: Create Group

**UI Flow:**
1. Open dashboard (user has edit permissions)
2. Click "Add Group"
3. Configure group properties:
   - Name, description, icon
   - Size (width: 15-100% multiples of 5, height: micro/small/medium/high/unlimited)
   - Collapsible behavior (enabled/disabled, default state)
4. Add widgets to group (drag and drop)
5. Set group position on dashboard grid
6. Save dashboard

**API Calls:**
```
PATCH /api/analytics/v1/gts/{dashboard_id}  # Add group item to dashboard
  # JSON Patch adding new item with type gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.group.v1~
  # Group contains settings/items array for nested widgets
```

---

### Scenario 8: Create Widget Preset

**UI Flow:**
1. Configure widget fully (datasource, template, item properties)
2. Click "Save as Preset"
3. Enter preset metadata (name, description, category)
4. Configure sharing:
   - Choose visibility: private, specific tenants, or all tenants
   - Select tenants if sharing with specific tenants
5. Save preset as widget instance in GTS registry
6. Preset appears in widget library for reuse

**API Calls:**
```
POST /api/analytics/v1/gts  # Create widget instance (preset)
  # Type: gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~
  # Instance contains full widget configuration
PUT /api/analytics/v1/gts/{widget_preset_id}/enablement  # Share widget preset with tenants
  # Body: { "enabled_for": ["tenant-1", "tenant-2"] } or { "enabled_for": "all" }
```

---

### Scenario 9: Create Group Preset

**UI Flow:**
1. Configure group with widgets (collapsible behavior, nested items)
2. Click "Save as Preset"
3. Enter preset metadata (name, description, category)
4. Configure sharing:
   - Choose visibility: private, specific tenants, or all tenants
   - Select tenants if sharing with specific tenants
5. Save preset as group instance in GTS registry
6. Preset available for reuse with pre-configured widget layout

**API Calls:**
```
POST /api/analytics/v1/gts  # Create group instance (preset)
  # Type: gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.group.v1~
  # Instance contains settings/collapsible config and settings/items (nested widgets)
PUT /api/analytics/v1/gts/{group_preset_id}/enablement  # Share group preset with tenants
  # Body: { "enabled_for": ["tenant-1", "tenant-2"] } or { "enabled_for": "all" }
```

---

### Scenario 10: Edit Widget Preset

**UI Flow:**
1. Browse widget presets library
2. Select widget preset to edit
3. Load preset configuration
4. Edit datasource configuration, template configuration, item properties
5. Preview changes with live data
6. Update preset metadata if needed
7. Save changes to preset

**API Calls:**
```
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~')&$select=...
GET /api/analytics/v1/gts/{widget_preset_id}
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query.v1~')&$select=...
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.datasource.v1~')&$select=...
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~')&$select=...
GET /api/analytics/v1/templates/{template_id}/bundle
GET /api/analytics/v1/gts/{query_returns_schema_id}
GET /api/analytics/v1/gts/{template_config_schema_id}
PUT /api/analytics/v1/gts/{widget_preset_id}  # Update widget preset
GET /api/analytics/v1/queries/{query_id}?...  # Preview data
```

---

### Scenario 11: Edit Group

**UI Flow:**
1. Open dashboard (user has edit permissions)
2. Select group to edit
3. Edit group properties (name, description, icon, size, collapsible behavior)
4. Manage nested widgets (add/remove/reorder)
5. Preview changes
6. Save group configuration

**API Calls:**
```
PATCH /api/analytics/v1/gts/{dashboard_id}
  # JSON Patch operations on dashboard/entity/items[{group_index}]
  # Update group properties and manage nested widgets
```

---

### Scenario 12: Delete Widget from Dashboard

**UI Flow:**
1. Open dashboard (user has edit permissions)
2. Select widget to delete
3. Click delete/remove button or press Delete key
4. Confirm deletion (optional)
5. Widget removed from dashboard
6. Dashboard layout automatically adjusts

**API Calls:**
```
PATCH /api/analytics/v1/gts/{dashboard_id}
  # JSON Patch operation: op: "remove", path: "/entity/items/{index}"
```

---

### Scenario 13: Delete Group from Dashboard

**UI Flow:**
1. Open dashboard (user has edit permissions)
2. Select group to delete
3. Click delete/remove button
4. Confirm deletion - warn if group contains widgets
5. Group and all nested widgets removed from dashboard
6. Dashboard layout automatically adjusts

**API Calls:**
```
PATCH /api/analytics/v1/gts/{dashboard_id}
  # JSON Patch operation: op: "remove", path: "/entity/items/{index}"
  # Removes group and all nested widgets
```

---

### Scenario 14: Delete Dashboard

**UI Flow:**
1. Navigate to Dashboards list
2. Select dashboard to delete
3. Click delete button
4. Confirm deletion - warn about permanent deletion
5. Dashboard soft-deleted (sets deleted_at timestamp)

**API Calls:**
```
DELETE /api/analytics/v1/gts/{dashboard_id}
  # Soft-delete dashboard (sets deleted_at timestamp)
  # Returns: 204 No Content
```

---

### Scenario 15: View Dashboard

**UI Flow:**
1. Login to Analytics Portal
2. Navigate to Dashboards list
3. Search/browse for desired dashboard
4. Click dashboard to open
5. View widgets with live data
6. Auto-refresh updates data periodically

**API Calls:**
```
GET /api/analytics/v1/gts?$filter=...&$select=...
GET /api/analytics/v1/gts/{dashboard_id}
GET /api/analytics/v1/templates/{template_id}/bundle.js
GET /api/analytics/v1/queries/{query_id}?$filter=...&$orderby=...&$top=...
```

---

### Scenario 16: Interact with Widget Controls and Drill Down

**UI Flow:**
1. User opens dashboard
2. Interacts with individual widget controls via datasource.render_options:
   - **Filters:** Selects date range, region, status from dropdowns
   - **Time Range:** Chooses quick range or custom date range with timezone
   - **Search:** Enters full-text search query
   - **Sorting:** Sorts by column, enables multi-column sort
   - **Pagination:** Changes page size, navigates pages
   - **Grouping:** Groups by category, applies aggregation functions
   - Each widget has independent controls configured via datasource.render_options
3. Widget refreshes with applied parameters
4. User clicks on data point in chart (drill-down)
5. Detail view opens with filtered/sorted/grouped data
6. User can drill down further or return

**API Calls:**
```
GET /api/analytics/v1/queries/{query_id}?$filter=date ge 2024-12-01 and region eq 'EMEA'&$search=urgent orders&$orderby=revenue desc&$top=25&$skip=0&$apply=groupby((category),aggregate(revenue with sum as total))
  # Each widget makes independent query call with its own parameters
  # All controls are widget-level, OData params generated from render_options UI control values
```

---

### Scenario 17: Export Dashboard/Widget Data

**UI Flow:**
1. User opens dashboard
2. Clicks "Export" button on widget or dashboard
3. Selects export format (PDF, CSV, Excel)
4. Optionally applies filters before export
5. System generates export file
6. User downloads file

**API Calls:**
```
POST /api/analytics/v1/queries/{query_id}/export
POST /api/analytics/v1/dashboards/{dashboard_id}/export
```

---

## Widget Settings API Calls Summary

When rendering widget settings dialog:

```
# Load widget configuration
GET /api/analytics/v1/gts/{dashboard_id}

# Load query capabilities for validation
GET /api/analytics/v1/gts/{capabilities_id}

# List available queries for dropdown
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query.v1~')

# List compatible templates for dropdown
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.template.v1~')

# Load template bundle for settings rendering
GET /api/analytics/v1/templates/{template_id}/bundle

# Load template metadata
GET /api/analytics/v1/gts/{template_id}

# Save updated widget configuration
PATCH /api/analytics/v1/gts/{dashboard_id}
```

---

## Common UI/API Patterns

### Authentication Flow
- All API calls require JWT Bearer token in `Authorization` header
- Token contains tenant context for multi-tenancy isolation
- Token expiration and refresh handled by client libraries

### Pagination Pattern
- Cursor-based pagination with `$skiptoken`
- Optional `$count` for total record count
- `@odata.nextLink` for next page URL
- Consistent across all list/query endpoints

### Error Handling
- RFC 7807 Problem Details format
- Consistent HTTP status codes
- Detailed error messages with troubleshooting hints
- Request ID for support tracking

---

### Testing Scenarios

**Unit Tests**:
- Widget configuration validation
- Grid layout auto-adjustment
- Schema compatibility checking
- Permission checks
- Error formatting

**Integration Tests**:
- Dashboard CRUD operations
- Widget preset creation
- Group management
- Sharing and enablement

**UI Tests**:
- Widget settings dialog rendering
- Drag-and-drop positioning
- Template bundle loading
- Live data preview
- Grid layout responsiveness

**E2E Tests**:
- Complete dashboard creation workflow
- Widget addition from preset
- Custom widget configuration
- Dashboard viewing and interaction
- Export functionality

**Performance Tests**:
- Dashboard load time (< 2s for 50 widgets)
- Widget configuration save time (< 200ms)
- Grid layout recalculation (< 100ms)
- Template bundle loading (< 100ms cached)

**Edge Cases**:
1. Dashboard with 100+ widgets
2. Deeply nested groups (3+ levels)
3. Invalid template/query compatibility
4. Concurrent edits by multiple users
5. Large dashboard export
6. Widget position conflicts

---

### OpenSpec Changes Plan

#### Change 001: Dashboard Manager
- **Type**: react
- **Files**: 
  - `ui/src/features/dashboards/DashboardManager.tsx`
- **Description**: Dashboard CRUD UI and orchestration
- **Dependencies**: None (foundational)
- **Effort**: 3.5 hours (AI agent)
- **Validation**: UI tests, integration tests

#### Change 002: Widget Configurator
- **Type**: react
- **Files**: 
  - `ui/src/features/dashboards/WidgetConfigurator.tsx`
- **Description**: Widget settings dialog with platform/template sections
- **Dependencies**: Change 001
- **Effort**: 4 hours (AI agent)
- **Validation**: UI tests, schema validation tests

#### Change 003: Grid Layout Engine
- **Type**: react
- **Files**: 
  - `ui/src/features/dashboards/GridLayout.tsx`
- **Description**: Responsive grid with drag-and-drop
- **Dependencies**: Change 001
- **Effort**: 3 hours (AI agent)
- **Validation**: UI tests, layout tests

#### Change 004: Template Bundle Loader
- **Type**: react
- **Files**: 
  - `ui/src/features/dashboards/TemplateBundleLoader.tsx`
- **Description**: Dynamic template loading and caching
- **Dependencies**: Change 002
- **Effort**: 2 hours (AI agent)
- **Validation**: Bundle loading tests, cache tests

#### Change 005: Widget Preset Manager
- **Type**: react
- **Files**: 
  - `ui/src/features/dashboards/PresetManager.tsx`
- **Description**: Create and manage widget/group presets
- **Dependencies**: Change 002
- **Effort**: 2 hours (AI agent)
- **Validation**: Preset CRUD tests

#### Change 006: Dashboard Sharing UI
- **Type**: react
- **Files**: 
  - `ui/src/features/dashboards/SharingDialog.tsx`
- **Description**: Configure dashboard sharing with tenants
- **Dependencies**: Change 001
- **Effort**: 1.5 hours (AI agent)
- **Validation**: Sharing tests, permission tests

#### Change 007: Widget Live Preview
- **Type**: react
- **Files**: 
  - `ui/src/features/dashboards/WidgetPreview.tsx`
- **Description**: Real-time widget preview with data
- **Dependencies**: Change 002, Change 004
- **Effort**: 2 hours (AI agent)
- **Validation**: Preview rendering tests

#### Change 008: Group Management
- **Type**: react
- **Files**: 
  - `ui/src/features/dashboards/GroupManager.tsx`
- **Description**: Create and manage collapsible groups
- **Dependencies**: Change 003
- **Effort**: 2 hours (AI agent)
- **Validation**: Group nesting tests

#### Change 009: Dashboard Export
- **Type**: react
- **Files**: 
  - `ui/src/features/dashboards/ExportDialog.tsx`
- **Description**: Export dashboard/widget data (PDF, CSV, Excel)
- **Dependencies**: Change 001
- **Effort**: 1.5 hours (AI agent)
- **Validation**: Export format tests

#### Change 010: Integration Testing Suite
- **Type**: react (tests)
- **Files**: 
  - `ui/tests/dashboards.test.tsx`
- **Description**: E2E dashboard workflow tests
- **Dependencies**: All previous changes
- **Effort**: 3.5 hours (AI agent)
- **Validation**: 100% scenario coverage

**Total Effort**: 24 hours (AI agent + OpenSpec)

---

## Dependencies

- **Depends On**: 
  - feature-dashboard-layouts (layout storage)
  - feature-widget-items (widget type definitions)
  - feature-group-items (group type definitions)
  - feature-widget-templates (template bundles)
  - feature-query-execution (data preview)
- **Blocks**: 
  - feature-reporting (reports build on dashboard patterns)
  - feature-export-sharing (uses dashboard data)

---

## References

- Overall Design: `architecture/DESIGN.md` Section 2 (Actors), Section 3 (System Capabilities)
- GTS Types: Dashboard and item schemas (owned by feature-dashboard-layouts, feature-widget-items)
- OpenAPI Spec: `architecture/openapi/v1/api.yaml` (dashboard endpoints)
- Feature Manifest: `architecture/features/FEATURES.md` (feature-dashboards entry)
