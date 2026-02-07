# Rate limit scope variants

## Goal

Lock behavior for each supported `scope`:
- `global`
- `tenant`
- `user`
- `ip`
- `route`

## Setup

Create one upstream with multiple routes and configure rate limit scope per route in separate test runs.

Example config skeleton:

```json
{ "rate_limit": { "sustained": { "rate": 2, "window": "second" }, "scope": "<scope>" } }
```

## Expected behavior

- `global`: limit shared across all tenants.
- `tenant`: limit shared within tenant.
- `user`: limit shared within authenticated principal.
- `ip`: limit shared for source IP.
- `route`: limit isolated per route.

For each scope, verify the counter keying by sending requests from different principals/tenants/IPs/routes.
