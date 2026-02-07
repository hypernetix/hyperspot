# Plain HTTP request/response passthrough

## Setup

- Upstream `alias=httpbin.org` (or test upstream) with `protocol=http`.
- Route matches `GET /get`.

## Inbound request

```http
GET /api/oagw/v1/proxy/httpbin.org/get HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
X-Request-ID: req-http-001
```

## Expected outbound request

```http
GET /get HTTP/1.1
Host: httpbin.org
X-Request-ID: req-http-001
```

## Expected response

- Status code and body forwarded as-is.
- If rate limiting headers are enabled, response includes `X-RateLimit-*`.
