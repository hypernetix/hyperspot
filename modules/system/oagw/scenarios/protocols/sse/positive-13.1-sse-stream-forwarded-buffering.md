# SSE stream forwarded without buffering

## Setup

- Upstream returns `Content-Type: text/event-stream` and sends multiple events.
- Route matches `POST /stream`.

## Inbound request

```http
POST /api/oagw/v1/proxy/<alias>/stream HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Accept: text/event-stream
Content-Type: application/json

{"stream":true}
```

## Expected response

- `200 OK`
- `Content-Type: text/event-stream`
- Events are delivered incrementally (no full buffering).
- If rate limit headers are enabled, they appear in initial headers.
