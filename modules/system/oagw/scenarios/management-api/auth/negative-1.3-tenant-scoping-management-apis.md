# Management API tenant scoping

## Setup

1. Tenant B creates an upstream.
2. Tenant A attempts to read Tenant B upstream.

## Step 1: Tenant B creates upstream

```http
POST /api/oagw/v1/upstreams HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-b-token>
Content-Type: application/json

{
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "httpbin.org", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "alias": "httpbin.org"
}
```

Expected: `201 Created` with `id` like `gts.x.core.oagw.upstream.v1~<uuid>`.

## Step 2: Tenant A reads Tenant B upstream

```http
GET /api/oagw/v1/upstreams/gts.x.core.oagw.upstream.v1~<uuid-from-step-1> HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-a-token>
```

## Expected response

- Must not leak cross-tenant data.
- Expected: `403 Forbidden` or `404 Not Found` (lock expected behavior).
- If body is present, it must be `application/problem+json`.
