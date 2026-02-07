# PRD — CredStore

<!--
=============================================================================
PRODUCT REQUIREMENTS DOCUMENT (PRD)
=============================================================================
PURPOSE: Define WHAT the system must do and WHY — business requirements,
functional capabilities, and quality attributes.

SCOPE:
  ✓ Business goals and success criteria
  ✓ Actors (users, systems) that interact with this module
  ✓ Functional requirements (WHAT, not HOW)
  ✓ Non-functional requirements (quality attributes, SLOs)
  ✓ Scope boundaries (in/out of scope)
  ✓ Assumptions, dependencies, risks

NOT IN THIS DOCUMENT (see other templates):
  ✗ Stakeholder needs (managed at project/task level by steering committee)
  ✗ Technical architecture, design decisions → DESIGN.md
  ✗ Why a specific technical approach was chosen → ADR/
  ✗ Detailed implementation flows, algorithms → features/

STANDARDS ALIGNMENT:
  - IEEE 830 / ISO/IEC/IEEE 29148:2018 (requirements specification)
  - IEEE 1233 (system requirements)
  - ISO/IEC 15288 / 12207 (requirements definition)

REQUIREMENT LANGUAGE:
  - Use "MUST" or "SHALL" for mandatory requirements (implicit default)
  - Do not use "SHOULD" or "MAY" — use priority p2/p3 instead
  - Be specific and clear; no fluff, bloat, duplication, or emoji
=============================================================================
-->

## 1. Overview

### 1.1 Purpose

CredStore provides per-tenant secret storage and retrieval for the platform. It abstracts backend differences behind a unified API, enabling platform modules to store and access credentials without coupling to a specific storage technology.

### 1.2 Background / Problem Statement

Platform modules — most notably the Outbound API Gateway (OAGW) — need access to secrets (API keys, tokens, credentials) for making upstream API calls on behalf of tenants. These secrets must be stored securely, scoped per tenant, and accessible only to authorized consumers.

Standard credential stores provide per-tenant isolation but do not support hierarchical multi-tenant sharing. In the platform's business model, parent tenants (partners) share API credentials with child tenants (customers). For example, a partner with an OpenAI API key and quota allows their customers to make requests through OAGW using the partner's key — without the customer ever seeing the actual secret value. This requires a hierarchical resolution model: when a customer requests a secret, the system walks up the tenant tree to find a shared secret from an ancestor.

Additionally, the platform runs in multiple environments: Kubernetes (where an external credential store like VendorA Credstore is available) and desktop/VM (where OS-level protected storage is appropriate). A plugin-based architecture allows runtime selection of the appropriate backend without changing consumer code.

### 1.3 Goals (Business Outcomes)

- Enable OAGW to retrieve tenant credentials for upstream API calls without exposing secret values to end users
- Support hierarchical credential sharing so partners can share API access with customers
- Decouple platform modules from specific credential storage backends
- Enforce least-privilege access: read vs write authorization, service-to-service vs tenant self-service

### 1.4 Glossary

| Term | Definition |
|------|------------|
| Secret | A key-value pair where the value is sensitive (API key, token, password) |
| Secret reference | A human-readable key identifying a secret within a tenant's namespace (e.g., `partner-openai-key`) |
| Sharing mode | Controls whether a secret is accessible to descendant tenants: `private` (owner only) or `shared` (owner + descendants) |
| Hierarchical resolution | Lookup algorithm that walks from child to parent to root tenant, returning the first matching shared secret |
| Secret shadowing | When a child tenant creates a secret with the same reference as a parent's shared secret, the child's own secret takes precedence |
| SecurityCtx | Request security context containing the authenticated tenant ID and permissions |

## 2. Actors

### 2.1 Human Actors

#### Tenant Admin

**ID**: `fdd-credstore-actor-tenant-admin`

<!-- fdd-id-content -->
**Role**: Authenticated user managing secrets for their tenant. Creates, updates, and deletes secrets. Configures sharing mode to control descendant access.
**Needs**: CRUD operations on secrets within their own tenant namespace. Ability to share secrets with descendants or keep them private.
<!-- fdd-id-content -->

### 2.2 System Actors

#### Outbound API Gateway (OAGW)

**ID**: `fdd-credstore-actor-oagw`

<!-- fdd-id-content -->
**Role**: Service that proxies outbound API calls to external services. Retrieves secrets on behalf of tenants using service-to-service authentication with explicit tenant_id. Primary consumer of hierarchical secret resolution.
<!-- fdd-id-content -->

#### Platform Module

**ID**: `fdd-credstore-actor-platform-module`

