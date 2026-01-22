# Alias uniqueness enforced per tenant

## Scenario A: duplicate alias in same tenant

### Request 1

Create upstream with `alias=my-service`.

### Request 2

Create another upstream with `alias=my-service` in the same tenant.

Expected: request 2 rejected with `409 Conflict` or `400 Bad Request` (lock expected behavior), `application/problem+json`.

## Scenario B: same alias across tenants

- Tenant A creates `alias=my-service`.
- Tenant B creates `alias=my-service`.

Expected: both succeed (tenant isolation).
