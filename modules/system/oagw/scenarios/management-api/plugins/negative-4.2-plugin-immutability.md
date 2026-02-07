# Plugin immutability (no update)

## Setup

Create a custom plugin (guard or transform).

## Attempt to update plugin

```http
PUT /api/oagw/v1/plugins/gts.x.core.oagw.plugin.guard.v1~<uuid> HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "description": "updated"
}
```

## Expected response

- `404 Not Found` or `405 Method Not Allowed` (lock expected behavior).
- If body is present:
  - `Content-Type: application/problem+json`
