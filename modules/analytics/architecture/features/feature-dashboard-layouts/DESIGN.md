# Feature: Dashboard Layouts

**Status**: NOT_STARTED  
**Feature Slug**: `feature-dashboard-layouts`

---

## A. Feature Context

### Overview

Dashboard layout type for real-time dashboards with responsive grid-based positioning. Implements masonry-style layout distribution algorithm for optimal space utilization.

**Purpose**: Provide layout storage, positioning algorithms, and size calculation for dashboard items.

**Scope**:
- Layout GTS type: `layout.v1~` (base) + `layout.v1~dashboard.v1~`
- Dashboard layout DB tables
- Real-time layout properties (auto-refresh, live updates)
- Layout-item relationships
- Dashboard-specific indexing (by user, by tenant, by category)
- Layout distribution algorithm
- Item preview rendering

**Out of Scope**:
- Report layouts - handled by feature-report-layouts
- Widget items - handled by feature-widget-items
- Group items - handled by feature-group-items
- Dashboard business logic - handled by feature-dashboards

### GTS Types

This feature owns:
- **`gts://gts.hypernetix.hyperspot.ax.layout.v1~`** - Base layout type
- **`gts://gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.dashboard.v1~`** - Dashboard-specific layout

References from `gts/types/`:
- [base.schema.json](../../../gts/types/layout/v1/base.schema.json) - Base layout schema
- [dashboard.schema.json](../../../gts/types/layout/v1/dashboard.schema.json) - Dashboard layout schema

### OpenAPI Endpoints

From `architecture/openapi/v1/api.yaml`:
- `POST /api/analytics/v1/gts` - Register dashboard layout instance
- `GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.layout.v1~dashboard.v1~')` - List dashboard layouts
- `GET /api/analytics/v1/gts/{dashboard-layout-id}` - Get specific layout
- `PUT /api/analytics/v1/gts/{dashboard-layout-id}` - Update layout
- `PATCH /api/analytics/v1/gts/{dashboard-layout-id}` - Partial update layout
- `DELETE /api/analytics/v1/gts/{dashboard-layout-id}` - Delete layout
- `PUT /api/analytics/v1/gts/{dashboard-layout-id}/enablement` - Configure tenant access

### Actors

**Human Actors** (from Overall Design):
- **Admin** - Creates and manages dashboard layouts, configures sharing
- **Dashboard Creator** - Designs layouts, positions widgets, configures layout properties
- **End User** - Views rendered dashboards (layout consumption)

**System Actors**:
- **Analytics Service** - Stores layouts, enforces tenant isolation
- **UI Renderer** - Executes layout distribution algorithm, calculates item positions
- **GTS Core** - Routes layout CRUD operations

**Service Roles** (from OpenAPI):
- `analytics:layout:read` - View dashboard layouts
- `analytics:layout:write` - Create/update layouts
- `analytics:layout:delete` - Delete layouts
- `analytics:layout:share` - Configure tenant enablement

---

## B. Actor Flows

### Flow 1: Admin Creates Dashboard Layout

**Actor**: Admin  
**Trigger**: Admin clicks "Create New Dashboard"  
**Goal**: Register empty dashboard layout for future widget additions

**UI Steps**:
1. Navigate to Dashboards → Create New
2. Enter dashboard metadata (name, description, icon)
3. Select category from dropdown
4. Configure auto-refresh settings (enabled/disabled, interval)
5. Configure sharing:
   - Private (owner only)
   - Specific tenants (select from list)
   - All tenants (global)
6. Click "Save"
7. Dashboard created with empty `items[]` array

**API Interactions**:
```
1. GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.category.v1~dashboard.v1~')
   → Returns: List of available categories

2. POST /api/analytics/v1/gts
   Body: {
     "id": "gts.hypernetix.hyperspot.ax.layout.v1~dashboard.v1~acme.sales._.executive.v1",
     "entity": {
       "name": "Executive Sales Dashboard",
       "description": "Real-time sales metrics",
       "icon": "dashboard",
       "category_id": "gts.hypernetix.hyperspot.ax.category.v1~dashboard.v1~sales",
       "items": [],
       "settings": {
         "auto_refresh": { "enabled": true, "interval": 60 },
         "theme": "light"
       }
     }
   }
   → Returns: Created layout with metadata

3. PUT /api/analytics/v1/gts/{dashboard_id}/enablement
   Body: { "enabled_for": ["tenant-1", "tenant-2"] }
   → Returns: 204 No Content
   → Side Effect: System automatically enables all future referenced entities
```

