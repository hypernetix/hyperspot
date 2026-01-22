# gRPC unary request proxied (native)

## Setup

- Upstream protocol: `gts.x.core.oagw.protocol.v1~x.core.grpc.v1`
- Upstream endpoint: `scheme=grpc`
- Route match:

```json
{
  "match": {
    "grpc": {
      "service": "example.v1.UserService",
      "method": "GetUser"
    }
  }
}
```

## Inbound request (native gRPC client)

- Client sends HTTP/2 request with:
  - `content-type: application/grpc`
  - `:path: /example.v1.UserService/GetUser`

## Expected behavior

- Gateway detects gRPC by content-type.
- Metadata headers are preserved.
- Response trailers (`grpc-status`, `grpc-message`) are forwarded.
