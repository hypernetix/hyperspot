# ADR: Rate Limiting Configuration and Overrides

- **Status**: Proposed
- **Date**: 2026-01-30
- **Deciders**: TBD

## Context and Problem Statement

OAGW must enforce rate limits to:

- prevent abuse and cost overruns
- protect upstreams from overload
- allow platform operators / partners to set hard ceilings for downstream tenants

The system is multi-tenant and hierarchical. Rate limit configuration must support **override at lower levels**, while still allowing ancestors to **enforce upper bounds**.

OAGW rate limiting applies in multiple places:

- **Upstream-level** (all requests routed to an upstream)
- **Route-level** (only requests that match a specific route)
- **Tenant-level bindings** (tenant-specific overrides)

### Configuration layering (Upstream < Route < Tenant)

OAGW applies **all** applicable limiters; it does not “pick one”.

Resolution order for an inbound request:

1. resolve upstream by alias (tenant hierarchy)
2. match route
3. resolve upstream effective rate limit for the current tenant (including ancestor sharing)
4. resolve route effective rate limit for the current tenant (including ancestor sharing)
5. enforce both (request must pass both)

Additionally, a parent tenant may share configuration with descendants with different override permissions.

## Decision Drivers

- Deterministic merge behavior across hierarchy (no surprises)
- “Parent can cap, child can tighten” semantics
- Works for HTTP and streaming (SSE/WebSocket/gRPC) without protocol-specific branching
- Supports multiple counting scopes (global/tenant/user/ip)
- Simple enough to implement correctly and to explain to tenants

## Considered Options

1. **Fixed window counter** (reset every N seconds)
2. **Sliding window** (log-based or ring buffer)
3. **Token bucket** (rate + burst capacity)
4. **GCRA / leaky bucket** (TAT-based)

## Decision Outcome

Use **token bucket** as the core algorithm and configuration model.

- Supports steady-state throughput (`rate`) and bursts (`capacity`).
- Efficient: O(1) per request with limited state.
- Standard behavior across gateways and load balancers.

Where we need “requests per minute” UX, we still express it as token bucket parameters.

## Rate Limit Configuration

### Data model

Rate limit configuration is an object with:

- `rate` (tokens per window)
- `window_seconds` (refill interval basis)
- `capacity` (max tokens accumulated; burst size)
- `cost` (tokens deducted per request)
- `scope` (which key to count against)
- `strategy` (what to do when exceeded)
- `sharing` (hierarchy visibility / override semantics)

Suggested schema (conceptual; matches existing DESIGN fields and adds `sharing` for hierarchy):

```json
{
  "sharing": "private|inherit|enforce",
  "rate": 1000,
  "window_seconds": 60,
  "capacity": 1000,
  "cost": 1,
  "scope": "global|tenant|user|ip",
  "strategy": "reject|queue|degrade"
}
```

Notes:

- `capacity` defaults to `rate` if omitted (no extra burst).
- `cost` defaults to `1`.
- `window_seconds` defaults to `60`.
- `scope` defaults to `tenant`.
- `strategy` defaults to `reject`.

### Effective limit is the strictest applicable

If multiple limits apply to one request (e.g., upstream limit and route limit), OAGW enforces **all** of them. Practically this means:

- request is allowed only if **every** limiter allows
- the effective throughput experienced by the caller is the **minimum** of the active limits

## Algorithms

### Token bucket (per limiter)

State per key:

- `tokens` (float)
- `last_refill_ts` (monotonic time)

Parameters:

- `refill_rate = rate / window_seconds` tokens per second
- `capacity` tokens

On request with `cost`:

1. Refill:
   - `tokens = min(capacity, tokens + refill_rate * (now - last_refill_ts))`
   - `last_refill_ts = now`
2. If `tokens >= cost`: allow and `tokens -= cost`
3. Else: exceeded → apply `strategy`

### Strategy behavior