**Outcome**: Empty dashboard layout registered, ready for widget additions

---

### Flow 2: Dashboard Creator Positions Widgets

**Actor**: Dashboard Creator  
**Trigger**: User opens dashboard editor, adds widgets  
**Goal**: Position widgets on grid using drag-and-drop

**UI Steps**:
1. Open dashboard in edit mode
2. Add widget to dashboard (becomes part of `items[]` array)
3. UI renders widget using Layout Distribution Algorithm
4. User drags widget to new position
5. UI recalculates positions for all items
6. Click "Save Layout"
7. Updated `items[]` array persisted

**API Interactions**:
```
1. GET /api/analytics/v1/gts/{dashboard_id}
   → Returns: Current layout with items[] array

2. PATCH /api/analytics/v1/gts/{dashboard_id}
   Body: [
     { "op": "add", "path": "/entity/items/-", "value": {widget_config} },
     { "op": "replace", "path": "/entity/items/0/settings/size/width", "value": 50 }
   ]
   → Returns: Updated layout
```

**Client-Side Processing** (Layout Distribution Algorithm runs in browser):
- Parse `items[]` array from API response
- For each item, calculate absolute position using algorithm
- Render items at calculated positions
- On drag, update item order in array, recalculate positions

**Outcome**: Widgets positioned optimally using masonry-style algorithm

---

### Flow 3: UI Renderer Calculates Item Positions

**Actor**: UI Renderer (System)  
**Trigger**: Dashboard loaded or layout modified  
**Goal**: Calculate absolute pixel positions for all items

**Processing Steps**:
1. Receive layout data from API (`items[]` array with relative sizes)
2. Get current viewport/container width
3. Execute Layout Distribution Algorithm (see Section C)
4. For each item:
   - Calculate absolute width: `(item.width / 100) × container_width`
   - Map height preset to pixels: `medium → 400px`
   - Determine x,y position using algorithm
5. Render items at calculated positions
6. On resize, recalculate widths and reposition

**API Interactions**:
```
GET /api/analytics/v1/gts/{dashboard_id}
→ Returns: Layout with items[] (relative sizes)
```

**Client-Side Algorithm Execution**: See Section C for detailed algorithm

**Outcome**: Items rendered at optimal positions with correct dimensions

---

### Flow 4: End User Views Dashboard

**Actor**: End User  
**Trigger**: User navigates to dashboard  
**Goal**: View live dashboard with auto-refresh

**UI Steps**:
1. Navigate to Dashboards list
2. Click dashboard name
3. UI loads layout and executes positioning algorithm
4. Widgets render with live data
5. Auto-refresh timer triggers (if enabled)
6. Dashboard updates periodically

**API Interactions**:
```
1. GET /api/analytics/v1/gts/{dashboard_id}
   → Returns: Layout configuration with items[]

2. (Auto-refresh) Periodically re-fetch data for each widget
   → Updates happen via widget-level queries
```

**Outcome**: User sees live dashboard with properly positioned widgets

---

### Flow 5: Analytics Service Enforces Tenant Isolation

**Actor**: Analytics Service (System)  
**Trigger**: Any layout API call  
**Goal**: Ensure tenant can only access enabled layouts

**Processing Steps**:
1. Extract `tenant_id` from JWT (SecurityCtx)
2. Query: `SELECT * FROM dashboard_layouts WHERE id = ? AND (enabled_for @> ? OR enabled_for = 'all')`
3. If no match: return 403 Forbidden
4. If match: return layout data
5. Apply tenant filter to all list queries

**Security Check**:
```sql
-- Tenant isolation query
SELECT * FROM dashboard_layouts
WHERE id = $1
  AND (
    enabled_for @> $2::jsonb  -- Tenant in array
    OR enabled_for = '"all"'::jsonb  -- Enabled for all
  )
  AND deleted_at IS NULL;
```

**Outcome**: Tenant isolation enforced, unauthorized access blocked

---

## C. Algorithms

### UI Algorithm 1: Layout Distribution Algorithm

**Algorithm: Position Dashboard Items**

Input: items array, container_width  
Output: x,y positions for each item  
Type: Client-side (browser)  
Complexity: O(n²)

