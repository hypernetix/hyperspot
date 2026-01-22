# gRPC JSON transcoding for HTTP clients

## Setup

- gRPC upstream + route.
- Transcoding enabled (based on proto descriptors; configuration is implementation-defined).

## Inbound request (HTTP/JSON)

```http
POST /api/oagw/v1/proxy/<grpc-alias>/example.v1.UserService/ListUsers HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json
Accept: application/x-ndjson

{"page_size":10}
```

## Expected behavior

- Gateway converts JSON body into protobuf request.
- For server streaming:
  - Response is `application/x-ndjson`.
  - Each gRPC message becomes one JSON line.
