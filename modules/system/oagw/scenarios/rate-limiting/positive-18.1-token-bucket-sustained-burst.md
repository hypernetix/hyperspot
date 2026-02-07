# Rate limiting: token bucket sustained + burst

## Setup

Configure upstream or route rate limit:

```json
{
  "rate_limit": {
    "algorithm": "token_bucket",
    "sustained": { "rate": 5, "window": "second" },
    "burst": { "capacity": 10 },
    "scope": "tenant",
    "strategy": "reject",
    "response_headers": true
  }
}
```

## Steps

1. Send 10 requests quickly (burst) → expect success until capacity consumed.
2. Send 1 more immediately → expect `429`.
3. Wait for refill (>= 1s) and retry → expect success.

## Expected responses

On success:
- `200 OK`
- Includes `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`.

On limit exceeded:
- `429 Too Many Requests`
- `Content-Type: application/problem+json`
- `X-OAGW-Error-Source: gateway`
- Includes `Retry-After`.