![Layout Distribution Example](diagrams/layout_distribution_example.drawio.svg)

**Horizontal Grid**:
- Container divided into 20 sections (5% each)
- Item widths: 15-100% (multiples of 5)
- Items fill left-to-right

**Height Presets**:
- `micro`: 100px
- `small`: 200px
- `medium`: 400px
- `high`: 600px
- `unlimited`: grows with content

**Positioning Steps**:

1. Initialize empty positions array
2. Place first item at top-left (x=0, y=0)
3. Add first item to positions
4. **FOR EACH** remaining item in items array:
   1. Set candidate_y = 0
   2. **FOR EACH** existing positioned item:
      1. Calculate item_bottom = item.y + item.height
      2. **IF** item_bottom > candidate_y:
         1. Set candidate_y = item_bottom
   3. Check if item fits below any existing item:
      1. **FOR EACH** existing item:
         1. **IF** item.x allows horizontal fit:
            1. Calculate possible_y = item.y + item.height
            2. **IF** no collision at (item.x, possible_y):
               1. Use this position
               2. **GO TO** step 4.5
   4. No vertical fit found:
      1. Calculate next_x from previous row
      2. **IF** next_x + item.width > 100:
         1. Set x = 0 (new row)
         2. Set y = candidate_y
      3. **ELSE**:
         1. Set x = next_x
         2. Set y = last_item.y
   5. Add positioned item to array
5. **RETURN** positions array

**Result**: Masonry-style layout with optimal space utilization

**API**: Data loaded via `GET /api/analytics/v1/gts/{dashboard_id}`

---

### UI Algorithm 2: Item Preview Rendering

**Algorithm: Calculate Item Dimensions**

Input: item, container_width, context  
Output: absolute_width_px, absolute_height_px

**Width Calculation**:

1. Get item.size.width (percentage: 15-100%, multiples of 5)
2. **MATCH** context:
   - **CASE** "dashboard_editor": Get viewport width minus sidebar
   - **CASE** "settings_preview": Use 600px fixed width
   - **CASE** "live_rendering": Get actual container width
3. Calculate: absolute_width = (item.width / 100) × container_width
4. Round to nearest pixel

**Height Calculation**:

1. Get item.size.height (preset enum)
2. **MATCH** item.size.height:
   - **CASE** "micro": Set height = 100px
   - **CASE** "small": Set height = 200px
   - **CASE** "medium": Set height = 400px
   - **CASE** "high": Set height = 600px
   - **CASE** "unlimited": Set min_height = 200px, allow content expansion
3. **RETURN** height

**Rendering Steps**:

1. Calculate absolute dimensions
2. **IF** context is "dashboard_editor":
   1. Apply Layout Distribution Algorithm for position
   2. Enable dragging (snapped to 5% grid)
   3. Enable resizing (multiples of 5%)
   4. Enable reordering
3. **ELSE IF** context is "settings_preview":
   1. Center item in preview panel
   2. Show static preview (no interaction)
3. **ELSE** (live_rendering):
   1. Apply Layout Distribution Algorithm
   2. Render item with calculated dimensions
4. **RETURN** rendered item
- Single item rendered in isolation
- Shows how item will appear at its configured size
- Non-interactive, read-only preview

#### 3. Live Rendering (Dashboard View)

- Layout container width = viewport width or dashboard container
- Full layout rendered with all items positioned
- Responsive: container width changes trigger re-calculation of absolute widths
- Heights remain fixed per preset values

**Example**:
- Item: width=50%, height="medium"
- Context: Editor with 1200px canvas
- Result: 600px × 400px

**Responsive Behavior**:
- Container resize → widths recalculate proportionally
- Heights stay constant (fixed presets)
- Layout algorithm re-runs for repositioning

**Complexity**: O(1) per item

---

## D. States

*(Not applicable - layouts are stateless entities with CRUD lifecycle only)*

---

## E. Technical Details

### Dashboard Layout Properties

### Layout Structure

