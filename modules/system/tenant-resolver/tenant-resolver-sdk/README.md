# Tenant Resolver SDK

Public API traits and models for tenant resolution.

## Quick Reference

- `TenantResolverGatewayClient` - Public API for consumers
- `TenantResolverPluginClient` - API for plugin implementations
- `TenantInfo` - Full tenant information (for `get_tenant`, `get_tenants`)
- `TenantRef` - Tenant reference without name (for `get_ancestors`, `get_descendants`)
- `TenantStatus`, `BarrierMode` - Query parameters
- `GetTenantsOptions`, `GetAncestorsOptions`, `GetDescendantsOptions`, `IsAncestorOptions` - Options structs
- `GetAncestorsResponse`, `GetDescendantsResponse` - Hierarchy responses
- `TenantResolverError` - Error types
