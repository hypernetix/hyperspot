# Rate limit response headers can be disabled

## Setup

Configure rate limiting with `response_headers=false`:

```json
{
  "rate_limit": {
    "algorithm": "token_bucket",
    "sustained": { "rate": 1, "window": "second" },
    "burst": { "capacity": 1 },
    "response_headers": false
  }
}
```

## Success request

```http
GET /api/oagw/v1/proxy/<alias>/resource HTTP/1.1
Host: oagw.example.com
Authorization: Bearer <tenant-token>
```

## Expected behavior

- Success response does NOT include `X-RateLimit-*` headers.

## Exceeded request

Send a second request within the same second.

Expected:
- `429 Too Many Requests`
- `Retry-After` is still present.
- Body is `application/problem+json` with `X-OAGW-Error-Source: gateway`.