<!-- fdd-id-content -->
**Role**: Any internal module consuming secrets via ClientHub in-process API. Reads or writes secrets using the calling tenant's SecurityCtx.
<!-- fdd-id-content -->

#### CredStore Backend

**ID**: `fdd-credstore-actor-backend`

<!-- fdd-id-content -->
**Role**: External storage system that persists encrypted secrets. Examples: VendorA Credstore (Go service with REST API), OS protected storage (macOS Keychain, Windows DPAPI). Accessed exclusively through plugins.
<!-- fdd-id-content -->

## 3. Operational Concept & Environment

> **Note**: Project-wide runtime, OS, architecture, lifecycle policy, and integration patterns defined in root PRD. Document only module-specific deviations here.

### 3.1 Module-Specific Environment Constraints

- VendorA Credstore plugin requires network access to the Credstore Go service and valid OAuth2 client credentials
- OS protected storage plugin requires platform-specific keychain/credential APIs (macOS Keychain, Windows DPAPI)
- Only one storage plugin is active per deployment (selected by configuration)

## 4. Scope

### 4.1 In Scope

- Store, retrieve, and delete per-tenant secrets
- Sharing modes: private (default) and shared
- Hierarchical secret resolution with walk-up through tenant ancestry
- Secret shadowing (child overrides parent)
- Service-to-service retrieval with explicit tenant_id (for OAGW)
- Gateway + Plugin architecture with runtime backend selection
- VendorA Credstore REST plugin (P1)
- OS protected storage plugin (P2)
- Module-level authorization enforcement (read vs write)

### 4.2 Out of Scope

- Full Credstore RAML API parity (only subset needed)
- Encryption key management (delegated to backend)
- Automatic secret rotation or expiration
- Secret versioning or history
- Cross-tenant secret transfer (secrets cannot change ownership)
- Direct end-user access to retrieve API (service-to-service only)
- Secret listing or search operations

## 5. Functional Requirements

### 5.1 P1 — Core Operations

#### Store Secret

- [ ] `p1` - **ID**: `fdd-credstore-req-put-secret`

<!-- fdd-id-content -->
The system **MUST** allow a tenant to store a secret with a reference (key), a value, and a sharing mode. If a secret with the same reference already exists for that tenant, the value and sharing mode are updated.

**Rationale**: Core capability — tenants need to manage their own API credentials.
**Actors**: `fdd-credstore-actor-tenant-admin`, `fdd-credstore-actor-platform-module`
<!-- fdd-id-content -->

#### Retrieve Own Secret

- [ ] `p1` - **ID**: `fdd-credstore-req-get-secret`

<!-- fdd-id-content -->
The system **MUST** allow a tenant to retrieve the decrypted value of their own secret by reference. Returns the secret value or not-found if no secret with that reference exists for the tenant.

**Rationale**: Tenants need to verify or use their own stored credentials.
**Actors**: `fdd-credstore-actor-tenant-admin`, `fdd-credstore-actor-platform-module`
<!-- fdd-id-content -->

#### Delete Secret

- [ ] `p1` - **ID**: `fdd-credstore-req-delete-secret`

<!-- fdd-id-content -->
The system **MUST** allow a tenant to delete their own secret by reference. Descendants using a shared secret lose access immediately upon deletion.

**Rationale**: Tenants must be able to revoke credentials.
**Actors**: `fdd-credstore-actor-tenant-admin`, `fdd-credstore-actor-platform-module`
<!-- fdd-id-content -->

#### Tenant Scoping

- [ ] `p1` - **ID**: `fdd-credstore-req-tenant-scoping`

<!-- fdd-id-content -->
The system **MUST** derive the owning tenant from the request SecurityCtx for all CRUD operations. Tenants MUST NOT create, update, or delete secrets belonging to other tenants.

**Rationale**: Prevents cross-tenant data manipulation.
**Actors**: `fdd-credstore-actor-tenant-admin`, `fdd-credstore-actor-platform-module`
<!-- fdd-id-content -->

### 5.2 P1 — Hierarchical Sharing

#### Sharing Modes

- [ ] `p1` - **ID**: `fdd-credstore-req-sharing-modes`

<!-- fdd-id-content -->
Each secret **MUST** have a sharing mode: `private` (default) or `shared`. Private secrets are accessible only to the owning tenant. Shared secrets are accessible to the owning tenant and all descendant tenants in the hierarchy.

**Rationale**: Partners need controlled credential sharing with customers without exposing secret values.
**Actors**: `fdd-credstore-actor-tenant-admin`
<!-- fdd-id-content -->

