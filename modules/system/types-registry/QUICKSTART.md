# Types Registry - Quickstart

Central repository for JSON schemas in the Global Type System (GTS). Manages type definitions with hierarchical namespacing for structured data validation.

**Features:**
- Hierarchical type IDs (e.g., `com.example.user.profile`)
- JSON Schema storage and retrieval
- Type versioning and evolution
- Schema validation

**Use cases:**
- Define data contracts between modules
- Validate API payloads against registered schemas
- Document data structures across the system

Full API documentation: <http://127.0.0.1:8087/docs>

## Examples

### List All GTS Entities

```bash
curl -s http://127.0.0.1:8087/types-registry/v1/entities | python3 -m json.tool | head -50
```

For additional endpoints, see <http://127.0.0.1:8087/docs>.
