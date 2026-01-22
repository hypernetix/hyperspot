# Budget modes: allocated/shared/unlimited

## Scenario A: allocated budget

Parent:

```json
{
  "rate_limit": {
    "sharing": "enforce",
    "budget": { "mode": "allocated", "total": 100, "overcommit_ratio": 1.0 },
    "sustained": { "rate": 100, "window": "minute" }
  }
}
```

Children request allocations that sum to > total.

Expected:
- Creation/update rejected when `sum(children) > total * overcommit_ratio`.

## Scenario B: shared pool

```json
{ "rate_limit": { "budget": { "mode": "shared", "total": 100 }, "sustained": { "rate": 100, "window": "minute" } } }
```

Expected:
- Multiple tenants share the same pool; first-come-first-served.

## Scenario C: unlimited

```json
{ "rate_limit": { "budget": { "mode": "unlimited" }, "sustained": { "rate": 100, "window": "minute" } } }
```

Expected:
- No budget allocation validation is enforced.
