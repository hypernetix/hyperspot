# Case-insensitive X-OAGW-Target-Host header matching

## Setup

- Upstream `vendor.com` with multiple endpoints:
  - `endpoints`: `[{"scheme": "https", "host": "us.vendor.com", "port": 443}, {"scheme": "https", "host": "eu.vendor.com", "port": 443}]`
  - `alias`: `vendor.com`

## Request

```http
POST /api/oagw/v1/proxy/vendor.com/v1/api/resource HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <token>
X-OAGW-Target-Host: US.VENDOR.COM
Content-Type: application/json

{"key": "value"}
```

## Expected behavior

- Header value `US.VENDOR.COM` matches endpoint `us.vendor.com` (case-insensitive)
- Request routes to `us.vendor.com:443`
- Response: `200 OK` with upstream response body

## Validation

- DNS hostname comparison is case-insensitive
- Mixed case in header value is accepted
- Header matching follows DNS standards
