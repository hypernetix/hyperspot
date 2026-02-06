# Hierarchical min() merge for descendant overrides

## Setup

Parent tenant upstream:

```json
{
  "rate_limit": {
    "sharing": "enforce",
    "sustained": { "rate": 1000, "window": "minute" }
  }
}
```

Child tenant binding sets stricter limit:

```json
{
  "rate_limit": {
    "sustained": { "rate": 100, "window": "minute" }
  }
}
```

## Expected behavior

- Effective limit for child is `min(parent, child)`.
- After 100 requests/min, child gets `429`.
