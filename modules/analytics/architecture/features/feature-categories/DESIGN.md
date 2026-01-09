# Feature: Categories

**Slug**: `feature-categories`
**Status**: ⏳ NOT_STARTED
**Dependencies**: [feature-gts-core](../feature-gts-core/)

---

## A. Feature Context

### 1. Feature Overview

**Feature**: Categories

**Purpose**: Hierarchical organization system for all GTS entities, enabling classification, grouping, and library management across the Analytics module.

**Scope**:
- Category GTS types (9 types: base + 8 domain categories)
- Category DB tables (single unified table - no domain-specific logic)
- Widget libraries (reusable widget collections)
- Template libraries (visualization marketplace)
- Datasource libraries (preconfigured data connectors)
- Query libraries (shareable query definitions)
- Hierarchical classification with parent-child relationships
- Category-based search and filtering across all GTS types

**References to OVERALL DESIGN**:

**GTS Types**: 
- `gts://gts.hypernetix.hyperspot.ax.category.v1~` - Base category type
- Category types for domains: Query, Template, Datasource, Widget, Item, Group, Dashboard, Layout (8 specialized types)

**OpenAPI Endpoints** (from `architecture/openapi/v1/api.yaml`):
- `POST /api/analytics/v1/gts` - Register category instance
- `GET /api/analytics/v1/gts` - List/search categories with OData
- `GET /api/analytics/v1/gts/{id}` - Get specific category
- `PUT /api/analytics/v1/gts/{id}` - Update category
- `PATCH /api/analytics/v1/gts/{id}` - Partial update category with JSON Patch
- `DELETE /api/analytics/v1/gts/{id}` - Delete category
- `GET /api/analytics/v1/$metadata` - OData metadata including category types

**Service Roles** (from OpenAPI):
- `analytics:gts_read` - Read category entities
- `analytics:gts_write` - Create/update categories
- `analytics:gts_delete` - Delete categories
- `analytics:metadata_read` - Read metadata including category schemas

**User Roles** (from Overall Design):
- Platform Administrator - Manage global category structures
- Tenant Administrator - Manage tenant-specific categories
- Dashboard Designer - Organize widgets and templates in categories
- Plugin Developer - Categorize custom plugins and templates
- Template Developer - Organize template libraries
- Business Analyst - Browse and filter by categories

**Actors**:
- Dashboard Designer - Creates and manages category hierarchies for organizing dashboards
- Plugin Developer - Categorizes custom datasource plugins
- Template Developer - Organizes widget templates into libraries
- Platform Administrator - Manages global category structures
- Tenant Administrator - Manages tenant-specific category hierarchies
- UI Application (HAI3) - Displays category trees and enables browsing
- Hyperspot Platform - Provides SecurityCtx and tenant isolation

---

## B. Actor Flows

### Dashboard Designer Flow

**UI Interactions**:
1. Navigate to category management section in HAI3
2. View hierarchical category tree for widgets/dashboards/templates
3. Create new category with name, description, parent reference
4. Drag-and-drop to reorganize category hierarchy
5. Assign widgets/dashboards to categories
6. Search entities by category filter

**API Interactions**:
1. **List Categories**: `GET /api/analytics/v1/gts?$filter=type eq 'category.v1~widget'&$orderby=name`
2. **Create Category**: `POST /api/analytics/v1/gts` with category payload
3. **Update Hierarchy**: `PATCH /api/analytics/v1/gts/{id}` to change parent_id
4. **Assign Entity to Category**: `PATCH /api/analytics/v1/gts/{widget_id}` to set category_id field
5. **Search by Category**: `GET /api/analytics/v1/gts?$filter=category_id eq '{cat_id}'`

### Template Developer Flow

**UI Interactions**:
1. Access template library management
2. Create category structure for template marketplace
3. Tag templates with multiple categories
4. Browse templates by category filters
5. Export/import category structures

