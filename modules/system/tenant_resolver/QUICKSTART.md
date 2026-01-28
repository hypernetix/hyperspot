# Tenant Resolver - Quickstart

Multi-tenant hierarchy management. Tenants form a tree structure with parent/child relationships.

> **Note:** Requires `make example` (includes `--features tenant-resolver-example`)

> **Full API Documentation:** <http://127.0.0.1:8087/docs> - Interactive docs with all endpoints, parameters, and "Try it out" buttons.

## Quick Example

### List All Tenants

```bash
curl -s http://127.0.0.1:8087/tenant-resolver/v1/tenants | python3 -m json.tool
```

**Output:**
```json
{
    "items": [
        {"id": "00000000000000000000000000000001", "parentId": "", "status": "ACTIVE"},
        {"id": "00000000000000000000000000000010", "parentId": "00000000000000000000000000000001", "status": "ACTIVE"}
    ],
    "page_info": {"next_cursor": null, "prev_cursor": null, "limit": 100}
}
```

## More Examples

For additional endpoints (`/root`, `/tenants/{id}`, `/tenants/{id}/children`, `/tenants/{id}/parents`, etc.), see the interactive documentation at <http://127.0.0.1:8087/docs>.
