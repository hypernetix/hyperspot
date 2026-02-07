# Create minimal HTTP upstream (single endpoint)

## Request

```http
POST /api/oagw/v1/upstreams HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "httpbin.org", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1"
}
```

## Expected response

- `201 Created`
- Response body contains:
  - `enabled: true` (default)
  - `alias: "httpbin.org"` (standard port 443 omitted)

```json
{
  "id": "gts.x.core.oagw.upstream.v1~<uuid>",
  "enabled": true,
  "alias": "httpbin.org",
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "httpbin.org", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1"
}
```