**API Interactions**:
1. **Create Template Categories**: `POST /api/analytics/v1/gts` for template categories
2. **Tag Template**: `PATCH /api/analytics/v1/gts/{template_id}` to add category references
3. **Browse by Category**: `GET /api/analytics/v1/gts?$filter=type eq 'template.v1~widget.v1~' and category_id eq '{id}'`
4. **Get Category Tree**: `GET /api/analytics/v1/gts?$filter=type eq 'category.v1~template' and parent_id eq null&$expand=children`

### Tenant Administrator Flow

**UI Interactions**:
1. View all categories within tenant scope
2. Create tenant-specific category hierarchies
3. Import global categories as templates
4. Manage access control for categories
5. Monitor category usage across tenant

**API Interactions**:
1. **List Tenant Categories**: `GET /api/analytics/v1/gts?$filter=type startswith 'category.v1~'` (SecurityCtx filters by tenant automatically)
2. **Create Tenant Category**: `POST /api/analytics/v1/gts` with category data
3. **Category Analytics**: `GET /api/analytics/v1/gts?$apply=groupby((category_id),aggregate($count as usage))`

---

## C. Algorithms

### 1. UI Algorithms

**Algorithm: Display Category Tree**

Input: domain_type (e.g., "widget", "template", "query")  
Output: Hierarchical category tree structure

1. **Fetch root categories** for domain type
2. **FOR EACH** root category:
   1. Fetch category details from API
   2. **Fetch children** recursively with `$expand=children`
   3. Build tree node with category metadata
   4. **IF** category has children:
      1. **Recursively render** child categories
   5. Render category node in UI tree component
3. **Display** complete category tree with expand/collapse controls
4. **Enable** drag-and-drop for hierarchy reorganization
5. **RETURN** rendered tree structure

**Algorithm: Assign Entity to Category**

Input: entity_id, category_id  
Output: Updated entity with category reference

1. **Validate** entity and category exist in same tenant
2. **IF** category not found:
   1. **RETURN** 404 error with message
3. **Construct** JSON Patch operation: `[{"op": "replace", "path": "/category_id", "value": "{category_id}"}]`
4. **Send** PATCH request to `/gts/{entity_id}`
5. **IF** request succeeds:
   1. Update local entity cache
   2. Refresh category tree UI
   3. Show success notification
6. **ELSE**:
   1. Show error message
   2. Revert UI changes
7. **RETURN** updated entity

**Algorithm: Browse Entities by Category**

Input: category_id, domain_type  
Output: Filtered entity list

1. **Construct** OData filter: `$filter=category_id eq '{category_id}' and type startswith '{domain_type}'`
2. **Add** pagination params: `$top=50&$skip={offset}`
3. **Send** GET request to `/gts?{odata_params}`
4. **Parse** response with entity list and total count
5. **Render** entity cards/list in UI
6. **Enable** pagination controls
7. **IF** user clicks category in breadcrumb:
   1. Navigate to parent category
   2. **Repeat** algorithm with parent category_id
8. **RETURN** entity list with pagination

### 2. Service Algorithms

**Algorithm: Register Category**

Input: security_ctx, category_payload  
Output: Registered category with generated ID

1. **Validate** SecurityCtx has `analytics:gts_write` scope
2. **Extract** tenant_id from SecurityCtx
3. **Validate** category payload against JSON schema
4. **Check** required fields: name, type (must start with "category.v1~")
5. **IF** parent_id provided:
   1. **Verify** parent category exists in same tenant
   2. **Verify** parent has same domain type
   3. **Check** for circular references in hierarchy
   4. **IF** circular reference detected:
      1. **RETURN** 400 error
6. **Generate** GTS identifier: `gts.hypernetix.hyperspot.ax.category.v1~{tenant}.{domain}.{name}.v1`
7. **Insert** category record into categories table
8. **IF** database constraint violation:
   1. **RETURN** 409 conflict error
9. **Index** category for search (name, description, path)
10. **RETURN** registered category with 201 status

**Algorithm: Update Category Hierarchy**

Input: security_ctx, category_id, patch_operations  
Output: Updated category

