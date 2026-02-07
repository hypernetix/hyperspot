# Method allowlist enforcement

## Setup

Route allows `POST` only.

## Inbound request (disallowed method)

```http
GET /api/oagw/v1/proxy/<alias>/post HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected response

- Gateway-generated rejection.
- Expected: `404` (route not found) or `400` (validation) (lock expected behavior).
- If body is present:
  - `Content-Type: application/problem+json`
  - `X-OAGW-Error-Source: gateway`
