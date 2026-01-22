# Outbound auth: noop

## Upstream configuration

```json
{
  "alias": "public.example.com",
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "public.example.com", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "auth": {
    "type": "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.noop.v1",
    "config": {}
  }
}
```

## Inbound request

```http
GET /api/oagw/v1/proxy/public.example.com/health HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected behavior

- Gateway forwards request without injecting credentials.
- Upstream does not receive additional auth headers from OAGW.
