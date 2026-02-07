# Multi-endpoint common-suffix alias requires Host header

## Setup

Create an upstream with multiple endpoints and a common-suffix alias:

- Endpoints:
  - `us.vendor.com:443`
  - `eu.vendor.com:443`
- Alias: `vendor.com`

## Scenario A: missing inbound Host

```http
GET /api/oagw/v1/proxy/vendor.com/health HTTP/1.1
Authorization: Bearer <tenant-token>
```

Expected:
- Gateway error (`400`)
- If body is present: `application/problem+json`
- `detail` indicates `Host` header required.

## Scenario B: Host selects endpoint

```http
GET /api/oagw/v1/proxy/vendor.com/health HTTP/1.1
Host: us.vendor.com
Authorization: Bearer <tenant-token>
```

Expected:
- Routes to `us.vendor.com` endpoint.

## Scenario C: Host not in pool

```http
GET /api/oagw/v1/proxy/vendor.com/health HTTP/1.1
Host: evil.vendor.com
Authorization: Bearer <tenant-token>
```

Expected:
- Gateway rejects selection (400/403; lock expected behavior).
