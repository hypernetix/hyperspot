# Domain Spec: GTS Types

## GTS Type System

GTS types are defined as **JSON Schema** documents following JSON Schema draft 2020-12. Each type is stored as a JSON file with accompanying mock/example instances.

### GTS Identifier Format

```
gts.<vendor>.<package>.<namespace>.<type>.v<MAJOR>[.<MINOR>][~[chain]]
```

**Validation Rules:**
- Each segment: alphanumeric + underscore, start with letter
- Version: major required, minor optional
- Type identifiers (schemas) end with `~`
- Instance identifiers do not end with `~`
- Chain: optional chained identifier after `~`

**Examples:**
- Type: `gts.hypernetix.hyperspot.ax.schema.v1~`
- Instance: `gts.hypernetix.hyperspot.ax.query.v1~acme.monitoring._.server_metrics.v1`
- Chained: `gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.dashboard.v1~`

### JSON Schema Structure

Each GTS type is defined as a JSON Schema document:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "gts://gts.hypernetix.hyperspot.ax.datasource.v1~",
  "type": "object",
  "properties": {
    "query_id": {
      "type": "string",
      "description": "GTS identifier of the query registration",
      "x-gts-ref": "gts.hypernetix.hyperspot.ax.query.v1~"
    },
    "params": {
      "description": "Query parameters",
      "$ref": "gts://gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_params.v1~"
    }
  },
  "required": ["query_id", "params"]
}
```

**Key Schema Extensions:**
- `x-gts-ref` - Declares that a string field contains a GTS identifier
- `$ref` with `gts://` prefix - References another GTS schema

### Instance Structure

Instances are JSON documents that conform to a GTS type schema:

```json
{
  "query_id": "gts.hypernetix.hyperspot.ax.query.v1~acme.monitoring._.server_metrics.v1",
  "params": {
    "filters": {
      "status": ["active"]
    },
    "pagination": {
      "limit": 50
    }
  }
}
```

## Directory Structure

GTS entities are organized in `gts/`:

```
gts/
└── types/              # Type definitions (schemas)
    ├── schema/v1/      # Schema type definitions
    ├── query/v1/       # Query type definitions
    ├── datasource/v1/  # Datasource type definitions
    ├── template/v1/    # Template type definitions
    ├── item/v1/        # Item type definitions
    ├── layout/v1/      # Layout type definitions
    └── category/v1/    # Category type definitions
```

Each type directory contains:
- `*.schema.json` - JSON Schema definition (type definition)

Future structure will include:
- `gts/instances/` - Concrete instances of types
- `gts/registry/` - Registry of GTS entities

## Type Definitions

All types are defined using GTS (Generic Type System) with JSON Schema format. Each type definition is stored in `gts/types/` directory with corresponding JSON Schema file.

### Schema Types

Schema types define data structures and validation rules for types referenced by other types and instances. All schema types inherit from base schema and must provide mock objects via `x-gts-mock` extension.

#### Base Schema
**Type ID:** `gts.hypernetix.hyperspot.ax.schema.v1~`  
**File:** `gts/types/schema/v1/base.schema.json`  
**Purpose:** Base type for all schema definitions. Requires mandatory `x-gts-mock` for validation.

#### Query Returns Schema
**Type ID:** `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~`  
**File:** `gts/types/schema/v1/query_returns.schema.json`  
**Purpose:** Schema for query result data. Defines paginated result sets with scalar-only field values (no nested objects/arrays).

#### Template Config Schema
**Type ID:** `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.template_config.v1~`  
**File:** `gts/types/schema/v1/template_config.schema.json`  
**Purpose:** Base schema for template configuration. Derived types define specific properties for different template types.

#### Query Values Schema
**Type ID:** `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~hypernetix.hyperspot.ax.values.v1~`  
**File:** `gts/types/schema/v1/values.schema.json`  
**Purpose:** Schema for value lists used in UI selectors and filters. Returns value/label/description/metadata structure for dropdowns and pickers.

---

### Query Types

Query types define data retrieval operations using OData v4 protocol for standardized query interface with built-in support across DWH systems, BI tools, and UI libraries.

#### Query
**Type ID:** `gts.hypernetix.hyperspot.ax.query.v1~`  
**File:** `gts/types/query/v1/query.schema.json`  
**Purpose:** Query registration with OData v4 integration. References capabilities and returns schemas for OData metadata generation.

