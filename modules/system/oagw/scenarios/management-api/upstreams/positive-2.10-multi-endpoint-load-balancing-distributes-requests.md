# Multi-endpoint load balancing distributes requests

## Upstream configuration

Create one upstream with multiple endpoints (same scheme/port/protocol).

```json
{
  "alias": "vendor.com",
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "us.vendor.com", "port": 443 },
      { "scheme": "https", "host": "eu.vendor.com", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1"
}
```

## Route configuration

One route `GET /health`.

## Invocation

Send N requests:

```http
GET /api/oagw/v1/proxy/vendor.com/health HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected behavior

- Requests are distributed across endpoints.
- Verify by checking one of:
  - Upstream access logs per endpoint host.
  - Response header injected by endpoint (test upstream), e.g. `X-Upstream-Host`.
  - Gateway audit logs that record chosen endpoint (if present).
