# Upstream `headers` config applies simple transformations

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
  "headers": {
    "request": {
      "set": {
        "User-Agent": "OAGW/1.0",
        "X-Forwarded-For": "<must-not-be-allowed-if-considered-sensitive>"
      }
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

## Expected behavior

- Outbound request has `User-Agent: OAGW/1.0`.
- If some headers are forbidden by policy, config update is rejected with `400` or the header is dropped; lock expected behavior.

## What to check

- Upstream echo verifies header values.
