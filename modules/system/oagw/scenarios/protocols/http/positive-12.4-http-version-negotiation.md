# HTTP version negotiation (HTTP/2 attempt + fallback)

## Setup

- Use an upstream host that does not support HTTP/2 (or a test upstream configured to reject ALPN h2).
- Gateway should try HTTP/2 first, then fall back to HTTP/1.1.

## Inbound request

```http
GET /api/oagw/v1/proxy/<alias>/get HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected behavior

- First request:
  - Attempts HTTP/2.
  - On failure, uses HTTP/1.1.
- Subsequent requests:
  - Use cached decision for ~1h TTL (lock via behavior/metrics/logs if available).
