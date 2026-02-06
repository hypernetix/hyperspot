# Tenant Resolver

Tenant hierarchy and information resolution for CyberFabric's multi-tenancy layer.

## Overview

The tenant model that CyberFabric operates on is described in [TENANT_MODEL.md](../../../docs/arch/authorization/TENANT_MODEL.md).

The **tenant_resolver** module provides a hierarchical tenant model with:

1. **Tenant information** — Retrieve tenant metadata (name, status, parent, type)
2. **Hierarchy traversal** — Navigate parent chains (ancestors) and children subtrees (descendants)
3. **Ancestry verification** — Check if one tenant is an ancestor of another

The module supports **barrier semantics** where self-managed tenants act as boundaries that
block hierarchy traversal from parent tenants (unless explicitly ignored).

## Public API

The gateway registers [`TenantResolverGatewayClient`](tenant_resolver-sdk/src/api.rs) in ClientHub:

- `get_tenant(ctx, id)` — Retrieve single tenant by ID
- `get_tenants(ctx, ids, filter)` — Retrieve multiple tenants by IDs (batch)
- `get_ancestors(ctx, id, options)` — Get parent chain from tenant to root
- `get_descendants(ctx, id, filter, options, max_depth)` — Get children subtree; `response.tenant` contains the starting tenant, `response.descendants` contains the subtree
- `is_ancestor(ctx, ancestor_id, descendant_id, options)` — Check ancestry relationship

The `SecurityContext` is passed to the plugin for use in access control decisions. Each plugin
implements its own logic for using (or ignoring) the context.

### Hierarchy Model

```
T1 (root)
├── T2 (self_managed=true) ← BARRIER
│   └── T3
└── T4
```

- **`parent_id`**: Links a tenant to its parent (None for root tenants)
- **`self_managed`**: When true, this tenant is a barrier that blocks parent traversal into its subtree

### `TenantFilter`

Filter tenants by status:

```rust
// No filter (all tenants)
let tenants = resolver.get_tenants(&ctx, &ids, None).await?;

// Only active tenants
let filter = TenantFilter {
    status: vec![TenantStatus::Active],
};
let tenants = resolver.get_tenants(&ctx, &ids, Some(&filter)).await?;
```

An empty `status` vector means "no constraint" (include all statuses).

### `HierarchyOptions`

Control barrier behavior during hierarchy traversal:

```rust
// Default: respect barriers
let ancestors = resolver.get_ancestors(&ctx, id, None).await?;

// Ignore barriers: traverse through self_managed tenants
let options = HierarchyOptions {
    barrier_mode: BarrierMode::Ignore,
};
let ancestors = resolver.get_ancestors(&ctx, id, Some(&options)).await?;
```

### Barrier Semantics

When `self_managed = true` (barrier exists):
- Parent cannot traverse into this subtree (with `BarrierMode::Respect`, the default)
- Using `BarrierMode::Ignore` ignores barriers and traverses through
- **Barriers never produce errors** — traversal stops silently and returns truncated results

**`get_ancestors`** — if the starting tenant is a barrier, ancestors are **empty**; if a barrier is encountered in the chain, it **is included** but traversal stops after it:
- `get_ancestors(T2, barrier_mode=Respect)` → tenant=T2, ancestors=[] (T2 is a barrier, cannot see parent chain)
- `get_ancestors(T3, barrier_mode=Respect)` → tenant=T3, ancestors=[T2] (T2 is the barrier, included; T1 is not reached)
- `get_ancestors(T3, barrier_mode=Ignore)` → tenant=T3, ancestors=[T2, T1]

**`get_descendants`** — barrier tenant **is excluded** along with its subtree; but a barrier can see its own subtree:
- `get_descendants(T1, barrier_mode=Respect)` → tenant=T1, descendants=[T4] (T2 is a barrier, excluded along with T3)
- `get_descendants(T2, barrier_mode=Respect)` → tenant=T2, descendants=[T3] (T2 can see its own subtree)
- `get_descendants(T1, barrier_mode=Ignore)` → tenant=T1, descendants=[T2, T3, T4]

**`is_ancestor`** — returns `false` if a barrier blocks the path:
- `is_ancestor(T1, T3, barrier_mode=Respect)` → false (blocked by T2)
- `is_ancestor(T1, T3, barrier_mode=Ignore)` → true

### Filter Semantics

Filter is supported only for methods where it provides significant value:

| Method | `TenantNotFound` error | Filter support |
|--------|------------------------|----------------|
| `get_tenant(id)` | `id` doesn't exist | — (no filter) |
| `get_tenants(ids, filter)` | — (skip missing) | filters results |
| `get_ancestors(id)` | `id` doesn't exist | — (no filter) |
| `get_descendants(id, filter)` | `id` doesn't exist | filters descendants |
| `is_ancestor(a, d)` | `a` or `d` doesn't exist | — (no filter) |

**Design rationale:**

- **`get_ancestors`** returns the full chain — caller can filter the typically small result
- **`is_ancestor`** answers "is A ancestor of D?" — filtering the path is a rare use case
- **`get_descendants`** supports filter because descendants can be many and "all active descendants" is a common access control pattern

**Principles:**

