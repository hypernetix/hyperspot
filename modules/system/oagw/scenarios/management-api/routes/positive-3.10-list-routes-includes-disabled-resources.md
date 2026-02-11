# List routes includes disabled resources

## Step 1: Create upstream

```http
POST /api/oagw/v1/upstreams HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
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

Expected: `201 Created` with upstream id.

## Step 2: Create enabled route

```http
POST /api/oagw/v1/routes HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "upstream_id": "gts.x.core.oagw.upstream.v1~<uuid>",
  "match": {
    "http": {
      "methods": ["GET"],
      "path": "/get"
    }
  }
}
```

Expected: `201 Created` with `enabled: true` (default).

## Step 3: Create disabled route

```http
POST /api/oagw/v1/routes HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "upstream_id": "gts.x.core.oagw.upstream.v1~<uuid>",
  "match": {
    "http": {
      "methods": ["POST"],
      "path": "/post"
    }
  },
  "enabled": false
}
```

Expected: `201 Created` with `enabled: false`.

## Step 4: List all routes

```http
GET /api/oagw/v1/routes HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected response

- `200 OK`
- Response contains both routes:
  - `/get` route with `enabled: true`
  - `/post` route with `enabled: false`
- Each resource includes `enabled` field explicitly

## Step 5: Filter to enabled only (optional OData)

```http
GET /api/oagw/v1/routes?$filter=enabled%20eq%20true HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected response

- `200 OK`
- Response contains only `/get` route
- `/post` route is excluded
