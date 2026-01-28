# Authentication & Authorization Design

## Table of Contents

- [Overview](#overview)
  - [PDP/PEP Model](#pdppep-model)
  - [Request Flow](#request-flow)
  - [Auth Resolver: Gateway + Plugin Architecture](#auth-resolver-gateway--plugin-architecture)
  - [Integration Architecture](#integration-architecture)
  - [Deployment Modes and Trust Model](#deployment-modes-and-trust-model)
- [Core Terms](#core-terms)
- [Authentication](#authentication)
  - [Configuration](#configuration)
  - [Token Introspection](#token-introspection)
  - [OpenID Connect Integration](#openid-connect-integration)
  - [Plugin Role](#plugin-role)
  - [Validation](#validation)
  - [Introspection Caching](#introspection-caching)
- [Authorization](#authorization)
  - [Why AuthZEN (and Why It's Not Enough)](#why-authzen-and-why-its-not-enough)
  - [PEP Enforcement](#pep-enforcement)
  - [API Specifications](#api-specifications)
  - [Predicate Types Reference](#predicate-types-reference)
  - [PEP Property Mapping](#pep-property-mapping)
  - [Capabilities -> Predicate Matrix](#capabilities---predicate-matrix)
  - [Table Schemas (Local Projections)](#table-schemas-local-projections)
- [Tenant Model](./TENANT_MODEL.md)
- [Usage Scenarios](./SCENARIOS.md)
- [Open Questions](#open-questions)
- [References](#references)

---

## Overview

This document describes HyperSpot's approach to authentication (AuthN) and authorization (AuthZ).

**Authentication** verifies the identity of the subject making a request. HyperSpot integrates with vendor's Identity Provider (IdP) to validate access tokens and extract subject identity.

**Authorization** determines what the authenticated subject can do. HyperSpot uses the Auth Resolver module (acting as PDP) to obtain access decisions and query-level constraints. The core challenge: HyperSpot modules need to enforce authorization at the **query level** (SQL WHERE clauses), not just perform point-in-time access checks. See [ADR 0001](../adrs/authorization/0001-pdp-pep-authorization-model.md) for the details.

### PDP/PEP Model

This document uses the PDP/PEP authorization model (per NIST SP 800-162):

- **PDP (Policy Decision Point)** — evaluates policies and returns access decisions with constraints
- **PEP (Policy Enforcement Point)** — enforces PDP decisions at resource access points

In HyperSpot's architecture:
- **Auth Resolver** (via vendor-specific plugin) serves as the **PDP**
- **Domain modules** act as **PEPs**, applying constraints to database queries

See [ADR 0001](../adrs/authorization/0001-pdp-pep-authorization-model.md) for the full rationale.

### Request Flow

```mermaid
sequenceDiagram
    participant Client
    participant AuthN as AuthN Layer<br/>(Gateway)
    participant PEP as PEP<br/>(Domain Module)
    participant PDP as PDP<br/>(Auth Resolver)
    participant DB as Database

    Client->>AuthN: Request + Token
    AuthN->>AuthN: Validate token, extract claims
    AuthN->>PEP: Request + SecurityContext
    PEP->>PDP: AuthZ Request<br/>(subject, action, resource, context)
    PDP->>PDP: Evaluate policies
    PDP-->>PEP: decision + constraints
    PEP->>PEP: Compile constraints to SQL
    PEP->>DB: Query with WHERE clauses
    DB-->>PEP: Filtered results
    PEP-->>Client: Response
```

**Separation of concerns:**
1. **AuthN Layer** — validates token, extracts identity → `SecurityContext`
2. **PEP** — builds AuthZ request, compiles constraints to SQL
3. **PDP** — evaluates policies, returns decision + constraints

### Auth Resolver: Gateway + Plugin Architecture

Since IdP and PDP are vendor-specific, HyperSpot cannot implement authentication and authorization directly. Instead, we use the **gateway + plugin** pattern:

- **Auth Resolver** — a HyperSpot gateway module that defines a unified interface for AuthN/AuthZ operations
- **Vendor Plugin** — implements the Auth Resolver interface, integrating with vendor's IdP and Authorization API

This allows HyperSpot domain modules (PEPs) to use a consistent API regardless of the vendor's identity and authorization infrastructure. Each vendor develops their own Auth Resolver plugin that bridges to their specific systems.

### Integration Architecture

```mermaid
flowchart TB
    subgraph Vendor["Vendor Platform"]
        IdP["IdP"]
        TenantSvc["Tenant Service"]
        RGSvc["RG Service"]
        AuthzSvc["Authz Service"]
    end

    subgraph HyperSpot
        subgraph TenantResolver["Tenant Resolver"]
            TenantGW["Gateway"]
            TenantPlugin["Plugin"]
        end

        subgraph RGResolver["RG Resolver"]
            RGGW["Gateway"]
            RGPlugin["Plugin"]
        end

        subgraph AuthResolver["Auth Resolver"]
            AuthGW["Gateway"]
            AuthPlugin["Plugin<br/>(AuthN + PDP)"]
        end

        subgraph PEP["Domain Module (PEP)"]
            Handler["Handler"]
            subgraph ModuleDB["Module Database"]
                DomainTables["Domain Tables<br/>(events, ...)"]
                LocalProj["Local Projections<br/>• tenant_closure<br/>• resource_group_closure<br/>• resource_group_membership"]
            end
        end
    end

    %% Tenant Resolver flow
    TenantGW --> TenantPlugin
    TenantPlugin --> TenantSvc

    %% RG Resolver flow
    RGGW --> RGPlugin
    RGPlugin --> RGSvc

    %% Auth Resolver flow
    AuthGW --> AuthPlugin
    AuthPlugin -->|introspection| IdP
    AuthPlugin -->|authz| AuthzSvc
    AuthPlugin -.->|tenant hierarchy| TenantGW
    AuthPlugin -.->|group hierarchy,<br/>membership| RGGW

    %% PEP flow
    Handler -->|token introspection| AuthGW
    Handler -->|/access/v1/evaluation| AuthGW
    Handler -->|constraints → SQL| ModuleDB
```

**Communication flow:**
1. **Handler → Auth Resolver (Gateway)** — PEP calls for token introspection and authorization
2. **Auth Resolver (Gateway) → Auth Resolver (Plugin)** — gateway delegates to vendor plugin (handles both AuthN and PDP)
3. **Auth Resolver (Plugin) → IdP** — token introspection
4. **Auth Resolver (Plugin) → Authz Svc** — authorization decisions
5. **Auth Resolver (Plugin) → Tenant Resolver (Gateway)** — tenant hierarchy queries
6. **Auth Resolver (Plugin) → RG Resolver (Gateway)** — group hierarchy and membership queries
7. **Tenant/RG Resolver (Gateway) → Plugin → Vendor Service** — gateway delegates to plugin, plugin syncs from vendor

### Deployment Modes and Trust Model

Auth Resolver (PDP) can run in two configurations with different trust models.

#### In-Process (Plugin)

```
┌─────────────────────────────────┐
│       HyperSpot Process         │
│  PEP ───function call───► PDP   │
└─────────────────────────────────┘
```

- PDP runs as a plugin in the same process
- Communication via direct function calls
- **Trust model:** implicit — same process, same memory space
- No additional authentication required

#### Out-of-Process (Separate Service)

```
┌─────────────┐        gRPC/mTLS        ┌───────────────┐
│  HyperSpot  │ ──────────────────────► │ Auth Resolver │
│    (PEP)    │                         │    (PDP)      │
└─────────────┘                         └───────────────┘
```

- PDP runs as a separate process (same machine or remote)
- Communication via gRPC
- **Trust model:** explicit authentication required
- **mTLS required:** PDP authenticates that the caller is a legitimate PEP

#### Trust Boundaries

| Aspect | In-Process | Out-of-Process |
|--------|------------|----------------|
| PEP → PDP AuthN | implicit | mTLS |
| Subject identity | PDP trusts PEP | PDP trusts authenticated PEP |
| Network exposure | none | internal network only |

**Note:** In both modes, PDP trusts subject identity data from PEP. The mTLS in out-of-process mode authenticates *which service* is calling, not the validity of subject claims. Subject identity originates from the AuthN layer (Gateway) and flows through PEP to PDP.

---

## Core Terms

- **Access Token** - Credential presented by the client to authenticate requests. Format is not restricted — can be opaque token (validated via introspection) or self-contained JWT. The key requirement: it must enable authentication and subject identification.
- **Subject / Principal** - Actor initiating the request (user or API client), identified via access token
- **Tenant** - Domain of ownership/responsibility and policy (billing, security, data isolation)
- **Subject Owner Tenant** - Tenant the subject belongs to (owning tenant of the subject)
- **Context Tenant** - Tenant scope root for the operation (may differ from subject owner tenant in cross-tenant scenarios)
- **Resource Owner Tenant** - Actual tenant owning the resource (`owner_tenant_id`)
- **Resource** - Object with owner tenant identifier
- **Resource Group** - Optional container for resources (project/workspace/folder)
- **Permission** - `{ resource_type, action }` - allowed operation identifier
- **Access Constraints** - Structured predicates returned by the PDP for query-time enforcement. NOT policies (which are stored vendor-side), but compiled, time-bound enforcement artifacts.

  **Why "constraints" not "grants":** The term "grant" is overloaded in authorization contexts—OAuth uses it for token acquisition flows (Authorization Code Grant), Zanzibar/ReBAC uses it for static relation tuples stored in the system. Constraints are fundamentally different: they are *computed predicates* returned by PDP at evaluation time, not stored permission facts. The term "constraints" accurately describes their role as query-level restrictions.
- **Security Context** - Result of successful authentication containing subject identity, tenant information, and optionally the original bearer token. Flows from authentication to authorization. Contains: `subject_id`, `subject_type`, `subject_tenant_id`, `bearer_token`.

---

# Authentication

## Overview

HyperSpot integrates with the vendor's Identity Provider (IdP) to authenticate requests. It supports two token formats:

- **JWT (JSON Web Token)** — Self-contained tokens (RFC 7519), can be validated locally via signature verification or via introspection for revocation checking and claim enrichment
- **Opaque tokens** — Tokens validated via Token Introspection endpoint (RFC 7662)

For JWT-based authentication, HyperSpot follows OpenID Connect Core 1.0 standards. Auto-configuration is supported via OpenID Connect Discovery 1.0 (`.well-known/openid-configuration`).

**Token type detection**: JWT tokens are identified by their structure (three base64url-encoded segments separated by dots). All other tokens are treated as opaque.

### Token Validation Modes

| Mode | When | How |
|------|------|-----|
| JWT local | JWT + introspection not required | Validate signature via JWKS, extract claims |
| Introspection | Opaque token OR JWT requiring enrichment/revocation check | Plugin calls `introspection_endpoint` |

### JWT Local Validation

```mermaid
sequenceDiagram
    participant Client
    participant Gateway as API Gateway (HyperSpot)
    participant AuthResolver as Auth Resolver (HyperSpot)
    participant IdP as Vendor's IdP
    participant Module as HyperSpot Module

    Client->>Gateway: Request + Bearer {JWT}
    Gateway->>Gateway: Extract iss from JWT (unverified)
    Gateway->>Gateway: Lookup iss in jwt.trusted_issuers

    alt iss not in jwt.trusted_issuers
        Gateway-->>Client: 401 Untrusted issuer
    end

    alt JWKS not cached or expired (1h)
        Gateway->>AuthResolver: get JWKS(discovery_url)
        AuthResolver->>IdP: GET {discovery_url}/.well-known/openid-configuration
        IdP-->>AuthResolver: { jwks_uri, ... }
        AuthResolver->>IdP: GET {jwks_uri}
        IdP-->>AuthResolver: JWKS
        AuthResolver-->>Gateway: JWKS (cached 1h)
    end

    Gateway->>Gateway: Validate signature (JWKS)
    Gateway->>Gateway: Check exp, aud
    Gateway->>Gateway: Extract claims → SecurityContext
    Gateway->>Module: Request + SecurityContext
    Module-->>Gateway: Response
    Gateway-->>Client: Response
```

### Token Introspection

```mermaid
sequenceDiagram
    participant Client
    participant Gateway as API Gateway (HyperSpot)
    participant AuthResolver as Auth Resolver (HyperSpot)
    participant IdP as Vendor's IdP
    participant Module as HyperSpot Module

    Client->>Gateway: Request + Bearer {token}

    Note over Gateway: Token is opaque OR introspection.mode=always

    Gateway->>AuthResolver: introspect(token)
    AuthResolver->>IdP: POST /introspect { token }
    IdP-->>AuthResolver: { active: true, sub, sub_tenant_id, sub_type, exp, ... }
    AuthResolver->>AuthResolver: Map response → SecurityContext
    AuthResolver-->>Gateway: SecurityContext
    Gateway->>Module: Request + SecurityContext
    Module-->>Gateway: Response
    Gateway-->>Client: Response
```

### AuthN Result: Security Context

Successful authentication produces a `SecurityContext` that flows to authorization:

```rust
SecurityContext {
    subject_id: String,           // from `sub` claim
    subject_type: GtsTypeId,      // vendor-specific subject type (optional)
    subject_tenant_id: TenantId,  // Subject Owner Tenant - tenant the subject belongs to
    bearer_token: Option<String>, // original token for forwarding and PDP validation
}
```

**Field sources by validation mode:**

| Field | JWT Local | Introspection |
|-------|-----------|---------------|
| `subject_id` | `sub` claim | Introspection response `sub` |
| `subject_type` | Custom claim (vendor-defined) | Plugin maps from response |
| `subject_tenant_id` | Custom claim (vendor-defined) | Plugin maps from response |
| `bearer_token` | Original token from `Authorization` header | Original token from `Authorization` header |

**Notes:**
- Token expiration (`exp`) is validated during authentication but not included in SecurityContext. Expiration is token metadata, not identity. The caching layer uses `exp` as upper bound for cache entry TTL.
- **Security:** `bearer_token` is a credential. It MUST NOT be logged, serialized to persistent storage, or included in error messages. Implementations should use opaque wrapper types (e.g., `Secret<String>`) and exclude from `Debug` output. The token is included for two purposes:
  1. **Forwarding** — Auth Resolver plugin may need to call external vendor services that require the original bearer token for authentication
  2. **PDP validation** — In out-of-process deployments, PDP may independently validate the token as defence-in-depth, not trusting the PEP's claim extraction

---

## Configuration

```yaml
auth:
  jwt:
    trusted_issuers:
      "https://accounts.google.com":
        discovery_url: "https://accounts.google.com"
      "my-corp-idp":
        discovery_url: "https://idp.corp.example.com"
    require_audience: true
    expected_audience:
      - "https://*.my-company.com"
      - "https://api.my-company.com"
  jwks:
    cache:
      ttl: 1h
  introspection:
    mode: opaque_only
    endpoint: "https://idp.corp.example.com/oauth2/introspect"
    cache:
      enabled: true
      max_entries: 10000
      ttl: 5m
    endpoint_discovery_cache:
      enabled: true
      max_entries: 10000
      ttl: 1h
```

### JWT Settings

- `auth.jwt.trusted_issuers` — map of issuer identifier to discovery config
  - **Key** — expected `iss` claim value in JWT
  - **`discovery_url`** — base URL for OpenID Discovery (`{value}/.well-known/openid-configuration`)
- `auth.jwt.require_audience` — whether to require `aud` claim validation (default: `false`)
- `auth.jwt.expected_audience` — list of glob patterns for valid audiences (e.g., `https://*.my-company.com`)

### JWKS Settings

- `auth.jwks.cache.ttl` — JWKS cache TTL (default: `1h`)

### Introspection Settings

- `auth.introspection.mode` — when to introspect: `never`, `opaque_only` (default), `always`
- `auth.introspection.endpoint` — global introspection endpoint URL (applies to all issuers)
- `auth.introspection.cache.enabled` — enable introspection result caching (default: `true`)
- `auth.introspection.cache.max_entries` — max cached introspection results (default: `10000`)
- `auth.introspection.cache.ttl` — introspection result cache TTL (default: `5m`)
- `auth.introspection.endpoint_discovery_cache.enabled` — cache discovered introspection endpoints (default: `true`)
- `auth.introspection.endpoint_discovery_cache.max_entries` — max cached endpoints (default: `10000`)
- `auth.introspection.endpoint_discovery_cache.ttl` — endpoint discovery cache TTL (default: `1h`)

---

## Token Introspection

Introspection (RFC 7662) is used in three scenarios:

1. **Opaque tokens** — token is not self-contained, must be validated by IdP
2. **JWT enrichment** — JWT lacks HyperSpot-specific claims (`sub_tenant_id`, `sub_type`), plugin fetches additional subject info via introspection
3. **Revocation checking** — even for valid JWTs, introspection provides central point to check if token was revoked (e.g., user logout, compromised token)

Configuration determines when introspection is triggered via `introspection.mode`:
- `introspection.mode: always` — all tokens (JWT and opaque) go through introspection
- `introspection.mode: opaque_only` — only opaque tokens (default)
- `introspection.mode: never` — JWT local validation only (no revocation check)

**Configuration Matrix:**

| Token Type | `introspection.mode` | `introspection.endpoint` | Behavior |
|------------|----------------------|--------------------------|----------|
| JWT | `never` | (any) | Local validation only, no introspection |
| JWT | `opaque_only` | (any) | Local validation only |
| JWT | `always` | configured | Use configured endpoint |
| JWT | `always` | not configured | Discover endpoint from issuer's OIDC config |
| Opaque | `never` | (any) | **401 Unauthorized** (cannot validate opaque without introspection) |
| Opaque | `opaque_only` / `always` | configured | Use configured endpoint |
| Opaque | `opaque_only` / `always` | not configured | **401 Unauthorized** (no `iss` claim to discover endpoint) |

**Note:** Discovery requires the `iss` claim to look up the issuer configuration. Opaque tokens don't contain claims, so discovery is only possible for JWTs. For opaque tokens, `introspection.endpoint` must be explicitly configured.

## OpenID Connect Integration

HyperSpot leverages OpenID Connect standards for authentication:

- **JWT validation** per OpenID Connect Core 1.0 — signature verification, claim validation
- **Discovery** via `.well-known/openid-configuration` (OpenID Connect Discovery 1.0) — automatic endpoint configuration
- **JWKS (JSON Web Key Set)** — public keys for JWT signature validation, fetched from `jwks_uri`
- **Token Introspection** (RFC 7662) — for opaque token validation, JWT enrichment, and revocation checking

### Issuer Configuration

The `trusted_issuers` map is required for JWT validation. This separation exists because:

1. **Trust anchor** — HyperSpot must know which issuers to trust before receiving tokens
2. **Flexible mapping** — `iss` claim may differ from discovery URL (e.g., custom identifiers)
3. **Bootstrap problem** — to validate JWT, we need JWKS; to get JWKS, we need discovery URL

**Lazy initialization flow:**
1. Admin configures `jwt.trusted_issuers` map
2. On first request, extract `iss` from JWT (unverified)
3. Look up `iss` in `jwt.trusted_issuers` → get discovery URL
4. If not found → reject (untrusted issuer)
5. Fetch `{discovery_url}/.well-known/openid-configuration`
6. Validate and cache JWKS, then verify JWT signature

### Discovery

Discovery is performed lazily on the first authenticated request (not at startup). HyperSpot fetches the OpenID configuration from `{issuer}/.well-known/openid-configuration` and extracts:

- `jwks_uri` — for fetching signing keys
- `introspection_endpoint` — for opaque token validation (optional)

**Caching:** JWKS is cached for `jwks.cache.ttl` (default: **1 hour**) and refreshed automatically on cache expiry or when signature validation fails with unknown `kid`.

---

## Plugin Role

The Auth Resolver plugin bridges HyperSpot to the vendor's IdP. The plugin is responsible for:

1. **IdP communication** — calling introspection endpoints, handling IdP-specific protocols
2. **Claim enrichment** — if the IdP doesn't include `subject_type` or `subject_tenant_id` in tokens, the plugin fetches this information from vendor services
3. **Response mapping** — converting IdP-specific responses to `SecurityContext`

**When a plugin is needed:**
- Vendor's IdP uses opaque tokens
- Standard claims don't include tenant information
- Custom subject type mapping is required
- Additional validation rules apply

---

## Validation

### Token Expiration

The `exp` (expiration) claim is always validated:
- JWT local: `exp` claim must be in the future
- Introspection: response `active` must be `true` and `exp` must be in the future

### Audience Validation

The `aud` (audience) claim validation is controlled by `jwt.require_audience` and `jwt.expected_audience`:

- If `require_audience: true` and JWT lacks `aud` claim → **401 Unauthorized**
- If `require_audience: false` (default) and JWT lacks `aud` claim → validation passes
- If JWT has `aud` claim and `expected_audience` is configured → at least one audience must match a pattern (glob pattern matching with `*` wildcard)
- If JWT has `aud` claim but `expected_audience` is empty/not configured → validation passes

---

## Introspection Caching

Introspection results MAY be cached to reduce IdP load and latency (`introspection.cache.*`). Trade-off: revoked tokens remain valid until cache expires. Cache TTL should be shorter than token lifetime; use token `exp` as upper bound for cache entry lifetime.

---

# Authorization

## Why AuthZEN (and Why It's Not Enough)

We chose [OpenID AuthZEN Authorization API 1.0](https://openid.net/specs/authorization-api-1_0.html) (approved 2026-01-12) as the foundation for Auth Resolver. See [ADR 0001](../adrs/authorization/0001-pdp-pep-authorization-model.md) for the full analysis of considered options.

**Why AuthZEN:**
- Industry standard with growing ecosystem
- Vendor-neutral: doesn't dictate policy model (RBAC/ABAC/ReBAC)
- Clean subject/action/resource/context structure
- Extensible via `context` field

However, AuthZEN out of the box doesn't solve HyperSpot's core requirement: **query-level authorization**.

### Why Access Evaluation API Alone Isn't Enough

AuthZEN's Access Evaluation API answers: "Can subject S perform action A on resource R?" — a point-in-time check returning `decision: true/false`.

#### LIST Operations

For **LIST operations** with Access Evaluation API, we'd need an iterative process:

1. Fetch a batch of resources from DB (e.g., `LIMIT 100` to get candidates for a page of 10)
2. Send batch to PDP for evaluation (AuthZEN supports batching via Access Evaluations API)
3. Filter results based on decisions
4. If filtered result < requested page size → fetch next batch, repeat

**The core problem**: unpredictable number of iterations. If the user has access to only 1% of resources, fetching a page of 10 items might require 10+ round-trips (DB → PDP → filter → not enough → repeat). Worst case: user has access to nothing, and we scan the entire table before returning empty result.

**Additional problems:**
- **Pagination cursor invalidation** — cursor points to DB offset, but after filtering the mapping breaks
- **Total count impossible** — can't know total accessible count without evaluating all resources
- **Inconsistent page sizes** — hard to guarantee exactly N items per page

#### Point Operations (GET/UPDATE/DELETE)

For **point operations**, Access Evaluation API could technically work, but requires an inefficient flow:

1. Query database to fetch the resource
2. Send resource to PDP for evaluation
3. If denied, return 403/404

**The problem**: the subject might not have rights to access this resource type at all. The database query is wasteful — we should fail fast before touching the database.

**What we want instead:**
1. Ask PDP first: "Can subject S perform action A on resource type T?"
2. If denied → 403 immediately (fail-fast, no database query)
3. If allowed → get constraints, execute query with `WHERE id = :id AND (constraints)`
4. If 0 rows → 404 (hides resource existence from unauthorized users)

### Why Search API Doesn't Work

AuthZEN's Resource Search API answers: "What resources can subject S perform action A on?" — returning a list of resource IDs.

This **assumes the PDP has access to resource data**. In HyperSpot's architecture, resources live in the PEP's database — the PDP cannot enumerate what it doesn't have.

This creates an architectural mismatch:
- **PDP** knows "who can access what" (authorization policies)
- **PEP** knows "what exists" (resources in database)

To use Search API, we'd need to sync all resources to the PDP — defeating the purpose of keeping data local.

### Our Solution: Extended Evaluation Response

We extend AuthZEN's evaluation response with optional `context.constraints`. Instead of returning resource IDs (enumeration), the PDP returns **predicates** that the PEP compiles to SQL WHERE clauses:

```jsonc
// PDP response
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          { "type": "in_tenant_subtree", "resource_property": "owner_tenant_id", "root_tenant_id": "tenant-123", "respect_barrier": true }
        ]
      }
    ]
  }
}
// PEP compiles to: WHERE owner_tenant_id IN (SELECT descendant_id FROM tenant_closure WHERE ancestor_id = 'tenant-123')
// Result: SELECT * FROM events WHERE (constraints) LIMIT 10 — correct pagination!
```

This gives us:
- **O(1) authorization overhead** per query (single PDP call)
- **Correct pagination** — constraints applied at SQL level before LIMIT
- **Accurate counts** — database handles filtering
- **No resource sync** — PDP never needs to know about individual resources

---

## PEP Enforcement

### Unified PEP Flow

All operations (LIST, GET, UPDATE, DELETE) follow the same flow:

```mermaid
sequenceDiagram
    participant Client
    participant PEP as Module (PEP)
    participant PDP as Auth Resolver (PDP)
    participant DB

    Client->>PEP: GET /events
    PEP->>PDP: evaluation request
    PDP-->>PEP: decision + constraints
    PEP->>DB: SQL with constraints
    DB-->>PEP: filtered results
    PEP-->>Client: response
```

The only difference between LIST and point operations (GET/UPDATE/DELETE) is whether `resource.id` is present.

### Constraint Compilation to SQL

When constraints are present, the PEP compiles each constraint to SQL WHERE clauses:

1. **Predicates within a constraint** (`predicates` array) are AND'd together
2. **Multiple constraints** (`constraints` array) are OR'd together
3. **Unknown predicate types** cause that constraint to be treated as false (fail-closed)

### Fail-Closed Rules

The PEP MUST:

1. **Validate decision** - `decision: false` or missing -> deny all (403 Forbidden)
2. **Enforce require_constraints** - If `require_constraints: true` and `decision: true` but no `constraints` -> deny all (403 Forbidden)
3. **Apply constraints when present** - If `constraints` array is present, apply to SQL; if all constraints evaluate to false -> deny all
4. **Trust decision when constraints not required** - `decision: true` without `constraints` AND `require_constraints: false` -> allow (e.g., CREATE operations)
5. **Handle unreachable PDP** - Network failure, timeout -> deny all
6. **Handle unknown predicate types** - Treat containing constraint as false; if all constraints false -> deny all
7. **Handle missing required fields** - Treat containing constraint as false
8. **Handle unknown property names** - Treat containing constraint as false (PEP doesn't know how to map)

---

## API Specifications

### Access Evaluation API (AuthZEN-extended)

Two endpoints for authorization checks, following AuthZEN structure:

- `POST /access/v1/evaluation` - Single evaluation request
- `POST /access/v1/evaluations` - Batch evaluation (array of requests -> array of responses)

PDP returns `decision` plus optional `constraints` for each evaluation.

#### Design Principles

1. **AuthZEN alignment** - Use same `subject`, `action`, `resource`, `context` structure
2. **Constraints are optional** - PDP decides when to include based on action type
3. **Constraint-first** - Return predicates, not enumerated IDs
4. **Capability negotiation** - PEP declares enforcement capabilities
5. **Fail-closed** - Unknown constraints or schemas result in deny
6. **OR/AND semantics** - Multiple constraints are OR'd (alternative access paths), predicates within constraint are AND'd
7. **Token passthrough** - Original bearer token optionally included in `context.bearer_token` for PDP validation and external service calls (MUST NOT be logged)

#### Request

```
POST /access/v1/evaluation
Content-Type: application/json
```

```jsonc
{
  // AuthZEN standard fields
  "subject": {
    "type": "gts.x.core.security.subject.user.v1~",
    "id": "a254d252-7129-4240-bae5-847c59008fb6",
    "properties": {
      "tenant_id": "51f18034-3b2f-4bfa-bb99-22113bddee68"
    }
  },
  "action": {
    "name": "list"  // or "read", "update", "delete", "create"
  },
  "resource": {
    "type": "gts.x.events.event.v1~",
    "id": "e81307e5-5ee8-4c0a-8d1f-bd98a65c517e",  // present for point ops, absent for list
    "properties": {
      "topic_id": "gts.x.core.events.topic.v1~z.app._.some_topic.v1"
    }
  },

  // HyperSpot extension: context with tenant and PEP capabilities
  "context": {
    // Tenant context — use ONE of: tenant_id OR tenant_subtree

    // Option 1: Single tenant (simple case)
    // "tenant_id": "51f18034-3b2f-4bfa-bb99-22113bddee68",

    // Option 2: Tenant subtree (with hierarchy options)
    "tenant_subtree": {
      "root_id": "51f18034-3b2f-4bfa-bb99-22113bddee68",
      "include_root": true,        // default: true
      "respect_barrier": true,     // default: false, honor self_managed barrier
      "tenant_status": ["active", "suspended"]  // optional, filter by status
    },

    // PEP enforcement mode
    "require_constraints": true,  // if true, decision without constraints = deny

    // PEP capabilities: what predicate types the caller can enforce locally
    "capabilities": ["tenant_hierarchy", "group_membership", "group_hierarchy"],

    // Original bearer token (optional) — see "Bearer Token in Context" below
    "bearer_token": "eyJhbGciOiJSUzI1NiIs..."
  }
}
```

#### Bearer Token in Context

The `context.bearer_token` field is optional. PEP includes it when PDP needs access to the original token. Use cases:

1. **PDP validation (defence-in-depth)** — In out-of-process deployments, PDP may not fully trust subject claims extracted by PEP. PDP can independently validate the token signature and extract claims to verify `subject.id` and `subject.properties` match the token.

2. **External service calls** — Auth Resolver plugin may need to call vendor's external APIs (authorization service, user info endpoint, etc.) that require the original bearer token for authentication.

3. **Token-embedded policies** — Some IdPs embed access policies directly in the token (e.g., `permissions`, `roles`, `scopes` claims in JWT). PDP extracts and evaluates these claims to generate constraints.

4. **Scope narrowing** — Token may contain scope restrictions (e.g., `scope: "read:events"`, resource-specific access tokens). PDP uses these to narrow the access decision beyond what static policies would allow.

5. **Audit and tracing** — Token may contain correlation IDs, session info, or other metadata useful for audit logging in PDP.

**When to omit:** If PDP fully trusts PEP's claim extraction and doesn't need to call external services, `bearer_token` can be omitted to reduce payload size and minimize credential exposure.

**Security:** `bearer_token` is a credential. PDP MUST NOT log it, persist it, or include it in error responses.

#### Response

The response contains a `decision` and, when `decision: true`, optional `context.constraints`. Each constraint is an object with a `predicates` array that the PEP compiles to SQL.

```jsonc
{
  "decision": true,
  "context": {
    // Multiple constraints are OR'd together (alternative access paths)
    // Each constraint's predicates are AND'd together
    "constraints": [
      {
        "predicates": [
          {
            // Tenant subtree predicate - uses local tenant_closure table
            "type": "in_tenant_subtree",
            "resource_property": "owner_tenant_id",
            "root_tenant_id": "51f18034-3b2f-4bfa-bb99-22113bddee68",
            "respect_barrier": true,
            "tenant_status": ["active", "suspended"]
          },
          {
            // Equality predicate
            "type": "eq",
            "resource_property": "topic_id",
            "value": "gts.x.core.events.topic.v1~z.app._.some_topic.v1"
          }
        ]
      }
    ]
  }
}
```

#### PEP Decision Matrix

| `decision` | `constraints` | `require_constraints` | PEP Action |
|------------|---------------|----------------------|------------|
| `false` | (any) | (any) | **403 Forbidden** |
| `true` | absent | `false` | Allow (trust PDP decision) |
| `true` | absent | `true` | **403 Forbidden** (constraints required but missing) |
| `true` | present | (any) | Apply constraints to SQL |

**Key insight:** PEP declares via `require_constraints` capability whether it needs constraints for the operation. For LIST operations, this should typically be `true`; for CREATE, it can be `false`.

#### Operation-Specific Behavior

**CREATE** (no constraints needed):
```jsonc
// PEP -> PDP
{
  "action": { "name": "create" },
  "resource": {
    "type": "gts.x.events.event.v1~",
    "properties": { "owner_tenant_id": "tenant-B", "topic_id": "..." }
  }
  // ... subject, context
}

// PDP -> PEP
{ "decision": true }  // no constraints - PEP trusts decision

// PEP: INSERT INTO events ...
```

**LIST** (constraints required):
```jsonc
// PEP -> PDP
{
  "action": { "name": "list" },
  "resource": { "type": "gts.x.events.event.v1~" }  // no id
  // ... subject, context
}

// PDP -> PEP
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          { "type": "in_tenant_subtree", "resource_property": "owner_tenant_id", "root_tenant_id": "tenant-A", "respect_barrier": true }
        ]
      }
    ]
  }
}

// PEP: SELECT * FROM events WHERE (constraints)
```

**GET/UPDATE/DELETE** (constraints for SQL-level enforcement):
```jsonc
// PEP -> PDP
{
  "action": { "name": "read" },
  "resource": { "type": "gts.x.events.event.v1~", "id": "evt-123" }
  // ... subject, context
}

// PDP -> PEP
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          { "type": "in_tenant_subtree", "resource_property": "owner_tenant_id", "root_tenant_id": "tenant-A", "respect_barrier": true }
        ]
      }
    ]
  }
}

// PEP: SELECT * FROM events WHERE id = :id AND (constraints)
// 0 rows -> 404 (hides resource existence)
```

#### Response with Resource Group Predicate

```jsonc
{
  "decision": true,
  "context": {
    "constraints": [
      {
        "predicates": [
          {
            // Tenant subtree predicate
            "type": "in_tenant_subtree",
            "resource_property": "owner_tenant_id",
            "root_tenant_id": "tenant-A",
            "respect_barrier": true
          },
          {
            // Group subtree predicate - uses resource_group_membership + resource_group_closure tables
            "type": "in_group_subtree",
            "resource_property": "id",
            "root_group_id": "project-root-group"
          }
        ]
      }
    ]
  }
}
```

#### Deny Response

```jsonc
{
  "decision": false
}
```

---

## Predicate Types Reference

All predicates filter resources based on their properties. The `resource_property` field specifies which property to filter on — these correspond directly to `resource.properties` in the request.

| Type | Description | Required Fields | Optional Fields |
|------|-------------|-----------------|-----------------|
| `eq` | Property equals value | `resource_property`, `value` | — |
| `in` | Property in value list | `resource_property`, `values` | — |
| `in_tenant_subtree` | Tenant subtree via closure table | `resource_property`, `root_tenant_id` | `respect_barrier`, `tenant_status` |
| `in_group` | Flat group membership | `resource_property`, `group_ids` | — |
| `in_group_subtree` | Group subtree via closure table | `resource_property`, `root_group_id` | — |

### 1. Equality Predicate (`type: "eq"`)

Compares resource property to a single value.

**Schema:**
- `type` (required): `"eq"`
- `resource_property` (required): Property name (e.g., `topic_id`, `owner_tenant_id`)
- `value` (required): Single value to compare

```jsonc
{ "type": "eq", "resource_property": "topic_id", "value": "uuid-123" }
// SQL: topic_id = 'uuid-123'
```

### 2. IN Predicate (`type: "in"`)

Compares resource property to a list of values.

**Schema:**
- `type` (required): `"in"`
- `resource_property` (required): Property name (e.g., `owner_tenant_id`, `status`)
- `values` (required): Array of values

```jsonc
{ "type": "in", "resource_property": "owner_tenant_id", "values": ["tenant-1", "tenant-2"] }
// SQL: owner_tenant_id IN ('tenant-1', 'tenant-2')

{ "type": "in", "resource_property": "status", "values": ["active", "pending"] }
// SQL: status IN ('active', 'pending')
```

### 3. Tenant Subtree Predicate (`type: "in_tenant_subtree"`)

Filters resources by tenant subtree using the closure table. The `resource_property` specifies which property contains the tenant ID.

**Schema:**
- `type` (required): `"in_tenant_subtree"`
- `resource_property` (required): Property containing tenant ID (e.g., `owner_tenant_id`)
- `root_tenant_id` (required): Root of tenant subtree
- `respect_barrier` (optional): Honor `self_managed` barrier in hierarchy traversal, default `false`
- `tenant_status` (optional): Filter by tenant status

```jsonc
{
  "type": "in_tenant_subtree",
  "resource_property": "owner_tenant_id",
  "root_tenant_id": "tenant-A",
  "respect_barrier": true,
  "tenant_status": ["active", "suspended"]
}
// SQL: owner_tenant_id IN (
//   SELECT descendant_id FROM tenant_closure
//   WHERE ancestor_id = 'tenant-A'
//     AND (barrier_ancestor_id IS NULL OR barrier_ancestor_id = 'tenant-A')
//     AND descendant_status IN ('active', 'suspended')
// )
```

### 4. Group Membership Predicate (`type: "in_group"`)

Filters resources by explicit group membership. The `resource_property` specifies which property is used for group membership join.

**Schema:**
- `type` (required): `"in_group"`
- `resource_property` (required): Property for group membership join (typically `id`)
- `group_ids` (required): Array of group IDs

```jsonc
{ "type": "in_group", "resource_property": "id", "group_ids": ["group-1", "group-2"] }
// SQL: id IN (
//   SELECT resource_id FROM resource_group_membership
//   WHERE group_id IN ('group-1', 'group-2')
// )
```

### 5. Group Subtree Predicate (`type: "in_group_subtree"`)

Filters resources by group subtree using the closure table. The `resource_property` specifies which property is used for group membership join.

**Schema:**
- `type` (required): `"in_group_subtree"`
- `resource_property` (required): Property for group membership join (typically `id`)
- `root_group_id` (required): Root of group subtree

```jsonc
{ "type": "in_group_subtree", "resource_property": "id", "root_group_id": "root-group" }
// SQL: id IN (
//   SELECT resource_id FROM resource_group_membership
//   WHERE group_id IN (
//     SELECT descendant_id FROM resource_group_closure
//     WHERE ancestor_id = 'root-group'
//   )
// )
```

---

## PEP Property Mapping

The `resource_property` in predicates corresponds to `resource.properties` in the request. Each module (PEP) defines a mapping from property names to physical SQL columns. PDP uses property names — **it doesn't know the database schema**.

**Example mapping for Event Manager:**

| Resource Property | SQL Column |
|-------------------|------------|
| `owner_tenant_id` | `events.tenant_id` |
| `topic_id` | `events.topic_id` |
| `id` | `events.id` |

**How PEP compiles predicates to SQL:**

| Predicate | SQL |
|-----------|-----|
| `{ "type": "eq", "resource_property": "topic_id", "value": "v" }` | `events.topic_id = 'v'` |
| `{ "type": "in", "resource_property": "owner_tenant_id", "values": ["t1", "t2"] }` | `events.tenant_id IN ('t1', 't2')` |
| `{ "type": "in_tenant_subtree", "resource_property": "owner_tenant_id", ... }` | `events.tenant_id IN (SELECT descendant_id FROM tenant_closure WHERE ...)` |
| `{ "type": "in_group", "resource_property": "id", "group_ids": ["g1", "g2"] }` | `events.id IN (SELECT resource_id FROM resource_group_membership WHERE ...)` |
| `{ "type": "in_group_subtree", "resource_property": "id", "root_group_id": "g1" }` | `events.id IN (SELECT ... FROM resource_group_membership WHERE group_id IN (SELECT ... FROM resource_group_closure ...))` |

**Conventions:**
- All IDs are UUIDs
- PDP may return GTS IDs (e.g., `gts.x.core.events.topic.v1~...`), PEP converts to UUIDv5

---

## Capabilities -> Predicate Matrix

The PEP declares its capabilities in the request. This determines what predicate types the PDP can return.

### `require_constraints` Flag

The `require_constraints` field (separate from capabilities array) controls PEP behavior when constraints are absent:

| `require_constraints` | `decision: true` without `constraints` |
|-----------------------|----------------------------------------|
| `true` | **deny** (constraints required but missing) |
| `false` | **allow** (trust PDP decision) |

**Usage:**
- For LIST operations: typically `true` (constraints needed for SQL WHERE)
- For CREATE operations: typically `false` (no query, just permission check)
- For GET/UPDATE/DELETE: depends on whether PEP wants SQL-level enforcement or trusts PDP decision

### Capabilities Array

Capabilities declare what predicate types the PEP can enforce locally:

| Capability | Enables Predicate Types |
|------------|---------------------|
| `tenant_hierarchy` | `in_tenant_subtree` |
| `group_membership` | `in_group` |
| `group_hierarchy` | `in_group_subtree` (implies `group_membership`) |

**Capability dependencies:**
- `group_hierarchy` implies `group_membership` — if PEP has the closure table, it necessarily has the membership table
- When declaring capabilities, `["group_hierarchy"]` is sufficient; `group_membership` is implied

**Predicate type availability by capability:**

| Predicate Type | Required Capability |
|-------------|---------------------|
| `eq`, `in` | (none — always available) |
| `in_tenant_subtree` | `tenant_hierarchy` |
| `in_group` | `group_membership` |
| `in_group_subtree` | `group_hierarchy` |

**Capability degradation**: If a PEP lacks a capability, the PDP must either:
1. Expand the predicate to explicit IDs (may be large)
2. Return `decision: false` if expansion is not feasible

---

## Table Schemas (Local Projections)

These tables are maintained locally by HyperSpot gateway modules (Tenant Resolver, Resource Group Resolver) and used by PEPs to execute constraint queries efficiently without calling back to the vendor platform.

### `tenant_closure`

Denormalized closure table for tenant hierarchy. Enables efficient subtree queries without recursive CTEs.

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| `ancestor_id` | UUID | No | Parent tenant in the hierarchy |
| `descendant_id` | UUID | No | Child tenant (the one we check ownership against) |
| `barrier_ancestor_id` | UUID | Yes | ID of tenant with `self_managed=true` between ancestor and descendant (NULL if no barrier) |
| `descendant_status` | TEXT | No | Status of descendant tenant (`active`, `suspended`, `deleted`) |

**Notes:**
- Status is denormalized into closure for query simplicity (avoids JOIN). When a tenant's status changes, all rows where it is `descendant_id` are updated.
- The `barrier_ancestor_id` column enables `respect_barrier` filtering: when set, only include descendants where `barrier_ancestor_id IS NULL OR barrier_ancestor_id = :root_tenant_id`.
- Self-referential rows exist: each tenant has a row where `ancestor_id = descendant_id`.
- **Predicate mapping:** `in_tenant_subtree` predicate compiles to SQL using this closure table.

**Example query (in_tenant_subtree):**
```sql
SELECT * FROM events
WHERE owner_tenant_id IN (
  SELECT descendant_id FROM tenant_closure
  WHERE ancestor_id = :root_tenant_id
    AND (barrier_ancestor_id IS NULL OR barrier_ancestor_id = :root_tenant_id)  -- respect_barrier
    AND descendant_status IN ('active', 'suspended')  -- tenant_status filter
)
```

### `resource_group_closure`

Closure table for resource group hierarchy. Similar structure to tenant_closure but simpler (no barrier or status).

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| `ancestor_id` | UUID | No | Parent group |
| `descendant_id` | UUID | No | Child group |

**Notes:**
- Self-referential rows exist: each group has a row where `ancestor_id = descendant_id`.
- **Predicate mapping:** `in_group_subtree` predicate compiles to SQL using this closure table.

### `resource_group_membership`

Association between resources and groups. A resource can belong to multiple groups.

| Column | Type | Nullable | Description |
|--------|------|----------|-------------|
| `resource_id` | UUID | No | ID of the resource (FK to resource table) |
| `group_id` | UUID | No | ID of the group (FK to resource_group_closure) |

**Notes:**
- The `resource_id` column joins with the resource table's ID column (configurable per module, default `id`).
- **Predicate mapping:** `in_group` and `in_group_subtree` predicates use this table for the resource-to-group join.

**Example query (in_group_subtree):**
```sql
SELECT * FROM events
WHERE id IN (
  SELECT resource_id FROM resource_group_membership
  WHERE group_id IN (
    SELECT descendant_id FROM resource_group_closure
    WHERE ancestor_id = :root_group_id
  )
)
```

---

## Usage Scenarios

For concrete examples demonstrating the authorization model in practice, see [SCENARIOS.md](./SCENARIOS.md).

The scenarios document covers:
- When to use projection tables (`tenant_closure`, `resource_group_membership`, `resource_group_closure`)
- Complete request/response flows for LIST, GET, UPDATE, DELETE, CREATE operations
- With and without closure tables (prefetch patterns)
- Resource group filtering (flat membership and hierarchy)
- Combined tenant + group constraints (AND/OR semantics)
- TOCTOU protection analysis

---

## Open Questions

1. **"Allow all" semantics** - Should there be a way for PDP to express "allow all resources of this type" (e.g., for platform support roles)? Currently, constraints must have concrete predicates. Future consideration: `predicates: []` with explicit "allow all" semantics.

2. **Empty `predicates` interpretation** - If a constraint has an empty `predicates: []` array, should it mean "match all" or "match none"? Currently undefined.

3. **Batch evaluation optimization** - We support `/access/v1/evaluations` for batch requests. Should PDP optimize constraint generation when multiple evaluations share the same subject/context? Use cases: bulk operations, permission checks for UI rendering.

4. **Constraint caching** - Can constraints be cached at the PEP level beyond TTL? What invalidation signals are needed?

5. **AuthZEN context structure** - Is embedding HyperSpot-specific fields in `context` the right approach, or should we use a dedicated extension namespace?

6. **IANA registration** - Should HyperSpot register its extension parameters with the AuthZEN metadata registry?

7. **AuthZEN Search API relationship** - Our extended evaluation response serves similar purposes to Resource Search. Should we document this as a constraint-based alternative, or position it separately?

---

## References

### Authentication
- [RFC 7519: JSON Web Token (JWT)](https://datatracker.ietf.org/doc/html/rfc7519)
- [RFC 7662: OAuth 2.0 Token Introspection](https://datatracker.ietf.org/doc/html/rfc7662)
- [OpenID Connect Core 1.0](https://openid.net/specs/openid-connect-core-1_0.html)
- [OpenID Connect Discovery 1.0](https://openid.net/specs/openid-connect-discovery-1_0.html)

### Authorization
- [OpenID AuthZEN Authorization API 1.0](https://openid.net/specs/authorization-api-1_0.html) (approved 2026-01-12)
- [ADR 0001: PDP/PEP Authorization Model](../adrs/authorization/0001-pdp-pep-authorization-model.md)

### Internal
- [TENANT_MODEL.md](./TENANT_MODEL.md) — Tenant topology, barriers, closure tables
- [SCENARIOS.md](./SCENARIOS.md) — Authorization usage scenarios
- [HyperSpot GTS (Global Type System)](../../modules/types-registry/)
