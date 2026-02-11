# Multi-endpoint upstream with explicit alias and X-OAGW-Target-Host header

## Setup

- Upstream `my-service` with multiple endpoints:
  - `endpoints`: `[{"scheme": "https", "host": "server-a.example.com", "port": 443}, {"scheme": "https", "host": "server-b.example.com", "port": 443}]`
  - `alias`: `my-service`

## Request

```http
GET /api/oagw/v1/proxy/my-service/v1/status HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <token>
X-OAGW-Target-Host: server-a.example.com
```

## Expected behavior

- Request routes directly to `server-a.example.com:443`
- Round-robin load balancing is bypassed
- `X-OAGW-Target-Host` header is stripped before forwarding
- Response: `200 OK` with upstream response body

## Validation

- `X-OAGW-Target-Host` header allows explicit endpoint selection
- Header value is validated against configured endpoints
- Load balancing is bypassed when header is present