1. **`TenantNotFound`** — Only raised when tenant **physically** doesn't exist
2. **Filter does NOT affect** existence check of the starting tenant
3. **Filter applies only to** results (descendants list), not to the starting tenant

### `get_descendants` Traversal Semantics

`get_descendants` uses **pre-order traversal**: each node is visited before its children.

**Filter as traversal barrier:** If a node doesn't pass the filter, it is excluded **along with its entire subtree**. This is intentional — if a parent tenant is suspended, its children should not be reachable.

**Example:**

```
A (active) → B (suspended) → C (active)
          → D (active)
```

```rust
// Without filter: returns [B, C, D] (pre-order)
resolver.get_descendants(&ctx, A, None, None, None).await?;

// With filter={status: Active}: returns [D] only
// B is excluded (suspended), so C is unreachable
resolver.get_descendants(&ctx, A, Some(&active_filter), None, None).await?;
```

Note: Sibling order within the same parent is not guaranteed.

### Models

See [`models.rs`](tenant_resolver-sdk/src/models.rs): `TenantId`, `TenantInfo`, `TenantRef`, `TenantStatus`, `TenantFilter`, `HierarchyOptions`, `BarrierMode`, `GetAncestorsResponse`, `GetDescendantsResponse`

**`TenantInfo`** — Full tenant information (for `get_tenant`, `get_tenants`):
- `id` — Unique tenant identifier
- `name` — Human-readable tenant name
- `status` — Lifecycle status (`Active`, `Suspended`, `Deleted`)
- `tenant_type` — Optional classification string (e.g., `"enterprise"`, `"trial"`)
- `parent_id` — Parent tenant ID (None for root tenants)
- `self_managed` — True if this tenant is a barrier

**`TenantRef`** — Tenant reference without name (for `get_ancestors`, `get_descendants`):
- All fields except `name`
- Use `get_tenants(ids)` if display names are needed

### Errors

See [`error.rs`](tenant_resolver-sdk/src/error.rs): `TenantNotFound`, `Unauthorized`, `NoPluginAvailable`, `ServiceUnavailable`, `Internal`

## Plugin API

Plugins implement [`TenantResolverPluginClient`](tenant_resolver-sdk/src/plugin_api.rs) and register via GTS.

CyberFabric includes two plugins out of the box:
- [`static_tr_plugin`](plugins/static_tr_plugin/) — Config-based plugin with hierarchical tenant support
- [`single_tenant_tr_plugin`](plugins/single_tenant_tr_plugin/) — Zero-config plugin for single-tenant deployments

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
        name: "Root Tenant"
        status: active
        type: enterprise
      - id: "550e8400-e29b-41d4-a716-446655440002"
        name: "Child Tenant"
        status: active
        parent_id: "550e8400-e29b-41d4-a716-446655440001"
        self_managed: false
```

## Usage

```rust
let resolver = hub.get::<dyn TenantResolverGatewayClient>()?;

// Get tenant info
let tenant = resolver.get_tenant(&ctx, tenant_id).await?;

// Get multiple tenants (batch)
let tenants = resolver.get_tenants(&ctx, &[id1, id2], None).await?;

// Get ancestor chain
let response = resolver.get_ancestors(&ctx, tenant_id, None).await?;
println!("Tenant: {:?}, Ancestors: {:?}", response.tenant, response.ancestors);

// Get descendants (max_depth=None means unlimited)
let response = resolver.get_descendants(&ctx, tenant_id, None, None, None).await?;
println!("Tenant: {:?}, Descendants: {:?}", response.tenant, response.descendants);

// Get only active descendants
let filter = TenantFilter { status: vec![TenantStatus::Active] };
let response = resolver.get_descendants(&ctx, tenant_id, Some(&filter), None, None).await?;

// Check ancestry
let is_parent = resolver.is_ancestor(&ctx, parent_id, child_id, None).await?;
```

## Technical Decisions

### Gateway + Plugin Pattern

Multiple backends are planned (config-based, DB-driven, external API). The gateway handles cross-cutting concerns consistently while plugins can be developed independently.

### Barrier Semantics

The `self_managed` field on tenants creates traversal barriers. This enables:
- Delegated administration: self-managed tenants control their own subtrees
- Privacy boundaries: parent organizations cannot see into self-managed subsidiaries
- Flexible traversal: callers can opt to ignore barriers when needed (e.g., system operations)

### Batch Semantics

The `get_tenants` method returns only found tenants — missing IDs are silently skipped.
This simplifies callers who want to fetch multiple tenants without handling per-item errors.

- **Order**: Output order is not guaranteed (may differ from input order)
- **Duplicates**: Duplicate IDs in the input are deduplicated
- **Empty input**: Returns an empty list when `ids` is empty

## Implementation Phases

### Phase 1: Core (Current)

- `get_tenant`, `get_tenants` APIs
- `get_ancestors`, `get_descendants`, `is_ancestor` for hierarchy traversal
- `TenantFilter` for status-based filtering
- `HierarchyOptions` with `BarrierMode` for traversal control
- Static plugin with config-driven hierarchy
- Single-tenant plugin for simple deployments
- ClientHub registration for in-process consumption

### Phase 2: gRPC (Planned)

- gRPC API for out-of-process consumers