#### Hierarchical Secret Resolution

- [ ] `p1` - **ID**: `fdd-credstore-req-hierarchical-resolve`

<!-- fdd-id-content -->
The system **MUST** support hierarchical secret resolution: given a secret reference and a tenant_id, the system walks from the specified tenant up through its ancestors (parent, grandparent, ... root), returning the first secret that matches the reference and is accessible (owned by the tenant, or shared by an ancestor). If no accessible secret is found, the system returns not-found.

**Rationale**: Enables the core business use case — OAGW retrieves a partner's shared API key when making calls on behalf of a customer.
**Actors**: `fdd-credstore-actor-oagw`
<!-- fdd-id-content -->

#### Secret Shadowing

- [ ] `p1` - **ID**: `fdd-credstore-req-secret-shadowing`

<!-- fdd-id-content -->
When a tenant owns a secret with the same reference as an ancestor's shared secret, the tenant's own secret **MUST** take precedence during hierarchical resolution. The ancestor's secret is never checked.

**Rationale**: Allows customers to override partner defaults with their own credentials.
**Actors**: `fdd-credstore-actor-oagw`
<!-- fdd-id-content -->

#### Service-to-Service Retrieval

- [ ] `p1` - **ID**: `fdd-credstore-req-service-retrieve`

<!-- fdd-id-content -->
The system **MUST** provide a retrieval operation that accepts an explicit tenant_id parameter (not derived from SecurityCtx). This operation is restricted to authorized service accounts (e.g., OAGW). The response **MUST** include the decrypted secret value, the owning tenant_id, and the sharing mode.

**Rationale**: OAGW operates as a service account and needs to retrieve secrets on behalf of arbitrary tenants.
**Actors**: `fdd-credstore-actor-oagw`
<!-- fdd-id-content -->

### 5.3 P1 — Authorization

#### Read Authorization

- [ ] `p1` - **ID**: `fdd-credstore-req-authz-read`

<!-- fdd-id-content -->
The system **MUST** require `Secrets:Read` permission for get and resolve operations.

**Rationale**: Least-privilege access control.
**Actors**: `fdd-credstore-actor-tenant-admin`, `fdd-credstore-actor-oagw`, `fdd-credstore-actor-platform-module`
<!-- fdd-id-content -->

#### Write Authorization

- [ ] `p1` - **ID**: `fdd-credstore-req-authz-write`

<!-- fdd-id-content -->
The system **MUST** require `Secrets:Write` permission for put and delete operations.

**Rationale**: Least-privilege access control.
**Actors**: `fdd-credstore-actor-tenant-admin`, `fdd-credstore-actor-platform-module`
<!-- fdd-id-content -->

#### Gateway-Level Enforcement

- [ ] `p1` - **ID**: `fdd-credstore-req-authz-gateway`

<!-- fdd-id-content -->
Authorization **MUST** be enforced in the gateway layer, not in plugins. Plugins are storage adapters and MUST NOT implement authorization logic.

**Rationale**: Prevents inconsistent authorization behavior across different backends.
**Actors**: `fdd-credstore-actor-platform-module`
<!-- fdd-id-content -->

### 5.4 P2 — Planned

#### OS Protected Storage Plugin

- [ ] `p2` - **ID**: `fdd-credstore-req-os-storage`

<!-- fdd-id-content -->
The system **MUST** provide an OS protected storage plugin for desktop/VM environments using platform-native secure storage (macOS Keychain, Windows DPAPI). This plugin supports basic get/put/delete operations. Hierarchical resolution returns only own secrets (no walk-up — single-tenant desktop environment).

**Rationale**: Desktop/VM environments lack access to VendorA Credstore.
**Actors**: `fdd-credstore-actor-platform-module`
<!-- fdd-id-content -->

#### Read-Only / Read-Write Credential Separation

- [ ] `p2` - **ID**: `fdd-credstore-req-rw-separation`

<!-- fdd-id-content -->
The VendorA Credstore plugin **MUST** support optional separate OAuth2 client credentials for read-only and read-write operations, enabling least-privilege deployment configurations.

**Rationale**: Production environments benefit from separate credentials: RO for get/resolve, RW for put/delete.
**Actors**: `fdd-credstore-actor-backend`
<!-- fdd-id-content -->

## 6. Non-Functional Requirements

### 6.1 Module-Specific NFRs

#### Secret Value Confidentiality

- [ ] `p1` - **ID**: `fdd-credstore-req-nfr-confidentiality`

<!-- fdd-id-content -->
Secret values **MUST NOT** appear in logs, error messages, or debug output at any level (gateway, plugin, transport). Secret values **MUST** be encrypted at rest in the backend storage.

