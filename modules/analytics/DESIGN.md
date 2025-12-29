# Hyperspot Analytics - Design Document

**Version**: 1.0  
**Platform**: Hyperspot with GTS Integration

---

## Table of Contents

1. [Overview](#overview)
2. [Domain Model Types](#domain-model-types)
3. [Behavior](#behavior)

---

## Overview

**Hyperspot Analytics** is a comprehensive framework for creating, managing, and displaying data visualizations and reports within the **Hyperspot Platform**. Built on **GTS (Generic Type System)** and a plugin architecture, it provides a flexible and extensible foundation for data analytics.

**GTS Specification**: All data schemas and type definitions follow the [GTS Specification](https://github.com/GlobalTypeSystem/gts-spec).

### Key Capabilities

- **Rich Visualization**: Charts, tables, maps, custom widgets
- **Interactive Features**: Drilldowns, tooltips, filtering
- **Flexible Layout**: Responsive grid-based dashboards and reports
- **Plugin Architecture**: Dynamic datasource registration via plugins
- **GTS Integration**: Native integration with Hyperspot GTS system
- **Export & Sharing**: Multiple formats (PDF, CSV, Excel)
- **Extensibility**: Third-party plugins and custom widgets
- **Multi-tenancy**: Full tenant isolation via SecurityCtx

### Architecture Principles

1. **Security First**: SecurityCtx enforced at every level
2. **Plugin-Based Extensibility**: Datasources as dynamically registered plugins
3. **GTS Native**: All plugin communication via GTS
4. **Strongly Typed**: All configuration validated with schemas
5. **OLAP Performance**: Centralized Analytics DWH for aggregations
6. **Event-Driven Data Ingestion**: Async data pipeline
7. **Modular Design**: Reusable layouts, items, widgets, templates
8. **API-First**: REST API with OpenAPI specification
9. **Horizontal Scalability**: Stateless services, distributed architecture
10. **Tenant Isolation**: Complete data separation per tenant

### Architecture Overview

The following diagram illustrates the layered architecture of Hyperspot Analytics:

![Architecture Overview](openspec/diagrams/architecture.drawio.svg)

The architecture consists of four distinct layers:

- **PRESENTATION**: Dashboards, Reports, Widgets, Datasources & Templates, and Admin Panel - all user-facing components with interactive features
- **FLEXIBLE API**: REST API with Multi-Tenancy support, Queries execution engine, GTS Registry for type management, and OData v4 compatibility layer
- **EXTENSIBLE BUSINESS LOGIC**: GTS Types & Instances management, Query plugins for different data sources, and Adapter plugins for protocol translations
- **DATA**: OLAP (Analytics Data Warehouse), OLTP (PostgreSQL), and ETL pipelines for data ingestion and transformation

### GTS-Based Data Layer

The platform uses **GTS (Generic Type System)** as the foundation for all data definitions and plugin communication. GTS provides:

- **Type Safety**: All datasource schemas are validated at runtime
- **Extensibility**: Plugins can define custom data structures
- **Interoperability**: Standardized communication protocol
- **Versioning**: Schema evolution support

### JWT-Based Tenancy Context Propagation

**Standard**: All query executions automatically include tenant context via JWT tokens. This behavior is **always enforced** and cannot be disabled.

#### Automatic JWT Generation

When Analytics Service executes any query to external APIs, it **always** generates a JWT token containing:
- `sub` - User identifier from SecurityCtx
- `tenant_id` - Current tenant identifier
- `org_id` - Organization identifier  
- `iat`, `exp` - Timestamps
- `scopes` - Access scopes

The JWT is **always** placed in the `Authorization: Bearer` header for all query executions.

#### Plugin Influence

Query plugins **can** influence JWT generation:

1. **Add custom claims** - Plugin can inject additional claims before signing
2. **Modify expiration** - Adjust token lifetime based on query type
3. **Add scopes** - Include plugin-specific scopes

**Plugin Hook:**
```rust
trait QueryPlugin {
    fn before_jwt_sign(&self, ctx: &SecurityCtx, claims: &mut JwtClaims) -> Result<()>;
}
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

#### For External API Providers

Your API **must**:

1. **Validate JWT signature** - Use shared signing key or public key
2. **Extract tenant_id** - Read from JWT claims
3. **Filter data by tenant** - Return only tenant-scoped data
4. **Return 403** if tenant context is invalid or missing

**Example validation (pseudocode):**
```rust
let jwt = extract_bearer_token(request)?;
let claims = validate_and_decode_jwt(jwt)?;
let tenant_id = claims.tenant_id;

// Filter data by tenant
let data = database.query("SELECT * FROM sales WHERE tenant_id = ?", tenant_id)?;
```

#### Security Guarantees

- **Cryptographic integrity** - JWT signature prevents tampering
- **Cannot forge tenant_id** - Claims are signed together
- **Automatic expiration** - Tokens are short-lived (default: 5 minutes)
- **Audit trail** - All claims logged with every request

### User Roles & Personas

Hyperspot Analytics serves multiple user personas with distinct responsibilities and access patterns:

#### 1. **Platform Administrator**
**Responsibilities:**
- Manage platform infrastructure and configuration
- Configure tenant isolation and security policies
- Monitor system health and performance
- Manage user access and permissions across tenants
- Configure DWH connections and storage backends

**Access:** Full administrative access to all layers, tenant management, system configuration

---

#### 2. **Data Engineer / ETL Developer**
**Responsibilities:**
- Design and implement ETL pipelines for data ingestion
- Configure Analytics DWH (OLAP) and data schemas
- Set up event-driven data ingestion workflows
- Optimize query performance and data aggregations
- Manage data quality and transformation rules

**Access:** Data Layer management, GTS Schema registration, query optimization tools

**Typical Tasks:**
- Register GTS schemas for data structures
- Configure PostgreSQL and DWH connections
- Build ETL adapters for external data sources
- Set up real-time event streaming pipelines

---

#### 3. **Plugin Developer**
**Responsibilities:**
- Develop custom datasource plugins
- Implement contract adapters for external APIs
- Create GTS type extensions
- Build custom query implementations
- Develop integration connectors

**Access:** GTS Registry (type registration), Plugin API, development sandbox environments

**Typical Tasks:**
- Register custom query types via GTS
- Implement `contract_format` adapters (native, odata, rest)
- Develop datasource plugins with authentication
- Create reusable query parameter specifications

**Technologies:** Rust (modkit plugins), GTS specification, REST/OData protocols

---

#### 4. **Dashboard Designer / Business Analyst**
**Responsibilities:**
- Create and configure dashboards and reports
- Design data visualizations and widgets
- Configure filters, drilldowns, and interactive features
- Set up scheduled reports and exports
- Organize layouts and items for business users

**Access:** Presentation Layer, GTS Registry (instance creation: dashboards, reports, widgets), query execution

**Typical Tasks:**
- Register dashboard layouts via `/gts` API
- Configure widget items with datasources and templates
- Set up filtering and parameter controls
- Design responsive grid layouts
- Schedule report generation (PDF, CSV, Excel)

**Tools:** Web UI, REST API, OpenAPI specification

---

#### 5. **Template Developer / Frontend Developer**
**Responsibilities:**
- Develop custom widget templates
- Create reusable visualization components
- Implement UI/UX for interactive features
- Build custom chart types and data displays
- Optimize rendering performance

**Access:** Template registration, GTS Registry (template types), frontend development tools

**Typical Tasks:**
- Register widget templates via GTS
- Implement template rendering logic
- Define template configuration schemas
- Create values selector templates (dropdowns, pickers)
- Build responsive and accessible components

**Technologies:** Frontend frameworks, GTS template schemas, data binding APIs

---

#### 6. **System Integrator / ISV Partner**
**Responsibilities:**
- Embed Hyperspot Analytics into third-party products
- Configure white-label analytics solutions
- Integrate with existing authentication systems
- Customize branding and theming
- Implement multi-tenant configurations for SaaS products

**Access:** Full API access, tenant provisioning, configuration management, embedding SDKs

**Typical Tasks:**
- Embed dashboards via iframe or SDK
- Integrate with SSO/OAuth providers
- Configure tenant-specific branding
- Set up data isolation for customer tenants
- Implement usage metering and billing integration

**Integration Methods:** REST API, OpenAPI client generation, iframe embedding, JWT authentication

---

#### 7. **Tenant Administrator**
**Responsibilities:**
- Manage tenant-specific configurations
- Control user access within tenant scope
- Configure tenant-specific datasources and dashboards
- Manage tenant data and privacy settings
- Monitor tenant usage and quotas

**Access:** Tenant-scoped GTS Registry access, user management, tenant configuration

**Typical Tasks:**
- Register tenant-specific queries and datasources
- Configure tenant user roles and permissions
- Set up tenant data retention policies
- Manage tenant-specific categories and organization

---

#### 8. **End User / Business Consumer**
**Responsibilities:**
- View and interact with dashboards and reports
- Apply filters and perform data exploration
- Export data and reports
- Subscribe to scheduled reports
- Use search and drilldown features

**Access:** Read-only access to assigned dashboards and reports, query execution (read-only), export functions

**Typical Tasks:**
- Browse dashboards and reports
- Apply filters and time ranges
- Drill down into detailed data
- Export to PDF, CSV, Excel
- Receive scheduled email reports

---

#### 9. **API Consumer / Application Developer**
**Responsibilities:**
- Integrate analytics capabilities into applications
- Execute queries programmatically
- Fetch data for custom visualizations
- Implement analytics in mobile or desktop apps
- Build automation workflows

**Access:** REST API, query execution endpoints, OData queries, read access to GTS registry

**Typical Tasks:**
- Execute queries via `/queries/{id}` endpoint
- Fetch data with OData filtering and pagination
- Implement custom data visualizations
- Build analytics-driven automation
- Integrate with BI tools

**Technologies:** REST clients, OpenAPI/Swagger, OData libraries, programming SDKs

---

### Component Registration Flow

Hyperspot Analytics uses a three-tier registration system:

1. **Schema Registration** - Define data structures and validation rules
2. **Datasource Registration** - Register data query endpoints
3. **Template Registration** - Register UI visualization components

**Plugin Architecture**: Optional extensions for local implementations and contract adapters.

See the [Behavior](#behavior) section for detailed API specifications and workflows.

---

## Domain Model Types

All types are defined using GTS (Generic Type System) with JSON Schema format.

![Domain Model Types](openspec/diagrams/domain-model.drawio.svg)

The domain model consists of several interconnected type categories managed through the unified GTS Registry:

- **Schemas**: Define data structures and validation rules (Base, Query Returns, Template Config, Query Values, Query Params)
- **Queries**: Define data retrieval operations with API endpoints and parameter specifications
- **Categories**: Organize and group related GTS entities by domain or purpose
- **Templates**: Define rendering logic and configuration for visual components
- **Datasources**: Connect queries with runtime parameters and UI controls
- **Items**: Reusable building blocks for layouts (Widgets and Groups)
- **Layouts**: Organize items into Dashboards and Reports

### Schemas

Schemas define data structures and validation rules for types that are referenced by other types and instances. All schema types inherit from the base schema and must provide mock objects via `examples[0]`.

**Schema Types:**

- **Base Schema** (`gts://gts.hypernetix.hyperspot.ax.schema.v1~`)  
  Base type for all schema definitions. Requires mandatory `examples[0]` for validation.  
  📄 [`gts/types/schema/v1/base.schema.json`](gts/types/schema/v1/base.schema.json)

- **Query Returns Schema** (`gts://gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~`)  
  Schema for query result data. Defines paginated result sets with scalar-only field values (no nested objects/arrays).  
  📄 [`gts/types/schema/v1/query_returns.schema.json`](gts/types/schema/v1/query_returns.schema.json)

- **Template Config Schema** (`gts://gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.template_config.v1~`)  
  Base schema for template configuration. Derived types define specific properties for different template types.  
  📄 [`gts/types/schema/v1/template_config.schema.json`](gts/types/schema/v1/template_config.schema.json)

- **Query Values Schema** (`gts://gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~hypernetix.hyperspot.ax.values.v1~`)  
  Schema for value lists used in UI selectors and filters. Returns value/label/description/metadata structure for dropdowns and pickers.  
  📄 [`gts/types/schema/v1/values.schema.json`](gts/types/schema/v1/values.schema.json)

### Queries

Queries define data retrieval operations using OData v4 protocol for standardized query interface with built-in support across DWH systems, BI tools, and UI libraries.

**Key Features:**
- ✅ **Standard query syntax**: OData query options (`$filter`, `$orderby`, `$top`, `$skip`, `$select`)
- ✅ **$metadata support**: Schema and capabilities defined via OData metadata
- ✅ **GET & POST support**: URL query strings for simple queries, JSON body for complex ones
- ✅ **Built-in UI libraries**: DevExtreme, Kendo UI, ag-Grid support OData out-of-the-box
- ✅ **Wide ecosystem**: Libraries for all popular programming languages

**Query Types:**

- **Query** (`gts://gts.hypernetix.hyperspot.ax.query.v1~`)  
  Query registration with OData v4 integration. References capabilities and returns schemas for OData metadata generation.  
  📄 [`gts/types/query/v1/query.schema.json`](gts/types/query/v1/query.schema.json)

- **Query Capabilities** (`gts://gts.hypernetix.hyperspot.ax.query_capabilities.v1~`)  
  OData Capabilities annotations in JSON format. Defines supported query operations ($filter, $orderby, $top, etc) and their restrictions. Maps to OData CSDL JSON v4.01 Capabilities vocabulary.  
  📄 [`gts/types/query/v1/query_capabilities.schema.json`](gts/types/query/v1/query_capabilities.schema.json)

- **Query Values** (`gts://gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_values.v1~`)  
  Default OData query options for datasources. Stores default `$filter`, `$orderby`, `$select` expressions.  
  📄 [`gts/types/query/v1/values.schema.json`](gts/types/query/v1/values.schema.json)

### Categories

Categories organize and group related GTS entities by domain, type, or purpose.

**Category Types:**

- **Base Category** (`gts://gts.hypernetix.hyperspot.ax.category.v1~`)  
  Base category type for organizing GTS entities. Provides hierarchical classification with name, description, and icon.  
  📄 [`gts/types/category/v1/base.schema.json`](gts/types/category/v1/base.schema.json)

- **Query Category** (`gts://gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.query.v1~`)  
  Category for query definitions. Organizes queries by domain, data source, or functional area.  
  📄 [`gts/types/category/v1/query.schema.json`](gts/types/category/v1/query.schema.json)

- **Template Category** (`gts://gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.template.v1~`)  
  Category for UI component templates. Organizes templates by visualization type or component family.  
  📄 [`gts/types/category/v1/template.schema.json`](gts/types/category/v1/template.schema.json)

- **Datasource Category** (`gts://gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.datasource.v1~`)  
  Category for datasource configurations. Organizes datasources by data domain or system integration.  
  📄 [`gts/types/category/v1/datasource.schema.json`](gts/types/category/v1/datasource.schema.json)

- **Widget Category** (`gts://gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.widget.v1~`)  
  Category for widget items. Organizes widgets by visualization type or data domain.  
  📄 [`gts/types/category/v1/widget.schema.json`](gts/types/category/v1/widget.schema.json)

- **Item Category** (`gts://gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.item.v1~`)  
  Category for layout items (widgets and groups). Organizes reusable item instances.  
  📄 [`gts/types/category/v1/item.schema.json`](gts/types/category/v1/item.schema.json)

- **Group Category** (`gts://gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.group.v1~`)  
  Category for group items. Organizes groups that serve as containers for related widgets.  
  📄 [`gts/types/category/v1/group.schema.json`](gts/types/category/v1/group.schema.json)

- **Dashboard Category** (`gts://gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.dashboard.v1~`)  
  Category for dashboard layouts. Organizes dashboards by business domain or team.  
  📄 [`gts/types/category/v1/dashboard.schema.json`](gts/types/category/v1/dashboard.schema.json)

- **Layout Category** (`gts://gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.layout.v1~`)  
  Category for layout definitions (dashboards and reports). Organizes layouts by use case or format.  
  📄 [`gts/types/category/v1/layout.schema.json`](gts/types/category/v1/layout.schema.json)

### Templates

Templates define rendering logic and configuration for visual components.

**Template Types:**

- **Base Template** (`gts://gts.hypernetix.hyperspot.ax.template.v1~`)  
  Base template type defining reusable UI component configurations. Specifies visual presentation, behavior, and configuration schema.  
  📄 [`gts/types/template/v1/base.schema.json`](gts/types/template/v1/base.schema.json)

- **Widget Template** (`gts://gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~`)  
  Widget template for data visualizations. Defines rendering logic, config schema, and expected data structure (query_schema_id).  
  📄 [`gts/types/template/v1/widget.schema.json`](gts/types/template/v1/widget.schema.json)

- **Values Selector Template** (`gts://gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.values_selector.v1~`)  
  Template for value selection UI components (dropdowns, multi-selects, autocomplete). Used in filter controls.  
  📄 [`gts/types/template/v1/values_selector.schema.json`](gts/types/template/v1/values_selector.schema.json)

### Datasources

Datasources connect query definitions with runtime parameters and UI controls.

- **Datasource** (`gts://gts.hypernetix.hyperspot.ax.datasource.v1~`)  
  Datasource connects a query with its runtime parameters and UI configuration. Encapsulates data retrieval logic and user interface controls (filters, sorting, pagination, grouping, time range selectors).  
  📄 [`gts/types/datasource/v1/datasource.schema.json`](gts/types/datasource/v1/datasource.schema.json)

### Items

Items are reusable building blocks that can be placed in layouts.

**Item Types:**

- **Base Item** (`gts://gts.hypernetix.hyperspot.ax.item.v1~`)  
  Base item type for dashboard and report components. Defines name, description, icon, category, and size (width: 15-100% multiples of 5, height: fixed presets micro/small/medium/high/unlimited).  
  📄 [`gts/types/item/v1/base.schema.json`](gts/types/item/v1/base.schema.json)

- **Widget Item** (`gts://gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~`)  
  Widget item for data visualizations. Combines template (defines rendering) with datasource (provides data). Primary building block for dashboards and reports.  
  📄 [`gts/types/item/v1/widget.schema.json`](gts/types/item/v1/widget.schema.json)

- **Group Item** (`gts://gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.group.v1~`)  
  Group item for organizing and containing other items. Supports collapsible sections and hierarchical layout structures.  
  📄 [`gts/types/item/v1/group.schema.json`](gts/types/item/v1/group.schema.json)

### Layouts

Layouts organize items into dashboards and reports.

**Layout Types:**

- **Base Layout** (`gts://gts.hypernetix.hyperspot.ax.layout.v1~`)  
  Base layout type for organizing items. Defines name, description, icon, category, and ordered array of items.  
  📄 [`gts/types/layout/v1/base.schema.json`](gts/types/layout/v1/base.schema.json)

- **Dashboard Layout** (`gts://gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.dashboard.v1~`)  
  Dashboard layout for interactive, real-time data monitoring. Supports auto-refresh and theme customization.  
  📄 [`gts/types/layout/v1/dashboard.schema.json`](gts/types/layout/v1/dashboard.schema.json)

- **Report Layout** (`gts://gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~`)  
  Report layout for scheduled, printable, and exportable presentations. Supports scheduled generation and multiple export formats.  
  📄 [`gts/types/layout/v1/report.schema.json`](gts/types/layout/v1/report.schema.json)

---

## Behavior

**OpenAPI Specification**: [`openapi/analytics.yaml`](openapi/analytics.yaml)

All REST API endpoints, request/response schemas, and authentication mechanisms are formally defined in the OpenAPI specification.

### GTS Registry

**Unified registry for all GTS-identified types and instances.**

All GTS entities (schemas, queries, datasources, templates, categories, items, layouts) are managed through a unified registry endpoint. The registry handles type registration, instance management, and tenant enablement.

**Key Features:**
- Type definitions registered once, instances created multiple times
- Tenant isolation via SecurityCtx
- Validation against registered schemas
- All extensions and instances registered via `/gts`
- Tenant access controlled via `/enablement` sub-resource with automatic reference enablement

**Service Implementation Requirements:**

The service implementing `/gts` endpoint must meet the following requirements:

1. **Base Types Preloading**
   - All base types (Query, Datasource, Template, Category, Item, Layout, Widget, etc.) are preloaded on service startup
   - Base types are loaded from `gts/types/` directory structure
   - **Alternative source:** If dynamic provisioning/centralized GTS storage is available, service must support loading base types from it
   - Base types cannot be registered or modified through API

2. **Type Extension Support**
   - Service allows registering type extensions (derived schemas) via API
   - Extensions must inherit from known base types (e.g., `gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.custom_query.v1~`)
   - Service validates extensions against their base type schemas

3. **Efficient Validation & Storage**
   - Service knows how to efficiently validate each base type and its extensions
   - Type-specific validation rules are applied based on base type
   - Custom validation logic per base type (e.g., Query requires `api_endpoint`, Datasource requires `query_id`)

4. **Indexing Strategy**
   - Service defines which fields to index for each base type
   - Common indexes: `id`, `type`, `tenant`, `registered_at`, `deleted_at`
   - Type-specific indexes:
     - **Query:** `api_endpoint`, `contract_format`
     - **Datasource:** `query_id`
     - **Template:** `asset_url`
     - **Category:** `name`
     - **Layout:** `layout_type`

5. **Database Schema Generation**
   - Service creates efficient DB schema for each base type
   - Separate tables or collections per base type for performance
   - **Base type instances:** Fully normalized schema adapted to DB engine capabilities
     - Fields mapped to native column types (string → VARCHAR, number → INTEGER/DECIMAL, etc.)
     - Relations properly indexed and optimized
     - Type-specific optimizations (e.g., Template assets stored separately, Layout grid data optimized)
   - **Type extensions:** May use JSON/JSONB for additional fields when appropriate
     - Base fields remain normalized
     - Extension-specific fields stored as structured data when normalization is impractical
   - All storage strategies optimized for the specific DB engine (PostgreSQL, MongoDB, etc.)

6. **Tenant Enablement and Automatic Reference Resolution**

   The service manages multi-tenant access control through the `/enablement` sub-resource with automatic dependency resolution:

   **Enablement Configuration:**
   - Each GTS entity can be enabled for specific tenants via `/gts/{id}/enablement`
   - `enabled_for` field accepts:
     - **Array of tenant IDs:** `["tenant-1", "tenant-2"]` - enables for specific tenants
     - **String "all":** `"all"` - enables for all tenants in the system
   - Enablement is inherited: when entity is enabled for tenant, all referenced entities are automatically enabled

   **Automatic Reference Enablement:**
   
   When entity is enabled for tenant(s), system **automatically enables all referenced entities** for the same tenant(s):

   - **Query** → `returns_schema_id`, `capabilities_id`
   - **Template (Widget)** → `config_schema_id`, `query_returns_schema_id`, `category_id`
   - **Template (Values Selector)** → `config_schema_id`, `values_schema_id`, `category_id`
   - **Datasource** → `query_id` (and transitively: query's schemas)
   - **Widget** → `template_id`, `datasource` reference (and transitively: all their dependencies)
   - **Group** → nested `items` array (widgets and their dependencies)
   - **Dashboard/Report** → all items in `entity.items` array (widgets, groups, and their transitive dependencies)

   **Transitive Dependency Resolution:**
   - System recursively resolves all reference chains
   - No circular dependency handling needed (GTS type system prevents cycles)
   - All schemas, capabilities, and referenced instances enabled automatically
   - Ensures tenants have complete access to all dependencies

   **Implementation Requirements:**
   - Service MUST track all reference fields for each base type
   - Service MUST implement recursive enablement propagation
   - Enablement operations MUST be transactional (all-or-nothing)
   - Service MUST prevent enablement of entity when referenced entities don't exist
   - Service SHOULD log enablement propagation for audit purposes

   **Example:**
   ```
   PUT /api/analytics/v1/gts/{dashboard_id}/enablement
   Body: { "enabled_for": ["tenant-1"] }
   
   System automatically enables:
   - Dashboard itself
   - All widgets in dashboard.entity.items[]
   - All templates referenced by widgets
   - All datasources referenced by widgets
   - All queries referenced by datasources
   - All schemas referenced by queries and templates
   - All capabilities referenced by queries
   ```

7. **Tolerant Reader Pattern**

   The API follows the **[Tolerant Reader](https://martinfowler.com/bliki/TolerantReader.html)** pattern, where the service intelligently understands field semantics and their usage across different scenarios:

   **Field Categories:**

   - **Client-provided fields (Create/Update):** Fields that clients send in requests
     - Example: `entity.name`, `entity.api_endpoint`, `entity.query_id`
   
   - **Server-managed fields (Read-only):** Fields automatically set by the service, ignored in requests
     - `id` - Generated based on GTS identifier rules or UUID for anonymous instances
     - `type` - Derived from `id` or `$id` field in schema
     - `registered_at`, `updated_at`, `deleted_at` - Timestamp metadata
     - `registered_by`, `updated_by`, `deleted_by` - User identity metadata
     - `tenant` - Extracted from security context
   
   - **Computed fields (Response-only):** Fields calculated or enriched by the service
     - `asset_path` - Server-computed local path for templates
     - Type-specific computed properties based on entity configuration
   
   - **Never-returned fields:** Sensitive data excluded from all responses
     - API keys, secrets, credentials stored in `entity` object
     - Internal system identifiers
     - Encryption keys or tokens

   **Scenario-Specific Field Handling:**

   **POST (Create):** Client provides `entity` data; Service adds `id`, `type`, `registered_at`, `tenant`
   
   **PUT (Update):** Client replaces `entity`; System fields (`id`, `type`, `registered_at`) ignored
   
   **PATCH (Partial Update):** JSON Patch on `/entity/*` paths; System fields rejected

   **JSON Schema `required` Fields:**
   - **POST/PUT:** Service validates all `required` fields present in request
   - **GET:** Service may omit fields (secrets, credentials) even if `required` in schema


#### Register GTS Type or Instance

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

#### List and Search GTS Entities

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

**Example 1: List All Queries**

```
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query.v1~')&$top=20&$select=...
```

**Example 2: Full-Text Search**
```
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query.v1~') and search.ismatch('monitoring metrics')
```

**Example 3: Filter by Entity Property**
```
GET /api/analytics/v1/gts?$filter=entity/api_endpoint eq 'https://api.acme.com/analytics/sales'
```

**Example 4: Filter by Nested Property**
```
GET /api/analytics/v1/gts?$filter=entity/query_id eq 'gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.sales.v1'
```

**Example 5: Property Value Pattern Matching**
```
GET /api/analytics/v1/gts?$filter=contains(entity/api_endpoint, 'monitoring')
```

**Example 6: Multiple Filters with Sorting**

```http
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.template.v1~') and contains(entity/asset_url, 'cdn.acme.com')&$orderby=registered_at desc
```

**Example 7: Filter by Vendor Segment**

```http
GET /api/analytics/v1/gts?$filter=gts_vendor eq 'acme'&$orderby=entity/name asc
```

**Example 8: Filter by Package and Type**

```http
GET /api/analytics/v1/gts?$filter=gts_package eq 'analytics' and gts_type eq 'query'
```

**Example 9: Combine Segment Filters**

```http
GET /api/analytics/v1/gts?$filter=gts_vendor eq 'acme' and gts_package eq 'monitoring' and gts_type eq 'datasource'
```

**Example 10: Filter by Tenant**

```http
GET /api/analytics/v1/gts?$filter=tenant eq '550e8400-e29b-41d4-a716-446655440000'&$top=100
```

**Example 11: Filter by Registration Date Range**
```
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query.v1~') and registered_at ge 2024-01-08T00:00:00Z
```

**Example 12: Filter by User and Date**
```
GET /api/analytics/v1/gts?$filter=registered_by eq 'user-7a1d2f34-...' and registered_at ge 2024-01-01T00:00:00Z
```

**Response Format:** `@odata.context`, `@odata.count`, `@odata.nextLink`, `items[]`

**Pagination Flow:**

```http
# First page
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query.v1~')&$top=50&$count=true

# Next page (use @odata.nextLink from response or $skiptoken)
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query.v1~')&$top=50&$skiptoken=eyJpZCI6Imd0cy5oeXBlcm5ldGl4LmhR...
```

**OData Response Fields:**
- `@odata.context` - Metadata context URL
- `@odata.count` - Total count (when `$count=true`)
- `@odata.nextLink` - URL for next page (`null` when no more results)
- `$skiptoken` is opaque - do not parse or modify

#### OData Metadata

```
GET /api/analytics/v1/$metadata
Accept: application/json
Returns: OData JSON CSDL with Capabilities vocabulary annotations
```

Service exposes full OData metadata in JSON CSDL format (OData v4.01) with capability annotations (FilterRestrictions, SortRestrictions, SearchRestrictions, SelectSupport, TopSupported, SkipSupported).

Spec: [OData JSON CSDL v4.01](https://docs.oasis-open.org/odata/odata-csdl-json/v4.01/odata-csdl-json-v4.01.html) | [Capabilities Vocabulary](https://github.com/oasis-tcs/odata-vocabularies/blob/master/vocabularies/Org.OData.Capabilities.V1.md)

#### Get GTS Item

```
GET /api/analytics/v1/gts/{gts-identifier}
Returns: GTS entity (id, type, entity, registered_at, tenant, metadata)
```

#### Update GTS Item (Full Replacement)

```
PUT /api/analytics/v1/gts/{gts-identifier}
Body: { "entity": { ... } }  # Full entity replacement
```

**Note:** Only API-registered entities can be updated. File-provisioned entities are read-only (HTTP 403).

#### Partially Update GTS Item

```
PATCH /api/analytics/v1/gts/{gts-identifier}
Content-Type: application/json-patch+json
Body: JSON Patch operations (RFC 6902) on /entity/* paths
```

**JSON Patch Operations:**

- `replace` - Replace a field value
- `add` - Add a new field or array element
- `remove` - Remove a field or array element
- `copy` - Copy a value from one location to another
- `move` - Move a value from one location to another
- `test` - Test that a value matches (for conditional updates)

**Example: Add and Remove Fields**

```http
PATCH /api/analytics/v1/gts/{gts-identifier}
Authorization: Bearer {token}
Content-Type: application/json-patch+json

[
  {
    "op": "add",
    "path": "/entity/tags",
    "value": ["analytics", "sales"]
  },
  {
    "op": "remove",
    "path": "/entity/deprecated_field"
  }
]
```

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

#### Delete GTS Item

```
DELETE /api/analytics/v1/gts/{gts-identifier}
Soft-delete (sets deleted_at timestamp)
Returns: 204 No Content
```

#### Manage Tenant Enablement

Enable/disable GTS entities for specific tenants:

```
GET /api/analytics/v1/gts/{gts-identifier}/enablement  # Returns enabled_for array
PUT /api/analytics/v1/gts/{gts-identifier}/enablement  # Update enabled_for list
```

Partial update with JSON Patch (RFC 6902):
```http
PATCH /api/analytics/v1/gts/{gts-identifier}/enablement
Content-Type: application/json-patch+json

[
  { "op": "add", "path": "/enabled_for/-", "value": "tenant-4" },
  { "op": "remove", "path": "/enabled_for/0" }
]
```

Common JSON Patch operations:
- **Add tenant**: `{ "op": "add", "path": "/enabled_for/-", "value": "tenant-id" }`
- **Remove tenant**: `{ "op": "remove", "path": "/enabled_for/0" }` (by index)
- **Replace all**: `{ "op": "replace", "path": "/enabled_for", "value": ["new-list"] }`
- **Test before change**: `{ "op": "test", "path": "/enabled_for/0", "value": "tenant-1" }`

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

#### Register Query

```
POST /api/analytics/v1/gts
Type: gts.hypernetix.hyperspot.ax.query.v1~
Instance: gts.hypernetix.hyperspot.ax.query.v1~acme.monitoring._.server_metrics.v1
Key fields:
  - category, name, api_endpoint
  - capabilities_id → query params schema
  - returns_schema_id → response data schema
  - contract_format (native|custom)
  - adapter_id (required if contract_format=custom)
```

#### Register Datasource

```
POST /api/analytics/v1/gts
Type: gts.hypernetix.hyperspot.ax.datasource.v1~
Instance: gts.hypernetix.hyperspot.ax.datasource.v1~acme.monitoring.metrics.cpu_usage.v1
Key fields:
  - query_id → links to query instance
  - params → OData parameters ($filter, $orderby, $top, $select, $count)
  - render_options → UI controls (filters, time, sorting, pagination)
```

#### Execute Query

```
GET /api/analytics/v1/queries/{gts-identifier}?$filter=...&$orderby=...&$top=...&$select=...&$count=...
POST /api/analytics/v1/queries/{gts-identifier}/$query  # OData params in JSON body
Response: OData v4 format (@odata.context, @odata.count, @odata.nextLink, value[])
```

##### OData Query Options Reference

**Filtering** (`$filter`):
- **Comparison**: `eq`, `ne`, `gt`, `ge`, `lt`, `le`
- **Logical**: `and`, `or`, `not`
- **String functions**: `contains()`, `startswith()`, `endswith()`, `length()`, `tolower()`, `toupper()`
- **Date functions**: `year()`, `month()`, `day()`, `hour()`, `minute()`, `second()`
- **Collections**: `in` (value in list), `any()`, `all()`

```odata
Examples:
$filter=status eq 'active'
$filter=revenue gt 1000 and region eq 'EU'
$filter=created_at ge 2024-01-01T00:00:00Z
$filter=contains(name, 'server')
$filter=region in ('EU','US','APAC')
```

**Sorting** (`$orderby`):
```odata
Examples:
$orderby=created_at desc
$orderby=region asc,revenue desc
$orderby=tolower(name) asc
```

**Pagination**:
- `$top=50` - page size (limit)
- `$skip=100` - offset
- Use `@odata.nextLink` from response for cursor-based pagination

**Field Selection** (`$select`):
```odata
Examples:
$select=id,name,revenue
$select=*
```

**Expand Navigation Properties** (`$expand`):
```odata
Examples:
$expand=customer
$expand=customer($select=id,name)
$expand=customer($filter=type eq 'enterprise')
```

**Full-Text Search** (`$search`):
```odata
Examples:
$search="server error"
$search=cpu OR memory
```

**Count** (`$count=true`):
Includes total count in `@odata.count` field.

##### Query Metadata Endpoint

```
GET /api/analytics/v1/queries/{gts-identifier}/$metadata
Accept: application/json
Returns: OData JSON CSDL with query-specific schema and capabilities
```

**Integration Scenarios**:

1. **Native Contract Implementation** (No plugin needed)
   - Service already implements Analytics contract
   - Direct registration via API call
   - Example: Custom analytics service built with Analytics contract from start

2. **Plugin-based Wrapper** (Plugin wraps existing API)
   - Service has API but doesn't implement Analytics contract
   - Write plugin that implements contract and calls your API
   - Plugin embedded in platform, registered as local datasource
   - Example: Legacy monitoring system with custom API format

3. **Adapter-based Integration** (Using 3rd-party contract)
   - Service implements known 3rd-party contract (e.g., Prometheus, Elasticsearch)
   - Use existing adapter plugin or write custom one
   - Adapter converts 3rd-party format to native contract
   - Optionally combine with plugin for complex auth/logic
   - Example: Prometheus exporter with adapter plugin

**Key Features**:
- **GTS-based identification** - Datasources identified by GTS instance identifiers
- **Dynamic registration** - Register at runtime without platform restart
- **Health monitoring** - Automatic health checks and status tracking
- **Multi-tenancy** - Full tenant isolation via SecurityCtx

### Plugin Architecture

Plugins are optional extensions that run inside the platform. They are **independent** from datasource registration.

**Plugin Capabilities**:

1. **Local Datasource Implementation**
   - Implement datasource logic directly in platform
   - No external API calls needed
   - Used when datasource is tightly coupled to platform

2. **Contract Adapters**
   - Convert 3rd-party API formats to native contract
   - Reusable across multiple datasources
   - Examples: Prometheus adapter, Elasticsearch adapter, OpenTelemetry adapter

3. **Custom Processing**
   - Add preprocessing/postprocessing logic
   - Handle complex authentication flows
   - Implement caching strategies

#### Plugin Loading

Plugins are loaded via filesystem and configuration, not API calls:

1. Write plugin code
2. Place in plugins directory (e.g., `/plugins/prometheus-adapter/`)
3. Enable in service config with optional auto-registration
4. Reload service config
5. Plugin loads, queries auto-register as full GTS entities, endpoints become available

**Auto-registration Requirements:**

Plugin can optionally auto-register GTS entities on load:

**Queries** (`auto_register_queries`):
- Plugin creates **one or more** full GTS Query entities
- Each Query gets proper **GTS identifier** (e.g., `gts.hypernetix.hyperspot.ax.query.v1~prometheus.monitoring._.server_metrics.v1`)
- Query registered with all required fields:
  - `category`, `name`, `description`
  - `api_endpoint` (plugin-internal endpoint)
  - `capabilities_id` (defines OData query capabilities)
  - `returns_schema_id` (defines response schema)
  - `contract_format: "custom"` (since plugin uses adapter)
  - `adapter_id` (references this plugin's adapter)
- Queries are **discoverable via `/gts` endpoint** like any other registered query
- Queries can be **used in dashboards** like manually registered queries

**Datasources** (`auto_register_datasources`):
- Plugin can optionally create **one or more** Datasource entities
- Each Datasource links to a Query via `query_id`
- Datasource includes `params` (OData parameters) and `render_options` (UI controls)
- Useful for providing pre-configured datasources with specific filters/parameters

**Validation:**
- Plugin declares entities it provides in config
- Service validates and registers them as GTS entities on startup
- If entity with same ID exists, plugin registration fails (conflict)
- Config format TBD in OpenSpec

**Key Difference**:
- **Datasource registration** = API call to register external/plugin endpoint
- **Plugin loading** = Config-based code extension (filesystem + config reload)

### Template Registration

Templates are pluggable UI elements (TypeScript/JavaScript assets) that render data visualizations, filters, and interactive components.

**Template Type Hierarchy** (via GTS inheritance):
- Base: `gts.hypernetix.hyperspot.ax.template.v1~`
  - Widget: `gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~`
  - Values Selector: `gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.values_selector.v1~`

#### Register Template

Two-step process: register metadata, upload bundle.

##### Step 1: Register Template Instance

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

##### Step 2: Upload JavaScript Bundle

```
POST /api/analytics/v1/templates/{template-id}/bundle  # Upload JS bundle, returns bundle_url and checksum
```

**Bundle Replacement**:
- Bundles can be uploaded **multiple times** for the same template
- Each upload **replaces** the previous bundle
- UI cache is invalidated on replacement (via ETag/Last-Modified headers)
- Template metadata remains unchanged

**Registration Flow**:
1. **Register Instance** - Create GTS template metadata with contract definition
2. **Upload Bundle** - POST JavaScript implementation to `/templates/{id}/bundle`
3. **Verification** - Service validates bundle syntax and security
4. **Storage** - Bundle stored with template ID mapping
5. **UI Loading** - UI fetches bundle via GET `/templates/{id}/bundle.js`
6. **Caching** - UI caches bundle with proper cache headers (ETag, max-age)

**Template Asset Requirements**:
- Template must export standard interface with required methods
- Must support rendering with config and data
- Must provide config editor for user customization
- Must validate config against schema
- Must support resource cleanup
- Interface details defined in OpenSpec

**Key Points**:
- **Self-contained bundle** - All dependencies must be bundled in the JavaScript asset
- **Schema references** - Both config schema and datasource schema must exist before registration
- **Config schema** - Defines structure of template configuration
- **Datasource schema** - Defines expected data format from datasource
- **Dynamic loading** - Templates loaded on-demand in browser
- **Version management** - Multiple versions can coexist via GTS identifiers
- **Security** - Checksum verification, sandboxed execution

### Query Execution Flow

Executes queries via `/queries/{id}` endpoint using OData v4 protocol.

**Flow Steps**:

1. **Resolve Query** - Fetch query definition from GTS Registry (`/gts/{query-id}`)
2. **Build OData Request** - Construct OData query parameters from datasource params + widget overrides
3. **Generate JWT** - Create JWT token with tenancy context (tenant_id, org_id, sub)
4. **Execute Request** - Call external API endpoint with OData parameters and JWT
5. **Validate Response** - Verify response against `returns_schema_id` from query definition
6. **Cache Result** - Store response for performance (with tenant isolation)
7. **Return Data** - Send OData response to widget renderer

**Query Request (GET with OData parameters)**:
```http
GET /api/analytics/v1/queries/gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.sales.v1?$filter=status%20eq%20%27active%27%20and%20revenue%20gt%201000&$orderby=created_at%20desc&$top=50&$skip=0&$select=id,name,revenue,status&$count=true
Authorization: Bearer <jwt-token-with-tenancy>
```

Or **POST with OData body** (for complex queries):
```http
POST /api/analytics/v1/queries/gts.hypernetix.hyperspot.ax.query.v1~acme.analytics._.sales.v1
Authorization: Bearer <jwt-token-with-tenancy>
Content-Type: application/json

{
  "$filter": "status eq 'active' and revenue gt 1000",
  "$orderby": "created_at desc",
  "$top": 50,
  "$skip": 0,
  "$select": "id,name,revenue,status",
  "$count": true
}
```

**Query Response:** OData v4 format with `@odata.context`, `@odata.count`, `@odata.nextLink`, `value[]`

**JWT Token:** Includes `sub`, `tenant_id`, `org_id`, `iat`, `exp`, `scopes`

---

### Layout Distribution Algorithm

The layout engine automatically positions items using a grid-based algorithm with vertical stacking.

**Horizontal Grid System:**
- Layout width is divided into **minimum horizontal sections** (e.g., 20 sections for 5% granularity)
- Each item occupies one or more sections based on its `width` percentage
- Items are placed left-to-right, filling available horizontal space

**Vertical Stacking Rules:**
- Items are positioned **top-to-bottom** in the order they appear in the `items[]` array
- **First item** always appears in the **top-left corner** of the layout
- If an item can fit **below** an existing item (vertical space available), it is placed there instead of to the right
- This creates a **masonry-style layout** where items stack vertically when possible
- Each item uses a fixed height based on its `height` preset:
  - `micro`: ~100px
  - `small`: ~200px  
  - `medium`: ~400px
  - `high`: ~600px
  - `unlimited`: grows with content (for tables with pagination)

**Positioning Algorithm:**
1. Start with first item → place at top-left (0, 0)
2. For each subsequent item:
   - Check all existing items from top to bottom
   - Find the first vertical position where the item can fit below an existing item
   - If no vertical fit found, place to the right of the previous row
3. Continue until all items are positioned

**Example Layout:**

![Layout Distribution Example](openspec/diagrams/layout_distribution_example.drawio.svg)

The diagram demonstrates masonry-style layout with 12 items of varying sizes:
- **Horizontal:** Items use percentage widths (25%, 30%, 35%, 50%, 100%) - all values are multiples of 5% within 15-100% range
- **Vertical:** Items use fixed height presets (micro ~100px, small ~200px, medium ~400px, high ~600px)
- **Positioning:** Items stack vertically when possible, following top-to-bottom order from the `items[]` array

This algorithm ensures optimal space utilization while maintaining predictable positioning based on the `items[]` array order.

**API Calls:**

None - this is a client-side algorithm that operates on dashboard/report layout data already loaded from:
```
GET /api/analytics/v1/gts/{dashboard_id}
```

---

### Item Preview Rendering

When rendering item previews (in editor, settings dialog, or live dashboard), the system must compute absolute dimensions from the item's relative sizing configuration.

**Size Calculation:**

**1. Horizontal Size (Width):**
- Item `width` is specified as **percentage** (15-100%, multiples of 5)
- Absolute width computed from **current layout container width**:
  ```
  absolute_width_px = (item.size.width / 100) × layout_container_width_px
  ```
- Layout container width depends on context:
  - **Dashboard/Report editor:** Current viewport width minus sidebar/chrome
  - **Settings preview panel:** Preview container width (typically 400-800px)
  - **Live rendering:** Actual dashboard container width

**2. Vertical Size (Height):**
- Item `height` is specified as **fixed preset** enum
- Absolute height mapped from preset to pixels:
  ```
  micro      → ~100px
  small      → ~200px
  medium     → ~400px
  high       → ~600px
  unlimited  → min-height with content expansion (for paginated tables)
  ```
- Height values are **constant** regardless of layout container size

**Rendering Contexts:**

**1. Editor Preview (Dashboard/Report Builder):**
- Layout container width = editor canvas width
- Items rendered with computed absolute dimensions
- Position calculated via Layout Distribution Algorithm
- Interactive: dragging, resizing (snapped to 5% grid), reordering

**2. Settings Preview (Widget/Group Configuration):**
- Layout container width = preview panel width (fixed, e.g., 600px)
- Single item rendered in isolation
- Shows how item will appear at its configured size
- Non-interactive, read-only preview

**3. Live Rendering (Dashboard View):**
- Layout container width = viewport width or dashboard container
- Full layout rendered with all items positioned
- Responsive: container width changes trigger re-calculation of absolute widths
- Heights remain fixed per preset values

**Example Calculation:**

```javascript
// Item configuration
item.size = { width: 50, height: "medium" }

// Context: Editor with 1200px canvas width
layout_container_width = 1200

// Computed dimensions
absolute_width = (50 / 100) × 1200 = 600px
absolute_height = 400px  // medium preset

// Rendered item: 600px × 400px
```

**Responsive Behavior:**
- When layout container resizes, all item widths recalculate proportionally
- Heights remain constant (fixed presets)
- Layout Distribution Algorithm re-runs to reposition items if needed
- Maintains visual consistency across different screen sizes

**API Calls:**
```
GET /api/analytics/v1/gts/{dashboard_id}  # Load dashboard with layout items
```

---

### Widget Rendering with Template Bundles

Widgets are rendered using dynamically loaded JavaScript template bundles. The UI follows a structured lifecycle to initialize, render, and update widgets based on template code and datasource data.

**Template Bundle Structure:**

Each template is a JavaScript module (ESM) uploaded to `/templates/{template_id}/bundle` that exports:

- **`init(container, config)`** - Initialize template instance
  - `container`: DOM element to render into
  - `config`: Template configuration from widget settings
  - Returns: template instance object

- **`render(instance, data)`** - Render/update visualization with data and config
  - `instance`: Object returned by init() (contains config and visualization state)
  - `data`: Query result from datasource
  - Template applies config (colors, axes, legends) and renders data

- **`updateConfig(instance, newConfig)`** *(optional)* - Update configuration
  - `newConfig`: Updated template settings

- **`destroy(instance)`** *(optional)* - Cleanup before removal
  - Clean up event listeners, timers, resources

- **`renderSettings(container, currentConfig, onChange)`** - Render settings UI
  - `container`: DOM element for settings form
  - `currentConfig`: Current template configuration
  - `onChange(newConfig)`: Callback to update config
  - Template must provide UI for all its configuration options

**Rendering Lifecycle:**

**1. Widget Initialization:**
```
Dashboard Load → Parse Layout Items → For Each Widget:
  - Read widget.settings.template.id
  - Read widget.settings.template.config
  - Read widget.settings.datasource
  - Compute absolute dimensions (via Item Preview Rendering algorithm)
  - Create DOM container with computed width/height
```

**2. Template Bundle Loading:**
```
GET /api/analytics/v1/templates/{template_id}/bundle
  → Download JavaScript bundle (cached in browser)
  → Import as ESM module
  → Call template.init(container, config)
  → Store returned instance for this widget
```

**3. Data Fetching:**
```
Extract datasource configuration
  → Build query URL: /api/analytics/v1/queries/{query_id}
  → Apply OData params from datasource.params
  → Execute query request
  → Receive data response
```

**4. Initial Render:**
```
Call template.render(instance, data)
  → Template applies config (colors, axes, legends, etc.) and renders data
  → Updates DOM inside container
  → Widget visible to user
```

**5. Update Cycle:**

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

**6. Widget Cleanup:**
```
Widget removed from dashboard or layout changed
  → Call template.destroy(instance)
  → Remove DOM container
  → Clear instance reference
```

**Template API Contract:**

**init(container, config):**
- Initialize template instance with initial configuration
- Returns opaque instance object (stores template state)

**render(instance, data):**
- Render/update visualization with new data using stored config
- Applies config settings (colors, axes, legends) and renders data
- Idempotent - can be called multiple times with different data

**updateConfig(instance, newConfig):**
- Update template configuration without full re-initialization
- Optional - not all templates need dynamic config updates

**destroy(instance):**
- Cleanup resources before widget removal
- Optional - only needed if template has event listeners, timers, WebSocket connections

**renderSettings(container, currentConfig, onChange):**
- Render UI for template configuration in settings dialog
- Required - template must provide settings UI, platform cannot auto-generate
- Template owns its configuration UI (color pickers, chart options, validators)

**Error Handling:**

**Bundle Load Failure:** 404 or network error → Show error placeholder with retry action

**Data Fetch Failure:** Query error → Call template.render(instance, null) or show error state

**Render Exception:** Catch exception → Display error overlay, keep container intact

**Caching Strategy:**
- **Template Bundles:** Cached by browser (ETag + Last-Modified headers)
- **Query Data:** Cached per query+params+tenant (configurable TTL)
- **Bundle Versioning:** Template ID includes version → cache invalidation on update

**Example Flow:**
```
Line Chart Template
  ├─ init() → Creates SVG element in container, stores visualization state
  ├─ render(data) → Draws line chart with data points, applies styling from config
  ├─ updateConfig(newColors) → Changes chart colors without reloading data
  └─ destroy() → Removes SVG element, cleans up event listeners
```

**API Calls:**
```
GET /api/analytics/v1/gts/{dashboard_id}              # Load dashboard with widgets
GET /api/analytics/v1/templates/{template_id}/bundle  # Load template bundle
GET /api/analytics/v1/queries/{query_id}?$filter=...  # Execute query for data
GET /api/analytics/v1/gts/{template_id}               # Load template metadata
```

---

### Widget Settings UI Rendering

When user opens widget settings dialog, the UI renders two types of configuration:
1. **Platform-managed settings** - standard widget properties (rendered by platform)
2. **Template-specific settings** - custom configuration (rendered by template bundle)

**Platform-Managed Settings:**

**1. Item Properties:** Name, description, icon, size (width %, height preset)

**2. Template Selection:** Searchable dropdown filtered by query compatibility

**3. Datasource Configuration:**
- Query selection
- OData parameters ($filter, $orderby, $top, $skip, $select, $expand, $search)
- Render options (filters, sorting, pagination, time range, search, grouping)

**OData Capabilities + Render Options Integration:**

The platform combines **OData metadata capabilities** (from query.capabilities_id) with **datasource.render_options** to render appropriate UI controls.

**Two-layer system:**

1. **OData Capabilities** - Define what query *technically supports*:
   - FilterFunctions, SortRestrictions, SearchRestrictions, SelectSupport, ExpandRestrictions, TopSupported, SkipSupported

2. **Render Options** - Define what UI *should show to user*:
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

**Settings Dialog Structure:**
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

**Key Differences:**

| Aspect | Platform Settings | Template Settings |
|--------|------------------|-------------------|
| **Rendered by** | Platform UI (standard controls) | Template bundle (custom code) |
| **Configuration** | Item properties, datasource, template selection | Template-specific options (colors, axes, legends) |
| **Schema source** | base.schema.json, widget.schema.json | template.config_schema_id |
| **UI generation** | Automatic from known schemas | Manual via renderSettings() |
| **Validation** | Platform validates against schemas | Template validates in onChange() |
| **Changes trigger** | May require template reload | Calls template.updateConfig() |

**API Calls:**
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

### User Scenarios & API Flows

This section defines all user scenarios for each persona, describing both UI interactions and the underlying API calls made during each workflow.

#### 1. Dashboard Designer / Business Analyst

##### Scenario 1.1: Create New Dashboard

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
  # Allows specified tenants or all tenants to access this dashboard
  # NOTE: System automatically enables all referenced entities (widgets, templates, datasources, queries, schemas)
  #       for the same tenants, ensuring complete dependency access
```

##### Scenario 1.2: Add Widget to Dashboard

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
  # Browse widget template instances (metadata) from GTS registry
  # System suggests compatible templates based on query schema
GET /api/analytics/v1/templates/{template_id}/bundle
  # Download widget template JavaScript bundle (implementation)
  # Returns application/javascript with ETag and cache headers
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.values_selector.v1~')&$select=...
  # Browse values selector template instances for render_options UI controls (dropdown, multi-select, date picker, etc.)
GET /api/analytics/v1/templates/{values_selector_template_id}/bundle
  # Download values selector template JavaScript bundle (implementation)
  # Returns application/javascript with ETag and cache headers

# Schemas for validation and compatibility checking
GET /api/analytics/v1/gts/{query_returns_schema_id}
  # Get query returns schema (from query.returns_schema_id) - defines query result structure
  # Used to match with template's expected data schema for compatibility checking
GET /api/analytics/v1/gts/{template_config_schema_id}
  # Get template config schema (from template.config_schema_id) - defines valid template configuration
  # Used to validate template.config against JSON Schema
GET /api/analytics/v1/gts/{values_schema_id}
  # Get values schema (from values_selector.values_schema_id) - defines filter values structure
  # Used to validate filter values for values selector templates

# Add widget to dashboard (both options)
PATCH /api/analytics/v1/gts/{dashboard_id}  # Add widget item inline to dashboard/entity/items
  # Widget type: gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~
  # If preset: reference widget instance ID
  # If custom: contains template_id, datasource (inline or ref), config, grid_position
GET /api/analytics/v1/queries/{query_id}?$filter=...&$orderby=...&$top=...  # Preview data
```

##### Scenario 1.3: Move Widget Position

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

##### Scenario 1.4: Edit Widget Settings

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

##### Scenario 1.5: Advanced Widget Editor

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
  # Browse queries when changing query_id
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.datasource.v1~')&$select=...
  # Browse datasource presets (optional - can configure inline or use preset)
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~')&$select=...
  # Browse widget template instances (metadata) from GTS registry when changing template_id
GET /api/analytics/v1/templates/{template_id}/bundle
  # Download widget template JavaScript bundle (implementation) if template_id changed
  # Returns application/javascript with ETag and cache headers
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.values_selector.v1~')&$select=...
  # Browse values selector template instances for render_options UI controls (dropdown, multi-select, date picker, etc.)
GET /api/analytics/v1/templates/{values_selector_template_id}/bundle
  # Download values selector template JavaScript bundle (implementation)
  # Returns application/javascript with ETag and cache headers

# Schemas for validation and compatibility checking
GET /api/analytics/v1/gts/{query_returns_schema_id}
  # Get query returns schema (from query.returns_schema_id) - defines query result structure
  # Used to match with template's expected data schema for compatibility checking
GET /api/analytics/v1/gts/{template_config_schema_id}
  # Get template config schema (from template.config_schema_id) - defines valid template configuration
  # Used to validate template.config against JSON Schema
GET /api/analytics/v1/gts/{values_schema_id}
  # Get values schema (from values_selector.values_schema_id) - defines filter values structure
  # Used to validate filter values for values selector templates

PATCH /api/analytics/v1/gts/{dashboard_id}  # Update widget settings
  # JSON Patch operations on dashboard/entity/items[n]/settings/datasource and /template
GET /api/analytics/v1/queries/{query_id}?...  # Preview data with new params
```

##### Scenario 1.6: Add Widget to Group

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

##### Scenario 1.7: Create Group

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

##### Scenario 1.8: Create Widget Preset

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
  # Instance contains full widget configuration: settings/template, settings/datasource, size (width %, height preset), etc.
PUT /api/analytics/v1/gts/{widget_preset_id}/enablement  # Share widget preset with tenants (optional)
  # Body: { "enabled_for": ["tenant-1", "tenant-2"] } or { "enabled_for": "all" }
  # Allows specified tenants or all tenants to use this widget preset
```

##### Scenario 1.9: Create Datasource Preset

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

##### Scenario 1.10: Create Group Preset

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
PUT /api/analytics/v1/gts/{group_preset_id}/enablement  # Share group preset with tenants (optional)
  # Body: { "enabled_for": ["tenant-1", "tenant-2"] } or { "enabled_for": "all" }
  # Allows specified tenants or all tenants to use this group preset
```

##### Scenario 1.11: Edit Widget Preset

**UI Flow:**
1. Browse widget presets library
2. Select widget preset to edit
3. Load preset configuration
4. Edit datasource configuration:
   - Change query_id
   - Modify OData params
   - Configure render_options
5. Edit template configuration:
   - Change template_id
   - Modify template config
6. Edit item properties (name, description, icon, size: width %, height preset)
7. Preview changes with live data
8. Update preset metadata if needed (name, description, category)
9. Save changes to preset

**API Calls:**
```
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~')&$select=...
  # Browse widget presets
GET /api/analytics/v1/gts/{widget_preset_id}
  # Get specific widget preset to edit
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.query.v1~')&$select=...
  # Browse queries when changing query_id
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.datasource.v1~')&$select=...
  # Browse datasource presets (optional)
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~')&$select=...
  # Browse widget template instances when changing template_id
GET /api/analytics/v1/templates/{template_id}/bundle
  # Download widget template bundle if template_id changed
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.values_selector.v1~')&$select=...
  # Browse values selector templates for render_options
GET /api/analytics/v1/templates/{values_selector_template_id}/bundle
  # Download values selector template bundle
GET /api/analytics/v1/gts/{query_returns_schema_id}
  # Get query returns schema for compatibility checking
GET /api/analytics/v1/gts/{template_config_schema_id}
  # Get template config schema for validation
GET /api/analytics/v1/gts/{values_schema_id}
  # Get values schema for filter validation
PUT /api/analytics/v1/gts/{widget_preset_id}
  # Update widget preset (full replacement)
  # Or use PATCH for partial updates with JSON Patch operations
GET /api/analytics/v1/queries/{query_id}?...  # Preview data with new configuration
```

##### Scenario 1.12: Edit Group

**UI Flow:**
1. Open dashboard (user has edit permissions)
2. Select group to edit
3. Edit group properties:
   - Name, description, icon
   - Size (width: 15-100% multiples of 5, height: micro/small/medium/high/unlimited)
   - Collapsible behavior (enabled/disabled, default collapsed/expanded state)
4. Manage nested widgets:
   - Add widgets to group (drag from dashboard or library)
   - Remove widgets from group
   - Reorder widgets within group
5. Preview changes
6. Save group configuration

**API Calls:**
```
PATCH /api/analytics/v1/gts/{dashboard_id}
  # JSON Patch operations on dashboard/entity/items[{group_index}]
  # Update group properties: name, description, icon, size (width %, height preset), settings/collapsible
  # Manage nested widgets: add/remove/reorder items in settings/items array
```

##### Scenario 1.13: Delete Widget from Dashboard

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
  # JSON Patch operation removing item from dashboard/entity/items array
  # op: "remove", path: "/entity/items/{index}"
```

##### Scenario 1.14: Delete Group from Dashboard

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
  # JSON Patch operation removing group from dashboard/entity/items array
  # op: "remove", path: "/entity/items/{index}"
  # Removes group and all nested widgets (settings/items)
```

##### Scenario 1.15: Delete Dashboard

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

##### Scenario 1.16: Delete Datasource Preset

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

##### Scenario 1.17: Create Report

**UI Flow:**
Similar to Scenario 1.1 (Create Dashboard), with report-specific differences:

1. Navigate to Reports → Create New
2. Choose starting point:
   - **Convert from dashboard** (import dashboard layout and widgets)
   - **Start blank** (create from scratch)
3. Enter report metadata (name, description, icon, category)
4. Configure report-specific settings:
   - Paper size (A4, Letter, etc.)
   - Orientation (portrait, landscape)
   - Header/footer templates (title, date, page numbers, company logo)
5. Add widgets to report (same as dashboard - see Scenario 1.2)
   - Widget size: width percentage (15-100%), height preset (micro/small/medium/high/unlimited)
6. Configure print/export settings:
   - Page breaks
   - Color mode (color, grayscale)
7. Set up schedule (optional):
   - Frequency (daily, weekly, monthly, custom cron)
   - Time of day / timezone
   - Recipients (email addresses, distribution lists)
   - Delivery format (PDF, Excel, CSV)
8. Configure sharing (optional):
   - Choose visibility: private, specific tenants, or all tenants
   - Select tenants if sharing with specific tenants
9. Test report generation
10. Save report

**API Calls:**
```
GET /api/analytics/v1/gts?$filter=startswith(id,'gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~')
  # Browse report categories
GET /api/analytics/v1/gts/{dashboard_id}  # If converting from dashboard
POST /api/analytics/v1/gts  # Create report instance
  # Type: gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~
  # Instance ID: gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~acme.sales._.weekly_report.v1
  # Contains: items (widgets/groups), settings (paper_size, orientation, header/footer, schedule, delivery)
PUT /api/analytics/v1/gts/{report_id}/enablement  # Share report with tenants (optional)
  # Body: { "enabled_for": ["tenant-1", "tenant-2"] } or { "enabled_for": "all" }
  # Allows specified tenants or all tenants to access this report
POST /api/analytics/v1/reports/{report_id}/generate  # Test report generation
  # Returns generated report file or download URL
```

Note: Adding/editing widgets and groups in reports uses the same API calls as dashboards (Scenarios 1.2-1.7, 1.12).

##### Scenario 1.18: Edit Report

**UI Flow:**
Similar to dashboard editing scenarios (1.2-1.7, 1.12), with report-specific additions:

1. Open report (user has edit permissions)
2. Edit report content:
   - Add/remove/move widgets (see Scenario 1.2, 1.3, 1.13)
   - Edit widget settings (see Scenario 1.4, 1.5)
   - Create/edit/delete groups (see Scenario 1.6, 1.7, 1.12, 1.14)
3. Edit report-specific settings:
   - Paper size, orientation
   - Header/footer templates
   - Page breaks
   - Print settings
4. Edit schedule configuration:
   - Enable/disable schedule
   - Change frequency, time, timezone
   - Update recipients list
   - Change delivery format
5. Edit report metadata (name, description, category)
6. Test report generation with new settings
7. Save changes

**API Calls:**
```
GET /api/analytics/v1/gts/{report_id}  # Load report
PATCH /api/analytics/v1/gts/{report_id}  # Update report settings
  # JSON Patch operations on:
  #   - report/entity/items (widgets/groups) - same as dashboard
  #   - report/settings (paper_size, orientation, header, footer, schedule, delivery)
POST /api/analytics/v1/reports/{report_id}/generate  # Test report generation
```

Note: Widget/group management uses same API patterns as dashboard scenarios.

##### Scenario 1.19: Delete Report

**UI Flow:**
1. Navigate to Reports list
2. Select report to delete
3. Click delete button
4. Confirm deletion - warn if report has active schedule
5. Report soft-deleted (sets deleted_at timestamp)
6. Associated schedule automatically disabled

**API Calls:**
```
DELETE /api/analytics/v1/gts/{report_id}
  # Soft-delete report (sets deleted_at timestamp)
  # Disables schedule if active
  # Returns: 204 No Content
```

---

#### 2. Template Developer / Frontend Developer

##### Scenario 2.1: Develop Custom Widget Template

**UI Flow:**
1. Set up local development environment
2. Create widget template project structure
3. Implement template interface:
   - render() method
   - renderConfigEditor() method
   - validateConfig() method
4. Define configuration schema
5. Test template with mock data
6. Bundle template assets (JS, CSS)
7. Navigate to Developer Console → Templates
8. Register template metadata
9. Configure sharing:
   - Choose visibility: private, specific tenants, or all tenants
   - Select tenants if sharing with specific tenants
10. Upload template bundle

**API Calls:**
```
POST /api/analytics/v1/gts  # Create template config schema type
  # Type ID: gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.template_config.v1~acme.charts._.line_config.v1~
  # JSON Schema for template configuration
POST /api/analytics/v1/gts  # Create widget template instance
  # Type: gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~
  # Instance ID: gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~acme.charts._.line_chart.v2
  # Links: config_schema_id, query_returns_schema_id, category_id
PUT /api/analytics/v1/gts/{template_id}/enablement  # Share widget template with tenants (optional)
  # Body: { "enabled_for": ["tenant-1", "tenant-2"] } or { "enabled_for": "all" }
  # Allows specified tenants or all tenants to use this widget template
POST /api/analytics/v1/templates/{template_id}/bundle  # Upload JS implementation
```

##### Scenario 2.2: Create Values Selector Template

**UI Flow:**
1. Develop custom selector component (e.g., hierarchical tree selector)
2. Implement selector interface
3. Define selector configuration schema
4. Test with sample values
5. Bundle component
6. Navigate to Developer Console → Templates
7. Register template metadata
8. Configure sharing:
   - Choose visibility: private, specific tenants, or all tenants
   - Select tenants if sharing with specific tenants
9. Upload template bundle

**API Calls:**
```
POST /api/analytics/v1/gts  # Create values selector template instance
  # Type: gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.values_selector.v1~
  # Instance ID: gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.values_selector.v1~acme.ui._.tree_selector.v1
  # Links: config_schema_id, values_schema_id
PUT /api/analytics/v1/gts/{template_id}/enablement  # Share values selector template with tenants (optional)
  # Body: { "enabled_for": ["tenant-1", "tenant-2"] } or { "enabled_for": "all" }
  # Allows specified tenants or all tenants to use this values selector template
POST /api/analytics/v1/templates/{template_id}/bundle  # Upload JS implementation
```

##### Scenario 2.3: Update Template Bundle

**UI Flow:**
1. Fix bugs or add features to existing template
2. Test changes locally with mock data
3. Bundle updated template assets
4. Navigate to Developer Console → Templates
5. Select template to update
6. Upload new bundle version
7. New bundle replaces previous version
8. UI cache invalidated (users get new version)

**API Calls:**
```
POST /api/analytics/v1/templates/{template_id}/bundle  # Upload new bundle
  # Replaces previous bundle
  # UI cache invalidated via ETag/Last-Modified headers
  # Template metadata (GTS entity) remains unchanged
```

##### Scenario 2.4: Edit Template Metadata

**UI Flow:**
1. Navigate to Developer Console → Templates
2. Select template to edit
3. Update template metadata:
   - Name, description, category
   - config_schema_id (if schema evolved)
   - query_returns_schema_id (for widget templates)
   - values_schema_id (for values selector templates)
4. Save changes

**API Calls:**
```
GET /api/analytics/v1/gts/{template_id}  # Load template metadata
PATCH /api/analytics/v1/gts/{template_id}  # Update template metadata
  # JSON Patch operations on template properties
  # Can update: name, description, category_id, schema references
```

##### Scenario 2.5: Delete Widget Template

**UI Flow:**
1. Navigate to Developer Console → Templates
2. Select widget template to delete
3. Click delete button
4. Confirm deletion - warn if template is used in widgets
5. Template soft-deleted (metadata + bundle)

**API Calls:**
```
DELETE /api/analytics/v1/gts/{template_id}
  # Soft-delete template metadata (sets deleted_at timestamp)
  # Bundle remains accessible for existing widgets but hidden from selection
  # Returns: 204 No Content
```

##### Scenario 2.6: Delete Values Selector Template

**UI Flow:**
1. Navigate to Developer Console → Templates
2. Select values selector template to delete
3. Click delete button
4. Confirm deletion - warn if template is used in datasource render_options
5. Template soft-deleted

**API Calls:**
```
DELETE /api/analytics/v1/gts/{template_id}
  # Soft-delete values selector template (sets deleted_at timestamp)
  # Bundle remains accessible for existing datasources but hidden from selection
  # Returns: 204 No Content
```

---

#### 3. End User / Business Consumer

##### Scenario 3.1: View Dashboard

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

##### Scenario 3.2: Interact with Widget Controls and Drill Down

**UI Flow:**
1. User opens dashboard
2. Interacts with individual widget controls via datasource.render_options:
   - **Filters:** Selects date range (last 30 days), region (EMEA), status from dropdowns
   - **Time Range:** Chooses quick range (Last 7 days, This month) or custom date range with timezone
   - **Search:** Enters full-text search query (e.g., "urgent orders")
   - **Sorting:** Sorts by column (revenue desc), enables multi-column sort
   - **Pagination:** Changes page size (10, 25, 50, 100), navigates pages
   - **Grouping:** Groups by category, applies aggregation functions (SUM, AVG, COUNT)
   - Each widget has independent controls configured via datasource.render_options
3. Widget refreshes with applied parameters
4. User clicks on data point in chart (drill-down)
5. Detail view opens with filtered/sorted/grouped data
6. User can drill down further or return

**API Calls:**
```
GET /api/analytics/v1/queries/{query_id}?$filter=date ge 2024-12-01 and region eq 'EMEA'&$search=urgent orders&$orderby=revenue desc&$top=25&$skip=0&$apply=groupby((category),aggregate(revenue with sum as total))
  # Each widget makes independent query call with its own parameters
  # All controls (filters, sorting, pagination, grouping, time, search) are widget-level
  # OData params generated from render_options UI control values
```

##### Scenario 3.3: Export Dashboard/Widget Data

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

##### Scenario 3.4: Subscribe to Scheduled Reports

**UI Flow:**
1. User browses available reports
2. Clicks "Subscribe" on report
3. Configures delivery preferences:
   - Frequency (daily, weekly, monthly)
   - Preferred format (PDF, Excel)
   - Delivery method (email)
4. Saves subscription
5. User receives reports via email on schedule

**API Calls:**
```
GET /api/analytics/v1/gts?$filter=...
POST /api/analytics/v1/gts  # Create subscription instance
  # Type: gts.hypernetix.hyperspot.ax.subscription.v1~
  # Instance ID: gts.hypernetix.hyperspot.ax.subscription.v1~tenant-123~user-456~weekly-sales
  # Contains: user_id, report_id, schedule, delivery, filters
PATCH /api/analytics/v1/gts/{subscription_id}  # Update entity/schedule or entity/delivery
DELETE /api/analytics/v1/gts/{subscription_id}  # Unsubscribe
```

---

#### 4. API Consumer / Application Developer

##### Scenario 4.1: Execute Query Programmatically

**UI Flow:**
N/A - Direct API integration

**API Calls:**
```
GET /api/analytics/v1/queries/{query_id}?$filter=...&$orderby=...&$top=...&$skip=...&$count=...
POST /api/analytics/v1/queries/{query_id}
```

##### Scenario 4.2: Build Custom Analytics Dashboard

**UI Flow:**
N/A - Custom application development

**API Calls:**
```
GET /api/analytics/v1/queries/{query_id}?$filter=...&$orderby=...&$top=...
GET /api/analytics/v1/gts?$filter=...&$select=...
```

##### Scenario 4.3: Integrate with BI Tools

**UI Flow:**
N/A - BI tool configuration (Tableau, Power BI, etc.)

**API Calls:**
```
GET /api/analytics/v1/queries/{query_id}/$metadata
GET /api/analytics/v1/queries/{query_id}?$filter=...&$orderby=...&$count=...
```

##### Scenario 4.4: Automate Analytics Workflows

**UI Flow:**
N/A - Automation scripts/CI-CD pipelines

**API Calls:**
```
GET /api/analytics/v1/queries/{query_id}?$filter=...&$orderby=...
POST /api/analytics/v1/dashboards/{dashboard_id}/export
POST /api/analytics/v1/reports/{report_id}/generate
```

---

## Summary: UI/API Interaction Patterns

### Common Patterns Across All Scenarios

1. **Authentication Flow**
   - All API calls require JWT Bearer token in `Authorization` header
   - Token contains tenant context for multi-tenancy isolation
   - Token expiration and refresh handled by client libraries

2. **GTS Registry Pattern**
   - All entities registered/retrieved via unified `/gts` endpoint
   - Type registration (schemas, base definitions)
   - Instance registration (dashboards, widgets, datasources)
   - Filtering and search using OData query syntax

3. **Query Execution Pattern**
   - Query metadata registered in GTS Registry
   - Query execution via `/queries/{id}` endpoint
   - OData v4 protocol for filtering, sorting, pagination
   - JWT token propagation to external datasources

4. **Widget Rendering Pattern**
   - Template metadata registered in GTS Registry
   - Template bundle uploaded/downloaded separately
   - Datasource configuration references query
   - Widget combines template + datasource + config

5. **Pagination Pattern**
   - Cursor-based pagination with `$skiptoken`
   - Optional `$count` for total record count
   - `@odata.nextLink` for next page URL
   - Consistent across all list/query endpoints

6. **Export Pattern**
   - Export endpoints accept format parameter (pdf, csv, excel)
   - Filters applied before export
   - Response as downloadable file
   - Async export for large datasets

7. **Error Handling**
   - RFC 7807 Problem Details format
   - Consistent HTTP status codes
   - Detailed error messages with troubleshooting hints
   - Request ID for support tracking

---

## TODO List

### High Priority

- [ ] **Dashboard Management**
  - Dashboard versioning and rollback
  - Dashboard export/import (JSON format)

- [ ] **Layout System**
  - Layout templates and presets

- [ ] **Widget Lifecycle**
  - Widget refresh strategies (real-time, polling, manual)

- [ ] **Admin Panel API**
  - System configuration endpoints
  - User management (create, list, permissions)
  - Tenant management operations
  - Audit log access and filtering

### Medium Priority

- [ ] **Template System Details**
  - Template asset loading (CSS, JS bundles)
  - Template versioning strategy
  - Template marketplace/registry concept
  - Custom template development guide

- [ ] **Query Execution Engine**
  - Query caching strategy (Redis, in-memory)
  - Query timeout and cancellation
  - Parallel query execution for multiple widgets
  - Query result pagination handling

- [ ] **Adapter System**
  - REST adapter implementation details
  - OData adapter query translation
  - GraphQL adapter support
  - Custom adapter development guide

- [ ] **Filters & Parameters**
  - Global dashboard filters (apply to all widgets)

- [ ] **Data Transformation**
  - Aggregation pipeline specification
  - Formula/expression language for computed fields
  - Data joins between multiple queries
  - Time-series data resampling

### Low Priority

- [ ] **Export & Reporting**
  - Dashboard PDF export
  - Widget CSV/Excel export
  - Scheduled report generation
  - Email delivery integration

- [ ] **Visualization Library**
  - Charting library selection (Chart.js, D3.js, etc.)
  - Chart type catalog (bar, line, pie, scatter, etc.)
  - Interactive chart behaviors (drill-down, tooltips)

- [ ] **Real-time Updates**
  - WebSocket integration for live data
  - Server-Sent Events (SSE) for updates
  - Change detection and delta updates
  - Connection resilience and reconnection

- [ ] **Collaboration Features**
  - Dashboard comments and annotations
  - Real-time collaborative editing
  - Change notifications
  - Activity feed

- [ ] **Monitoring & Observability**
  - Query performance metrics
  - Widget render time tracking
  - Error rate monitoring
  - User analytics (widget usage, popular dashboards)

### Architecture & Infrastructure

- [ ] **Database Schema**
  - PostgreSQL table definitions for GTS entities
  - Indexes for query optimization
  - Migration strategy
  - Backup and restore procedures

- [ ] **Caching Strategy**
  - Query result caching (TTL, invalidation)
  - Template asset caching
  - User permission caching
  - Distributed cache considerations

- [ ] **Testing Strategy**
  - Unit test guidelines
  - Integration test patterns
  - End-to-end test scenarios
  - Performance test benchmarks

- [ ] **Deployment & Operations**
  - Kubernetes deployment manifests
  - Environment configuration
  - Health checks and readiness probes
  - Rolling update strategy

### Documentation

- [ ] **API Examples**
  - Complete request/response examples for all endpoints
  - Common workflow scenarios
  - Error handling patterns
  - Rate limiting guidelines

- [ ] **Developer Guides**
  - Plugin development tutorial
  - Custom template creation guide
  - Query adapter implementation guide
  - GTS type extension guide

- [ ] **User Documentation**
  - Dashboard creation tutorial
  - Widget configuration guide
  - Filter usage examples
  - Troubleshooting guide

---

## Notes

**Review Priority**: Focus on High Priority items first, particularly Authentication/Authorization and Dashboard Management, as they form the foundation for the entire system.

**Dependencies**: Some TODO items depend on decisions in other areas (e.g., Template System depends on visualization library choice).

**Timeline**: Estimate completion order based on OpenSpec workflow and implementation phases.