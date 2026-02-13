---
status: accepted
date: 2026-01-22
decision-makers: Cyber Fabric Architects Committee
---

# Adopt AuthZEN-based PDP/PEP Authorization Model

## Context and Problem Statement

Cyber Fabric is a modular platform for building multi-tenant vendor platforms. Each vendor has its own identity provider (IdP), authorization model, tenant service, and policy manager. Cyber Fabric must integrate with these vendor-specific systems without assuming a particular policy model (RBAC/ABAC/ReBAC).

**Key requirements:**

1. **Performance at scale** — Authorization must be efficient for all operations, including mass-management scenarios in multi-tenant environments (bulk updates, cross-tenant queries, hierarchical access). Access check complexity varies by vendor and should not bottleneck Cyber Fabric.

2. **Simplicity for module developers** — Authorization enforcement must be hard to get wrong. Authorization logic in modules should be minimal — ideally just "ask PDP, apply response". Complex policy evaluation belongs in vendor's PDP, not in Cyber Fabric code (even shared libraries are costly to update across deployments).

3. **Seamless vendor integration** — Cyber Fabric must integrate into vendor's existing infrastructure without requiring significant changes on their side:
   - **No resource sync** — Resources stay in Cyber Fabric's DB; vendors don't need to replicate millions of resources or all their relationships to their authorization service
   - **No policy format requirements** — Vendors keep their existing policy storage (RBAC tables, ReBAC tuples, custom DSL); Cyber Fabric only defines the response contract
   - **Leverage existing infrastructure** — Works with vendor's IdP, tenant service, and policy manager as-is

### PDP/PEP Architecture Model

Industry best practices (NIST SP 800-162, XACML, AuthZEN) recommend separating authorization into:

- **PDP (Policy Decision Point)** — Evaluates policies and returns access decisions. In Cyber Fabric, this is the vendor's authorization service accessed via AuthZ Resolver module.
- **PEP (Policy Enforcement Point)** — Enforces PDP decisions at resource access points. In Cyber Fabric, domain modules act as PEPs, with ModKit providing shared enforcement infrastructure.
- **PAP (Policy Administration Point)** — Where policies are authored and managed. This is entirely vendor-controlled (their admin UI, policy DSL, etc.). Cyber Fabric never sees or stores policies.
- **PIP (Policy Information Point)** — Provides additional attributes for decision-making (user roles, tenant hierarchy, resource metadata). In Cyber Fabric, Tenant Resolver and Resource Group Resolver serve as PIPs.

Benefits of PDP/PEP separation:

- Centralized policy management and auditability
- Consistent enforcement across all modules
- Separation of concerns (business logic vs authorization logic)
- Easier security audits and compliance

Cyber Fabric modules act as PEPs; AuthZ Resolver integrates with vendor's PDP; Tenant/RG Resolvers act as PIPs.

## Decision Drivers

- **Performance** — O(1) authorization overhead per query, not O(N) per resource
- **Simplicity** — Module developers use shared ModKit library, not manual authorization code
- **Vendor integration** — No resource sync, no policy format requirements, leverage existing infrastructure
- **Vendor-neutral** — No assumption about policy model (RBAC/ABAC/ReBAC)
- **Standards-based** — Build on industry standards where possible
- **Query-time enforcement** — Authorization as SQL WHERE constraints, not post-fetch filtering

## Considered Options

- **Option A**: Module-level authorization (PEP = PDP)
- **Option B**: Google Zanzibar / ReBAC
- **Option C**: OpenID AuthZEN 1.0 (as-is)
- **Option D**: OPA Partial Evaluation
- **Option E**: OpenID AuthZEN 1.0 + Constraint Extensions

## Decision Outcome

Chosen option: **Option E - AuthZEN 1.0 + Constraint Extensions**, because it provides a standards-based foundation with targeted extensions for SQL-level constraint enforcement, maintaining vendor neutrality while solving the core architectural challenge of PDP not having access to resources.

Implementation:

- Use AuthZEN standard request/response structure
- Extend evaluation response with optional `context.constraints`
- Constraints contain typed filters (field, group_membership) with known operators (eq, in, in_closure)
- PEP compiles filters directly to SQL without intermediate translation
- Endpoints: `/access/v1/evaluation`, `/access/v1/evaluations`

### Consequences

**Positive:**

- Standards-based foundation (AuthZEN 1.0)
- Single endpoint for all operations (LIST, GET, UPDATE, DELETE)
- Vendor-neutral: no assumption about policy model
- SQL-first: constraints designed for efficient database enforcement
- Fail-closed: structural guarantees against authorization bypass

**Negative:**

- Non-standard extension requires documentation and tooling
- PDP complexity: must understand both AuthZEN and constraint extensions
- May not be compatible with off-the-shelf AuthZEN PDPs

**Risks:**

- AuthZEN may add constraint-like features in future versions (alignment opportunity or divergence risk)

## Pros and Cons of the Options

### Option A: Module-level Authorization (PEP = PDP)

