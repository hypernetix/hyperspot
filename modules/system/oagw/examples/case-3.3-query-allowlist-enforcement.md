# Query allowlist enforcement

## Setup

Route config:
- `query_allowlist`: `["tag"]`

## Inbound request (unknown query param)

```http
POST /api/oagw/v1/proxy/<alias>/post?tag=ok&debug=1 HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{"name":"test"}
```

## Expected response

- `400 Bad Request`
- `Content-Type: application/problem+json`
- `X-OAGW-Error-Source: gateway`
- `detail` mentions unknown query parameter `debug`.
