# Types Registry - Quickstart

GTS (Global Type System) schema registry. Stores JSON schemas with hierarchical IDs.

Full API documentation: <http://127.0.0.1:8087/docs>

## Examples

### List All GTS Entities

```bash
curl -s http://127.0.0.1:8087/types-registry/v1/entities | python3 -m json.tool | head -50
```

For additional endpoints, see <http://127.0.0.1:8087/docs>.
