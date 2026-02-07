# Enforced ancestor limits apply when alias is shadowed

## Setup

- Parent upstream `alias=api.example.com`:
  - `rate_limit.sharing=enforce` with high cap (example: 1000/min)
- Child upstream shadows same alias:
  - sets its own rate_limit lower (example: 100/min)

## Inbound requests

- Send >100 requests/min from child tenant.

## Expected behavior

- Effective rate limit for child is `min(parent_enforced, child)`.
- Requests beyond effective cap return `429` (gateway error source).
- If enabled, rate-limit headers reflect the effective limit.
