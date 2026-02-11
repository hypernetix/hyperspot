# Multi-endpoint upstream with common suffix alias and X-OAGW-Target-Host header

## Setup

- Upstream `vendor.com` with multiple endpoints sharing common suffix:
  - `endpoints`: `[{"scheme": "https", "host": "us.vendor.com", "port": 443}, {"scheme": "https", "host": "eu.vendor.com", "port": 443}]`
  - `alias`: `vendor.com` (auto-generated common suffix)

## Request

```http
POST /api/oagw/v1/proxy/vendor.com/v1/api/resource HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <token>
X-OAGW-Target-Host: us.vendor.com
Content-Type: application/json

{"key": "value"}
```

## Expected behavior

- Request routes to `us.vendor.com:443`
- `X-OAGW-Target-Host` header is required for common suffix alias
- Header is stripped before forwarding to upstream
- Response: `200 OK` with upstream response body

## Validation

- Common suffix alias requires `X-OAGW-Target-Host` header for disambiguation
- Header value must match one of the configured endpoints
- Request succeeds when valid header is provided