**Key Fields:**
- `name`, `description`, `icon` - Basic metadata
- `category` - Reference to query category
- `api_endpoint` - External API endpoint URL
- `capabilities_id` - Reference to query capabilities
- `returns_schema_id` - Reference to returns schema
- `contract_format` - Protocol format (native, odata, rest)

#### Query Capabilities
**Type ID:** `gts.hypernetix.hyperspot.ax.query_capabilities.v1~`  
**File:** `gts/types/query/v1/query_capabilities.schema.json`  
**Purpose:** OData Capabilities annotations in JSON format. Defines supported query operations ($filter, $orderby, $top, etc) and their restrictions. Maps to OData CSDL JSON v4.01 Capabilities vocabulary.

**Key Fields:**
- `filterFunctions` - Supported filter operators
- `filterableProperties` - Fields that can be filtered
- `sortableProperties` - Fields that can be sorted
- `topSupported`, `skipSupported`, `countSupported`, `selectSupported` - Feature flags

#### Query Values
**Type ID:** `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_values.v1~`  
**File:** `gts/types/query/v1/values.schema.json`  
**Purpose:** Default OData query options for datasources. Stores default `$filter`, `$orderby`, `$select` expressions.

---

### Category Types

Category types organize and group related GTS entities by domain, type, or purpose. All category types provide hierarchical classification with name, description, and icon.

#### Base Category
**Type ID:** `gts.hypernetix.hyperspot.ax.category.v1~`  
**File:** `gts/types/category/v1/base.schema.json`  
**Purpose:** Base category type for organizing GTS entities.

#### Query Category
**Type ID:** `gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.query.v1~`  
**File:** `gts/types/category/v1/query.schema.json`  
**Purpose:** Category for query definitions. Organizes queries by domain, data source, or functional area.

#### Template Category
**Type ID:** `gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.template.v1~`  
**File:** `gts/types/category/v1/template.schema.json`  
**Purpose:** Category for UI component templates. Organizes templates by visualization type or component family.

#### Datasource Category
**Type ID:** `gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.datasource.v1~`  
**File:** `gts/types/category/v1/datasource.schema.json`  
**Purpose:** Category for datasource configurations. Organizes datasources by data domain or system integration.

#### Widget Category
**Type ID:** `gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.widget.v1~`  
**File:** `gts/types/category/v1/widget.schema.json`  
**Purpose:** Category for widget items. Organizes widgets by visualization type or data domain.

#### Item Category
**Type ID:** `gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.item.v1~`  
**File:** `gts/types/category/v1/item.schema.json`  
**Purpose:** Category for layout items (widgets and groups). Organizes reusable item instances.

#### Group Category
**Type ID:** `gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.group.v1~`  
**File:** `gts/types/category/v1/group.schema.json`  
**Purpose:** Category for group items. Organizes groups that serve as containers for related widgets.

#### Dashboard Category
**Type ID:** `gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.dashboard.v1~`  
**File:** `gts/types/category/v1/dashboard.schema.json`  
**Purpose:** Category for dashboard layouts. Organizes dashboards by business domain or team.

#### Layout Category
**Type ID:** `gts.hypernetix.hyperspot.ax.category.v1~hypernetix.hyperspot.ax.layout.v1~`  
**File:** `gts/types/category/v1/layout.schema.json`  
**Purpose:** Category for layout definitions (dashboards and reports). Organizes layouts by use case or format.

---

### Template Types

Template types define rendering logic and configuration for visual components.

#### Base Template
**Type ID:** `gts.hypernetix.hyperspot.ax.template.v1~`  
**File:** `gts/types/template/v1/base.schema.json`  
**Purpose:** Base template type defining reusable UI component configurations. Specifies visual presentation, behavior, and configuration schema.

**Key Fields:**
- `name`, `description`, `icon` - Basic metadata
- `category` - Reference to template category
- `version` - Semantic version
- `config_schema_id` - Reference to configuration schema
- `asset_url` - URL to template JavaScript bundle
- `thumbnail_url` - Preview image URL

#### Widget Template
**Type ID:** `gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~`  
**File:** `gts/types/template/v1/widget.schema.json`  
**Purpose:** Widget template for data visualizations. Defines rendering logic, config schema, and expected data structure (query_schema_id).

