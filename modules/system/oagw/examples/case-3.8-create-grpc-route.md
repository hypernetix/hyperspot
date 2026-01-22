# Create gRPC route by service + method

## Upstream configuration

```json
{
  "alias": "grpc.example.com",
  "server": {
    "endpoints": [
      { "scheme": "grpc", "host": "grpc.example.com", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.grpc.v1"
}
```

## Route configuration

```http
POST /api/oagw/v1/routes HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json

{
  "upstream_id": "gts.x.core.oagw.upstream.v1~<upstream-uuid>",
  "match": {
    "grpc": {
      "service": "user.v1.UserService",
      "method": "ListUsers"
    }
  }
}
```

## Expected behavior

- gRPC request is routed to HTTP/2 `:path` `/user.v1.UserService/ListUsers`.
- Wrong service/method returns gateway `404` route not found.
