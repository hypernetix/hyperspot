# List upstreams includes disabled resources

## Step 1: Create enabled upstream

```http
POST /api/oagw/v1/upstreams HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "api.enabled.com", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "alias": "api.enabled.com"
}
```

Expected: `201 Created` with `enabled: true` (default).

## Step 2: Create disabled upstream

```http
POST /api/oagw/v1/upstreams HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "api.disabled.com", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "alias": "api.disabled.com",
  "enabled": false
}
```

Expected: `201 Created` with `enabled: false`.

## Step 3: List all upstreams

```http
GET /api/oagw/v1/upstreams HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected response

- `200 OK`
- Response contains both upstreams:
  - `api.enabled.com` with `enabled: true`
  - `api.disabled.com` with `enabled: false`
- Each resource includes `enabled` field explicitly

## Step 4: Filter to enabled only (optional OData)

```http
GET /api/oagw/v1/upstreams?$filter=enabled%20eq%20true HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected response

- `200 OK`
- Response contains only `api.enabled.com`
- `api.disabled.com` is excluded
