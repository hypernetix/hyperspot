# Authorization Usage Scenarios

This document demonstrates the authorization model through concrete examples.
Each scenario shows the full flow: HTTP request → PDP evaluation → SQL execution.

For the core authorization design, see [AUTH.md](./AUTH.md).

All examples use a Task Management domain:
- **Resource:** `tasks` table with `id`, `owner_tenant_id`, `title`, `status`
- **Resource Groups:** Projects (tasks belong to projects)
- **Tenant Model:** Hierarchical multi-tenancy — see [TENANT_MODEL.md](./TENANT_MODEL.md) for details on topology, barriers, and closure tables

---

## Table of Contents

- [Authorization Usage Scenarios](#authorization-usage-scenarios)
  - [Table of Contents](#table-of-contents)
  - [Projection Tables](#projection-tables)
    - [What Are Projection Tables?](#what-are-projection-tables)
    - [Choosing Projection Tables](#choosing-projection-tables)
    - [Capabilities and PDP Response](#capabilities-and-pdp-response)
    - [When No Projection Tables Are Needed](#when-no-projection-tables-are-needed)
    - [When to Use `tenant_closure`](#when-to-use-tenant_closure)
    - [When to Use `resource_group_membership`](#when-to-use-resource_group_membership)
    - [When to Use `resource_group_closure`](#when-to-use-resource_group_closure)
    - [Combinations Summary](#combinations-summary)
  - [Scenarios](#scenarios)
    - [With `tenant_closure`](#with-tenant_closure)
      - [S01: LIST, tenant subtree, PEP has tenant\_closure](#s01-list-tenant-subtree-pep-has-tenant_closure)
      - [S02: GET, tenant subtree, PEP has tenant\_closure](#s02-get-tenant-subtree-pep-has-tenant_closure)
      - [S03: UPDATE, tenant subtree, PEP has tenant\_closure](#s03-update-tenant-subtree-pep-has-tenant_closure)
      - [S04: CREATE, tenant context](#s04-create-tenant-context)
      - [S05: LIST, billing data, ignore barriers (barrier\_mode: "none")](#s05-list-billing-data-ignore-barriers-barrier_mode-none)
    - [Without `tenant_closure`](#without-tenant_closure)
      - [S06: LIST, tenant subtree, PEP without tenant\_closure](#s06-list-tenant-subtree-pep-without-tenant_closure)
      - [S07: GET, tenant subtree, PEP without tenant\_closure](#s07-get-tenant-subtree-pep-without-tenant_closure)
      - [S08: UPDATE, tenant subtree, PEP without tenant\_closure (prefetch)](#s08-update-tenant-subtree-pep-without-tenant_closure-prefetch)
      - [S09: GET, context tenant only (no subtree)](#s09-get-context-tenant-only-no-subtree)
    - [Resource Groups](#resource-groups)
      - [S10: LIST, group membership, PEP has resource\_group\_membership](#s10-list-group-membership-pep-has-resource_group_membership)
      - [S11: LIST, group subtree, PEP has resource\_group\_closure](#s11-list-group-subtree-pep-has-resource_group_closure)
      - [S12: UPDATE, group membership, PEP has resource\_group\_membership](#s12-update-group-membership-pep-has-resource_group_membership)
      - [S13: UPDATE, group subtree, PEP has resource\_group\_closure](#s13-update-group-subtree-pep-has-resource_group_closure)
      - [S14: GET, group membership, PEP without resource\_group\_membership](#s14-get-group-membership-pep-without-resource_group_membership)
      - [S15: LIST, group subtree, PEP has membership but no closure](#s15-list-group-subtree-pep-has-membership-but-no-closure)
    - [Advanced Patterns](#advanced-patterns)
      - [S16: LIST, tenant subtree and group membership (AND)](#s16-list-tenant-subtree-and-group-membership-and)
      - [S17: LIST, tenant subtree and group subtree](#s17-list-tenant-subtree-and-group-subtree)
      - [S18: LIST, multiple access paths (OR)](#s18-list-multiple-access-paths-or)
      - [S19: Access denied](#s19-access-denied)
  - [TOCTOU Analysis](#toctou-analysis)
    - [When TOCTOU Matters](#when-toctou-matters)
    - [How Each Scenario Handles TOCTOU](#how-each-scenario-handles-toctou)
    - [Key Insight: Prefetch + Constraint for Mutations](#key-insight-prefetch--constraint-for-mutations)
  - [References](#references)

---

## Projection Tables

### What Are Projection Tables?

**Projection tables** are local copies of hierarchical or relational data that enable efficient SQL-level authorization. Instead of calling external services during query execution, PEP uses these pre-synced tables to enforce constraints directly in the database.

**The problem they solve:** When PDP returns constraints like "user can access resources in tenant subtree T1", the PEP needs to translate this into SQL. Without local data, PEP would need to:
1. Call an external service to resolve the tenant hierarchy, or
2. Receive thousands of explicit tenant IDs from PDP (doesn't scale)

Projection tables allow PEP to JOIN against local data, making authorization O(1) regardless of hierarchy size.

**Types of projection tables:**

| Table | Purpose | Enables |
|-------|---------|---------|
| `tenant_closure` | Denormalized tenant hierarchy (ancestor→descendant pairs) | `in_tenant_subtree` predicate — efficient subtree queries without recursive CTEs |
| `resource_group_membership` | Resource-to-group associations | `in_group` predicate — filter by group membership |
| `resource_group_closure` | Denormalized group hierarchy | `in_group_subtree` predicate — filter by group subtree |

**Closure tables** specifically solve the hierarchy traversal problem. A closure table contains all ancestor-descendant pairs, allowing subtree queries with a simple `WHERE ancestor_id = X` instead of recursive tree walking.

### Choosing Projection Tables

The choice depends on the application's tenant structure, resource organization, and **endpoint requirements**. Even with a hierarchical tenant model, specific endpoints may operate within a single context tenant (see S09).

### Capabilities and PDP Response

| PEP Capability | Closure Table | Prefetch | PDP Response |
|----------------|---------------|----------|--------------|
| `tenant_hierarchy` | tenant_closure ✅ | **No** | `in_tenant_subtree` predicate |
| (none) | ❌ | **Yes** | `eq`/`in` or decision only |
| `group_hierarchy` | resource_group_closure ✅ | **No** | `in_group_subtree` predicate |
| `group_membership` | resource_group_membership ✅ | **No** | `in_group` predicate |
| (none for groups) | ❌ | **Yes** | explicit resource IDs |

### When No Projection Tables Are Needed

| Condition | Why Tables Aren't Required |
|-----------|---------------------------|
| Endpoint operates in context tenant only | No subtree traversal → `eq` on `owner_tenant_id` is sufficient (see S09) |
| Few tenants per vendor | PDP can return explicit tenant IDs in `in` predicate |
| Flat tenant structure | No hierarchy → `in_tenant_subtree` not needed |
| No resource groups | `in_group*` predicates not used |
| Low frequency LIST requests | Prefetch overhead is acceptable |

**Important:** The first condition applies regardless of overall tenant model. Even in a hierarchical multi-tenant system, specific endpoints may be designed to work within a single context tenant without subtree access. This is an endpoint-level decision, not a system-wide constraint.

**Example:** Internal enterprise tool with 10 tenants, flat structure. Or: a "My Tasks" endpoint that shows only tasks in user's direct tenant, even though the system supports tenant hierarchy for other operations.

### When to Use `tenant_closure`

| Condition | Why Closure Is Needed |
|-----------|----------------------|
| Tenant hierarchy (parent-child) + many tenants | PDP cannot return all IDs in `in` predicate |
| Frequent LIST requests by subtree | Subtree JOINs more efficient than explicit ID lists |

**Note:** Self-managed tenants (barriers) and tenant status filtering can be checked by PDP on its side — this doesn't require closure on PEP side.

**Example:** Multi-tenant SaaS with organization hierarchy (org → teams → projects) and thousands of tenants.

### When to Use `resource_group_membership`

| Condition | Why Membership Table Is Needed |
|-----------|-------------------------------|
| Resources belong to groups | Projects, workspaces, folders |
| Frequent group-based filters | "Show all tasks in Project X" |
| Access control via groups | Role assignments at group level |

**Example:** Project management tool where tasks belong to projects.

### When to Use `resource_group_closure`

| Condition | Why Group Closure Is Needed |
|-----------|----------------------------|
| Group hierarchy | Nested folders, sub-projects |
| Subtree queries by groups | "Show all in folder and subfolders" |
| Many groups | PDP cannot expand entire hierarchy to explicit IDs |

**Example:** Document management with nested folders.

### Combinations Summary

| Use Case | tenant_closure | group_membership | group_closure |
|----------|----------------|------------------|---------------|
| Simple SaaS (flat tenants, no groups) | ❌ | ❌ | ❌ |
| Enterprise SaaS (tenant hierarchy) | ✅ | ❌ | ❌ |
| Project-based SaaS (flat tenants + projects) | ❌ | ✅ | ❌ |
| Complex SaaS (hierarchy + nested projects) | ✅ | ✅ | ✅ |

---

## Scenarios

> **Note:** SQL examples use subqueries for clarity. Production implementations
> may use JOINs or EXISTS for performance optimization.

### With `tenant_closure`

PEP has local tenant_closure table → can enforce `in_tenant_subtree` predicates.

---

#### S01: LIST, tenant subtree, PEP has tenant_closure

`GET /tasks?tenant_subtree=true`

User requests all tasks visible in their tenant subtree.

**Request:**
```http
GET /tasks?tenant_subtree=true
Authorization: Bearer <token>
```

**PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "list" },
  "resource": { "type": "gts.x.tasks.task.v1~" },
  "context": {
    "tenant_context": {
      "mode": "subtree",
      "root_id": "T1",
      "barrier_mode": "all"
    },
    "require_constraints": true,
    "capabilities": ["tenant_hierarchy"],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**
```json
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            "type": "in_tenant_subtree",
            "resource_property": "owner_tenant_id",
            "root_tenant_id": "T1",
            "barrier_mode": "all"
          }
        ]
      }
    ]
  }
}
```

**SQL:**
```sql
SELECT * FROM tasks
WHERE owner_tenant_id IN (
  SELECT descendant_id FROM tenant_closure
  WHERE ancestor_id = 'T1'
    AND barrier = 0
)
```

---

#### S02: GET, tenant subtree, PEP has tenant_closure

`GET /tasks/{id}?tenant_subtree=true`

User requests a specific task; PEP enforces tenant subtree access at query level.

**Request:**
```http
GET /tasks/task-456?tenant_subtree=true
Authorization: Bearer <token>
```

**PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "read" },
  "resource": {
    "type": "gts.x.tasks.task.v1~",
    "id": "task-456"
  },
  "context": {
    "tenant_context": {
      "mode": "subtree",
      "root_id": "T1"
    },
    "require_constraints": true,
    "capabilities": ["tenant_hierarchy"],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**
```json
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            "type": "in_tenant_subtree",
            "resource_property": "owner_tenant_id",
            "root_tenant_id": "T1"
          }
        ]
      }
    ]
  }
}
```

**SQL:**
```sql
SELECT * FROM tasks
WHERE id = 'task-456'
  AND owner_tenant_id IN (
    SELECT descendant_id FROM tenant_closure
    WHERE ancestor_id = 'T1'
      AND barrier = 0  -- barrier_mode defaults to "all"
  )
```

**Result interpretation:**
- 1 row → return task
- 0 rows → **404 Not Found** (hides resource existence from unauthorized users)

---

#### S03: UPDATE, tenant subtree, PEP has tenant_closure

`PUT /tasks/{id}?tenant_subtree=true`

User updates a task; constraint ensures atomic authorization check.

**Request:**
```http
PUT /tasks/task-456?tenant_subtree=true
Authorization: Bearer <token>
Content-Type: application/json

{"status": "completed"}
```

**PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "update" },
  "resource": {
    "type": "gts.x.tasks.task.v1~",
    "id": "task-456"
  },
  "context": {
    "tenant_context": {
      "mode": "subtree",
      "root_id": "T1"
    },
    "require_constraints": true,
    "capabilities": ["tenant_hierarchy"],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**
```json
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            "type": "in_tenant_subtree",
            "resource_property": "owner_tenant_id",
            "root_tenant_id": "T1"
          }
        ]
      }
    ]
  }
}
```

**SQL:**
```sql
UPDATE tasks
SET status = 'completed'
WHERE id = 'task-456'
  AND owner_tenant_id IN (
    SELECT descendant_id FROM tenant_closure
    WHERE ancestor_id = 'T1'
      AND barrier = 0  -- barrier_mode defaults to "all"
  )
```

**Result interpretation:**
- 1 row affected → success
- 0 rows affected → **404 Not Found** (task doesn't exist or no access)

---

#### S04: CREATE, tenant context

`POST /tasks`

User creates a new task. No constraints needed — PDP just validates permission.

**Request:**
```http
POST /tasks
Authorization: Bearer <token>
Content-Type: application/json

{"title": "New Task", "owner_tenant_id": "T2"}
```

**PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "create" },
  "resource": {
    "type": "gts.x.tasks.task.v1~",
    "properties": {
      "owner_tenant_id": "T2"
    }
  },
  "context": {
    "tenant_context": {
      "mode": "root_only",
      "root_id": "T2"
    },
    "require_constraints": false,
    "capabilities": ["tenant_hierarchy"],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**
```json
{
  "decision": true
}
```

**SQL:**
```sql
INSERT INTO tasks (id, owner_tenant_id, title, status)
VALUES ('task-new', 'T2', 'New Task', 'pending')
```

**Note:** No constraints returned — `require_constraints: false` allows PEP to trust the decision for CREATE operations.

---

#### S05: LIST, billing data, ignore barriers (barrier_mode: "none")

`GET /billing/usage?tenant_subtree=true&barrier_mode=none`

Billing service needs usage data from all tenants in subtree, including self-managed tenants (barriers ignored). This is a cross-barrier operation for administrative purposes.

**Tenant hierarchy:**
```
T1 (parent)
├── T2 (self_managed=true)  ← barrier (ignored for billing)
│   └── T3
└── T4
```

**Request:**
```http
GET /billing/usage?tenant_subtree=true&barrier_mode=none
Authorization: Bearer <token>
```

**PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "list" },
  "resource": { "type": "gts.x.billing.usage.v1~" },
  "context": {
    "tenant_context": {
      "mode": "subtree",
      "root_id": "T1",
      "barrier_mode": "none"
    },
    "require_constraints": true,
    "capabilities": ["tenant_hierarchy"],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**
```json
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            "type": "in_tenant_subtree",
            "resource_property": "owner_tenant_id",
            "root_tenant_id": "T1",
            "barrier_mode": "none"
          }
        ]
      }
    ]
  }
}
```

**SQL:**
```sql
SELECT * FROM billing_usage
WHERE owner_tenant_id IN (
  SELECT descendant_id FROM tenant_closure
  WHERE ancestor_id = 'T1'
  -- no barrier clause because barrier_mode = "none"
)
```

**Result:** Returns usage data from T1, T2, T3, and T4. Barriers are ignored for billing operations.

**tenant_closure data example:**

| ancestor_id | descendant_id | barrier |
|-------------|---------------|---------|
| T1 | T1 | 0 |
| T1 | T2 | 1 |
| T1 | T3 | 1 |
| T1 | T4 | 0 |
| T2 | T2 | 0 |
| T2 | T3 | 0 |

When querying from T1 with `barrier_mode=all`, only rows where `barrier = 0` match → T1, T4.

**Key insight:** T2 → T2 and T2 → T3 have `barrier = 0` because barriers are tracked **strictly between** ancestor and descendant, not including the ancestor itself. When T2 is the query root, its self_managed status doesn't block access to its own subtree.

---

### Without `tenant_closure`

PEP has no tenant_closure table → PDP returns explicit IDs or PEP prefetches attributes.

---

#### S06: LIST, tenant subtree, PEP without tenant_closure

`GET /tasks?tenant_subtree=true`

PEP doesn't have tenant_closure. PDP resolves the subtree and returns explicit tenant IDs.

**Request:**
```http
GET /tasks?tenant_subtree=true
Authorization: Bearer <token>
```

**PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "list" },
  "resource": { "type": "gts.x.tasks.task.v1~" },
  "context": {
    "tenant_context": {
      "mode": "subtree",
      "root_id": "T1"
    },
    "require_constraints": true,
    "capabilities": [],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**

PDP resolves the subtree internally and returns explicit IDs:

```json
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            "type": "in",
            "resource_property": "owner_tenant_id",
            "values": ["T1", "T2", "T3"]
          }
        ]
      }
    ]
  }
}
```

**SQL:**
```sql
SELECT * FROM tasks
WHERE owner_tenant_id IN ('T1', 'T2', 'T3')
```

**Trade-off:** PDP must know the tenant hierarchy and resolve it. Works well for small tenant counts; may not scale for thousands of tenants.

---

#### S07: GET, tenant subtree, PEP without tenant_closure

`GET /tasks/{id}?tenant_subtree=true`

PEP doesn't have tenant_closure. For read-only operations, PEP fetches the resource first, then asks PDP for a decision based on resource attributes. No second query needed — TOCTOU is not a concern for reads.

**Request:**
```http
GET /tasks/task-456?tenant_subtree=true
Authorization: Bearer <token>
```

**Step 1 — PEP fetches resource:**
```sql
SELECT * FROM tasks WHERE id = 'task-456'
```
Result: full task record with `owner_tenant_id = 'T2'`

**Step 2 — PEP → PDP Request (with resource properties):**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "read" },
  "resource": {
    "type": "gts.x.tasks.task.v1~",
    "id": "task-456",
    "properties": {
      "owner_tenant_id": "T2"
    }
  },
  "context": {
    "tenant_context": {
      "mode": "subtree",
      "root_id": "T1"
    },
    "require_constraints": false,
    "capabilities": [],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**

PDP validates that T2 is in T1's subtree and returns decision only (no constraints needed for read):

```json
{
  "decision": true
}
```

**Step 3 — Return result:**
- `decision: true` → return already-fetched task
- `decision: false` → **404 Not Found** (hides resource existence)
- Resource not found in Step 1 → **404 Not Found**

**Why no TOCTOU concern:** For GET, the "use" is returning data to the client. Even if `owner_tenant_id` changed between check and response, no security violation occurs — the client either gets data they had access to at query time, or gets 404. For mutations (UPDATE/DELETE), see S08.

---

#### S08: UPDATE, tenant subtree, PEP without tenant_closure (prefetch)

`PUT /tasks/{id}?tenant_subtree=true`

Unlike S07 (GET), mutations require TOCTOU protection. PEP prefetches `owner_tenant_id`, gets `eq` constraint from PDP, and applies it in UPDATE's WHERE clause. This ensures atomic check-and-modify.

**Request:**
```http
PUT /tasks/task-456?tenant_subtree=true
Authorization: Bearer <token>
Content-Type: application/json

{"status": "completed"}
```

**Step 1 — PEP prefetches:**
```sql
SELECT owner_tenant_id FROM tasks WHERE id = 'task-456'
```
Result: `owner_tenant_id = 'T2'`

**Step 2 — PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "update" },
  "resource": {
    "type": "gts.x.tasks.task.v1~",
    "id": "task-456",
    "properties": {
      "owner_tenant_id": "T2"
    }
  },
  "context": {
    "tenant_context": {
      "mode": "subtree",
      "root_id": "T1"
    },
    "require_constraints": true,
    "capabilities": [],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**
```json
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            "type": "eq",
            "resource_property": "owner_tenant_id",
            "value": "T2"
          }
        ]
      }
    ]
  }
}
```

**Step 3 — SQL with constraint:**
```sql
UPDATE tasks
SET status = 'completed'
WHERE id = 'task-456'
  AND owner_tenant_id = 'T2'
```

**TOCTOU protection:** If another request changed `owner_tenant_id` between prefetch and UPDATE, the WHERE clause won't match → 0 rows affected → **404**. This prevents unauthorized modification even in a race condition.

---

#### S09: GET, context tenant only (no subtree)

`GET /tasks/{id}`

Simplest case — access limited to context tenant only, no subtree traversal. User can only access resources directly owned by their tenant.

**Request:**
```http
GET /tasks/task-456
Authorization: Bearer <token>
```

**PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "read" },
  "resource": {
    "type": "gts.x.tasks.task.v1~",
    "id": "task-456"
  },
  "context": {
    "tenant_context": {
      "mode": "root_only",
      "root_id": "T1"
    },
    "require_constraints": true,
    "capabilities": [],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**
```json
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            "type": "eq",
            "resource_property": "owner_tenant_id",
            "value": "T1"
          }
        ]
      }
    ]
  }
}
```

**SQL:**
```sql
SELECT * FROM tasks
WHERE id = 'task-456'
  AND owner_tenant_id = 'T1'
```

**Note:** No prefetch needed, no closure table required. PDP returns direct `eq` constraint based on context tenant. This pattern applies when the endpoint operates within a single tenant context, regardless of whether the overall tenant model is hierarchical.

---

### Resource Groups

> **Note:** Resource groups are tenant-scoped. **PDP guarantees** that any `group_ids` or `root_group_id` returned in constraints belong to the request context tenant. PEP trusts this guarantee — it has no group metadata to validate against (only `resource_group_membership` table).
>
> All group-based constraints also include a tenant predicate on the resource (typically `eq` on `owner_tenant_id`) as defense in depth, ensuring tenant isolation at the resource level.

---

#### S10: LIST, group membership, PEP has resource_group_membership

`GET /tasks`

User has access to specific projects (flat group membership, no hierarchy).

**Request:**
```http
GET /tasks
Authorization: Bearer <token>
```

**PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "list" },
  "resource": { "type": "gts.x.tasks.task.v1~" },
  "context": {
    "tenant_context": {
      "mode": "root_only",
      "root_id": "T1"
    },
    "require_constraints": true,
    "capabilities": ["group_membership"],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**

Tenant constraint is always included — groups don't bypass tenant isolation:

```json
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            "type": "eq",
            "resource_property": "owner_tenant_id",
            "value": "T1"
          },
          {
            "type": "in_group",
            "resource_property": "id",
            "group_ids": ["ProjectA", "ProjectB"]
          }
        ]
      }
    ]
  }
}
```

**SQL:**
```sql
SELECT * FROM tasks
WHERE owner_tenant_id = 'T1'
  AND id IN (
    SELECT resource_id FROM resource_group_membership
    WHERE group_id IN ('ProjectA', 'ProjectB')
  )
```

---

#### S11: LIST, group subtree, PEP has resource_group_closure

`GET /tasks`

User has access to a project folder and all its subfolders.

**Request:**
```http
GET /tasks
Authorization: Bearer <token>
```

**PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "list" },
  "resource": { "type": "gts.x.tasks.task.v1~" },
  "context": {
    "tenant_context": {
      "mode": "root_only",
      "root_id": "T1"
    },
    "require_constraints": true,
    "capabilities": ["group_hierarchy"],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**

Tenant constraint is always included:

```json
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            "type": "eq",
            "resource_property": "owner_tenant_id",
            "value": "T1"
          },
          {
            "type": "in_group_subtree",
            "resource_property": "id",
            "root_group_id": "FolderA"
          }
        ]
      }
    ]
  }
}
```

**SQL:**
```sql
SELECT * FROM tasks
WHERE owner_tenant_id = 'T1'
  AND id IN (
    SELECT resource_id FROM resource_group_membership
    WHERE group_id IN (
      SELECT descendant_id FROM resource_group_closure
      WHERE ancestor_id = 'FolderA'
  )
)
```

---

#### S12: UPDATE, group membership, PEP has resource_group_membership

`PUT /tasks/{id}`

User updates a task; PEP has resource_group_membership table. Similar to tenant-based S03, but filtering by group membership.

**Request:**
```http
PUT /tasks/task-456
Authorization: Bearer <token>
Content-Type: application/json

{"status": "completed"}
```

**PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "update" },
  "resource": {
    "type": "gts.x.tasks.task.v1~",
    "id": "task-456"
  },
  "context": {
    "tenant_context": {
      "mode": "root_only",
      "root_id": "T1"
    },
    "require_constraints": true,
    "capabilities": ["group_membership"],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**

Tenant constraint is always included:

```json
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            "type": "eq",
            "resource_property": "owner_tenant_id",
            "value": "T1"
          },
          {
            "type": "in_group",
            "resource_property": "id",
            "group_ids": ["ProjectA", "ProjectB"]
          }
        ]
      }
    ]
  }
}
```

**SQL:**
```sql
UPDATE tasks
SET status = 'completed'
WHERE id = 'task-456'
  AND owner_tenant_id = 'T1'
  AND id IN (
    SELECT resource_id FROM resource_group_membership
    WHERE group_id IN ('ProjectA', 'ProjectB')
  )
```

**Result interpretation:**
- 1 row affected → success
- 0 rows affected → task doesn't exist or not in user's accessible groups → **404**

---

#### S13: UPDATE, group subtree, PEP has resource_group_closure

`PUT /tasks/{id}`

User updates a task; PEP has both resource_group_membership and resource_group_closure tables.

**Request:**
```http
PUT /tasks/task-456
Authorization: Bearer <token>
Content-Type: application/json

{"status": "completed"}
```

**PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "update" },
  "resource": {
    "type": "gts.x.tasks.task.v1~",
    "id": "task-456"
  },
  "context": {
    "tenant_context": {
      "mode": "root_only",
      "root_id": "T1"
    },
    "require_constraints": true,
    "capabilities": ["group_hierarchy"],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**

Tenant constraint is always included:

```json
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            "type": "eq",
            "resource_property": "owner_tenant_id",
            "value": "T1"
          },
          {
            "type": "in_group_subtree",
            "resource_property": "id",
            "root_group_id": "FolderA"
          }
        ]
      }
    ]
  }
}
```

**SQL:**
```sql
UPDATE tasks
SET status = 'completed'
WHERE id = 'task-456'
  AND owner_tenant_id = 'T1'
  AND id IN (
    SELECT resource_id FROM resource_group_membership
    WHERE group_id IN (
      SELECT descendant_id FROM resource_group_closure
      WHERE ancestor_id = 'FolderA'
    )
  )
```

---

#### S14: GET, group membership, PEP without resource_group_membership

`GET /tasks/{id}`

PEP doesn't have resource_group_membership table. PDP resolves group membership internally and returns a tenant constraint for defense in depth.

**Request:**
```http
GET /tasks/task-456
Authorization: Bearer <token>
```

**Step 1 — PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "read" },
  "resource": {
    "type": "gts.x.tasks.task.v1~",
    "id": "task-456"
  },
  "context": {
    "tenant_context": {
      "mode": "root_only",
      "root_id": "T1"
    },
    "require_constraints": true,
    "capabilities": [],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP internally:**
1. Resolves resource's group membership (via PIP or own storage)
2. Checks if subject has access to any of those groups
3. Validates tenant access

**PDP → PEP Response:**

PDP returns tenant constraint as defense in depth (group check already done):

```json
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            "type": "eq",
            "resource_property": "owner_tenant_id",
            "value": "T1"
          }
        ]
      }
    ]
  }
}
```

**Step 2 — SQL with constraint:**
```sql
SELECT * FROM tasks
WHERE id = 'task-456'
  AND owner_tenant_id = 'T1'
```

**Result interpretation:**
- 1 row → return task
- 0 rows → **404 Not Found**

**Note:** This pattern requires PDP to have access to group membership data. For LIST operations without resource_group_membership on PEP side, PDP would need to return explicit resource IDs (impractical for large datasets). This scenario works best for point operations (GET, UPDATE, DELETE by ID).

---

#### S15: LIST, group subtree, PEP has membership but no closure

`GET /tasks`

PEP has resource_group_membership but not resource_group_closure. PDP expands group hierarchy to explicit group IDs.

**Request:**
```http
GET /tasks
Authorization: Bearer <token>
```

**PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "list" },
  "resource": { "type": "gts.x.tasks.task.v1~" },
  "context": {
    "tenant_context": {
      "mode": "root_only",
      "root_id": "T1"
    },
    "require_constraints": true,
    "capabilities": ["group_membership"],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**Note:** PEP declares `group_membership` capability (has the membership table) but NOT `group_hierarchy` (no closure table).

**PDP → PEP Response:**

PDP knows user has access to FolderA and its subfolders. Since PEP can't handle `in_group_subtree`, PDP expands the hierarchy to explicit group IDs. Tenant constraint is always included:

```json
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            "type": "eq",
            "resource_property": "owner_tenant_id",
            "value": "T1"
          },
          {
            "type": "in_group",
            "resource_property": "id",
            "group_ids": ["FolderA", "FolderA-Sub1", "FolderA-Sub2", "FolderA-Sub1-Deep"]
          }
        ]
      }
    ]
  }
}
```

**SQL:**
```sql
SELECT * FROM tasks
WHERE owner_tenant_id = 'T1'
  AND id IN (
    SELECT resource_id FROM resource_group_membership
    WHERE group_id IN ('FolderA', 'FolderA-Sub1', 'FolderA-Sub2', 'FolderA-Sub1-Deep')
  )
```

**Trade-off:** PDP must know the group hierarchy and expand it. Works well for shallow hierarchies or small group counts; may not scale for deep/wide hierarchies with thousands of groups.

---

### Advanced Patterns

---

#### S16: LIST, tenant subtree and group membership (AND)

`GET /tasks?tenant_subtree=true`

User has access to tasks in their tenant subtree AND in specific projects. Both conditions must be satisfied.

**Request:**
```http
GET /tasks?tenant_subtree=true
Authorization: Bearer <token>
```

**PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "list" },
  "resource": { "type": "gts.x.tasks.task.v1~" },
  "context": {
    "tenant_context": {
      "mode": "subtree",
      "root_id": "T1"
    },
    "require_constraints": true,
    "capabilities": ["tenant_hierarchy", "group_membership"],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**

Single constraint with multiple predicates (AND semantics):

```json
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            "type": "in_tenant_subtree",
            "resource_property": "owner_tenant_id",
            "root_tenant_id": "T1"
          },
          {
            "type": "in_group",
            "resource_property": "id",
            "group_ids": ["ProjectA"]
          }
        ]
      }
    ]
  }
}
```

**SQL:**
```sql
SELECT * FROM tasks
WHERE owner_tenant_id IN (
    SELECT descendant_id FROM tenant_closure
    WHERE ancestor_id = 'T1'
      AND barrier = 0  -- barrier_mode defaults to "all"
  )
  AND id IN (
    SELECT resource_id FROM resource_group_membership
    WHERE group_id = 'ProjectA'
  )
```

---

#### S17: LIST, tenant subtree and group subtree

`GET /tasks?tenant_subtree=true`

User has access to tasks that are owned by tenants in their subtree AND belong to a folder or any of its subfolders. This scenario demonstrates the most complex constraint combination using all three projection tables.

**Use case:** Manager can see tasks from their department (tenant subtree) that are in the "Q1 Projects" folder or any nested subfolder.

**Request:**
```http
GET /tasks?tenant_subtree=true
Authorization: Bearer <token>
```

**PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "list" },
  "resource": { "type": "gts.x.tasks.task.v1~" },
  "context": {
    "tenant_context": {
      "mode": "subtree",
      "root_id": "T1"
    },
    "require_constraints": true,
    "capabilities": ["tenant_hierarchy", "group_hierarchy"],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**

Single constraint with two predicates (AND semantics):

```json
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            "type": "in_tenant_subtree",
            "resource_property": "owner_tenant_id",
            "root_tenant_id": "T1"
          },
          {
            "type": "in_group_subtree",
            "resource_property": "id",
            "root_group_id": "FolderA"
          }
        ]
      }
    ]
  }
}
```

**SQL:**
```sql
SELECT * FROM tasks
WHERE owner_tenant_id IN (
    SELECT descendant_id FROM tenant_closure
    WHERE ancestor_id = 'T1'
      AND barrier = 0
  )
  AND id IN (
    SELECT resource_id FROM resource_group_membership
    WHERE group_id IN (
      SELECT descendant_id FROM resource_group_closure
      WHERE ancestor_id = 'FolderA'
    )
  )
```

**Projection tables used:**
- `tenant_closure` — resolves tenant subtree (T1 and all descendants)
- `resource_group_closure` — resolves folder hierarchy (FolderA and all subfolders)
- `resource_group_membership` — maps resources to groups

**Note:** This is the most demanding query pattern. For large datasets, ensure proper indexing on all three projection tables and consider the scalability considerations in [AUTH.md Open Questions](./AUTH.md#open-questions).

---

#### S18: LIST, multiple access paths (OR)

`GET /tasks`

User has multiple ways to access tasks: (1) via project membership, (2) via explicitly shared tasks.

**Request:**
```http
GET /tasks
Authorization: Bearer <token>
```

**PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "list" },
  "resource": { "type": "gts.x.tasks.task.v1~" },
  "context": {
    "tenant_context": {
      "mode": "root_only",
      "root_id": "T1"
    },
    "require_constraints": true,
    "capabilities": ["group_membership"],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**

Multiple constraints (OR semantics). Tenant constraint is included in each path:

```json
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            "type": "eq",
            "resource_property": "owner_tenant_id",
            "value": "T1"
          },
          {
            "type": "in_group",
            "resource_property": "id",
            "group_ids": ["ProjectA"]
          }
        ]
      },
      {
        "predicates": [
          {
            "type": "eq",
            "resource_property": "owner_tenant_id",
            "value": "T1"
          },
          {
            "type": "in",
            "resource_property": "id",
            "values": ["task-shared-1", "task-shared-2"]
          }
        ]
      }
    ]
  }
}
```

**SQL:**
```sql
SELECT * FROM tasks
WHERE (
    owner_tenant_id = 'T1'
    AND id IN (
      SELECT resource_id FROM resource_group_membership
      WHERE group_id = 'ProjectA'
    )
  )
  OR (
    owner_tenant_id = 'T1'
    AND id IN ('task-shared-1', 'task-shared-2')
  )
```

---

#### S19: Access denied

`GET /tasks`

User doesn't have permission to access the requested resources.

**Request:**
```http
GET /tasks
Authorization: Bearer <token>
```

**PEP → PDP Request:**
```json
{
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "user-123",
    "properties": { "tenant_id": "T1" }
  },
  "action": { "name": "list" },
  "resource": { "type": "gts.x.tasks.task.v1~" },
  "context": {
    "tenant_context": {
      "mode": "root_only",
      "root_id": "T1"
    },
    "require_constraints": true,
    "capabilities": ["tenant_hierarchy"],
    "supported_properties": ["owner_tenant_id", "id"]
  }
}
```

**PDP → PEP Response:**
```json
{
  "decision": false,
  "context": {
    "deny_reason": {
      "error_code": "gts.x.core.errors.err.v1~x.authz.errors.insufficient_permissions.v1",
      "details": "Subject 'user-123' lacks 'list' permission on 'gts.x.tasks.task.v1~' in tenant 'T1'"
    }
  }
}
```

**PEP Action:**
- No SQL query is executed
- Use `error_code` for programmatic handling (e.g., metrics, error categorization)
- Log `deny_reason` for audit/debugging (includes `error_code` and `details`)
- Return **403 Forbidden** to client without exposing `details`

**Fail-closed principle:** The PEP never executes a database query when `decision: false`. This prevents any data leakage and ensures authorization is enforced before resource access.

**Note on deny_reason:** The `deny_reason` is required when `decision: false`. PEP uses `error_code` for programmatic handling and logs `details` for troubleshooting, but returns a generic 403 response to prevent leaking authorization policy details to clients.

---

## TOCTOU Analysis

[Time-of-check to time-of-use (TOCTOU)](https://en.wikipedia.org/wiki/Time-of-check_to_time-of-use) is a class of race condition where a security check is performed at one point in time, but the protected action occurs later when conditions may have changed.

### When TOCTOU Matters

TOCTOU is a security concern only for **mutations** (UPDATE, DELETE). For **reads** (GET, LIST), there's no security violation if the resource changes between check and response — the client receives data they had access to at query time.

| Operation | TOCTOU Concern | Why |
|-----------|----------------|-----|
| GET | ❌ No | Read returns point-in-time snapshot; no state change |
| LIST | ❌ No | Same as GET — read-only |
| UPDATE | ✅ Yes | Must ensure authorization at mutation time |
| DELETE | ✅ Yes | Must ensure authorization at mutation time |
| CREATE | ❌ No | No existing resource to race against |

### How Each Scenario Handles TOCTOU

**Tenant-based scenarios:**

| Scenario | Operation | Closure | Constraint | TOCTOU Protection |
|----------|-----------|---------|------------|-------------------|
| S01-S03, S05 | LIST/GET/UPDATE | ✅ | `in_tenant_subtree` | ✅ Atomic SQL check |
| S07 | GET | ❌ | decision only | N/A (read-only) |
| S08 | UPDATE | ❌ | `eq` (prefetched) | ✅ Atomic SQL check |
| S04 | CREATE | N/A | None | N/A (no existing resource) |

**Resource group scenarios:**

| Scenario | Operation | Projection Tables | Constraint | TOCTOU Protection |
|----------|-----------|-------------------|------------|-------------------|
| S10, S11 | LIST | ✅ | `in_group` / `in_group_subtree` | ✅ Atomic SQL check |
| S12, S13 | UPDATE | ✅ | `in_group` / `in_group_subtree` | ✅ Atomic SQL check |
| S14 | GET | ❌ | `eq` (tenant) | N/A (read-only) |
| S15 | LIST | membership only | `in_group` (expanded) | ✅ Atomic SQL check |

### Key Insight: Prefetch + Constraint for Mutations

Without closure tables, mutations (UPDATE/DELETE) use a two-step pattern:

1. **Prefetch:** PEP reads `owner_tenant_id = 'T2'` from database
2. **PDP check:** PDP validates T2 is accessible, returns `eq: owner_tenant_id = 'T2'`
3. **SQL execution:** `UPDATE tasks SET ... WHERE id = 'X' AND owner_tenant_id = 'T2'`
4. **If tenant changed:** WHERE clause won't match → 0 rows affected → 404

The constraint acts as a [compare-and-swap](https://en.wikipedia.org/wiki/Compare-and-swap) mechanism — if the value changed between check and use, the operation atomically fails.

**For reads (S07):** No second query needed. PEP fetches the resource, asks PDP for decision, and returns the already-fetched data if allowed.

---

## References

- [AUTH.md](./AUTH.md) — Core authorization design
- [TENANT_MODEL.md](./TENANT_MODEL.md) — Tenant topology, barriers, closure tables
- [RESOURCE_GROUP_MODEL.md](./RESOURCE_GROUP_MODEL.md) — Resource group topology, membership, hierarchy
- [TOCTOU - Wikipedia](https://en.wikipedia.org/wiki/Time-of-check_to_time-of-use)
- [Race Conditions - PortSwigger](https://portswigger.net/web-security/race-conditions)
- [AWS Multi-tenant Authorization](https://docs.aws.amazon.com/prescriptive-guidance/latest/saas-multitenant-api-access-authorization/introduction.html)
