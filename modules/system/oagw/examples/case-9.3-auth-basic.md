# Outbound auth: Basic

## Upstream configuration

```json
{
  "alias": "legacy.example.com",
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "legacy.example.com", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "auth": {
    "type": "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.basic.v1",
    "config": {
      "username_ref": "cred://legacy/basic/username",
      "password_ref": "cred://legacy/basic/password"
    }
  }
}
```

## Inbound request

```http
GET /api/oagw/v1/proxy/legacy.example.com/health HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected outbound request

- `Authorization: Basic <base64(user:pass)>` is present.
- No credentials appear in logs.