1. **Validate** SecurityCtx and tenant access
2. **Fetch** category from database
3. **IF** category not found or wrong tenant:
   1. **RETURN** 404 error
4. **FOR EACH** JSON Patch operation:
   1. **IF** operation targets `/parent_id`:
      1. **Validate** new parent exists in same tenant
      2. **Validate** new parent has same domain type
      3. **Check** for circular references
      4. **IF** would create cycle:
         1. **RETURN** 400 error with details
      5. **Update** category path for efficient querying
      6. **Update** all descendant paths recursively
   2. **IF** operation targets `/name`:
      1. **Validate** name uniqueness within parent
      2. **Update** GTS identifier if needed
   3. **Apply** operation to category record
5. **Save** updated category to database
6. **Reindex** category and descendants
7. **RETURN** updated category with 200 status

**Algorithm: Delete Category**

Input: security_ctx, category_id, cascade (boolean)  
Output: Deletion confirmation

1. **Validate** SecurityCtx has `analytics:gts_delete` scope
2. **Fetch** category from database with tenant check
3. **IF** category not found:
   1. **RETURN** 404 error
4. **Count** child categories
5. **Count** entities referencing this category
6. **IF** has children or references and NOT cascade:
   1. **RETURN** 400 error: "Category in use, enable cascade or reassign entities"
7. **IF** cascade is true:
   1. **FOR EACH** child category:
      1. **Recursively** delete child category
   2. **FOR EACH** entity referencing category:
      1. **Set** entity category_id to null (unassign)
      2. **OR** move to parent category if specified
8. **Delete** category record from database
9. **Remove** from search index
10. **RETURN** 204 no content

**Algorithm: Search Categories**

Input: security_ctx, odata_query  
Output: Filtered category list with metadata

1. **Validate** SecurityCtx
2. **Extract** tenant_id for automatic filtering
3. **Parse** OData query parameters: $filter, $orderby, $top, $skip, $search
4. **Build** SQL query with tenant isolation
5. **IF** $filter contains domain type:
   1. **Add** type prefix filter: `type LIKE 'category.v1~{domain}%'`
6. **IF** $search provided:
   1. **Add** full-text search on name, description, path
7. **IF** hierarchical query (parent_id filter):
   1. **Optimize** with indexed path column
8. **Execute** query with pagination
9. **Count** total results for OData @odata.count
10. **FOR EACH** category result:
    1. **Format** as GTS entity with type, id, entity fields
    2. **IF** $expand includes children:
       1. **Fetch** immediate children
       2. **Include** in response
11. **RETURN** OData response with categories and metadata

**Algorithm: Get Category Path**

Input: security_ctx, category_id  
Output: Full category path from root to target

1. **Validate** SecurityCtx and fetch category
2. **Initialize** path array
3. **Set** current_category to target category
4. **WHILE** current_category has parent_id:
   1. **Add** current_category to path
   2. **Fetch** parent category
   3. **Set** current_category to parent
   4. **IF** loop count exceeds max depth (safety check):
      1. **RETURN** 500 error: "Corrupted category hierarchy"
5. **Add** root category to path
6. **Reverse** path array (root to target)
7. **RETURN** category path with names and IDs

---

## D. States

### 1. State Machines (Optional)

**Category Lifecycle States**:

- **ACTIVE** - Category is available and can be assigned to entities
- **ARCHIVED** - Category is hidden but entities can still reference it (read-only)
- **DELETED** - Category marked for deletion (soft delete with retention period)

**State Transitions**:
- ACTIVE → ARCHIVED (when administrator archives unused categories)
- ARCHIVED → ACTIVE (when reactivating archived category)
- ACTIVE/ARCHIVED → DELETED (when administrator deletes category)
- DELETED cannot transition (permanent after retention period)

**Rules**:
- Cannot delete category with active children (must cascade or archive)
- Archived categories not shown in UI but entities can keep references
- Deleted categories trigger reassignment of all entity references

---

## E. Technical Details

### 1. High-Level DB Schema

**Table: categories**