```json
{
  "id": "gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.dashboard.v1~acme.sales._.executive.v1",
  "type": "gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.dashboard.v1~",
  "entity": {
    "name": "Executive Sales Dashboard",
    "description": "Real-time sales metrics and performance indicators",
    "icon": "dashboard",
    "category_id": "gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.dashboard.v1~sales",
    "items": [
      {
        "type": "gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~",
        "settings": {
          "name": "Revenue Trend",
          "template": {
            "id": "gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~charts.line.v1",
            "config": { /* template-specific config */ }
          },
          "datasource": {
            "query_id": "gts.hypernetix.hyperspot.ax.query.v1~sales.revenue.v1",
            "params": { /* OData params */ }
          },
          "size": {
            "width": 50,
            "height": "medium"
          }
        }
      }
    ],
    "settings": {
      "auto_refresh": {
        "enabled": true,
        "interval": 60
      },
      "theme": "light"
    }
  }
}
```

### Real-Time Properties

- **auto_refresh.enabled** - Enable automatic data refresh
- **auto_refresh.interval** - Refresh interval in seconds
- **theme** - Dashboard theme (light, dark, custom)

---

### Database Schema

```sql
CREATE TABLE dashboard_layouts (
    id VARCHAR(500) PRIMARY KEY,
    type VARCHAR(500) NOT NULL,
    tenant VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    icon VARCHAR(100),
    category_id VARCHAR(500),
    items JSONB NOT NULL,
    settings JSONB,
    registered_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    registered_by VARCHAR(255),
    updated_by VARCHAR(255),
    deleted_by VARCHAR(255)
);

CREATE INDEX idx_dashboard_layouts_tenant ON dashboard_layouts(tenant);
CREATE INDEX idx_dashboard_layouts_category ON dashboard_layouts(category_id);
CREATE INDEX idx_dashboard_layouts_registered_by ON dashboard_layouts(registered_by);
CREATE INDEX idx_dashboard_layouts_deleted_at ON dashboard_layouts(deleted_at) WHERE deleted_at IS NULL;
CREATE INDEX idx_dashboard_layouts_items_gin ON dashboard_layouts USING GIN(items);
```

**Indexes Rationale**:
- `idx_dashboard_layouts_tenant` - Tenant isolation filtering (SecurityCtx)
- `idx_dashboard_layouts_category` - Category browsing
- `idx_dashboard_layouts_registered_by` - User's dashboards listing
- `idx_dashboard_layouts_deleted_at` - Exclude soft-deleted (partial index for performance)
- `idx_dashboard_layouts_items_gin` - JSON search on items array (widget search)

---

### Item Size Configuration

### Width

- **Type:** Percentage
- **Range:** 15-100%
- **Step:** 5% (multiples of 5 only)
- **Valid values:** 15, 20, 25, 30, 35, 40, 45, 50, 55, 60, 65, 70, 75, 80, 85, 90, 95, 100

### Height

- **Type:** Fixed preset enum
- **Values:**
  - `micro` - ~100px - Small indicators, KPIs
  - `small` - ~200px - Compact charts, gauges
  - `medium` - ~400px - Standard charts, tables
  - `high` - ~600px - Detailed visualizations
  - `unlimited` - Min-height with content expansion - Paginated tables, long lists

**Validation Rules**:
- Width: Must be multiple of 5 between 15-100
- Height: Must be one of predefined presets
- Both values required for each item

---

### Access Control

**SecurityCtx Integration**:

All layout operations require SecurityCtx with `tenant_id` extracted from JWT.

**Example Flow**:
1. Extract tenant_id from JWT (SecurityCtx)
2. Query database with tenant filter
3. **IF** layout found AND tenant has access:
   1. **RETURN** layout
4. **ELSE**:
   1. **RETURN** 403 Forbidden

**Permission Checks**:

| Operation | Required Role | SecurityCtx Check |
|-----------|---------------|-------------------|
| List layouts | `analytics:layout:read` | Filter by tenant_id |
| Get layout | `analytics:layout:read` | Verify tenant access |
| Create layout | `analytics:layout:write` | Set owner = tenant_id |
| Update layout | `analytics:layout:write` | Verify tenant ownership |
| Delete layout | `analytics:layout:delete` | Verify tenant ownership |
| Share layout | `analytics:layout:share` | Verify tenant ownership |

**Tenant Enablement**:
- Layouts created by tenant A can be shared with tenants B, C
- System automatically enables all referenced entities (widgets, templates, etc.)
- See feature-tenancy-enablement for automatic dependency resolution

---

### Error Handling

**Common Errors**:

