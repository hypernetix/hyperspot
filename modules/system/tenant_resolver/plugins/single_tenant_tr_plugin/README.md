# Single Tenant Resolver Plugin

Zero-config plugin for single-tenant deployments.

See [../../README.md](../../README.md) for full documentation.

## Quick Reference

- No configuration required
- Returns tenant from security context as the only tenant (name: "Default")
- No cross-tenant access allowed (single-tenant mode)
- Implements `TenantResolverPluginClient`
