# Proxy cannot access upstream not owned or shared

## Setup

- Tenant B creates upstream `alias=httpbin.org` (private by default).
- Tenant A does not have access to Tenant B upstream.

## Inbound request (Tenant A)

```http
POST /api/oagw/v1/proxy/httpbin.org/post HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-a-token>
Content-Type: application/json

{"name":"test"}
```

## Expected response

- Must not leak upstream existence.
- Expected: `403 Forbidden` or `404 Not Found` (lock expected behavior).
- If body is present:
  - `Content-Type: application/problem+json`
  - `X-OAGW-Error-Source: gateway`
