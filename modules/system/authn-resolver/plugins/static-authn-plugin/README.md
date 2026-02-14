# Static AuthN Plugin

> **Temporary plugin** — this is a development/testing stub that will be replaced by a production-ready AuthN plugin (e.g., OIDC/JWT-based) in a future release.

Static token-to-identity mapping for the AuthN Resolver gateway.

## Purpose

Provides a simple, config-driven authentication mechanism so that the platform can run end-to-end without an external identity provider. Useful for:

- Local development (`make quickstart`, `make example`)
- E2E / integration tests that need distinct user identities
- Demos and prototyping

**Do not use in production.**

## Modes

| Mode | Description |
|------|-------------|
| `accept_all` (default) | Accepts any non-empty `Bearer` token and returns a configured default identity. Replaces the legacy `auth_disabled` flag. |
| `static_tokens` | Maps specific tokens to specific identities. Useful for E2E tests with multiple distinct users. |

## Configuration

```yaml
modules:
  static_authn_plugin:
    config:
      vendor: "hyperspot"
      priority: 100
      mode: accept_all               # or "static_tokens"
      default_identity:
        subject_id: "11111111-6a88-4768-9dfc-6bcd5187d9ed"
        subject_tenant_id: "00000000-df51-5b42-9538-d2b56b7ee953"
        token_scopes: ["*"]
      tokens: []                      # populated in static_tokens mode
```

## Feature Flag

The server binary includes this plugin only when built with the `static-authn` feature:

```bash
cargo build --bin hyperspot-server --features static-authn
```

The `make example` target enables this feature automatically.
