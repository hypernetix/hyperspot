# Types Registry - Quickstart

GTS (Global Type System) schema registry. Stores JSON schemas with hierarchical IDs.

## Examples

### List All GTS Entities

```bash
curl -s http://127.0.0.1:8087/types-registry/v1/entities | python3 -m json.tool | head -50
```

### Get a Specific GTS Entity

```bash
curl -s "http://127.0.0.1:8087/types-registry/v1/entities/gts.x.core.modkit.plugin.v1~" | python3 -m json.tool
```
