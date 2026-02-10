# Tenant Resolver Gateway

Gateway module that routes requests to the selected plugin.

## Quick Reference

- Discovers plugins via GTS (types-registry)
- Selects plugin by vendor + priority
- Enforces self-access (source == target always allowed)
- Registers `TenantResolverGatewayClient` in ClientHub