**Threshold**: Zero plaintext secret values in any log output
**Rationale**: Secrets are the most sensitive data in the platform. Leaking them through logs or error messages would be a critical security incident.
**Architecture Allocation**: See DESIGN.md for implementation approach
<!-- fdd-id-content -->

## 7. Public Library Interfaces

### 7.1 Public API Surface

#### CredStoreGatewayClient

- [ ] `p1` - **ID**: `fdd-credstore-interface-gateway-client`

<!-- fdd-id-content -->
**Type**: Rust trait (async)
**Stability**: stable
**Description**: Public API for platform modules to store, retrieve, and delete secrets. Registered in ClientHub without scope. Operations: `get`, `put`, `delete`, `resolve`.
**Breaking Change Policy**: Major version bump required
<!-- fdd-id-content -->

#### CredStorePluginClient

- [ ] `p1` - **ID**: `fdd-credstore-interface-plugin-client`

<!-- fdd-id-content -->
**Type**: Rust trait (async)
**Stability**: unstable
**Description**: Plugin SPI for backend implementations. Registered in ClientHub with GTS instance scope. Operations mirror gateway client but without authorization enforcement.
**Breaking Change Policy**: Minor version bump (unstable API)
<!-- fdd-id-content -->

### 7.2 External Integration Contracts

#### REST API

- [ ] `p1` - **ID**: `fdd-credstore-contract-rest-api`

<!-- fdd-id-content -->
**Direction**: provided by library
**Protocol/Format**: HTTP/REST, JSON
**Compatibility**: Versioned URL path (`/api/credstore/v1/...`), backward-compatible within major version
<!-- fdd-id-content -->

#### VendorA Credstore REST

- [ ] `p1` - **ID**: `fdd-credstore-contract-vendor_a-rest`

<!-- fdd-id-content -->
**Direction**: required from client (outbound to VendorA Credstore)
**Protocol/Format**: HTTP/REST, JSON. OAuth2 client credentials for authentication. Credstore RAML API subset.
**Compatibility**: Plugin adapts to Credstore API version. Breaking Credstore API changes require plugin update.
<!-- fdd-id-content -->

## 8. Use Cases

#### UC-001: Partner Creates Shared Secret

- [ ] `p1` - **ID**: `fdd-credstore-uc-create-shared`

<!-- fdd-id-content -->
**Actor**: `fdd-credstore-actor-tenant-admin`

**Preconditions**:
- Tenant is authenticated with `Secrets:Write` permission

**Main Flow**:
1. Partner tenant calls put with reference `partner-openai-key`, value `sk-proj-PARTNER_KEY`, sharing `shared`
2. Gateway verifies `Secrets:Write` authorization
3. Gateway delegates to plugin
4. Plugin stores secret in backend

**Postconditions**:
- Secret is stored and accessible to partner and all descendant tenants

**Alternative Flows**:
- **Secret already exists**: Value and sharing mode are updated
<!-- fdd-id-content -->

#### UC-002: OAGW Retrieves Secret for Customer (Hierarchical Resolution)

- [ ] `p1` - **ID**: `fdd-credstore-uc-hierarchical-resolve`

<!-- fdd-id-content -->
**Actor**: `fdd-credstore-actor-oagw`

**Preconditions**:
- OAGW has valid service token with `Secrets:Read` permission
- Partner tenant has created a shared secret with reference `partner-openai-key`
- Customer is a descendant of partner in tenant hierarchy

**Main Flow**:
1. OAGW calls resolve with reference `partner-openai-key` and tenant_id `customer-123`
2. Gateway verifies service authorization
3. Gateway delegates to plugin with hierarchical resolution
4. Plugin checks customer-123 — no own secret with this reference
5. Plugin walks up to parent (partner-acme) — finds shared secret
6. Plugin returns decrypted value, owner_tenant_id `partner-acme`, sharing `shared`
7. OAGW uses the secret value for upstream API call

**Postconditions**:
- OAGW has the decrypted secret. Customer never sees the actual value.

**Alternative Flows**:
- **Customer has own secret**: Customer's secret returned (shadowing), parent not checked
- **Secret is private**: Walk-up continues to next ancestor
- **No secret in hierarchy**: Not-found error returned
<!-- fdd-id-content -->

#### UC-003: Customer Overrides Parent Secret (Shadowing)

- [ ] `p1` - **ID**: `fdd-credstore-uc-shadowing`

<!-- fdd-id-content -->
**Actor**: `fdd-credstore-actor-tenant-admin`

