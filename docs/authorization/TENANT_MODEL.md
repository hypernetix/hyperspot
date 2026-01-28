# Tenant Model

This document describes HyperSpot's multi-tenancy model, tenant topology, and isolation mechanisms.

## Table of Contents

- [Overview](#overview)
- [Tenant Topology: Forest](#tenant-topology-forest)
- [Tenant Properties](#tenant-properties)
- [Barriers (Self-Managed Tenants)](#barriers-self-managed-tenants)
- [Context Tenant vs Subject Tenant](#context-tenant-vs-subject-tenant)
- [Tenant Subtree Queries](#tenant-subtree-queries)
- [Closure Table](#closure-table)

---

## Overview

HyperSpot uses a **hierarchical multi-tenancy** model where tenants form a forest (multiple independent trees). Each tenant can have child tenants, creating organizational structures like:

```
Vendor
├── Organization A
│   ├── Team A1
│   └── Team A2
└── Organization B
    ├── Team B1
    └── Team B2
```

Key principles:
- **Isolation by default** — tenants cannot access each other's data
- **Hierarchical access** — parent tenants may access child tenant data (configurable)
- **Barriers** — child tenants can opt out of parent visibility via `self_managed` flag

---

## Tenant Topology: Forest

The tenant structure is a **forest** — a collection of independent trees with no single global root.

```
       [T1]              [T5]           ← Root tenants (no parent)
      /    \               |
   [T2]    [T3]          [T6]
     |
   [T4]
```

**Properties:**
- Each tree has exactly one root tenant (`parent_id = NULL`)
- A tenant belongs to exactly one tree
- Trees are completely isolated from each other
- Depth is unlimited (but deep hierarchies may impact performance)

**Why forest, not single tree?**
- Supports multiple independent vendors/organizations
- No artificial "super-root" that would complicate access control
- Each tree can have different policies and configurations
- Enables datacenter migration — vendor can gradually move tenant trees between regions/datacenters without cross-tree dependencies

---

## Tenant Properties

| Property | Type | Description |
|----------|------|-------------|
| `id` | UUID | Unique tenant identifier |
| `parent_id` | UUID? | Parent tenant (NULL for root tenants) |
| `status` | enum | `active`, `suspended`, `deleted` |
| `self_managed` | bool | If true, creates a barrier — parent cannot access this subtree |

**Status semantics:**
- `active` — normal operation
- `suspended` — tenant temporarily disabled (e.g., billing issue), data preserved
- `deleted` — soft-deleted, may be purged after retention period

---

## Barriers (Self-Managed Tenants)

A **barrier** is created when a tenant sets `self_managed = true`. This prevents parent tenants from accessing the subtree rooted at the barrier tenant.

**Example:**

```
T1 (parent)
├── T2 (self_managed=true)  ← BARRIER
│   └── T3
└── T4
```

**Access from T1's perspective:**
- ✅ Can access T1's own resources
- ❌ Cannot access T2's resources (barrier)
- ❌ Cannot access T3's resources (behind barrier)
- ✅ Can access T4's resources

**Access from T2's perspective:**
- ✅ Can access T2's own resources
- ✅ Can access T3's resources (T3 is in T2's subtree, no barrier between them)

**Use cases:**
- Enterprise customer wants data isolation from reseller/partner
- Compliance requirements (data sovereignty)
- Organizational autonomy within a larger structure

**Barrier interpretation is context-dependent:**

Barriers are not absolute — their enforcement depends on the type of data and operation. The same parent-child relationship may have different access rules for different resource types:

| Data Type | Barrier Enforced? | Rationale |
|-----------|-------------------|-----------|
| Business data (tasks, documents) | ✅ Yes | Core isolation requirement |
| Usage/metrics for billing | ❌ No | Parent needs to bill child tenant |
| Audit logs | ⚠️ Configurable | Compliance may require parent visibility |
| Tenant metadata (name, status) | ❌ No | Parent needs to manage child tenants |

**Example:** Reseller T1 has enterprise customer T2 (`self_managed=true`):
- T1 ❌ cannot read T2's business data (tasks, files, etc.)
- T1 ✅ can read T2's usage metrics for billing purposes
- T1 ✅ can see T2's tenant metadata (name, status, plan)

This means `respect_barrier` in authorization requests applies to specific resource types, not globally. Each module/endpoint decides whether barriers apply to its resources.

**Implementation:** The `tenant_closure` table includes a `barrier_ancestor_id` column that tracks the nearest barrier between any ancestor-descendant pair. See [Closure Table](#closure-table).

---

## Context Tenant vs Subject Tenant

Two different tenant concepts appear in authorization:

| Concept | Description | Example |
|---------|-------------|---------|
| **Subject Tenant** | Tenant the user belongs to (from token/identity) | User's "home" organization |
| **Context Tenant** | Tenant scope for the current operation | May differ for cross-tenant operations |

**Typical case:** Subject tenant = Context tenant (user operates in their own tenant)

**Cross-tenant case:** Admin from parent tenant T1 operates in child tenant T2's context:
- Subject tenant: T1 (where admin belongs)
- Context tenant: T2 (where operation is scoped)

**In authorization requests:**
```json
{
  "subject": {
    "properties": { "tenant_id": "T1" }  // Subject tenant
  },
  "context": {
    "tenant_id": "T2"  // Context tenant (single tenant)
    // OR
    "tenant_subtree": { "root_id": "T1" }  // Context tenant subtree
  }
}
```

---

## Tenant Subtree Queries

Many operations need to query "all resources in tenant T and its children". This is a **subtree query**.

**Options for subtree queries:**

| Approach | Pros | Cons |
|----------|------|------|
| Recursive CTE | No extra tables | Slow for deep hierarchies, not portable |
| Explicit ID list from PDP | Simple SQL | Doesn't scale (thousands of IDs) |
| Closure table | O(1) JOIN, scales well | Requires sync, storage overhead |

HyperSpot recommends **closure tables** for production deployments with hierarchical tenants.

**Subtree query parameters:**

| Parameter | Default | Description |
|-----------|---------|-------------|
| `root_id` | required | Root tenant of the subtree |
| `include_root` | `true` | Include root tenant in results |
| `respect_barrier` | `false` | Stop at `self_managed` tenants |
| `tenant_status` | all | Filter by tenant status (`active`, `suspended`) |

---

## Closure Table

The `tenant_closure` table is a denormalized representation of the tenant hierarchy. It contains all ancestor-descendant pairs, enabling efficient subtree queries.

**Schema:**

| Column | Type | Description |
|--------|------|-------------|
| `ancestor_id` | UUID | Ancestor tenant |
| `descendant_id` | UUID | Descendant tenant |
| `barrier_ancestor_id` | UUID? | Nearest barrier between ancestor and descendant (NULL if none) |
| `descendant_status` | enum | Status of descendant tenant (denormalized for query efficiency) |

**Example data for the hierarchy:**

```
T1
├── T2 (self_managed=true)
│   └── T3
└── T4
```

| ancestor_id | descendant_id | barrier_ancestor_id | descendant_status |
|-------------|---------------|---------------------|-------------------|
| T1 | T1 | NULL | active |
| T1 | T2 | T2 | active |
| T1 | T3 | T2 | active |
| T1 | T4 | NULL | active |
| T2 | T2 | NULL | active |
| T2 | T3 | NULL | active |
| T3 | T3 | NULL | active |
| T4 | T4 | NULL | active |

**Query: "All tenants in T1's subtree, respecting barriers"**

```sql
SELECT descendant_id FROM tenant_closure
WHERE ancestor_id = 'T1'
  AND (barrier_ancestor_id IS NULL OR barrier_ancestor_id = 'T1')
```

Result: T1, T4 (T2 and T3 excluded due to barrier)

**Query: "All tenants in T2's subtree"**

```sql
SELECT descendant_id FROM tenant_closure
WHERE ancestor_id = 'T2'
```

Result: T2, T3 (barrier doesn't apply when querying from T2)

**Maintenance:**
- Closure table is maintained by Tenant Resolver module
- Synced from vendor's tenant service
- Updates propagate on tenant create/move/delete/status change

---

## References

- [AUTH.md](./AUTH.md) — Core authorization design
- [SCENARIOS.md](./SCENARIOS.md) — Authorization scenarios with tenant examples
