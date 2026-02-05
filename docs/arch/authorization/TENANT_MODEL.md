# Tenant Model

This document describes HyperSpot's multi-tenancy model, tenant topology, and isolation mechanisms.

## Table of Contents

- [Tenant Model](#tenant-model)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Tenant Topology: Forest](#tenant-topology-forest)
  - [Tenant Properties](#tenant-properties)
  - [Barriers (Self-Managed Tenants)](#barriers-self-managed-tenants)
  - [Context Tenant vs Subject Tenant](#context-tenant-vs-subject-tenant)
  - [Tenant Subtree Queries](#tenant-subtree-queries)
  - [Closure Table](#closure-table)
  - [References](#references)

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

This means `barrier_mode` in authorization requests applies to specific resource types, not globally. Each module/endpoint decides whether barriers apply to its resources.

**Implementation:** The `tenant_closure` table includes a `barrier` column that indicates whether a barrier exists between ancestor and descendant. See [Closure Table](#closure-table).

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
```jsonc
{
  "subject": {
    "properties": { "tenant_id": "T1" }  // Subject tenant (optional)
  },
  "context": {
    "tenant_context": {
      "mode": "root_only",  // Single tenant T2
      "root_id": "T2"
    }
    // OR for subtree:
    // "tenant_context": {
    //   "mode": "subtree",   // T1 + descendants
    //   "root_id": "T1"
    // }
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

**Tenant scope parameters (in `context.tenant_context`):**

| Parameter | Default | Description |
|-----------|---------|-------------|
| `mode` | `"subtree"` | `"root_only"` (single tenant) or `"subtree"` (tenant + descendants) |
| `root_id` | — | Root tenant. Optional — PDP can determine from `token_scopes` or `subject.properties.tenant_id` |
| `barrier_mode` | `"all"` | `"all"` (respect barriers) or `"none"` (ignore). See [DESIGN.md](./DESIGN.md#3-tenant-subtree-predicate-type-in_tenant_subtree). |
| `tenant_status` | all | Filter by tenant status (`active`, `suspended`) |

---

## Closure Table

The `tenant_closure` table is a denormalized representation of the tenant hierarchy. It contains all ancestor-descendant pairs, enabling efficient subtree queries.

**Schema:**

| Column | Type | Description |
|--------|------|-------------|
| `ancestor_id` | UUID | Ancestor tenant |
| `descendant_id` | UUID | Descendant tenant |
| `barrier` | INT NOT NULL DEFAULT 0 | 0 = no barrier on path, 1 = barrier exists between ancestor and descendant |
| `descendant_status` | enum | Status of descendant tenant (denormalized for query efficiency) |

**Barrier semantics:** The `barrier` column stores whether a barrier exists **strictly between** ancestor and descendant, **not including the ancestor itself**. This means:
- When querying from T2 (a self_managed tenant), rows with `ancestor_id = T2` have `barrier = 0` because T2 is the ancestor, not "between" itself and its descendants
- When querying from T1 (parent of T2), rows with `ancestor_id = T1` and `descendant_id` in T2's subtree have `barrier = 1` because T2 is between T1 and its descendants

**Example data for the hierarchy:**

```
T1
├── T2 (self_managed=true)
│   └── T3
└── T4
```

| ancestor_id | descendant_id | barrier | descendant_status |
|-------------|---------------|---------|-------------------|
| T1 | T1 | 0 | active |
| T1 | T2 | 1 | active |
| T1 | T3 | 1 | active |
| T1 | T4 | 0 | active |
| T2 | T2 | 0 | active |
| T2 | T3 | 0 | active |
| T3 | T3 | 0 | active |
| T4 | T4 | 0 | active |

**Key observations:**
- `T1 → T2`: barrier = 1 because T2 (self_managed) is on the path
- `T1 → T3`: barrier = 1 because T2 is on the path from T1 to T3
- `T2 → T2` and `T2 → T3`: barrier = 0 because T2 is the **ancestor**, not between T2 and its descendants

**Query: "All tenants in T1's subtree, with `barrier_mode: "all"`"**

```sql
-- barrier_mode: "all" (default) adds the barrier clause
SELECT descendant_id FROM tenant_closure
WHERE ancestor_id = 'T1'
  AND barrier = 0
-- barrier_mode: "none" omits the barrier clause
```

Result: T1, T4 (T2 and T3 excluded due to barrier = 1)

**Query: "All tenants in T2's subtree"**

```sql
SELECT descendant_id FROM tenant_closure WHERE ancestor_id = 'T2' AND barrier = 0
```

Result: T2, T3 (barrier = 0 for both rows because T2 is the ancestor, not between T2 and its descendants)

**Future extensibility:** The `barrier` column is INT to allow future use as a bitmask for multiple barrier types (e.g., bit 0 for self_managed, bit 1 for data_sovereignty). SQL would change from `barrier = 0` to `(barrier & mask) = 0` for selective enforcement.

**Synchronization:** How projection tables are synchronized with vendor systems, consistency guarantees, and conflict resolution are out of scope for this document. See Tenant Resolver design documentation (TBD).

---

## References

- [DESIGN.md](./DESIGN.md) — Core authorization design
- [RESOURCE_GROUP_MODEL.md](./RESOURCE_GROUP_MODEL.md) — Resource group topology, membership, hierarchy
- [AUTHZ_USAGE_SCENARIOS.md](./AUTHZ_USAGE_SCENARIOS.md) — Authorization scenarios with tenant examples
