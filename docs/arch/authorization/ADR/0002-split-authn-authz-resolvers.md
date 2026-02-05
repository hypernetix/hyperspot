---
status: accepted
date: 2026-01-29
decision-makers: Vasily
---

# Use Separate AuthN Resolver and AuthZ Resolver

## Context and Problem Statement

HyperSpot requires integration with vendor-specific identity and authorization infrastructure. This integration involves two distinct concerns:

1. **Authentication (AuthN)** — Token validation (JWT signature verification, introspection), JWKS management, claim extraction, and SecurityContext production
2. **Authorization (AuthZ)** — Policy Decision Point (PDP) functionality, policy evaluation, and constraint generation for Policy Enforcement Points (PEPs)

These are conceptually separate responsibilities governed by different standards:

- **AuthN** — OpenID Connect Core 1.0, RFC 7519 (JWT), RFC 7662 (Token Introspection)
- **AuthZ** — OpenID AuthZEN Authorization API 1.0, NIST SP 800-162 (PDP/PEP model)

**Key architectural question:** Should HyperSpot use a single unified resolver module (Auth Resolver) that handles both AuthN and AuthZ, or should these be split into two independent gateway modules (AuthN Resolver and AuthZ Resolver)?

This decision impacts deployment flexibility, vendor integration patterns, security boundaries, caching strategies, and component reusability.

## Decision Drivers

- **Separation of Concerns** — AuthN and AuthZ are distinct responsibilities with different standards, data flows, and security properties
- **Deployment Flexibility** — AuthN and AuthZ have different scaling characteristics and may benefit from independent deployment (especially in out-of-process/distributed scenarios)
- **Security Boundaries** — AuthN handles credentials (bearer tokens), AuthZ handles validated identity; separating these creates clearer security boundaries for audit
- **Caching Strategy** — AuthN and AuthZ have fundamentally different caching patterns:
  - AuthN: cache by token, TTL bounded by `exp`, sensitive data
  - AuthZ: cache by `(subject, action, resource, context)`, potentially longer TTL, less sensitive
- **Vendor Integration** — Some vendors have separate services for IdP and Authorization Engine; unified resolver forces coupling
- **Mix & Match** — Enable using different vendors for AuthN (e.g., Auth0, Okta) and AuthZ (e.g., OpenFGA, Oso, custom PDP)
- **Component Reusability** — AuthN Resolver can be reused for non-AuthZ scenarios (WebSocket auth, gRPC auth, audit, metrics); AuthZ Resolver can be used for bulk checks, UI pre-flight, admin tools

## Considered Options

- **Option A**: Unified Auth Resolver (single module for both AuthN and AuthZ)
- **Option B**: Separate AuthN Resolver and AuthZ Resolver (two independent modules)

## Decision Outcome

Chosen option: **Option B - Separate AuthN Resolver and AuthZ Resolver**, because it provides clearer separation of concerns aligned with industry standards (OIDC vs AuthZEN), enables deployment flexibility for distributed scenarios, creates proper security boundaries, and allows mix-and-match vendor integration while maintaining simplicity for in-process deployments.

**Implementation:**

1. **AuthN Resolver** (gateway + plugin):
   - Responsibilities: token validation, JWT signature verification, JWKS management, token introspection (RFC 7662), claim extraction
   - Output: `SecurityContext` (subject_id, subject_type, subject_tenant_id, token_scopes, bearer_token)
   - Used by: AuthN middleware in modules accepting requests (API Gateway module, Domain Module, gRPC Gateway module, WebSocket handlers, etc.)
   - Standards: OpenID Connect Core 1.0, RFC 7519 (JWT), RFC 7662 (Token Introspection)

2. **AuthZ Resolver** (gateway + plugin):
   - Responsibilities: PDP functionality, policy evaluation, constraint generation
   - Input: `SecurityContext` + evaluation request (subject, action, resource, context)
   - Output: decision + constraints
   - Used by: PEPs (domain modules)
   - Standards: OpenID AuthZEN Authorization API 1.0, NIST SP 800-162

3. **Request Flow:**

   ```text
   Client -> API Gateway -> AuthN Resolver -> SecurityContext
   API Gateway -> PEP (Domain Module) -> Request + SecurityContext
   PEP -> AuthZ Resolver -> Evaluation Request
   AuthZ Resolver -> PEP -> Decision + Constraints
   PEP -> Database -> Query with WHERE (constraints)
   ```

4. **Unified Plugin Support:**
   - Vendors with integrated AuthN+AuthZ API can provide a unified plugin wrapper
   - Shared cache mechanism for coordination between AuthN and AuthZ plugins
   - Single configuration section for simple cases

### Consequences

**Good:**

- **Standards alignment** — Clear mapping to OIDC (AuthN) and AuthZEN (AuthZ) specifications
- **Deployment flexibility** — AuthN can be edge-deployed (close to clients), AuthZ can be centralized (shared PDP across instances)
- **Independent scaling** — Different load patterns (AuthN: per-request, AuthZ: cacheable) can be scaled independently in distributed deployments
- **Security boundaries** — Credentials (tokens) handled only in AuthN layer; AuthZ works with validated identity (reduces attack surface)
- **Reusability** — AuthN Resolver can authenticate non-HTTP protocols (gRPC, WebSocket); AuthZ Resolver can serve bulk checks, UI permissions, admin tools
- **Mix & match vendors** — Use standard OpenID provider for AuthN (Auth0, Okta) + specialized policy engine for AuthZ (OpenFGA, Oso, custom)
- **Caching independence** — AuthN and AuthZ can implement optimal caching strategies independently
- **Simpler plugins** — Single responsibility plugins are easier to develop, test, and maintain