```
categories
├── id (TEXT PRIMARY KEY) - GTS identifier
├── tenant_id (UUID NOT NULL) - Tenant isolation
├── type (TEXT NOT NULL) - Category GTS type (category.v1~, category.v1~widget, etc.)
├── name (TEXT NOT NULL) - Category display name
├── description (TEXT) - Optional description
├── parent_id (TEXT) - Reference to parent category (NULL for root)
├── path (TEXT NOT NULL) - Materialized path for efficient queries (e.g., "/root/child1/child2")
├── depth (INTEGER NOT NULL) - Hierarchy depth (0 for root)
├── sort_order (INTEGER DEFAULT 0) - Display order within parent
├── icon (TEXT) - Optional icon identifier
├── color (TEXT) - Optional UI color code
├── metadata (JSONB) - Additional domain-specific metadata
├── state (TEXT DEFAULT 'ACTIVE') - Lifecycle state
├── created_at (TIMESTAMPTZ NOT NULL)
├── updated_at (TIMESTAMPTZ NOT NULL)
├── created_by (UUID NOT NULL) - User who created category
├── FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE CASCADE
```

**Indexes**:
- `idx_categories_tenant_type` - (tenant_id, type) for fast filtering by domain
- `idx_categories_parent` - (parent_id, sort_order) for child queries
- `idx_categories_path` - (path) GIN index for hierarchical queries
- `idx_categories_search` - (name, description) GIN for full-text search
- `idx_categories_state` - (state, tenant_id) for filtering active categories

**Unique Constraints**:
- `uq_category_name_parent` - (tenant_id, parent_id, name) prevents duplicate names in same parent

**Relationships**:
- Self-referential: categories.parent_id → categories.id
- Referenced by all GTS entity tables via category_id field (optional foreign key)

### 2. Database Operations

**Query Patterns**:

1. **Get Root Categories**: `SELECT * FROM categories WHERE tenant_id = ? AND parent_id IS NULL AND state = 'ACTIVE' ORDER BY sort_order`

2. **Get Children**: `SELECT * FROM categories WHERE tenant_id = ? AND parent_id = ? AND state = 'ACTIVE' ORDER BY sort_order`

3. **Get Subtree**: `SELECT * FROM categories WHERE tenant_id = ? AND path LIKE '/root/parent/%' AND state = 'ACTIVE'`

4. **Count Entity Usage**: `SELECT COUNT(*) FROM widgets WHERE tenant_id = ? AND category_id = ?` (similar for other entity types)

5. **Search Categories**: `SELECT * FROM categories WHERE tenant_id = ? AND state = 'ACTIVE' AND (name ILIKE ? OR description ILIKE ?) ORDER BY name LIMIT ? OFFSET ?`

**Transaction Requirements**:
- Category creation/update within single transaction
- Hierarchy updates require serializable isolation to prevent race conditions
- Cascade deletes wrapped in transaction with entity reassignment

**Performance Considerations**:
- Materialized path enables efficient subtree queries without recursion
- Depth field allows quick validation of max hierarchy depth
- Separate indexes for different query patterns (parent-child, path-based, search)

### 3. Access Control

**SecurityCtx Usage**:
- All category operations require SecurityCtx with tenant_id
- Automatic tenant isolation applied to all queries via WHERE tenant_id = ?
- No cross-tenant category access allowed

**Permission Checks**:
- `analytics:gts_write` - Required for POST, PUT, PATCH operations
- `analytics:gts_read` - Required for GET operations
- `analytics:gts_delete` - Required for DELETE operations

**Row-Level Security**:
- Database enforces tenant_id matching in all queries
- Application validates SecurityCtx before database access
- No shared categories between tenants (tenant-specific hierarchies only)

**Special Cases**:
- Global/system categories managed by Platform Administrator with special scope
- Template marketplace categories may have cross-tenant visibility (read-only)

### 4. Error Handling

**Error Scenarios**:

1. **Circular Reference**: When updating parent_id would create a cycle
   - Detect: Traverse parent chain from new parent, check if contains current category
   - Response: 400 Bad Request with error details
   - Fallback: Reject operation, maintain existing hierarchy

