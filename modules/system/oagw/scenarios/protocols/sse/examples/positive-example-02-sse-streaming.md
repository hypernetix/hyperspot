# Server-Sent Events (SSE) (positive)

## Upstream Configuration

```json
{
  "alias": "<sse-upstream-host>",
  "server": {
    "endpoints": [
      { "scheme": "https", "host": "<sse-upstream-host>", "port": 443 }
    ]
  },
  "protocol": "gts.x.core.oagw.protocol.v1~x.core.http.v1",
  "auth": {
    "type": "gts.x.core.oagw.plugin.auth.v1~x.core.oagw.apikey.v1",
    "config": {
      "header": "Authorization",
      "prefix": "Bearer ",
      "secret_ref": "cred://sse/upstream-token"
    }
  },
  "headers": {
    "request": {
      "set": {
        "User-Agent": "OAGW/1.0"
      }
    }
  }
}
```

## Route Configuration

```json
{
  "upstream_id": "gts.x.core.oagw.upstream.v1~<upstream-uuid>",
  "match": {
    "http": {
      "methods": ["POST"],
      "path": "/v1/stream",
      "query_allowlist": [],
      "path_suffix_mode": "disabled"
    }
  }
}
```

## Inbound Request

```http
POST /api/oagw/v1/proxy/<sse-upstream-host>/v1/stream HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Accept: text/event-stream
Content-Type: application/json
X-Request-ID: req-sse-001

{"stream": true}
```

## Outbound Request (to upstream)

```http
POST /v1/stream HTTP/1.1
Host: <sse-upstream-host>
Authorization: Bearer <secret-from-cred-store>
Accept: text/event-stream
X-Request-ID: req-sse-001

{"stream": true}
```

## Upstream Response (SSE stream)

```http
HTTP/1.1 200 OK
Content-Type: text/event-stream

data: {"event":"start"}

data: {"event":"delta","text":"hello"}

data: [DONE]
```

## Outbound Response (to client)

```http
HTTP/1.1 200 OK
Content-Type: text/event-stream
X-Request-ID: req-sse-001

data: {"event":"start"}

data: {"event":"delta","text":"hello"}

data: [DONE]
```