**Preconditions**:
- Partner has shared secret with reference `partner-openai-key`
- Customer is a descendant of partner

**Main Flow**:
1. Customer creates own secret with same reference `partner-openai-key`, value `sk-proj-CUSTOMER_KEY`, sharing `private`
2. OAGW calls resolve for `partner-openai-key`, tenant_id `customer-123`
3. System finds customer's own secret first — returns `sk-proj-CUSTOMER_KEY`
4. Partner's secret is never checked

**Postconditions**:
- Customer uses own key. Partner's shared secret remains available to other descendants.
<!-- fdd-id-content -->

#### UC-004: Private Secret Access Denied

- [ ] `p1` - **ID**: `fdd-credstore-uc-private-denied`

<!-- fdd-id-content -->
**Actor**: `fdd-credstore-actor-oagw`

**Preconditions**:
- Partner has secret with reference `internal-admin-key`, sharing `private`
- Customer is a descendant of partner

**Main Flow**:
1. OAGW calls resolve for `internal-admin-key`, tenant_id `customer-123`
2. Customer has no own secret with this reference
3. System walks up to partner — finds `internal-admin-key` but sharing is `private`
4. Walk-up continues (or reaches root) — no accessible secret found
5. System returns access-denied or not-found error

**Postconditions**:
- Customer cannot access partner's private secret
<!-- fdd-id-content -->

#### UC-005: Tenant CRUD Own Secrets

- [ ] `p1` - **ID**: `fdd-credstore-uc-crud`

<!-- fdd-id-content -->
**Actor**: `fdd-credstore-actor-tenant-admin`

**Preconditions**:
- Tenant is authenticated with appropriate permissions

**Main Flow**:
1. Tenant creates secret: put(reference, value, sharing)
2. Tenant reads secret: get(reference) — returns value
3. Tenant updates secret: put(reference, new_value, new_sharing) — overwrites
4. Tenant deletes secret: delete(reference) — removed

**Postconditions**:
- Secret lifecycle managed. Descendants of shared secrets lose access on delete.

**Alternative Flows**:
- **Get non-existent secret**: Not-found error
- **Delete non-existent secret**: Not-found error or no-op (plugin-dependent)
<!-- fdd-id-content -->

## 9. Acceptance Criteria

- [ ] Tenant can store, retrieve, and delete secrets via both ClientHub and REST API
- [ ] Shared secrets are accessible to descendant tenants via hierarchical resolution
- [ ] Private secrets are never accessible to descendants
- [ ] Secret shadowing works: child's own secret takes precedence over parent's
- [ ] OAGW can retrieve secrets on behalf of any tenant using service-to-service authentication
- [ ] Authorization is enforced at the gateway level for all operations
- [ ] Secret values never appear in log output

## 10. Dependencies

| Dependency | Description | Criticality |
|------------|-------------|-------------|
| `tenant_resolver` | Provides tenant hierarchy information (parent lookup) for hierarchical resolution | `p1` |
| OAGW | Primary consumer of service-to-service secret retrieval | `p1` |
| OAuth/token provider | Shared component for Credstore REST authentication tokens | `p1` |
| VendorA Credstore | External Go service for secret persistence (Kubernetes environments) | `p1` |
| `types_registry` | GTS-based plugin registration and discovery | `p1` |

## 11. Assumptions

- VendorA Credstore implements hierarchical secret resolution natively (the walk-up algorithm and sharing mode enforcement happen in the Credstore backend)
- Tenant hierarchy is managed externally and accessible via `tenant_resolver`
- OAGW is the only consumer of the service-to-service resolve endpoint (no direct end-user access)
- One storage plugin is active per deployment

## 12. Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Credstore API changes break plugin | Plugin stops working, secrets inaccessible | Pin Credstore API version, integration tests against Credstore |
| Secret values leaked through logs | Critical security incident | NFR enforcement, code review, log scrubbing |
| Hierarchy walk-up performance at deep nesting | Increased latency for resolve operations | Credstore backend implements efficient lookup; monitor resolution depth |
| ExternalID encoding collision | Wrong secret returned | Deterministic encoding with base64url; comprehensive test coverage |

## 13. Open Questions

- What is the exact error response when a secret exists in the hierarchy but is private — 403 (access denied) or 404 (not found)? Leaking existence may be a security concern.
- Should `resolve` support batch retrieval (multiple references in one call) for OAGW efficiency?

## 14. Traceability

- **Design**: [DESIGN.md](./DESIGN.md)
- **ADRs**: [ADR/](./ADR/)
- **Features**: [features/](./features/)