2. **Duplicate Name**: When creating category with existing name in same parent
   - Detect: Database unique constraint violation
   - Response: 409 Conflict with suggestion to use different name
   - Fallback: Return existing category if idempotent creation requested

3. **Category In Use**: When deleting category with children or entity references
   - Detect: Count children and referencing entities
   - Response: 400 Bad Request with usage details and cascade option
   - Fallback: Offer to archive instead of delete

4. **Max Depth Exceeded**: When hierarchy depth exceeds configured limit
   - Detect: Check depth field before insert/update
   - Response: 400 Bad Request with max depth limit
   - Fallback: Suggest flattening hierarchy or using different parent

5. **Parent Not Found**: When parent_id references non-existent category
   - Detect: Foreign key constraint or explicit check
   - Response: 404 Not Found for parent category
   - Fallback: Offer to create as root category

6. **Cross-Domain Parent**: When assigning parent from different domain type
   - Detect: Compare type prefixes of category and parent
   - Response: 400 Bad Request with explanation
   - Fallback: Suggest correct domain category or create new hierarchy

**Validation Logic**:
- JSON Schema validation for category payload
- Domain type validation (type must match parent's domain)
- Name sanitization and length limits
- Path validation and normalization

---

## F. Validation & Implementation

### 1. Testing Scenarios

**Unit Tests**:
- Category registration with valid payload
- Hierarchy path calculation
- Circular reference detection
- Name uniqueness validation
- Depth limit enforcement
- JSON Patch operations on category fields

**Integration Tests**:
- Create category hierarchy with multiple levels
- Move category to different parent
- Delete category with cascade
- Search categories with OData filters
- Assign entities to categories
- Get category path from root
- Archive and reactivate categories

**Edge Cases**:
- Create root category (parent_id = null)
- Move category to become root
- Attempt circular parent assignment
- Delete category with 1000+ descendants (cascade performance)
- Concurrent hierarchy updates (race conditions)
- Unicode and special characters in category names
- Very deep hierarchies (10+ levels)
- Categories with no entities vs heavily used categories

**Performance Tests**:
- Search across 10,000+ categories
- Subtree query for 5-level hierarchy with 1000 nodes
- Concurrent category updates on same parent
- Batch category creation (100+ categories)

**Security Tests**:
- Cross-tenant category access attempts
- Missing SecurityCtx handling
- Invalid JWT token scenarios
- Insufficient permissions for delete operation
- SQL injection in category name/description

### 2. OpenSpec Changes Plan

**Total Changes**: 8
**Estimated Effort**: 28 hours with AI agent

### Change 001: Category DB Schema & Table

**Status**: ⏳ NOT_STARTED

**Scope**: Create categories database table with indexes and constraints

**Tasks**:
- [ ] Create migration file for categories table
- [ ] Add all columns with correct types and constraints
- [ ] Create indexes for tenant_id, parent_id, path, search
- [ ] Add unique constraint for name within parent
- [ ] Add foreign key for parent_id self-reference
- [ ] Write migration tests

**Files**:
- Backend: `modules/analytics/src/db/migrations/`
- Tests: `modules/analytics/tests/db/schema_categories.rs`

**Dependencies**: None

**Effort**: 3 hours (AI agent)

---

### Change 002: Category Domain Model

**Status**: ⏳ NOT_STARTED

**Scope**: Define Category struct and domain types

**Tasks**:
- [ ] Create Category struct with all fields
- [ ] Implement domain type for each category variant (9 types)
- [ ] Add serialization/deserialization
- [ ] Implement builder pattern
- [ ] Add validation methods
- [ ] Create type conversion traits

**Files**:
- Backend: `modules/analytics/src/domain/category/mod.rs`
- Backend: `modules/analytics/src/domain/category/types.rs`
- Tests: `modules/analytics/tests/domain/category_model.rs`

**Dependencies**: Change 001

**Effort**: 3 hours (AI agent)

---

### Change 003: Category Repository

**Status**: ⏳ NOT_STARTED

**Scope**: Database access layer for categories

**Tasks**:
- [ ] Create CategoryRepository trait
- [ ] Implement CRUD operations
- [ ] Add hierarchical query methods (get_children, get_path, get_subtree)
- [ ] Implement search with OData filters
- [ ] Add circular reference detection
- [ ] Write repository tests with test database

**Files**:
- Backend: `modules/analytics/src/repositories/category.rs`
- Tests: `modules/analytics/tests/repositories/category_repo.rs`

**Dependencies**: Change 002

**Effort**: 4 hours (AI agent)

---

### Change 004: Category Service Layer

**Status**: ⏳ NOT_STARTED

**Scope**: Business logic for category operations

**Tasks**:
- [ ] Create CategoryService with SecurityCtx
- [ ] Implement register_category method
- [ ] Implement update_hierarchy method
- [ ] Implement delete_category with cascade
- [ ] Add search_categories with OData
- [ ] Add get_category_path method
- [ ] Implement validation logic
- [ ] Write service layer tests

**Files**:
- Backend: `modules/analytics/src/services/category.rs`
- Tests: `modules/analytics/tests/services/category_service.rs`

**Dependencies**: Change 003

**Effort**: 4 hours (AI agent)

---

### Change 005: Category GTS Integration

**Status**: ⏳ NOT_STARTED

**Scope**: Integrate categories with GTS Registry

**Tasks**:
- [ ] Register 9 category GTS types in registry
- [ ] Implement GTS entity conversion for categories
- [ ] Add category routing in GTS core
- [ ] Handle category-specific OData queries
- [ ] Implement $expand for children
- [ ] Write GTS integration tests

**Files**:
- Backend: `modules/analytics/src/gts/handlers/category.rs`
- Backend: `modules/analytics/src/gts/routing.rs`
- Tests: `modules/analytics/tests/gts/category_integration.rs`

**Dependencies**: Change 004

**Effort**: 4 hours (AI agent)

---

### Change 006: Category REST API Endpoints

**Status**: ⏳ NOT_STARTED

**Scope**: REST API implementation via GTS unified endpoints

**Tasks**:
- [ ] Ensure /gts endpoints handle category operations
- [ ] Add category-specific validation middleware
- [ ] Implement JSON Patch for hierarchy updates
- [ ] Add response serialization for categories
- [ ] Write API endpoint tests

**Files**:
- Backend: `modules/analytics/src/api/routes/gts.rs`
- Tests: `modules/analytics/tests/api/category_endpoints.rs`

**Dependencies**: Change 005

**Effort**: 3 hours (AI agent)

---

### Change 007: Category Search & Indexing

**Status**: ⏳ NOT_STARTED

**Scope**: Full-text search and filtering for categories

**Tasks**:
- [ ] Implement full-text search on name and description
- [ ] Add hierarchical path queries
- [ ] Optimize index usage for performance
- [ ] Add category aggregation queries
- [ ] Write search performance tests

**Files**:
- Backend: `modules/analytics/src/search/category.rs`
- Tests: `modules/analytics/tests/search/category_search.rs`

**Dependencies**: Change 003

**Effort**: 3 hours (AI agent)

---

### Change 008: Category E2E Tests

**Status**: ⏳ NOT_STARTED

**Scope**: End-to-end integration tests for complete category workflows

**Tasks**:
- [ ] Test create category hierarchy (5 levels)
- [ ] Test move category between parents
- [ ] Test delete with cascade
- [ ] Test assign entities to categories
- [ ] Test search and filter workflows
- [ ] Test cross-tenant isolation
- [ ] Test concurrent updates
- [ ] Test error scenarios

**Files**:
- Tests: `testing/e2e/modules/analytics/test_categories.py`

**Dependencies**: Change 006

**Effort**: 4 hours (AI agent)

---

**Status Legend**:
- ⏳ **NOT_STARTED** - Change not yet started
- 🔄 **IN_PROGRESS** - Change currently being implemented
- ✅ **COMPLETED** - Change implemented and archived