**Additional Fields:**
- `query_schema_id` - Reference to expected data schema from datasource

#### Values Selector Template
**Type ID:** `gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.values_selector.v1~`  
**File:** `gts/types/template/v1/values_selector.schema.json`  
**Purpose:** Template for value selection UI components (dropdowns, multi-selects, autocomplete). Used in filter controls.

---

### Datasource Type

#### Datasource
**Type ID:** `gts.hypernetix.hyperspot.ax.datasource.v1~`  
**File:** `gts/types/datasource/v1/datasource.schema.json`  
**Purpose:** Datasource connects a query with its runtime parameters and UI configuration. Encapsulates data retrieval logic and user interface controls (filters, sorting, pagination, grouping, time range selectors).

**Key Fields:**
- `query_id` - Reference to query definition
- `default_query_values` - Default OData parameters ($filter, $orderby, $select)
- `refresh_interval` - Auto-refresh interval in seconds
- `ui_controls` - Filter controls, buttons, and UI elements configuration

---

### Item Types

Item types are reusable building blocks that can be placed in layouts.

#### Base Item
**Type ID:** `gts.hypernetix.hyperspot.ax.item.v1~`  
**File:** `gts/types/item/v1/base.schema.json`  
**Purpose:** Base item type for dashboard and report components. Defines name, description, icon, category, size (percentage-based), and type-specific settings.

#### Widget Item
**Type ID:** `gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~`  
**File:** `gts/types/item/v1/widget.schema.json`  
**Purpose:** Widget item for data visualizations. Combines template (defines rendering) with datasource (provides data). Primary building block for dashboards and reports.

**Key Fields:**
- `template_id` - Reference to widget template
- `datasource_id` - Reference to datasource
- `template_config` - Configuration object matching template's config schema
- `size` - Width/height settings

#### Group Item
**Type ID:** `gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.group.v1~`  
**File:** `gts/types/item/v1/group.schema.json`  
**Purpose:** Group item for organizing and containing other items. Supports collapsible sections and hierarchical layout structures.

**Key Fields:**
- `items` - Array of child item references
- `collapsible` - Whether group can be collapsed
- `default_collapsed` - Initial collapsed state

---

### Layout Types

Layout types organize items into dashboards and reports.

#### Base Layout
**Type ID:** `gts.hypernetix.hyperspot.ax.layout.v1~`  
**File:** `gts/types/layout/v1/base.schema.json`  
**Purpose:** Base layout type for organizing items. Defines name, description, icon, category, and ordered array of items.

#### Dashboard Layout
**Type ID:** `gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.dashboard.v1~`  
**File:** `gts/types/layout/v1/dashboard.schema.json`  
**Purpose:** Dashboard layout for interactive, real-time data monitoring. Supports auto-refresh and theme customization.

**Key Fields:**
- `items` - Array of item references with grid positions
- `auto_refresh` - Auto-refresh configuration
- `theme` - Visual theme (light, dark)
- `global_filters` - Dashboard-wide filter controls

#### Report Layout
**Type ID:** `gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~`  
**File:** `gts/types/layout/v1/report.schema.json`  
**Purpose:** Report layout for scheduled, printable, and exportable presentations. Supports scheduled generation and multiple export formats.

**Key Fields:**
- `items` - Array of item references
- `paper_size` - Paper size (A4, Letter, etc.)
- `orientation` - Portrait or landscape
- `header_template`, `footer_template` - Page header/footer HTML
- `schedule` - Scheduling configuration
- `delivery` - Email delivery configuration

## Implementation Status

**Completed:**
- ✅ 26 GTS type schemas defined (8 category + 18 other types)
- ✅ All schemas follow JSON Schema draft 2020-12
- ✅ All schemas use `gts://` prefix per GTS spec v0.4
- ✅ All `$id` and `$ref` fields validated
- ✅ All `x-gts-ref` fields validated
- ✅ Category schemas include explicit `type: object`
- ✅ Validation with GTS CLI tools (`~/bin/gts`)
- ✅ Directory structure reorganized to `gts/types/`

**Location:** `gts/types/` directory in module root

**Validation:** 
- GTS CLI: `~/bin/gts --path gts/types list`
- GTS Kit: Electron app or web viewer

**Next Steps:**
- Create instance definitions in `gts/instances/`
- Implement Rust types for GTS schemas
- Build API endpoints for GTS registry
