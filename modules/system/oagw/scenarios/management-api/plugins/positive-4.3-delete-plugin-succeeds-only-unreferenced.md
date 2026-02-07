# Delete plugin succeeds only when unreferenced

## Scenario A: delete unreferenced plugin

1. Create plugin `gts.x.core.oagw.plugin.guard.v1~<uuid>`.
2. Ensure it is not referenced by any upstream/route.

```http
DELETE /api/oagw/v1/plugins/gts.x.core.oagw.plugin.guard.v1~<uuid> HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

Expected:
- `204 No Content`

## Scenario B: delete referenced plugin

1. Create plugin `...~<uuid>`.
2. Attach it to an upstream or route.

```http
DELETE /api/oagw/v1/plugins/gts.x.core.oagw.plugin.guard.v1~<uuid> HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

Expected:
- `409 Conflict`
- `Content-Type: application/problem+json`
- Body includes:
  - `type` = `gts.x.core.errors.err.v1~x.oagw.plugin.in_use.v1`
  - `referenced_by` listing upstreams/routes (if implemented).
