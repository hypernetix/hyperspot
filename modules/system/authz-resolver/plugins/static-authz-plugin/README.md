# Static AuthZ Plugin

> **Temporary plugin** — this is a development/testing stub that will be replaced by a production-ready AuthZ plugin (e.g., OPA/Cedar-based policy engine) in a future release.

Static allow-all authorization policy for the AuthZ Resolver gateway.

## Purpose

Provides a permissive authorization policy so that the platform can run end-to-end without an external policy engine. Useful for:

- Local development (`make quickstart`, `make example`)
- E2E / integration tests that need authorization to pass
- Demos and prototyping

**Do not use in production.**

## Behavior

In `allow_all` mode (the only mode currently supported):

| Scenario | Decision | Constraints |
|----------|----------|-------------|
| `require_constraints = false` | `true` | none |
| `require_constraints = true` | `true` | `in` predicate on `owner_tenant_id` scoped to the caller's tenant |

This ensures that the Secure ORM receives the tenant scope it needs for queries, while still granting access to every action.

## Configuration

```yaml
modules:
  static_authz_plugin:
    config:
      vendor: "hyperspot"
      priority: 100
      mode: allow_all
```

## Feature Flag

The server binary includes this plugin only when built with the `static-authz` feature:

```bash
cargo build --bin hyperspot-server --features static-authz
```

The `make example` target enables this feature automatically.
