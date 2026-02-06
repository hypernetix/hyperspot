# Create HTTP route with method + path

## Setup

Create an HTTP upstream (example: `alias=httpbin.org`).

## Request

```http
POST /api/oagw/v1/routes HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "upstream_id": "gts.x.core.oagw.upstream.v1~<upstream-uuid>",
  "match": {
    "http": {
      "methods": ["POST"],
      "path": "/post",
      "query_allowlist": ["tag"],
      "path_suffix_mode": "disabled"
    }
  }
}
```

## Expected response

- `201 Created`

```json
{
  "id": "gts.x.core.oagw.route.v1~<route-uuid>",
  "upstream_id": "gts.x.core.oagw.upstream.v1~<upstream-uuid>",
  "match": {
    "http": {
      "methods": ["POST"],
      "path": "/post",
      "query_allowlist": ["tag"],
      "path_suffix_mode": "disabled"
    }
  }
}
```
