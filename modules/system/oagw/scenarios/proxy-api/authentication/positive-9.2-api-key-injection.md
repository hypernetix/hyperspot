# Outbound auth: API key injection

## Upstream configuration

```json
{
  "alias": "httpbin.org",
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "httpbin.org", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "auth": {
    "type": "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.apikey.v1",
    "config": {
      "header": "X-Api-Key",
      "secret_ref": "cred://httpbin/api-key"
    }
  }
}
```

## Inbound request

```http
GET /api/oagw/v1/proxy/httpbin.org/get HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected outbound request

```http
GET /get HTTP/1.1
Host: httpbin.org
X-Api-Key: <resolved-secret>
```

## What to check

- Upstream receives `X-Api-Key`.
- Secret value is not logged.
