# Outbound auth: Bearer token injection

## Upstream configuration

```json
{
  "alias": "api.example.com",
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "api.example.com", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "auth": {
    "type": "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.bearer.v1",
    "config": {
      "secret_ref": "cred://api/static-bearer-token"
    }
  }
}
```

## Inbound request

```http
GET /api/oagw/v1/proxy/api.example.com/v1/me HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected outbound request

- `Authorization: Bearer <resolved-secret>` is set.