Each module implements its own authorization logic: extracts permissions/roles from token, calls PIPs (Tenant Resolver, Resource Group Resolver, vendor's Policy Manager API) as needed, and makes access decisions internally.

- Good, because no single point of failure — modules are self-contained
- Good, because flexible — each module can implement exactly the logic it needs
- Bad, because **violates PDP/PEP separation** recommended by NIST SP 800-162
- Bad, because authorization logic scattered across modules — hard to audit, easy to make mistakes
- Bad, because each module must understand and correctly implement policy evaluation
- Bad, because inconsistent enforcement across modules, difficult compliance audits
- Bad, because complex authorization logic lives in Cyber Fabric (even if in shared library — bugs, versioning, update rollout across deployments)

### Option B: Google Zanzibar / ReBAC

Relationship-based access control (SpiceDB, Authzed, etc.). Check tuples: `user:alice` -> `viewer` -> `document:123`

- Good, because powerful relationship modeling, proven at scale (Google)
- Bad, because two approaches, both problematic:
  - (1) Double SELECT: fetch resources from DB -> check each in Zanzibar -> filter results
  - (2) Store all relationships in Zanzibar: strong vendor lock-in, must sync all resource metadata
- Bad, because LIST operations require Lookup API which returns IDs (not constraints) - O(N) checks
- Bad, because filtering/pagination becomes painful: fetch all, check each, then paginate
- Bad, because vendor lock-in to ReBAC model if using approach (2)

### Option C: OpenID AuthZEN 1.0 (as-is)

[OpenID AuthZEN Authorization API 1.0](https://openid.net/specs/authorization-api-1_0.html), approved by OpenID Foundation on January 12, 2026. Standard Access Evaluation API: subject + action + resource + context -> decision. Resource Search API for LIST operations.

- Good, because industry standard, vendor-neutral, growing ecosystem
- Bad, because PDP doesn't know resource metadata (owner_tenant_id, properties)
- Bad, because must either (1) sync resources to PDP or (2) SELECT all -> evaluate each -> filter
- Bad, because cannot return constraints for SQL-level enforcement - O(N) evaluations for LIST
- Bad, because filtering/pagination becomes painful: fetch all, evaluate each, then paginate

### Option D: OPA Partial Evaluation

Open Policy Agent returns residual Rego policies when not all input is available. PEP evaluates residual policy and translates to SQL predicates.

- Good, because powerful, can express complex constraints in Rego
- Good, because conceptually similar to Option E (query-time constraints)
- Bad, because **policies must be written in Rego** - OPA's proprietary policy language:
  - Vendors using RBAC tables, ReBAC tuples, or custom policy formats must transpile to Rego or maintain duplicate policy definitions
  - Creates policy-language lock-in even though OPA itself is open source
  - Violates our "vendor-neutral" decision driver - we would dictate how vendors store policies
- Bad, because Rego learning curve for policy authors who may already have working policies in other formats
- Bad, because not a standard API format, tight coupling to OPA implementation

### Option E: OpenID AuthZEN 1.0 + Constraint Extensions

Extend [AuthZEN 1.0](https://openid.net/specs/authorization-api-1_0.html) (approved January 12, 2026) evaluation response with `context.constraints`. PDP returns typed filters using logical field names (DSL), not physical schema. PEP maps fields to columns and compiles constraints to SQL WHERE clauses.

- Good, because standards-based foundation (AuthZEN 1.0) with targeted extension
- Good, because purpose-built for SQL compilation, simpler PEP implementation
- Good, because **shared ModKit library handles enforcement** — module developers call one method, constraints automatically applied to queries
- Good, because **vendor-neutral at policy storage level** — we define only the response format:
  - Vendors can use any internal policy format (RBAC tables, ReBAC tuples, ABAC rules, custom DSL)
  - PDP translates from vendor's native format to constraints JSON at runtime
  - No policy-language lock-in, vendors keep full control over their PAP
- Good, because similar to Option D conceptually, but with cleaner PDP-PEP contract
- Good, because **proven approach** — similar patterns used by:
  - [Permit.io Data Filtering](https://docs.permit.io/how-to/enforce-permissions/data-filtering/)
  - [Oso Data Filtering](https://www.osohq.com/post/authorization-logic-into-sql)
- Bad, because non-standard extension requires documentation

**Comparison with Option D (OPA Partial Evaluation):**

Both options solve the same problem (query-time constraint enforcement) but differ in approach:

| Aspect | OPA (D) | AuthZEN + Constraints (E) |
|--------|---------|---------------------------|
| Policy format | Must be Rego | Any (vendor's choice) |
| PDP output | Rego AST | SQL-friendly JSON filters |
| PEP complexity | Parse Rego, translate to SQL | Direct SQL compilation |
| Standards | None | AuthZEN 1.0 foundation |

Option E provides a cleaner contract between PDP and PEP with less implementation complexity, while maintaining standards compliance through AuthZEN foundation.

## More Information

**Technical Specification:**

- [`DESIGN.md`](../DESIGN.md)

**Authorization Architecture:**

- NIST SP 800-162 (ABAC Guide): https://csrc.nist.gov/publications/detail/sp/800-162/final

**References for Considered Options:**

- **Option A** (Module-level Authorization):
  - Common pattern, no specific reference — each module implements its own PDP logic
- **Option B** (Google Zanzibar / ReBAC):
  - Google Zanzibar paper: https://research.google/pubs/pub48190/
  - SpiceDB (OSS implementation): https://authzed.com/docs
  - Authzed: https://authzed.com/
- **Option C** (AuthZEN 1.0):
  - OpenID AuthZEN Authorization API 1.0: https://openid.net/specs/authorization-api-1_0.html
- **Option D** (OPA Partial Evaluation):
  - OPA Partial Evaluation: https://www.openpolicyagent.org/docs/latest/philosophy/#partial-evaluation
  - OPA Documentation: https://www.openpolicyagent.org/docs/latest/
- **Option E** (AuthZEN + Constraints - CHOSEN):
  - OpenID AuthZEN 1.0 (base): https://openid.net/specs/authorization-api-1_0.html
  - Cyber Fabric constraint extension: [`DESIGN.md`](../DESIGN.md)

**Prior Art (Data Filtering / Query-time Authorization):**

- Permit.io Data Filtering: https://docs.permit.io/how-to/enforce-permissions/data-filtering/
- Oso Authorization to SQL: https://www.osohq.com/post/authorization-logic-into-sql
- OPA Partial Evaluation: https://www.openpolicyagent.org/docs/latest/philosophy/#partial-evaluation