| Error | HTTP Status | Cause | Resolution |
|-------|-------------|-------|------------|
| Layout not found | 404 | Invalid layout_id or tenant mismatch | Verify ID and tenant access |
| Validation error | 400 | Invalid size values or missing fields | Check width/height constraints |
| Unauthorized | 401 | Missing or invalid JWT | Refresh authentication |
| Forbidden | 403 | Tenant not enabled for layout | Request access or check sharing |
| Conflict | 409 | Duplicate layout ID | Use different ID |
| Schema validation | 422 | Entity doesn't match GTS schema | Fix entity structure |

**Error Response Format (RFC 7807)**:
```json
{
  "type": "https://example.com/problems/layout-not-found",
  "title": "Dashboard Layout Not Found",
  "status": 404,
  "detail": "Layout 'acme.sales._.executive.v1' not found or not accessible to tenant 'tenant-123'",
  "instance": "/api/analytics/v1/gts/{layout-id}",
  "trace_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Retry Strategy**:
- 404/403: Do not retry (permission issue)
- 500/503: Retry with exponential backoff (max 3 attempts)
- 422: Do not retry (fix request first)

---

## F. Validation & Implementation

### Testing Scenarios

**Unit Tests**:

1. **Layout Distribution Algorithm**
   - Input: 12 items with varying widths (25%, 50%, 100%)
   - Expected: Masonry-style positioning, no overlaps
   - Verify: First item at (0,0), vertical stacking prioritized

2. **Size Calculation Algorithm**
   - Input: width=50%, height=medium, container=1200px
   - Expected: 600px × 400px
   - Verify: Percentage calculation accurate, preset mapping correct

3. **Responsive Recalculation**
   - Input: Container resize from 1200px → 800px
   - Expected: All widths recalculated, heights unchanged
   - Verify: Proportional scaling maintained

4. **Validation Rules**
   - Input: width=33 (invalid, not multiple of 5)
   - Expected: Validation error
   - Verify: Error message specifies constraint

**Integration Tests**:

1. **CRUD Operations with Tenant Isolation**
   - Create layout as tenant A
   - Attempt access as tenant B (should fail)
   - Share with tenant B
   - Verify tenant B can now access

2. **Layout with 50 Widgets**
   - Create layout with 50 items
   - Verify: Algorithm completes in <100ms
   - Verify: All items positioned without overlap

3. **Auto-Refresh Settings**
   - Create layout with auto_refresh enabled
   - Verify: Settings persisted correctly
   - Verify: Retrieved layout has correct interval

**UI Tests** (Browser/E2E):

1. **Drag-and-Drop Positioning**
   - Open dashboard editor
   - Drag widget to new position
   - Save layout
   - Reload page, verify position persisted

2. **Responsive Rendering**
   - Open dashboard at 1920px width
   - Resize browser to 1024px
   - Verify: Widgets resize proportionally
   - Verify: Layout remains readable

3. **Multi-Context Rendering**
   - Render same layout in editor, preview, live view
   - Verify: Algorithm produces consistent results
   - Verify: Interactive controls work in editor only

**Performance Tests**:

1. **Large Layout (100 widgets)**
   - Create layout with 100 items
   - Measure: Algorithm execution time
   - Target: <200ms for full layout calculation

2. **Rapid Resize**
   - Resize container 100 times rapidly
   - Verify: No UI lag or freezing
   - Verify: Recalculation debounced properly

**Edge Cases**:

1. Empty layout (items = [])
2. Single widget (100% width)
3. All micro widgets (minimal height)
4. Mixed height presets in same row
5. Container width < minimum item width (15%)

---

### OpenSpec Changes Plan

#### Change 001: GTS Layout Type Definition
- **Type**: gts
- **Files**: 
  - [layout.v1.schema.json](../../../gts/types/layout/v1/base.schema.json) (base type)
  - [layout.v1.dashboard.v1.schema.json](../../../gts/types/layout/v1/dashboard.schema.json) (dashboard extension)
- **Description**: Define GTS schema for dashboard layout with items array, size config, auto-refresh settings
- **Dependencies**: None (foundational)
- **Effort**: 0.5 hours (AI agent)
- **Validation**: JSON Schema validation, sample instances

#### Change 002: Database Schema Creation
- **Type**: database
- **Files**: 
  - `modules/analytics/migrations/001_create_dashboard_layouts.sql`
- **Description**: Create `dashboard_layouts` table with tenant isolation, indexes for performance
- **Dependencies**: 001
- **Effort**: 0.5 hours (AI agent)
- **Validation**: Run migration, verify indexes created

#### Change 003: OpenAPI Layout Endpoints
- **Type**: openapi
- **Files**: 
  - `architecture/openapi/v1/api.yaml`
- **Description**: Add layout CRUD endpoints to OpenAPI spec with OData parameters
- **Dependencies**: 001
- **Effort**: 0.5 hours (AI agent)
- **Validation**: Swagger validator, endpoint completeness check

#### Change 004: Layout CRUD Implementation
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/api/rest/dashboard_layouts/handlers.rs`
  - `modules/analytics/src/domain/dashboard_layouts/repository.rs`
  - `modules/analytics/src/domain/dashboard_layouts/validation.rs`
