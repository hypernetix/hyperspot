# Plugin type enforcement

## Setup

Create a guard plugin id:
- `gts.x.core.oagw.plugin.guard.v1~<uuid>`

## Attempt: attach guard plugin as upstream auth

```http
PUT /api/oagw/v1/upstreams/gts.x.core.oagw.upstream.v1~<upstream-uuid> HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "httpbin.org", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "alias": "httpbin.org",
  "auth": {
    "type": "gts.x.core.oagw.plugin.guard.v1~<uuid>",
    "config": {}
  }
}
```

## Expected response

- `400 Bad Request`
- `Content-Type: application/problem+json`
- `detail` mentions plugin type mismatch (`guard` vs `auth`).
