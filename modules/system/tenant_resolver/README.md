# Tenant Resolver

Tenant information and access resolution for HyperSpot's security layer.

## Overview

The **tenant_resolver** module answers two fundamental questions:

1. **What is this tenant?** — Retrieve tenant metadata (name, status)
2. **Can I access this tenant's data?** — Resolve access relationships from the current security context

The module is **topology-agnostic** — it makes no assumptions about tenant hierarchy structure. Whether the integrating system uses flat tenants, tree hierarchies, DAGs (directed acyclic graph) with multiple parents, or arbitrary graphs, the API remains the same.

## Architecture

```
modules/system/tenant_resolver/
├── tenant_resolver-sdk/         # Public API traits and models
├── tenant_resolver-gw/          # Gateway module (routes to plugins)
└── plugins/
    ├── static_tr_plugin/        # Config-based multi-tenant plugin
    └── single_tenant_tr_plugin/ # Zero-config single-tenant plugin
```

| Layer | Responsibility |
|-------|----------------|
| **SDK** | Public API, plugin API, models |
| **Gateway** | Plugin discovery via GTS, request routing |
| **Plugins** | Actual tenant data and access rule implementations |

## Public API

The gateway registers [`TenantResolverGatewayClient`](tenant_resolver-sdk/src/api.rs) in ClientHub:

- `get_tenant(ctx, id)` — Retrieve tenant by ID
- `can_access(ctx, target, options)` — Check if current tenant can access target
- `get_accessible_tenants(ctx, filter, options)` — List accessible tenants

Source tenant is always taken from `ctx.tenant_id()`.

> [!IMPORTANT]
> All API methods require a valid tenant in the security context. Calls with an
> empty/anonymous context (nil tenant ID) will return `Unauthorized` error.

### Access Rules

- **Non-transitive**: A→B and B→C does NOT imply A→C
- **Non-symmetric**: A→B does NOT imply B→A

> [!NOTE]
> Self-access behavior is plugin-determined. The built-in plugins allow full self-access,
> but custom plugins can implement restrictions (e.g., a tenant cannot delete itself,
> or a suspended tenant may have restricted functionality).

### TenantFilter

`TenantFilter` is used only in `get_accessible_tenants` to filter the returned list:

```rust
// No filter (all tenants)
let tenants = resolver.get_accessible_tenants(&ctx, None, None).await?;

// Only active tenants
let filter = TenantFilter {
    status: vec![TenantStatus::Active],
    ..Default::default()
};
let tenants = resolver.get_accessible_tenants(&ctx, Some(&filter), None).await?;

// Specific tenant IDs that are active
let filter = TenantFilter {
    id: vec![tenant_a, tenant_b],
    status: vec![TenantStatus::Active],
};
let tenants = resolver.get_accessible_tenants(&ctx, Some(&filter), None).await?;
```

Empty vectors mean "no constraint" (include all). This avoids `Option<Vec<T>>` ambiguity.

### AccessOptions

`AccessOptions` specifies permission requirements for access checks:

```rust
// Basic access check (no specific permission required)
let can = resolver.can_access(&ctx, target_id, None).await?;

// Check for specific permission
let options = AccessOptions {
    permission: vec!["read".to_string()],
};
let can = resolver.can_access(&ctx, target_id, Some(&options)).await?;

// Multiple permissions (all required - AND semantics)
let options = AccessOptions {
    permission: vec!["read".to_string(), "write".to_string()],
};
```

### Models

See [`models.rs`](tenant_resolver-sdk/src/models.rs): `TenantId`, `TenantInfo`, `TenantStatus`, `TenantFilter`, `AccessOptions`

`TenantInfo` fields:
- `id` — Unique tenant identifier
- `name` — Human-readable tenant name
- `status` — Lifecycle status (`active`, `suspended`, `deleted`)
- `type` — Optional classification string (e.g., `"enterprise"`, `"trial"`)

### Errors

See [`error.rs`](tenant_resolver-sdk/src/error.rs): `NotFound`, `AccessDenied`, `Unauthorized`, `NoPluginAvailable`, `Internal`

## Plugin API

Plugins implement [`TenantResolverPluginClient`](tenant_resolver-sdk/src/plugin_api.rs) and register via GTS. The gateway handles self-access before delegating to plugins.

HyperSpot includes two plugins out of the box:
- [`static_tr_plugin`](plugins/static_tr_plugin/) — Config-based plugin for testing multi-tenant deployments
- [`single_tenant_tr_plugin`](plugins/single_tenant_tr_plugin/) — Zero-config plugin for single-tenant deployments

## Integration with External Systems

The plugin architecture enables HyperSpot to integrate with existing multi-tenant systems where tenant data and access rules are managed externally.

