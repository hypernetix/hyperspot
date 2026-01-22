# Multi-endpoint upstream pool compatibility rules

## Scenario A: mismatched scheme in endpoint pool

### Request

```http
POST /api/oagw/v1/upstreams HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "us.vendor.com", "port": 443 },
      { "scheme": "wss", "host": "eu.vendor.com", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "alias": "vendor.com"
}
```

### Expected response

- `400 Bad Request`
- `Content-Type: application/problem+json`
- `detail` mentions incompatible endpoint pool fields (`scheme`).

## Scenario B: mismatched port in endpoint pool

```http
POST /api/oagw/v1/upstreams HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "us.vendor.com", "port": 443 },
      { "scheme": "https", "host": "eu.vendor.com", "port": 8443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "alias": "vendor.com"
}
```

Expected: `400` with `detail` mentioning incompatible endpoint pool fields (`port`).

## Scenario C: mismatched protocol vs intended pool

```http
POST /api/oagw/v1/upstreams HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "api.vendor.com", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.grpc.v1",
  "alias": "api.vendor.com"
}
```

Expected: `400` (config validation) or a later `502 ProtocolError` on invoke (lock expected behavior).
