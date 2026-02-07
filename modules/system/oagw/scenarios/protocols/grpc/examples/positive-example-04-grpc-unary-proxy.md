# Streaming gRPC (positive)

## Upstream Configuration

```json
{
  "alias": "<grpc-upstream-host>",
  "server": {
    "endpoints": [
      { "scheme": "grpc", "host": "<grpc-upstream-host>", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.grpc.v1",
  "auth": {
    "type": "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.apikey.v1",
    "config": {
      "header": "x-api-key",
      "secret_ref": "cred://grpc/api-key"
    }
  }
}
```

## Route Configuration

```json
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

## Inbound Request (native gRPC)

```http
:method: POST
:path: /user.v1.UserService/ListUsers
:authority: oagw.example.com
content-type: application/grpc
x-request-id: req-grpc-001
```

## Outbound Request (to upstream)

```http
:method: POST
:path: /user.v1.UserService/ListUsers
:authority: <grpc-upstream-host>
content-type: application/grpc
x-api-key: <secret-from-cred-store>
x-request-id: req-grpc-001
```

## Upstream Response (server streaming)

- Streamed gRPC messages followed by trailers (`grpc-status: 0`).

## Outbound Response

- Streamed gRPC messages forwarded to client.

## HTTP/JSON transcoding variant (optional)

If transcoding is enabled:

```http
POST /api/oagw/v1/proxy/<grpc-upstream-host>/user.v1.UserService/ListUsers HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/json
Accept: application/x-ndjson

{"page_size": 10}
```

Expected:
- Response is `application/x-ndjson` with one JSON object per streamed message.
