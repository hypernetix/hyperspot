# IP address in X-OAGW-Target-Host when hostname endpoint configured

## Setup

- Upstream `vendor.com` with hostname-based endpoints:
  - `endpoints`: `[{"scheme": "https", "host": "us.vendor.com", "port": 443}, {"scheme": "https", "host": "eu.vendor.com", "port": 443}]`
  - `alias`: `vendor.com`

## Request

```http
POST /api/oagw/v1/proxy/vendor.com/v1/api/resource HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <token>
X-OAGW-Target-Host: 192.168.1.10
Content-Type: application/json

{"key": "value"}
```

## Expected behavior

- Request is rejected with `400 Bad Request`
- Error response:

```http
HTTP/1.1 400 Bad Request
X-OAGW-Error-Source: gateway
Content-Type: application/problem+json

{
  "type": "gts.x.core.errors.err.v1~x.oagw.routing.unknown_target_host.v1",
  "title": "Unknown Target Host",
  "status": 400,
  "detail": "X-OAGW-Target-Host '192.168.1.10' does not match any configured endpoint. Valid hosts: [us.vendor.com, eu.vendor.com]",
  "instance": "/api/oagw/v1/proxy/vendor.com/v1/api/resource",
  "upstream_id": "gts.x.core.oagw.upstream.v1~...",
  "invalid_value": "192.168.1.10",
  "valid_hosts": ["us.vendor.com", "eu.vendor.com"]
}
```

## Validation

- Header value must match configured endpoint host (hostname or IP)
- Type mismatch (IP vs hostname) is treated as unknown host
- Allowlist validation enforces exact matching
