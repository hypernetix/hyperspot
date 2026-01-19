# Type Registry — PRD (LLM Gateway Scope)

GTS schema storage for LLM Gateway tool definitions.

## Scenarios

### S1 Get Schema by ID

LLM Gateway resolves tool schema by ID for function calling.

```mermaid
sequenceDiagram
    participant GW as LLM Gateway
    participant TR as Type Registry

    GW->>TR: get_schema(schema_id)
    TR-->>GW: GTS schema
```

**Schema ID format**: `gts.hx.core.faas.func.v1~<vendor>.<app>.<namespace>.<func_name>.v1`

### S2 Batch Get Schemas

LLM Gateway resolves multiple tool schemas in single request.

```mermaid
sequenceDiagram
    participant GW as LLM Gateway
    participant TR as Type Registry

    GW->>TR: get_schemas([schema_id, ...])
    TR-->>GW: [GTS schema, ...]
```

## Errors

| Error | HTTP | Description |
|-------|------|-------------|
| `schema_not_found` | 404 | Schema ID does not exist |
| `invalid_schema_id` | 400 | Malformed schema ID |
