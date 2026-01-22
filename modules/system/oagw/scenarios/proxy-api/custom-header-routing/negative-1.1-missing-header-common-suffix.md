# Missing X-OAGW-Target-Host header for common suffix alias

## Setup

- Upstream `vendor.com` with multiple endpoints sharing common suffix:
  - `endpoints`: `[{"scheme": "https", "host": "us.vendor.com", "port": 443}, {"scheme": "https", "host": "eu.vendor.com", "port": 443}]`
  - `alias`: `vendor.com`

## Request

```http
POST /api/oagw/v1/proxy/vendor.com/v1/api/resource HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <token>
Content-Type: application/json

{"key": "value"}
```

## Expected behavior

- Request is rejected with `400 Bad Request`
- Error response follows RFC 9457 Problem Details format:

```http
HTTP/1.1 400 Bad Request
X-OAGW-Error-Source: gateway
Content-Type: application/problem+json

{
  "type": "gts.x.core.errors.err.v1~x.oagw.routing.missing_target_host.v1",
  "title": "Missing Target Host Header",
  "status": 400,
  "detail": "X-OAGW-Target-Host header required for multi-endpoint upstream with common suffix alias. Valid hosts: [us.vendor.com, eu.vendor.com]",
  "instance": "/api/oagw/v1/proxy/vendor.com/v1/api/resource",
  "upstream_id": "gts.x.core.oagw.upstream.v1~...",
  "alias": "vendor.com",
  "valid_hosts": ["us.vendor.com", "eu.vendor.com"]
}
```

## Validation

- Common suffix alias requires `X-OAGW-Target-Host` header
- Error response includes list of valid endpoint hosts
- Error type allows programmatic error handling
