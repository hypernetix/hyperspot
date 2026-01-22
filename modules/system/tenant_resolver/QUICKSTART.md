# Tenant Resolver - Quickstart

Multi-tenant hierarchy management. Tenants form a tree structure with parent/child relationships.

> **Note:** Requires `make example` (includes `--features tenant-resolver-example`)

## Examples

### Get Root Tenant

```bash
curl -s http://127.0.0.1:8087/tenant-resolver/v1/root | python3 -m json.tool
```

**Output:**
```json
{
    "id": "00000000000000000000000000000001",
    "parentId": "",
    "status": "ACTIVE",
    "isAccessibleByParent": true
}
```

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

### Get Children of a Tenant

```bash
TENANT_ID="00000000000000000000000000000001"
curl -s "http://127.0.0.1:8087/tenant-resolver/v1/tenants/$TENANT_ID/children" | python3 -m json.tool
```

### Get Parent Chain

```bash
curl -s "http://127.0.0.1:8087/tenant-resolver/v1/tenants/$TENANT_ID/parents" | python3 -m json.tool
```
