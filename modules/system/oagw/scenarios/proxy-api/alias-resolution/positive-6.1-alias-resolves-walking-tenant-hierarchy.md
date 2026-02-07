# Alias resolves by walking tenant hierarchy (shadowing)

## Setup

- Parent tenant creates upstream:
  - `alias=api.example.com`
  - endpoint host `prod.example.com`
- Child tenant creates upstream with same alias:
  - `alias=api.example.com`
  - endpoint host `staging.example.com`

## Inbound request from child tenant

```http
GET /api/oagw/v1/proxy/api.example.com/health HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <child-tenant-token>
```

## Expected behavior

- Gateway resolves alias to child upstream (closest match wins).
- Verify by:
  - upstream-specific response header (test upstream), or
  - endpoint access logs.
