# gRPC status mapping and error source

## Scenario A: gateway-generated error

Trigger a gateway error (e.g., rate limit) on a gRPC route.

Expected:
- `X-OAGW-Error-Source: gateway`
- Error returned in a gateway error format appropriate for the client type.

## Scenario B: upstream gRPC error

Upstream returns gRPC error status (e.g., `UNAVAILABLE`, `RESOURCE_EXHAUSTED`).

Expected:
- Error is attributed to upstream:
  - `X-OAGW-Error-Source: upstream`
- Mapping rules are consistent:
  - `RESOURCE_EXHAUSTED` maps to rate limit semantics if OAGW maps it (lock expected behavior).