**Example: Integration with [Zanzibar](https://research.google/pubs/zanzibar-googles-consistent-global-authorization-system/)-style authorization**

Systems like [SpiceDB](https://authzed.com/spicedb), [OpenFGA](https://openfga.dev/), or [Ory Keto](https://www.ory.sh/keto/) provide relationship-based access control. A plugin can bridge HyperSpot to these systems:

```
┌─────────────────────────────────────────────────────────┐
│                      HyperSpot                          │
│                                                         │
│  ┌─────────────────┐      ┌─────────────────────────┐   │
│  │ tenant_resolver │      │  zanzibar_tr_plugin     │   │
│  │    gateway      │─────▶│  (adapter)              │   │
│  └─────────────────┘      └────────┬────────────────┘   │
│                                    │                    │
└────────────────────────────────────┼────────────────────┘
                                     │
                   ┌─────────────────┴─────────────────┐
                   ▼                                   ▼
          ┌──────────────┐                    ┌──────────────┐
          │  Tenant DB   │                    │   Zanzibar   │
          │  (external)  │                    │  (external)  │
          └──────────────┘                    └──────────────┘
```

In this scenario:
- **Tenant DB** — External database storing tenant metadata (name, status)
- **Zanzibar** — External authorization service storing access relationships
- **Plugin** — Adapter that bridges HyperSpot to both external systems

| Method | Data Source |
|--------|-------------|
| `get_tenant` | Tenant DB |
| `can_access` | Zanzibar `Check` API |
| `get_accessible_tenants` | Zanzibar `LookupResources` + Tenant DB |

**Example Zanzibar schema:**

```
definition tenant {
    relation accessor: tenant    // direct access grant
    relation parent: tenant      // optional hierarchy

    permission access = accessor + parent + parent->access
}

// Example relationships:
// tenant:acme#accessor@tenant:partner    → partner can access acme
// tenant:acme#parent@tenant:corp         → corp (and its accessors) can access acme
```

This pattern allows HyperSpot to operate as a component within a larger system without owning the tenant or authorization data.

Similar plugins can integrate with other authorization systems (LDAP, custom APIs, etc.) — the gateway remains agnostic to the backend.

## Configuration

### Gateway

See [`config.rs`](tenant_resolver-gw/src/config.rs)

```yaml
modules:
  tenant_resolver:
    vendor: "hyperspot"  # Selects plugin by matching vendor
```

### Static Plugin

See [`config.rs`](plugins/static_tr_plugin/src/config.rs)

```yaml
modules:
  static_tr_plugin:
    vendor: "hyperspot"
    priority: 100           # Lower = higher priority
    tenants:
      - id: "550e8400-e29b-41d4-a716-446655440001"
        name: "Tenant A"
        status: active
        type: enterprise  # optional
    access_rules:
      - source: "550e8400-e29b-41d4-a716-446655440001"
        target: "550e8400-e29b-41d4-a716-446655440002"
```

## Usage

```rust
let resolver = hub.get::<dyn TenantResolverGatewayClient>()?;

// Get tenant info (returns any status)
let tenant = resolver.get_tenant(&ctx, tenant_id).await?;

// Check basic access
let can_access = resolver.can_access(&ctx, target_id, None).await?;

// Get all accessible tenants
let accessible = resolver.get_accessible_tenants(&ctx, None, None).await?;

// Get only active accessible tenants
let filter = TenantFilter {
    status: vec![TenantStatus::Active],
    ..Default::default()
};
let active_tenants = resolver.get_accessible_tenants(&ctx, Some(&filter), None).await?;
```

## Technical Decisions

### Gateway + Plugin Pattern

Multiple backends are planned (config-based, DB-driven, external API). The gateway handles cross-cutting concerns consistently while plugins can be developed independently.

### Source Tenant from security_context

Using `security_context.tenant_id()` as the source tenant reduces API surface, prevents misuse, and aligns with existing patterns.

### Self-Access Enforcement

Self-access is plugin-determined, not gateway-enforced. This allows plugins to implement
nuanced access policies (e.g., restricting certain operations on the tenant itself, or
limiting functionality for suspended tenants). The built-in plugins (`static_tr_plugin`,
`single_tenant_tr_plugin`) allow full self-access.

## API Rationale

### Why `get_tenant` Has No Filter

The `get_tenant` method returns tenant info regardless of status. Rationale:

1. **Information preservation**: Consumers can decide how to handle different statuses (active, suspended, deleted) based on their business logic
2. **Better error messages**: Consumers can show "Tenant is suspended" vs "Tenant not found"
3. **Flexibility**: Some consumers may need to access deleted tenants for audit purposes

### Why `can_access` Has No Filter

Access rules (including status-based restrictions) are **plugin-determined**. Rationale:

1. **Plugin autonomy**: A plugin may allow read-only access to suspended tenants
2. **Consumer flexibility**: If consumers need status-specific logic, they can call `get_tenant` separately
3. **Cleaner semantics**: `can_access` answers "can I access?" — not "can I access if active?"

### Why Filter Only in `get_accessible_tenants`

Filtering makes sense for list operations:

1. **Performance**: Filter at source, not after fetching potentially large lists
2. **Common pattern**: "Give me all active tenants I can access" is a frequent use case
3. **No information loss**: Single-item operations (`get_tenant`, `can_access`) benefit from returning full info

### Why `AccessOptions` Uses AND Semantics

Multiple permissions in `AccessOptions` require **all** to be satisfied:

```rust
// Must have BOTH read AND write permission
let options = AccessOptions {
    permission: vec!["read".to_string(), "write".to_string()],
};
```

Rationale:
- AND is the safer, more restrictive default
- OR can be achieved client-side with multiple calls
- Most permission systems use AND for "must have these capabilities"

## Implementation Phases

### Phase 1: Core (Current)

- `get_tenant`, `can_access`, `get_accessible_tenants` APIs
- `TenantFilter` for id/status-based filtering
- `AccessOptions` for permission-based access checks
- Static plugin with config-driven access rules
- Single-tenant plugin for simple deployments
- ClientHub registration for in-process consumption

### Phase 2: gRPC (Planned)

- gRPC API for out-of-process consumers
