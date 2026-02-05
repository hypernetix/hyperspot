# Resource Group Model

This document describes HyperSpot's resource group model for authorization: group topology, membership mechanisms, and how groups are used in access control.

## Table of Contents

- [Resource Group Model](#resource-group-model)
  - [Table of Contents](#table-of-contents)
  - [Overview](#overview)
  - [Resource Group Topology: Forest](#resource-group-topology-forest)
  - [Resource Group Properties](#resource-group-properties)
  - [Resource-to-Group Membership](#resource-to-group-membership)
  - [Closure Table](#closure-table)
  - [Membership Table](#membership-table)
  - [Relationship with Tenant Model](#relationship-with-tenant-model)
  - [References](#references)

---

## Overview

HyperSpot uses **resource groups** as an optional organizational layer for grouping resources. The primary purpose is **access control** — granting permissions at the group level rather than per-resource.

Vendors may implement various group types depending on their domain. Examples include:

- Projects (task management, issue tracking)
- Workspaces (collaboration spaces)
- Folders (document organization with nesting)
- Teams, Departments, Campaigns, etc.

The specific group types and their semantics are vendor-defined. HyperSpot provides the infrastructure for hierarchical grouping and membership resolution without prescribing what groups represent.

```
Tenant T1
├── [Group A]
│   ├── Resource 1
│   ├── Resource 2
│   └── [Group A.1]
│       └── Resource 3
├── [Group B]
│   ├── Resource 1
│   └── Resource 4
└── (ungrouped resources)
```

Key principles:
- **Optional** — resources may exist without group membership
- **Many-to-many** — a resource can belong to multiple groups
- **Hierarchical** — groups can form nested structures
- **Tenant-scoped** — groups exist within tenant boundaries

---

## Resource Group Topology: Forest

The group structure is a **forest** — a collection of independent trees within a tenant.

```
Tenant T1:
    [G1]              [G4]           ← Root groups (no parent)
   /    \               |
 [G2]   [G3]          [G5]
   |
 [G6]

Tenant T2:
    [G7]
     |
    [G8]
```

**Properties:**
- Each tree has exactly one root group (`parent_id = NULL`)
- A group belongs to exactly one tree within a tenant
- Trees are completely isolated from each other
- Groups in different tenants are isolated by tenant boundaries
- Depth is unlimited (but deep hierarchies may impact performance)

---

## Resource Group Properties

Resource groups are stored on the **vendor side** (in the vendor's Resource Group service). HyperSpot does not store the full group entity — only local projections for authorization (closure and membership tables).

The following properties are the **minimum** HyperSpot expects from the vendor's group model:

| Property | Type | Description |
|----------|------|-------------|
| `id` | UUID | Unique group identifier |
| `tenant_id` | UUID | Owning tenant (groups are tenant-scoped) |
| `parent_id` | UUID? | Parent group (NULL for root groups) |

Vendors typically maintain additional fields (name, description, type, status, metadata, etc.) in their own systems. HyperSpot's RG Resolver plugin syncs only the hierarchy structure needed for authorization.

---

## Resource-to-Group Membership

Resources are associated with groups via the **membership** relationship. This is a many-to-many relationship:

- A resource can belong to multiple groups
- A group can contain multiple resources

```
┌─────────────┐         ┌─────────────┐
│  Resource   │◄───────►│    Group    │
│  (Task 1)   │   M:N   │ (Project A) │
└─────────────┘         └─────────────┘
       │
       ▼
┌─────────────┐
│    Group    │
│ (Project B) │
└─────────────┘
```

**Membership properties (minimum):**

| Property | Type | Description |
|----------|------|-------------|
| `resource_id` | UUID | ID of the resource |
| `group_id` | UUID | ID of the group |

**Design decisions:**

1. **Explicit membership, inherited access** — a resource is added to a specific group (explicit membership). However, access is inherited top-down: a user with access to parent group G1 can access resources in all descendant groups (G2, G3, etc.) via `in_group_subtree` predicate.

> **TODO:** Design resource-to-group membership operations (add, remove, move). This includes API contract, validation rules, sync mechanism with vendor systems, and behavior on resource deletion.

---

## Closure Table

The `resource_group_closure` table is a denormalized representation of the group hierarchy. It contains all ancestor-descendant pairs, enabling efficient subtree queries.

**Schema:**

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| `ancestor_id` | UUID | No | Ancestor group |
| `descendant_id` | UUID | No | Descendant group |

**Notes:**
- Self-referential rows exist: each group has a row where `ancestor_id = descendant_id`
- The table is scoped per module database (not global)

**Example data for the hierarchy:**

```
G1
├── G2
│   └── G6
└── G3
```

| ancestor_id | descendant_id |
|-------------|---------------|
| G1 | G1 |
| G1 | G2 |
| G1 | G3 |
| G1 | G6 |
| G2 | G2 |
| G2 | G6 |
| G3 | G3 |
| G6 | G6 |

**Query: "All groups in G1's subtree"**

```sql
SELECT descendant_id FROM resource_group_closure
WHERE ancestor_id = 'G1'
```

Result: G1, G2, G3, G6

**Synchronization:** How closure tables are synchronized with vendor systems, consistency guarantees, and conflict resolution are out of scope for this document. See Resource Group Resolver design documentation (TBD).

---

## Membership Table

The `resource_group_membership` table stores the many-to-many relationship between resources and groups.

**Schema:**

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| `resource_id` | UUID | No | ID of the resource |
| `group_id` | UUID | No | ID of the group |

**Notes:**
- Primary key: `(resource_id, group_id)`
- The `resource_id` column joins with the resource table's ID column
- The table is scoped per module database (each module has its own membership table)

**Example data:**

| resource_id | group_id |
|-------------|----------|
| task-1 | ProjectA |
| task-1 | ProjectB |
| task-2 | ProjectA |
| task-3 | FolderA-Sub1 |

**Query: "All resources in groups G1 and G2 (flat)"**

```sql
SELECT * FROM tasks
WHERE id IN (
  SELECT resource_id FROM resource_group_membership
  WHERE group_id IN ('G1', 'G2')
)
```

**Synchronization:** How projection tables are synchronized with vendor systems, consistency guarantees, and conflict resolution are out of scope for this document. See Resource Group Resolver design documentation.

---

## Relationship with Tenant Model

**Tenants** and **Resource Groups** serve different purposes:

| Aspect | Tenant | Resource Group |
|--------|--------|----------------|
| **Purpose** | Ownership, isolation, billing | Grouping for access control |
| **Scope** | System-wide | Per-tenant |
| **Resource relationship** | Ownership (1:N) | Membership (M:N) |
| **Hierarchy** | Forest (multiple roots) | Forest (multiple roots per tenant) |

Resource groups operate **within** tenant boundaries. They provide additional organizational structure but do not override tenant isolation.

**Key rules:**

1. **Groups are tenant-scoped** — a group belongs to exactly one tenant
2. **Cross-tenant groups are forbidden** — a group cannot span multiple tenants
3. **Tenant constraint always applies** — authorization always includes a tenant constraint alongside group predicates (see [AUTH.md](./AUTH.md) for details)

---

## References

- [AUTH.md](./AUTH.md) — Core authorization design
- [TENANT_MODEL.md](./TENANT_MODEL.md) — Tenant topology, barriers, closure tables
- [AUTHZ_USAGE_SCENARIOS.md](./AUTHZ_USAGE_SCENARIOS.md) — Authorization scenarios with resource group examples
