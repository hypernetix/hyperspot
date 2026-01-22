# Multi-endpoint upstream with explicit alias, no X-OAGW-Target-Host header

## Setup

- Upstream `my-service` with multiple endpoints:
  - `endpoints`: `[{"scheme": "https", "host": "server-a.example.com", "port": 443}, {"scheme": "https", "host": "server-b.example.com", "port": 443}]`
  - `alias`: `my-service` (explicit, no common suffix)

## Request

```http
GET /api/oagw/v1/proxy/my-service/v1/status HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <token>
```

## Expected behavior

- Request routes using round-robin load balancing between `server-a.example.com` and `server-b.example.com`
- No `X-OAGW-Target-Host` header required for explicit alias
- Response: `200 OK` with upstream response body

## Validation

- Multi-endpoint upstreams with explicit alias support round-robin without header
- Load balancing distributes requests across all endpoints
