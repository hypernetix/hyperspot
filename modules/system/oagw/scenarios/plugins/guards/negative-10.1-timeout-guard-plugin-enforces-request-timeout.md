# Timeout guard plugin enforces request timeout

## Setup

Attach builtin timeout guard plugin to the route:
- `gts.x.core.oagw.plugin.guard.v1~x.core.oagw.timeout.v1`

Route plugin list includes the timeout guard with a low timeout (example: 100ms) via its config mechanism (exact config shape is implementation-defined).

## Inbound request

Send a request that the upstream intentionally delays beyond the timeout.

```http
GET /api/oagw/v1/proxy/<alias>/slow HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected response

- `504 Gateway Timeout`
- `Content-Type: application/problem+json`
- `X-OAGW-Error-Source: gateway`
- `type` corresponds to request timeout (`...timeout.request...`) or configured timeout error type.

## What to check

- Upstream call is cancelled/terminated.
- Audit log records failure with error_type timeout.
