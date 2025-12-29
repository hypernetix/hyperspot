# Domain Spec Delta: GTS Types

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

### Example Files

Each type consists of two files:

**1. Schema file** (`.schema.json`)
- Defines the structure using JSON Schema
- Uses GTS identifier as `$id`
- References other schemas with `$ref`
- Marks GTS identifier fields with `x-gts-ref`

**2. Example file** (`.example.json`)
- Provides concrete instance
- Must validate against schema
- Shows realistic usage

## Directory Structure

GTS type definitions will be organized as:

```
gts-types/
├── schema/
│   └── v1/
│       ├── base.schema.json
│       ├── query_returns.schema.json
│       ├── query_params.schema.json
│       ├── template_config.schema.json
│       └── values.schema.json
├── query/
│   └── v1/
│       ├── base.schema.json
│       └── values.schema.json
├── datasource/
│   └── v1/
│       └── base.schema.json
├── template/
│   └── v1/
│       ├── base.schema.json
│       ├── widget.schema.json
│       └── values_selector.schema.json
├── item/
│   └── v1/
│       ├── base.schema.json
│       ├── widget.schema.json
│       └── group.schema.json
├── layout/
│   └── v1/
│       ├── base.schema.json
│       ├── dashboard.schema.json
│       └── report.schema.json
└── category/
    └── v1/
        └── base.schema.json
```

Each `.schema.json` file is a complete JSON Schema definition with:
- `$schema` field
- `$id` field (GTS URI)
- `properties`, `required`, etc.
- Schema-specific extensions (`x-gts-ref`, `x-gts-mock`)

## Default Type Definitions

### Schema Types

All schema types inherit from base schema type and require `x-gts-mock` field.

**Base Schema Type:**
```
gts.hypernetix.hyperspot.ax.schema.v1~
```

**Schema Subtypes:**
- `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~`
- `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_params.v1~`
- `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.template_config.v1~`
- `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.values.v1~`

### Query Types

**Query Params Spec:**
```
gts.hypernetix.hyperspot.ax.query_params.v1~
```
Describes query parameter capabilities and constraints.

**Query Registration:**
```
gts.hypernetix.hyperspot.ax.query.v1~
```
Base query registration type.

**Query Values Registration:**
```
gts.hypernetix.hyperspot.ax.query.v1~hypernetix.hyperspot.ax.values.v1~
```
Specialized query for returning values (dropdown options, etc).

### Datasource Type

```
gts.hypernetix.hyperspot.ax.datasource.v1~
```
Query configuration with parameters and UI config.

### Template Types

**Base Template:**
```
gts.hypernetix.hyperspot.ax.template.v1~
```

**Widget Template:**
```
gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~
```

**Values Selector Template:**
```
gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.values_selector.v1~
```

### Item Types

**Base Item:**
```
gts.hypernetix.hyperspot.ax.item.v1~
```

**Widget:**
```
gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~
```

**Group:**
```
gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.group.v1~
```

### Layout Types

**Base Layout:**
```
gts.hypernetix.hyperspot.ax.layout.v1~
```

**Dashboard:**
```
gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.dashboard.v1~
```

**Report:**
```
gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~
```

### Category Type

```
gts.hypernetix.hyperspot.ax.category.v1~
```
For organizing types into categories.

## Security Requirements

- All registry methods require `SecurityCtx`
- Tenant isolation: users can only see types enabled for their tenant
- Base types: visible to all tenants, cannot be modified/deleted
- Custom types: only creator tenant can modify/delete
- Admin role required for enablement changes
