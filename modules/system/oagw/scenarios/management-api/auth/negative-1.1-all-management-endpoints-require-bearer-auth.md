# Management API requires bearer auth

## Request

```http
POST /api/oagw/v1/upstreams HTTP/1.1
Host: oagw.example.com
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

- `401 Unauthorized`
- `Content-Type: application/problem+json`

```json
{
  "type": "gts.x.core.errors.err.v1~x.oagw.auth.failed.v1",
  "title": "Authentication Failed",
  "status": 401,
  "detail": "Missing or invalid bearer token",
  "instance": "/api/oagw/v1/upstreams"
}
```
