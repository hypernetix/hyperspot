# WebSocket upgrade is proxied

## Setup

- Upstream endpoint uses `scheme=wss`.
- Route matches `GET /v1/realtime` (example).

## Inbound request

```http
GET /api/oagw/v1/proxy/<alias>/v1/realtime HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: <base64>
Sec-WebSocket-Version: 13
Sec-WebSocket-Protocol: <subprotocol>
X-Request-ID: req-ws-001
```

## Expected response

- `101 Switching Protocols`
- Upgrade headers forwarded.
- After upgrade, frames are proxied bidirectionally.