- `reject`: return 429 and `Retry-After` derived from time-to-next-token.
- `queue`: bounded wait up to a per-request timeout; if still exceeded, reject.
- `degrade`: allow request but mark it (header/metric) and optionally reduce upstream concurrency (future work). In MVP, degrade can be treated as reject if not implemented.

### Key derivation by scope

Key for limiter state depends on `scope`:

- `global`: `oagw/global`
- `tenant`: `oagw/tenant/{tenant_id}`
- `user`: `oagw/tenant/{tenant_id}/user/{user_id}` (requires authenticated user)
- `ip`: `oagw/tenant/{tenant_id}/ip/{client_ip}`

For upstream / route we add a namespace:

- upstream limiter key prefix: `.../upstream/{upstream_binding_id}`
- route limiter key prefix: `.../route/{route_id}`

This avoids collisions and ensures correct isolation.

## Hierarchical Overrides (Parent → Child)

### Sharing modes

- `private`: not visible to descendants (descendants do not inherit it)
- `inherit`: visible; descendant may set its own value, but cannot become less strict than ancestor
- `enforce`: visible; descendant value cannot relax it; descendant may still tighten it

### Merge rule: “min across the chain”

For any configuration chain where an ancestor limit is visible to a descendant (`inherit` or `enforce`), the effective limit is the **minimum** (strictest) across the ancestor+descendant values.

This yields the desired property:

- Parent can set a maximum allowed throughput
- Child can only reduce it

### Field-by-field strictness

We define “stricter” per field:

- `rate`: lower is stricter → `min(rate)`
- `capacity`: lower is stricter → `min(capacity)`
- `cost`: higher is stricter (each request consumes more) → `max(cost)`
- `window_seconds`: smaller window can be stricter or looser depending on interpretation; to avoid ambiguity:
  - treat `window_seconds` as a **unit** chosen by the ancestor when sharing is enabled
  - if an ancestor is visible (`inherit|enforce`) and defines `window_seconds`, descendant must use the same `window_seconds`
- `scope`: stricter means “more granular” (harder to abuse at aggregate level). To keep behavior predictable:
  - require `scope` to be identical when inheriting/enforcing (otherwise child could change accounting semantics)
- `strategy`: does not change strictness of the limit itself; enforce `reject` if any visible ancestor is `reject`.

### Merge algorithm

Given a tenant `T` and a visible configuration chain from root → … → `T`:

1. Collect visible limits from ancestors (skip `private` limits from ancestors).
2. If no limit exists in the chain: no limiter.
3. If one or more limits exist:
   - `rate = min(all.rate)`
   - `capacity = min(all.capacity)`
   - `cost = max(all.cost)`
   - `window_seconds = first_visible.window_seconds` (must match all; else validation error)
   - `scope = first_visible.scope` (must match all; else validation error)
   - `strategy = reject if any.strategy == reject else first_visible.strategy`

Validation errors should be surfaced at configuration time (CRUD), not at request time.

## Examples

### Example A: Partner enforces ceiling, customer tightens

Partner tenant config (shared):

```json
{ "sharing": "enforce", "rate": 1000, "window_seconds": 60, "capacity": 1000, "cost": 1, "scope": "tenant", "strategy": "reject" }
```

Customer config:

```json
{ "rate": 100, "window_seconds": 60, "capacity": 100, "cost": 1, "scope": "tenant" }
```

Effective:

- `rate = min(1000, 100) = 100/min`
- `capacity = 100`

### Example B: Route limit tighter than upstream

Upstream: 1000/min
Route: 50/min

Request matching the route must pass both → effective observed limit is 50/min.

## Consequences

Pros:

- Clear, monotonic override model (descendants can only tighten visible limits)
- Token bucket supports burst + steady throughput with simple state
- Consistent enforcement across upstream and route levels

Cons:

- Requires validation rules for `window_seconds`/`scope` to avoid semantic drift
- `queue`/`degrade` strategies add complexity; may need to be treated as future work

## Links

- [OAGW Design Document](../DESIGN.md)
- [ADR: Resource Identification and Discovery](./adr-resource-identification.md)
