# Rate limiting: sliding window strictness

## Setup

Configure sliding window:

```json
{
  "rate_limit": {
    "algorithm": "sliding_window",
    "sustained": { "rate": 10, "window": "second" },
    "scope": "tenant",
    "strategy": "reject"
  }
}
```

## Steps

Send 10 requests near the end of a second, then 10 more immediately after the second boundary.

## Expected behavior

- Sliding window prevents the effective 2x boundary burst.
- Some of the second burst is rejected with `429`.
- Rejections include `Retry-After` and are `application/problem+json`.