**Bad:**

- **Configuration complexity** — Two modules instead of one (mitigated by unified config section with subsections)
- **Vendor coordination** — For vendors with unified AuthN+AuthZ API, requires either:
  - Unified plugin wrapper with shared state
  - Two separate plugins with cache coordination
  - Two API calls (less efficient)
- **Version coordination** — When vendor updates API, both AuthN and AuthZ plugins may need updates

**Neutral:**

- **No latency overhead** — AuthN and AuthZ are inherently separate steps in the request flow; having separate resolvers doesn't add network hops compared to unified approach (both require AuthN before AuthZ)
- **Token passthrough** — SecurityContext will include `bearer_token` field for passing token from AuthN to AuthZ when needed

## Pros and Cons of the Options

### Option A: Unified Auth Resolver

Single gateway module (Auth Resolver) that handles both AuthN and AuthZ concerns.

**Pros:**

- Simpler configuration (single module, single plugin)
- Easier for vendors with tightly integrated AuthN+AuthZ APIs (single implementation)
- No coordination needed between AuthN and AuthZ plugins
- Fewer moving parts in simple in-process deployments

**Cons:**

- Violates separation of concerns (OIDC ≠ AuthZEN)
- Limited deployment flexibility (AuthN and AuthZ must be co-located)
- Mixing credentials handling (sensitive) with policy decisions (less sensitive) in same component
- Cannot independently scale AuthN (high-frequency) and AuthZ (cacheable) in distributed scenarios
- Cannot mix vendors (e.g., standard OpenID provider + custom policy engine)
- Caching strategy must compromise between AuthN and AuthZ needs
- Reduced reusability (AuthN functionality tied to AuthZ context)
- Larger plugin surface area (more complex to implement and test)

### Option B: Separate AuthN Resolver and AuthZ Resolver

Two independent gateway modules with separate plugin interfaces.

**Pros:**

- **Separation of concerns** — OIDC (AuthN) and AuthZEN (AuthZ) are distinct standards and layers
- **Deployment flexibility** — AuthN on edge, AuthZ centralized; independent scaling in distributed scenarios
- **Security boundaries** — Credentials isolated in AuthN layer; AuthZ works with validated identity only
- **Caching independence** — Optimal strategies for each (AuthN: by token, short TTL; AuthZ: by decision, longer TTL)
- **Mix & match vendors** — Standard OpenID provider + specialized policy engine
- **Reusability** — AuthN for gRPC/WebSocket/audit; AuthZ for bulk checks/UI/admin
- **Simpler plugins** — Single responsibility, easier to develop and test

**Cons:**

- More complex configuration (two modules)
- Vendors with unified API need unified plugin wrapper or cache coordination
- Version coordination when vendor updates API

**Mitigation strategies:**

1. **Unified configuration pattern:**

   ```yaml
   auth:
     authn:
       plugin: vendor-authn-plugin
       jwt: { ... }
     authz:
       plugin: vendor-authz-plugin
       endpoint: ...
   ```

2. **Unified plugin wrapper:**

   ```rust
   trait UnifiedAuthPlugin: AuthNProvider + AuthZProvider {
       // Single plugin implements both interfaces with shared state
   }
   ```

3. **Shared cache mechanism:**
   - AuthN caches introspection result (including vendor-specific data)
   - AuthZ plugin can access cached data to avoid duplicate vendor API calls

## More Information

**Related Documentation:**

- [DESIGN.md](../DESIGN.md) — Authentication and authorization design specification
- [ADR 0001: PDP/PEP Authorization Model](./0001-pdp-pep-authorization-model.md) — Authorization architecture foundation

**Standards References:**

- OpenID Connect Core 1.0: https://openid.net/specs/openid-connect-core-1_0.html
- RFC 7519 (JSON Web Token): https://datatracker.ietf.org/doc/html/rfc7519
- RFC 7662 (Token Introspection): https://datatracker.ietf.org/doc/html/rfc7662
- OpenID AuthZEN Authorization API 1.0: https://openid.net/specs/authorization-api-1_0.html
- NIST SP 800-162 (ABAC Guide): https://csrc.nist.gov/publications/detail/sp/800-162/final

**Prior Art (Separate AuthN/AuthZ):**

- AWS IAM: Separate Authentication (STS, Cognito) and Authorization (IAM Policies, resource policies)
- Google Cloud: Separate Authentication (Identity Platform) and Authorization (IAM, Zanzibar-based)
- Azure: Separate Authentication (Azure AD) and Authorization (Azure RBAC, resource policies)
- Kubernetes: Separate Authentication (various providers) and Authorization (RBAC, ABAC, Webhook)

**Implementation Notes:**

- SecurityContext flows from AuthN Resolver to PEP to AuthZ Resolver
- bearer_token in SecurityContext enables AuthZ plugin to call vendor APIs requiring authentication
- In-process deployment: both resolvers are function calls (no network latency)
- Out-of-process deployment: AuthN on edge (low latency), AuthZ can be centralized (shared across instances)
