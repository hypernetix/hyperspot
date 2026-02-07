# Streaming request bodies are not buffered (where supported)

## Setup

Use a route/upstream that accepts streaming uploads (e.g., chunked transfer) and ensure gateway supports streaming request bodies.

## Inbound request

```http
POST /api/oagw/v1/proxy/<alias>/upload HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
Content-Type: application/octet-stream
Transfer-Encoding: chunked

<streamed-chunks>
```

## Expected behavior

- Gateway forwards chunks without buffering the full body.

## What to check

- Memory does not grow proportional to uploaded size.
- If streaming request bodies are not supported, gateway returns a clear `400`/`502` with `application/problem+json`.
