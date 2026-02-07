# Delete upstream cascades routes

## Setup

1. Create upstream `alias=httpbin.org`.
2. Create one or more routes referencing that upstream.

## Delete upstream

```http
DELETE /api/oagw/v1/upstreams/gts.x.core.oagw.upstream.v1~<uuid> HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected response

- `204 No Content`

## Post-conditions

- `GET /api/oagw/v1/routes?$filter=upstream_id eq '<uuid>'` returns empty (or routes are deleted).
- Proxy invocation:

```http
POST /api/oagw/v1/proxy/httpbin.org/post HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

Expected: `404` gateway error (`UPSTREAM_NOT_FOUND` or `ROUTE_NOT_FOUND`), `application/problem+json`.
