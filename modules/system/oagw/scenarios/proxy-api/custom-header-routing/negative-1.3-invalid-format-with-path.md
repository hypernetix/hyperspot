# Invalid X-OAGW-Target-Host format with path component

## Setup

- Upstream `vendor.com` with multiple endpoints:
  - `endpoints`: `[{"scheme": "https", "host": "us.vendor.com", "port": 443}, {"scheme": "https", "host": "eu.vendor.com", "port": 443}]`
  - `alias`: `vendor.com`

## Request

```http
POST /api/oagw/v1/proxy/vendor.com/v1/api/resource HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <token>
X-OAGW-Target-Host: us.vendor.com/api
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
  "type": "gts.x.core.errors.err.v1~x.oagw.routing.invalid_target_host.v1",
  "title": "Invalid Target Host Format",
  "status": 400,
  "detail": "X-OAGW-Target-Host must be a valid hostname or IP address (no port, path, or special characters)",
  "instance": "/api/oagw/v1/proxy/vendor.com/v1/api/resource",
  "upstream_id": "gts.x.core.oagw.upstream.v1~...",
  "invalid_value": "us.vendor.com/api"
}
```

## Validation

- Header must not include path component
- Only hostname or IP address is valid
- Path routing is handled separately in URL
