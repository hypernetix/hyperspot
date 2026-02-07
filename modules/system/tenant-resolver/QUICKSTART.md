# Tenant Resolver - Quickstart

Manages multi-tenant hierarchies for data isolation and access control. Tenants are organized in a tree structure where child tenants inherit access constraints from parents.

**Features:**
- Hierarchical tenant relationships (parent/child)
- Tenant status management (ACTIVE, INACTIVE)
- Ancestor and descendant queries
- Root tenant identification

**Use cases:**
- SaaS applications with organizational hierarchies
- Data isolation between customers
- Role-based access control at tenant level

> **Note:** Requires `make example` (includes `--features tenant-resolver-example`)

Full API documentation: <http://127.0.0.1:8087/docs>

## Examples

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

For additional endpoints, see <http://127.0.0.1:8087/docs>.
