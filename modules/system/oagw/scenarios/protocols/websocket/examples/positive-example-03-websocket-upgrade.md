# Streaming WebSockets (positive)

## Upstream Configuration

```json
{
  "alias": "<ws-upstream-host>",
  "server": {
    "endpoints": [
      { "scheme": "wss", "host": "<ws-upstream-host>", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "auth": {
    "type": "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.apikey.v1",
    "config": {
      "header": "Authorization",
      "prefix": "Bearer ",
      "secret_ref": "cred://ws/upstream-token"
    }
  },
  "rate_limit": {
    "algorithm": "token_bucket",
    "sustained": { "rate": 10, "window": "minute" },
    "scope": "tenant",
    "strategy": "reject"
  }
}
```

## Route Configuration

```json
{
  "upstream_id": "gts.x.core.oagw.upstream.v1~<upstream-uuid>",
  "match": {
    "http": {
      "methods": ["GET"],
      "path": "/v1/realtime",
      "query_allowlist": ["model"],
      "path_suffix_mode": "disabled"
    }
  }
}
```

## WebSocket Upgrade - Inbound Request

```http
GET /api/oagw/v1/proxy/<ws-upstream-host>/v1/realtime?model=realtime-model HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: <base64>
Sec-WebSocket-Version: 13
Sec-WebSocket-Protocol: realtime
X-Request-ID: req-ws-001
```

## WebSocket Upgrade - Outbound Request (to upstream)

```http
GET /v1/realtime?model=realtime-model HTTP/1.1
Host: <ws-upstream-host>
Authorization: Bearer <secret-from-cred-store>
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: <base64>
Sec-WebSocket-Version: 13
Sec-WebSocket-Protocol: realtime
X-Request-ID: req-ws-001
```

## WebSocket Upgrade - Outbound Response (to client)

```http
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Protocol: realtime
X-Request-ID: req-ws-001
```

## WebSocket Messages

Client → upstream (forwarded):

```json
{"type":"ping"}
```

Upstream → client (forwarded):

```json
{"type":"pong"}
```
