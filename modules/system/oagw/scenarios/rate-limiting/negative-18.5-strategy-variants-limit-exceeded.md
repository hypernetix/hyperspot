# Rate limit strategy variants when exceeded

## Scenario A: `strategy=reject`

Expected:
- Exceeded requests return `429` immediately.

## Scenario B: `strategy=queue`

Configure queueing for rate limit:

```json
{
  "rate_limit": {
    "sustained": { "rate": 1, "window": "second" },
    "strategy": "queue",
    "queue": { "max_depth": 10, "timeout": "2s" }
  }
}
```

Expected:
- Second request waits and then succeeds when tokens refill, or times out with:
  - `503`
  - `type` = `...queue.timeout...`

## Scenario C: `strategy=degrade`

If implemented:
- Exceeded requests use configured fallback behavior.

If not implemented:
- Configuration rejected or treated as `reject`.
- Lock expected behavior.
