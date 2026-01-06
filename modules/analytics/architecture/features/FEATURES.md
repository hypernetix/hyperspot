# Features: Analytics

**Status Overview**: 22 features total (2 completed, 13 in progress, 7 not started)

**Last Updated**: 2026-01-06

**Design Principle**: Each domain type is self-contained with its own DB schema, CRUD operations, and search logic.

**CRITICAL**: [feature-init-module](feature-init-module/) MUST be implemented FIRST before any other features.

---

## Features

### 0. [feature-init-module](feature-init-module/) ✅ CRITICAL
**Purpose**: Initialize analytics module structure following SDK pattern with ModKit compliance
**Status**: IMPLEMENTED
**Depends On**: None
**Blocks**: All other features

**Scope**:
- Create SDK crate with empty API trait (no business methods)
- Create module crate with empty layer folders (domain/, infra/, api/)
- Define basic configuration with defaults
- Register module with ModKit (#[modkit::module])
- Create stub local client (no implementations)
- Integrate into workspace (Cargo.toml)

---

### 1. [feature-gts-core](feature-gts-core/) ✅ CRITICAL
**Purpose**: Thin routing layer for GTS unified API - delegates to domain-specific features
**Status**: IMPLEMENTED
**Depends On**: None
**Blocks**: All domain type features

**Scope**:
- GTS API routing (`/gts`, `/gts/{id}`) - routes to domain features
- Common middleware (auth, tenant context injection)
- Request validation (structure only, not domain logic)
- **NO database layer** - purely routing
- **NO domain-specific logic** - delegates to features

---

### 2. [feature-schema-query-returns](feature-schema-query-returns/) 🔄 HIGH
**Purpose**: Query result schema type for paginated OData responses
**Status**: IN_PROGRESS
**Depends On**: [feature-gts-core](feature-gts-core/)
**Blocks**: [feature-query-definitions](feature-query-definitions/)

**Scope**:
- Schema GTS type: `schema.v1~` (base) + `schema.v1~query_returns.v1~`
- Query result schema DB tables
- Schema validation for paginated results
- Scalar-only field enforcement
- Custom search/query for schemas

---

### 3. [feature-schema-template-config](feature-schema-template-config/) ⏳ HIGH
**Purpose**: Template configuration schema type for widget settings
**Status**: NOT_STARTED
**Depends On**: [feature-gts-core](feature-gts-core/)
**Blocks**: [feature-widget-templates](feature-widget-templates/)

**Scope**:
- Schema GTS type: `schema.v1~template_config.v1~`
- Template config schema DB tables
- Widget configuration validation
- Schema-specific indexing

---

### 4. [feature-schema-values](feature-schema-values/) 🔄 HIGH
**Purpose**: Value lists schema for UI selectors (dropdowns, pickers)
**Status**: IN_PROGRESS
**Depends On**: [feature-gts-core](feature-gts-core/)
**Blocks**: [feature-values-selector-templates](feature-values-selector-templates/)

**Scope**:
- Schema GTS type: `schema.v1~values.v1~`
- Values schema DB tables
- Value list validation
- Custom indexing for value lists

---

### 5. [feature-query-definitions](feature-query-definitions/) 🔄 CRITICAL
**Purpose**: Query type registration and metadata management
**Status**: IN_PROGRESS
**Depends On**: [feature-gts-core](feature-gts-core/), [feature-schema-query-returns](feature-schema-query-returns/)
**Blocks**: [feature-query-execution](feature-query-execution/)

**Scope**:
- Query GTS type: `query.v1~` (main query type)
- Query definition DB tables
- Query metadata (category, returns_schema_id, capabilities_id)
- Query registration API
- Custom search for queries

---

### 6. [feature-query-capabilities](feature-query-capabilities/) 🔄 HIGH
**Purpose**: OData capabilities annotations for query restrictions
**Status**: IN_PROGRESS
**Depends On**: [feature-gts-core](feature-gts-core/)
**Blocks**: [feature-query-definitions](feature-query-definitions/)

**Scope**:
- Query capabilities GTS type: `query_capabilities.v1~`
- Capabilities DB tables (FilterRestrictions, SortRestrictions, etc.)
- OData annotations management
- Capability indexing

---

### 7. [feature-query-values](feature-query-values/) 🔄 HIGH
**Purpose**: Default OData query options for queries
**Status**: IN_PROGRESS
**Depends On**: [feature-gts-core](feature-gts-core/)
**Blocks**: [feature-query-definitions](feature-query-definitions/)

**Scope**:
- Query values GTS type: `query.v1~values.v1~`
- Query values DB tables
- Default OData options storage
- Values search logic

---

### 8. [feature-plugins](feature-plugins/) 🔄 CRITICAL
**Purpose**: Plugin management system for query adapters and datasource plugins
**Status**: IN_PROGRESS
**Depends On**: [feature-gts-core](feature-gts-core/)
**Blocks**: [feature-query-execution](feature-query-execution/)

**Scope**:
- Plugin registration and discovery
- Plugin metadata storage (DB tables)
- Query adapter management (OData, REST, Prometheus, Elasticsearch, etc.)
- Datasource plugin lifecycle (enable/disable/update)
- Plugin configuration storage
- Plugin security and sandboxing
- Contract format adapters (3rd-party to native)
- Plugin health monitoring
- Plugin API endpoints

---

### 9. [feature-query-execution](feature-query-execution/) 🔄 CRITICAL
**Purpose**: Query execution engine with OData v4 support using registered plugins
**Status**: IN_PROGRESS
**Depends On**: [feature-query-definitions](feature-query-definitions/), [feature-query-capabilities](feature-query-capabilities/), [feature-query-values](feature-query-values/), [feature-plugins](feature-plugins/)
**Blocks**: [feature-datasources](feature-datasources/)

**Scope**:
- OData v4 execution engine (GET/POST)
- Query metadata endpoint (`/queries/{id}/$metadata`)
- Plugin invocation and orchestration
- JWT generation for external APIs
- Query result caching layer
- Multi-datasource orchestration
- `/queries/{id}` and `/queries/{id}/$query` endpoints

---

### 10. [feature-widget-templates](feature-widget-templates/) 🔄 HIGH
**Purpose**: Widget visualization templates (charts, tables, maps)
**Status**: IN_PROGRESS
**Depends On**: [feature-gts-core](feature-gts-core/), [feature-schema-template-config](feature-schema-template-config/)
**Blocks**: [feature-widget-items](feature-widget-items/)

**Scope**:
- Template GTS type: `template.v1~` (base) + `template.v1~widget.v1~`
- Widget template DB tables
- JavaScript bundle upload/download (`/templates/{id}/bundle`)
- Chart type library (line, bar, pie, scatter, heatmap)
- Template configuration schemas
- Custom template search

---

### 11. [feature-values-selector-templates](feature-values-selector-templates/) ⏳ HIGH
**Purpose**: UI control templates for parameter inputs (dropdowns, autocomplete, pickers)
**Status**: NOT_STARTED
**Depends On**: [feature-gts-core](feature-gts-core/), [feature-schema-values](feature-schema-values/)
**Blocks**: [feature-datasources](feature-datasources/)

**Scope**:
- Template GTS type: `template.v1~values_selector.v1~`
- Values selector DB tables
- UI control templates (dropdowns, autocomplete, pickers)
- Selector configuration schemas
- Custom indexing for selectors

---

### 12. [feature-datasources](feature-datasources/) 🔄 HIGH
**Purpose**: Datasource instances with query + parameter binding
**Status**: IN_PROGRESS
**Depends On**: [feature-gts-core](feature-gts-core/), [feature-query-execution](feature-query-execution/), [feature-values-selector-templates](feature-values-selector-templates/)
**Blocks**: [feature-widget-items](feature-widget-items/)

**Scope**:
- Datasource GTS type: `datasource.v1~`
- Datasource DB tables
- Query + parameters binding
- Values selector integration for parameter inputs
- Runtime parameter injection
- Datasource reusability
- Custom datasource search

---

### 13. [feature-widget-items](feature-widget-items/) ⏳ HIGH
**Purpose**: Widget item instances for data visualizations
**Status**: NOT_STARTED
**Depends On**: [feature-gts-core](feature-gts-core/), [feature-widget-templates](feature-widget-templates/), [feature-datasources](feature-datasources/)
**Blocks**: [feature-dashboard-layouts](feature-dashboard-layouts/)

**Scope**:
- Item GTS type: `item.v1~` (base) + `item.v1~widget.v1~`
- Widget item DB tables
- Widget instance lifecycle (create, update, delete)
- Widget state management
- Widget refresh strategies (real-time, polling, manual)
- Datasource + template binding
- Widget-specific indexing

---

### 14. [feature-group-items](feature-group-items/) ⏳ MEDIUM
**Purpose**: Group item containers for organizing widgets
**Status**: NOT_STARTED
**Depends On**: [feature-gts-core](feature-gts-core/)
**Blocks**: [feature-dashboard-layouts](feature-dashboard-layouts/)

**Scope**:
- Item GTS type: `item.v1~group.v1~`
- Group item DB tables
- Container management (children array)
- Group configuration
- Hierarchical structure support
- Group-specific indexing

---

### 15. [feature-dashboard-layouts](feature-dashboard-layouts/) 🔄 HIGH
**Purpose**: Dashboard layout type for real-time dashboards
**Status**: IN_PROGRESS
**Depends On**: [feature-gts-core](feature-gts-core/), [feature-widget-items](feature-widget-items/), [feature-group-items](feature-group-items/)
**Blocks**: [feature-dashboards](feature-dashboards/)

**Scope**:
- Layout GTS type: `layout.v1~` (base) + `layout.v1~dashboard.v1~`
- Dashboard layout DB tables
- Real-time layout properties (auto-refresh, live updates)
- Layout-item relationships
- Dashboard-specific indexing (by user, by tenant, by category)

---

### 16. [feature-report-layouts](feature-report-layouts/) ⏳ MEDIUM
**Purpose**: Report layout type for scheduled reports
**Status**: NOT_STARTED
**Depends On**: [feature-gts-core](feature-gts-core/), [feature-widget-items](feature-widget-items/), [feature-group-items](feature-group-items/)
**Blocks**: [feature-reporting](feature-reporting/)

**Scope**:
- Layout GTS type: `layout.v1~report.v1~`
- Report layout DB tables
- Scheduled report properties (exportable, scheduled)
- Report-specific indexing (by schedule, by format)
- Report parameter configuration

---

### 17. [feature-categories](feature-categories/) ⏳ MEDIUM
**Purpose**: Hierarchical organization system for all GTS entities
**Status**: NOT_STARTED
**Depends On**: [feature-gts-core](feature-gts-core/)
**Blocks**: None

**Scope**:
- Category GTS types (9 types: base + 8 domain categories)
- Category DB tables (single unified table - no domain-specific logic)
- Widget libraries (reusable collections)
- Template libraries (marketplace)
- Datasource libraries (preconfigured connectors)
- Query libraries (shareable definitions)
- Hierarchical classification

---

### 18. [feature-tenancy-enablement](feature-tenancy-enablement/) 🔄 HIGH
**Purpose**: Multi-tenant access control and automatic dependency enablement
**Status**: IN_PROGRESS
**Depends On**: [feature-gts-core](feature-gts-core/)
**Blocks**: None

**Scope**:
- Tenant enablement configuration via `/gts/{id}/enablement`
- Automatic dependency enablement (query → schema, template → config_schema)
- Tenant isolation enforcement
- Enablement API (GET/PUT/PATCH)
- JSON Patch support for enablement updates
- Enablement DB tables

---

### 19. [feature-dashboards](feature-dashboards/) 🔄 HIGH
**Purpose**: Dashboard UI management (grid layout, drag-and-drop, templates)
**Status**: IN_PROGRESS
**Depends On**: [feature-dashboard-layouts](feature-dashboard-layouts/)
**Blocks**: [feature-reporting](feature-reporting/), [feature-export-sharing](feature-export-sharing/)

**Scope**:
- Dashboard CRUD operations (business logic layer)
- Grid-based responsive layouts
- Drag-and-drop widget positioning
- Dashboard templates
- Version history
- Dashboard-specific business logic (NOT layout storage)

---

### 20. [feature-reporting](feature-reporting/) 🔄 MEDIUM
**Purpose**: Report generation with scheduling and delivery via platform services
**Status**: IN_PROGRESS
**Depends On**: [feature-report-layouts](feature-report-layouts/), [feature-dashboards](feature-dashboards/)
**Blocks**: None

**Scope**:
- Report generation (on-demand, scheduled)
- Report templates based on dashboards
- Multi-format export (PDF, CSV, Excel)
- Report history and versioning
- Schedule management via **Hyperspot Platform Scheduling Service**
- Report delivery via **Hyperspot Platform Email Service**
- Report access control
- Report parameters and filters

---

### 21. [feature-export-sharing](feature-export-sharing/) ⏳ LOW
**Purpose**: Dashboard and widget sharing and embedding
**Status**: NOT_STARTED
**Depends On**: [feature-dashboards](feature-dashboards/)
**Blocks**: None

**Scope**:
- Dashboard export to multiple formats (ad-hoc, no scheduling)
- Dashboard sharing with permissions
- Public/private dashboard URLs
- Widget embedding in external apps (iframe, SDK)
- Share links with expiration
- Sharing access control

---

## Implementation Order

**Parallelization Strategy**: UI and business features can develop against mocked GTS types, enabling massive parallelization after Phase 1.

### Phase 0: Module Initialization (REQUIRED - COMPLETED)

0. [`feature-init-module`](feature-init-module/) - ✅ **IMPLEMENTED** - Module structure with SDK pattern

**Status**: ✅ Complete - Module foundation ready for business features

---

### Phase 1: Foundation (1 feature)

1. [`feature-gts-core`](feature-gts-core/) - Thin routing layer + GTS unified API

**Blocks**: All other features (provides core routing infrastructure)

---

### Phase 2: Domain Types & Data Layer (14 features - ALL PARALLEL)

All features depend only on `feature-gts-core`. Each owns its GTS types, DB schema, and CRUD operations. Can be developed in parallel using mocked dependencies where needed.

**Schema Types**:
2. [`feature-schema-query-returns`](feature-schema-query-returns/) - Query result schemas
3. [`feature-schema-template-config`](feature-schema-template-config/) - Widget config schemas
4. [`feature-schema-values`](feature-schema-values/) - Values list schemas

**Query Types**:
5. [`feature-query-capabilities`](feature-query-capabilities/) - OData capability annotations
6. [`feature-query-values`](feature-query-values/) - Default OData options
7. [`feature-query-definitions`](feature-query-definitions/) - Query type registration (can use mocked schemas initially)

**Template Types**:
8. [`feature-widget-templates`](feature-widget-templates/) - Widget visualization templates
9. [`feature-values-selector-templates`](feature-values-selector-templates/) - UI control templates

**Item Types**:
10. [`feature-widget-items`](feature-widget-items/) - Widget instances (can use mocked templates/datasources)
11. [`feature-group-items`](feature-group-items/) - Container items

**Layout Types**:
12. [`feature-dashboard-layouts`](feature-dashboard-layouts/) - Dashboard layouts (can use mocked items)
13. [`feature-report-layouts`](feature-report-layouts/) - Report layouts (can use mocked items)

**Data & Infrastructure**:
14. [`feature-datasources`](feature-datasources/) - Datasource instances (can use mocked query-execution)
15. [`feature-categories`](feature-categories/) - Hierarchical organization
16. [`feature-tenancy-enablement`](feature-tenancy-enablement/) - Multi-tenant access control

---

### Phase 3: Runtime Engines + Business Features (5 features - ALL PARALLEL)

**Runtime Engines** (real data flows):
17. [`feature-plugins`](feature-plugins/) - Plugin management system
18. [`feature-query-execution`](feature-query-execution/) - OData execution engine (depends on plugins + query types)

**Business & UI Features** (can use mocks during development):
19. [`feature-dashboards`](feature-dashboards/) - Dashboard UI management (uses mocked layouts initially)
20. [`feature-reporting`](feature-reporting/) - Report generation + scheduling (uses mocked layouts initially)
21. [`feature-export-sharing`](feature-export-sharing/) - Sharing and embedding (uses mocked dashboards initially)

**Note**: Business/UI features can start development in parallel with runtime engines by using mocked GTS data. Integration happens when runtime engines are ready.

---

## Status Legend

- ✅ **IMPLEMENTED** - Feature complete and in production
- 🔄 **IN_PROGRESS** - Currently being developed
- ⏳ **NOT_STARTED** - Planned but not yet started
- 🚫 **BLOCKED** - Blocked by dependencies or design issues

---

## GTS Type Distribution (26 types → 20 features)

| Feature | GTS Types Count | Types Owned |
|---------|----------------|-------------|
| feature-gts-core | 0 | (routing only) |
| feature-schema-query-returns | 2 | schema.v1~, schema.v1~query_returns.v1~ |
| feature-schema-template-config | 1 | schema.v1~template_config.v1~ |
| feature-schema-values | 1 | schema.v1~values.v1~ |
| feature-query-definitions | 1 | query.v1~ |
| feature-query-capabilities | 1 | query_capabilities.v1~ |
| feature-query-values | 1 | query.v1~values.v1~ |
| feature-plugins | 0 | (plugin infrastructure) |
| feature-query-execution | 0 | (runtime engine) |
| feature-widget-templates | 2 | template.v1~, template.v1~widget.v1~ |
| feature-values-selector-templates | 1 | template.v1~values_selector.v1~ |
| feature-datasources | 1 | datasource.v1~ |
| feature-widget-items | 2 | item.v1~, item.v1~widget.v1~ |
| feature-group-items | 1 | item.v1~group.v1~ |
| feature-dashboard-layouts | 2 | layout.v1~, layout.v1~dashboard.v1~ |
| feature-report-layouts | 1 | layout.v1~report.v1~ |
| feature-categories | 9 | category.v1~ + 8 domain categories |
| feature-tenancy-enablement | 0 | (business logic) |
| feature-dashboards | 0 | (UI logic) |
| feature-reporting | 0 | (business logic) |
| feature-export-sharing | 0 | (business logic) |

**Total**: 26 GTS types + 5 runtime/business features = 21 features

---

## Design Principles

**1. Self-Contained Features**:
Each domain type feature owns:
- GTS type definitions
- DB tables and schema
- CRUD operations
- Search/query implementation
- Domain-specific validation
- Custom indexing strategy

**2. Thin Core Layer**:
[`feature-gts-core`](feature-gts-core/) is purely routing - no DB, no domain logic

**3. Clear Separation**:
- **Type Features** - Own GTS types + DB + CRUD  
  Examples: [`feature-schema-query-returns`](feature-schema-query-returns/), [`feature-query-definitions`](feature-query-definitions/), [`feature-widget-templates`](feature-widget-templates/)
- **Runtime Features** - Execution engines  
  Examples: [`feature-query-execution`](feature-query-execution/), [`feature-plugins`](feature-plugins/)
- **Business Features** - Business logic  
  Examples: [`feature-dashboards`](feature-dashboards/), [`feature-reporting`](feature-reporting/)

**4. Granular Dependencies**:
Dependencies are explicit at type level for better parallel development
