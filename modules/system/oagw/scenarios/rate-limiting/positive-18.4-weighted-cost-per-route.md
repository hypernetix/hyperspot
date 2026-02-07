# Weighted cost per route

## Setup

Tenant has budget of 10 tokens per window.

- Route A: `cost=10`
- Route B: `cost=1`

Example:

```json
{
  "rate_limit": {
    "sustained": { "rate": 10, "window": "minute" },
    "cost": 10
  }
}
```

## Expected behavior

- One call to Route A consumes all tokens and subsequent calls in window are rejected.
- Ten calls to Route B consume the same budget.
- Rejections return `429` with `Retry-After`.
