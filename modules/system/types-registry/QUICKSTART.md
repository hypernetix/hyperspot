# Types Registry - Quickstart

GTS (Global Type System) schema registry. Stores JSON schemas with hierarchical IDs.

> **Full API Documentation:** <http://127.0.0.1:8087/docs> - Interactive docs with all endpoints, parameters, and "Try it out" buttons.

## Quick Example

### List All GTS Entities

```bash
curl -s http://127.0.0.1:8087/types-registry/v1/entities | python3 -m json.tool | head -50
```

## More Examples

For additional endpoints (`/entities/{id}`, etc.), see the interactive documentation at <http://127.0.0.1:8087/docs>.
