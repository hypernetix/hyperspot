# Disable route blocks proxy traffic

## Setup

1. Create upstream `alias=httpbin.org`.
2. Create route `POST /post`.

## Disable route

```http
PUT /api/oagw/v1/routes/gts.x.core.oagw.route.v1~<route-uuid> HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "id": "gts.x.core.oagw.route.v1~<route-uuid>",
  "enabled": false,
  "upstream_id": "gts.x.core.oagw.upstream.v1~<upstream-uuid>",
  "match": {
    "http": {
      "methods": ["POST"],
      "path": "/post",
      "query_allowlist": [],
      "path_suffix_mode": "disabled"
    }
  }
}
```

## Invoke proxy

```http
POST /api/oagw/v1/proxy/httpbin.org/post HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{"name":"test"}
```

## Expected response

- Rejected by gateway.
- `404` (route not found) or `503` (implementation-defined) (lock expected behavior).
- If body is present: `application/problem+json` and `X-OAGW-Error-Source: gateway`.
