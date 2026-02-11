# Unknown X-OAGW-Target-Host not in endpoint list

## Setup

- Upstream `vendor.com` with multiple endpoints:
  - `endpoints`: `[{"scheme": "https", "host": "us.vendor.com", "port": 443}, {"scheme": "https", "host": "eu.vendor.com", "port": 443}]`
  - `alias`: `vendor.com`

## Request

```http
POST /api/oagw/v1/proxy/vendor.com/v1/api/resource HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <token>
X-OAGW-Target-Host: apac.vendor.com
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
  "detail": "X-OAGW-Target-Host 'apac.vendor.com' does not match any configured endpoint. Valid hosts: [us.vendor.com, eu.vendor.com]",
  "instance": "/api/oagw/v1/proxy/vendor.com/v1/api/resource",
  "upstream_id": "gts.x.core.oagw.upstream.v1~...",
  "invalid_value": "apac.vendor.com",
  "valid_hosts": ["us.vendor.com", "eu.vendor.com"]
}
```

## Validation

- Header value must exactly match one of the configured endpoint hosts
- Allowlist validation prevents routing to arbitrary servers
- Error response includes list of valid hosts for debugging
