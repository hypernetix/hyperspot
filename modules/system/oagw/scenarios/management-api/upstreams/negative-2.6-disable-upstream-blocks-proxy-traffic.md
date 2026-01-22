# Disable upstream blocks proxy traffic (including descendants)

## Step 1: Create upstream + route

Create upstream `alias=httpbin.org`, then route `POST /post`.

## Step 2: Disable upstream

```http
PUT /api/oagw/v1/upstreams/gts.x.core.oagw.upstream.v1~<uuid> HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "id": "gts.x.core.oagw.upstream.v1~<uuid>",
  "enabled": false,
  "alias": "httpbin.org",
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "httpbin.org", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1"
}
```

Expected: `200 OK`, `enabled=false`.

## Step 3: Invoke proxy

```http
POST /api/oagw/v1/proxy/httpbin.org/post HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{"name":"test"}
```

## Expected response

- Gateway-generated rejection.
- `503 Service Unavailable` (or implementation-defined status)
- `Content-Type: application/problem+json`
- `X-OAGW-Error-Source: gateway`

## Descendant tenant variant

- Disable upstream at ancestor tenant.
- Descendant invoking the same alias is rejected as well.