- **Description**: Implement CRUD handlers with SecurityCtx, tenant isolation, validation
- **Dependencies**: 002, 003
- **Effort**: 2 hours (AI agent)
- **Validation**: Unit tests for each handler, integration tests with DB

#### Change 005: Size Validation Logic
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/dashboard_layouts/validation.rs`
- **Description**: Validate width (15-100%, multiples of 5), height (preset enum)
- **Dependencies**: 004
- **Effort**: 1 hour (AI agent)
- **Validation**: Unit tests with valid/invalid inputs

#### Change 006: Client-Side Algorithm Implementation
- **Type**: typescript
- **Files**: 
  - `ui/src/features/dashboards/layoutAlgorithm.ts`
  - `ui/src/features/dashboards/sizeCalculation.ts`
- **Description**: Implement Layout Distribution Algorithm and Item Preview Rendering in TypeScript
- **Dependencies**: None (client-side only)
- **Effort**: 3 hours (AI agent)
- **Validation**: Jest unit tests, visual regression tests

#### Change 007: Tenant Enablement Integration
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/dashboard_layouts/enablement.rs`
- **Description**: Integrate with feature-tenancy-enablement for sharing
- **Dependencies**: 004, feature-tenancy-enablement
- **Effort**: 1 hour (AI agent)
- **Validation**: Integration tests with multi-tenant scenarios

#### Change 008: UI Editor Integration
- **Type**: typescript
- **Files**: 
  - `ui/src/features/dashboards/LayoutEditor.tsx`
  - `ui/src/features/dashboards/ItemRenderer.tsx`
- **Description**: Build drag-and-drop editor using algorithm, real-time preview
- **Dependencies**: 006
- **Effort**: 4 hours (AI agent)
- **Validation**: E2E tests with Playwright, visual testing

#### Change 009: Performance Optimization
- **Type**: rust + typescript
- **Files**: 
  - `modules/analytics/src/domain/dashboard_layouts/repository.rs` (DB queries)
  - `ui/src/features/dashboards/layoutAlgorithm.ts` (algorithm optimization)
- **Description**: Optimize DB queries (pagination, indexes), debounce resize calculations
- **Dependencies**: 004, 006
- **Effort**: 1.5 hours (AI agent)
- **Validation**: Performance benchmarks, load testing

#### Change 010: Documentation
- **Type**: documentation
- **Files**: 
  - `docs/features/dashboard-layouts.md`
  - `ui/src/features/dashboards/README.md`
- **Description**: Document API usage, algorithm details, UI integration guide
- **Dependencies**: All previous
- **Effort**: 1 hour (AI agent)
- **Validation**: Documentation review, completeness check

**Total Effort Estimate**: 15 hours (AI agent + OpenSpec)

**Implementation Order**: 001 → 002 → 003 → 004 → 005 → 007 → 006 → 008 → 009 → 010

**Critical Path**: 001 → 002 → 004 → 006 → 008 (core functionality)

---

## Dependencies

- **Depends On**: 
  - feature-gts-core (routing, GTS registry)
  - feature-widget-items (widget item type definition)
  - feature-group-items (group item type definition)
- **Blocks**: 
  - feature-dashboards (business logic layer needs layouts)

---

## References

- Overall Design: `architecture/DESIGN.md` Section 2 (Actors), Section 3 (System Capabilities)
- GTS Types: [base.schema.json](../../../gts/types/layout/v1/base.schema.json), [dashboard.schema.json](../../../gts/types/layout/v1/dashboard.schema.json)
- OpenAPI Spec: `architecture/openapi/v1/api.yaml` (layout endpoints)
- Feature Manifest: `architecture/features/FEATURES.md` (feature-dashboard-layouts entry)
