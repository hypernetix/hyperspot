# OAGW Documentation Alignment Plan

**Date**: 2026-01-30

## Current State Analysis

### Document Inconsistencies

#### 1. Plugin Taxonomy Mismatch

**DESIGN.md** uses:

- `gts.x.core.oagw.plugin.auth.v1~` - Auth plugins
- `gts.x.core.oagw.plugin.filter.v1~` - Filter plugins (with phases: on_request, on_response, on_error)

**PRD.md** uses:

- Auth plugins
- Guard plugins (validation/policy enforcement)
- Transform plugins (mutation)

**Issue**: DESIGN has 2 types (auth + filter), PRD has 3 types (auth + guard + transform). Filter plugins in DESIGN seem to combine both guard and transform behavior.

**Decision needed**:

- Keep 2-type taxonomy (auth + filter with sub-behaviors)?
- Or 3-type taxonomy (auth + guard + transform)?

#### 2. Hierarchical Configuration Coverage

**DESIGN.md**:

- No explicit section on tenant hierarchy and sharing modes
- TODO notes mention "Per-Tenant Upstream and Route Overrides" but not covered in detail
- Feedback #6 references ADR for per-tenant overrides

**PRD.md**:

- Has "Hierarchical Configuration Override" with sharing modes (private/inherit/enforce)
- Detailed example of partner→leaf tenant credential override

**Issue**: PRD has implementation details not present in DESIGN.

#### 3. Alias Resolution and Shadowing

**DESIGN.md**:

- Has "Upstream Alias" concept with shadowing behavior
- Multi-endpoint pooling with compatibility validation
- References `adr-resource-identification.md` for details

**PRD.md**:

- Mentions "Resolves upstream by alias" but no shadowing/pooling details

**Issue**: DESIGN has richer alias model not reflected in PRD.

#### 4. Built-in Plugins List Inconsistency

**DESIGN.md** (Filter Plugins):

- logging, metrics, timeout, retry, circuit_breaker, cors, request_id

**PRD.md** splits into:

- Guard: timeout, circuit_breaker, cors
- Transform: logging, metrics, retry, request_id

**Issue**: Same plugins categorized differently due to taxonomy mismatch.

### Coverage Gaps

#### DESIGN.md "To Be Covered" Items

| #  | Topic                                                            | Status in PRD        | Notes                                                          |
|----|------------------------------------------------------------------|----------------------|----------------------------------------------------------------|
| 1  | Configuration REST / Static Type Registry / Database persistence | ❌ Not covered        | Implementation detail                                          |
| 2  | Protocol Negotiation (HTTP/1.1, HTTP/2, HTTP/3, gRPC)            | ❌ Not covered        | NFR or capability?                                             |
| 3  | Authentication                                                   | ⚠️ Partially covered | PRD has auth injection FR, DESIGN has detailed plugin examples |
| 4  | Plugin Versioning and Lifecycle Management                       | ❌ Not covered        | Operational concern                                            |
| 5  | Cache Management                                                 | ❌ Not covered        | Performance optimization                                       |
| 6  | TLS certificate pinning                                          | ❌ Not covered        | Security feature                                               |
| 7  | mTLS support                                                     | ❌ Not covered        | Security feature                                               |
| 8  | Rust ABI / Client Libraries                                      | ❌ Not covered        | SDK concern                                                    |
| 9  | Audit logging                                                    | ⚠️ Covered as NFR    | Observability requirement                                      |
| 10 | Metrics                                                          | ⚠️ Covered as NFR    | Observability requirement                                      |

#### DESIGN.md "Feedback To Be Covered" Items

| #  | Topic                                      | Status                         | Notes                                                          |
|----|--------------------------------------------|--------------------------------|----------------------------------------------------------------|
| 1  | Distinguish upstream error vs OAGW error   | ❌ Not covered                  | Error handling strategy, has ADR reference                     |
| 2  | Rate limiting strategies and algorithms    | ⚠️ Basic coverage              | PRD mentions scopes/strategies, DESIGN lacks algorithm details |
| 3  | Validation and Mutation Plugins            | ⚠️ Covered via Guard/Transform | Taxonomy-dependent                                             |
| 4  | Query plugin transformation                | ❌ Not covered                  | Plugin capability                                              |
| 5  | Body validation rules                      | ⚠️ Mentioned in NFR            | Input validation NFR exists                                    |
| 6  | Per-Tenant Upstream and Route Overrides    | ✅ Covered in PRD               | Has ADR, PRD has hierarchical override section                 |
| 7  | Retry and Circuit Breaker Strategies       | ⚠️ Mentioned as plugins        | No detailed strategy/config                                    |
| 8  | In-flight Limits and Backpressure Handling | ❌ Not covered                  | Performance/reliability feature                                |
| 9  | Rate Limit as a plugin                     | ❌ Not aligned                  | Currently rate_limit is config field, not plugin               |
| 10 | Full custom overrides                      | ⚠️ Via sharing modes           | PRD has inherit/enforce modes                                  |
| 11 | Enable/disable upstreams on tenant level   | ❌ Not covered                  | Visibility/access control                                      |
| 12 | Question of uniqueness of alias            | ⚠️ Covered in DESIGN           | Shadowing behavior addresses this                              |
| 13 | Use anonymous GTS for upstream/route       | ❌ Not covered                  | Implementation detail                                          |

### TODO Notes in DESIGN.md

Lines 941-977 contain scratch notes:

- Mention "PRD create" - but PRD now exists
- Draft rules about tenant hierarchy (1. tenant for itself, 2. partner for children, 3. parent with shared creds)
- Should be cleaned up or formalized

## Alignment Plan

### Phase 1: Resolve Taxonomy ✅ COMPLETE (2026-01-30)

**Goal**: Decide on plugin taxonomy and make consistent across both docs.

**Decision**: Adopted PRD's 3-type model (Auth + Guard + Transform) for clearer separation of concerns.

**Completed Actions**:

- [x] Updated DESIGN.md plugin taxonomy section to 3-type model
- [x] Updated DESIGN.md built-in plugins table (split filter → guard + transform)
- [x] Updated schema definitions in DESIGN.md (`plugin.filter.v1` → `plugin.guard.v1` and `plugin.transform.v1`)
- [x] Added full GTS identifiers to all plugin references in both documents
- [x] Updated PRD.md with full GTS identifiers for all plugins
- [x] Added phase clarification (guards only implement `on_request`, transforms implement `on_request/on_response/on_error`)
- [x] Split plugin examples in DESIGN.md into separate Guard and Transform examples

### Phase 2: Sync Hierarchical Configuration ✅ COMPLETE (2026-01-30)

**Goal**: Align tenant hierarchy model between docs.

**Completed Actions**:

- [x] Add "Hierarchical Configuration" section to DESIGN.md
- [x] Include sharing modes (private/inherit/enforce) with examples
- [x] Reference `adr-resource-identification.md` for detailed rules
- [x] Ensure PRD's example matches DESIGN's intended behavior
- [x] Update Upstream/Route schema in DESIGN.md to include `sharing` field
- [x] Added merge strategies for auth, rate limits, and plugins
- [x] Included configuration resolution algorithm in Go pseudocode
- [x] Added detailed partner→customer example showing auth override
- [x] Documented secret access control integration with `cred_store`
- [x] Added permissions and access control section

### Phase 3: Sync Alias Resolution ✅ COMPLETE (2026-01-30)

**Goal**: Add alias resolution details to PRD and ensure consistency across all docs.

**Completed Actions**:

- [x] Add "Alias Resolution and Shadowing" section to PRD
- [x] Include multi-endpoint pooling explanation
- [x] Add compatibility validation rules (protocol, scheme, port must match)
- [x] Reference ADR for implementation details
- [x] Documented enforced alias rules with 3-scenario table (single host, common suffix, explicit)
- [x] Included alias auto-generation algorithm
- [x] Added examples for all scenarios (single host, multi-region, IP-based, heterogeneous)
- [x] Documented enforced limits across shadowing
- [x] Updated ADR with common suffix extraction algorithm
- [x] Added "Alias Resolution" section to DESIGN.md
- [x] Updated Upstream schema description in DESIGN.md for alias field

### Phase 4: Address "To Be Covered" Items (Priority: Variable)

#### High Priority (Include in PRD)

- [ ] **mTLS support** - Add to capabilities and NFR (security)
- [ ] **TLS certificate pinning** - Add to NFR (security, SSRF protection)
- [ ] **Protocol negotiation** - Add to capabilities or NFR
- [ ] **Enable/disable upstreams** - Add to hierarchical configuration (visibility control)
- [ ] **In-flight limits** - Add to rate limiting section (backpressure)

#### Medium Priority (Add to DESIGN.md)

- [ ] **Retry strategies** - Detail retry plugin config (exponential backoff, max attempts, retryable status codes)
- [ ] **Circuit breaker strategies** - Detail circuit breaker config (failure threshold, timeout, half-open state)
- [ ] **Rate limit algorithms** - Specify token bucket, leaky bucket, sliding window (currently vague)
- [ ] **Error source distinction** - Reference ADR, add to error handling section

#### Low Priority (Implementation Details - defer or add to separate docs)

- [ ] Configuration REST API implementation
- [ ] Database schema and persistence
- [ ] Plugin versioning and lifecycle
- [ ] Cache management strategy
- [ ] Rust ABI / Client libraries
- [ ] Anonymous GTS usage

### Phase 5: Address "Feedback To Be Covered" Items (Priority: Variable)

#### Resolved (Mark as done)

- [x] **#6 Per-Tenant Overrides** - Covered in PRD, has ADR
- [x] **#12 Alias uniqueness** - Covered in DESIGN with shadowing

#### Needs Action

- [ ] **#1 Error source distinction** - Add section referencing ADR `adr-error-source-distinction.md`
- [ ] **#4 Query transformation in plugins** - Add to Starlark plugin context API
- [ ] **#9 Rate limit as plugin** - Decide: keep as config field or make pluggable? (Breaking change if changed)

### Phase 6: Cleanup (Priority: Low)

**Actions**:

- [ ] Remove TODO section from DESIGN.md (lines 941-977)
- [ ] Remove "To Be Covered" section after items are addressed or deferred
- [ ] Remove "Feedback To Be Covered" section after items are addressed
- [ ] Add "Future Considerations" section for deferred items

## Proposed Document Structure Post-Alignment

### PRD.md

```
1. Overview (purpose, concepts, users, problems, success criteria)
2. Actors
3. Functional Requirements
   - Upstream/Route/Plugin Management
   - Request Proxying
   - Auth Injection
   - Rate Limiting (+ in-flight limits, backpressure)
   - Header Transformation
   - Plugin System (3-type taxonomy)
   - Streaming
   - Configuration Layering
   - Hierarchical Override (with sharing modes)
   - Alias Resolution and Shadowing
   - Upstream Visibility Control (enable/disable)
4. Use Cases
5. Non-Functional Requirements
   - Latency, Availability, Security (SSRF, TLS, mTLS), Observability, Sandbox, Multi-tenancy
6. Built-in Plugins (auth, guard, transform)
7. Error Codes
8. API Endpoints
9. Dependencies
```

### DESIGN.md

```
1. Context
2. Architecture
   - Dependencies
   - Key Concepts
   - Out of Scope
   - Security Considerations
3. Routing (detailed flow)
4. Plugins System
   - 3-type taxonomy (auth, guard, transform)
   - Execution order
   - Starlark context API (+ query transformation)
   - Sandbox restrictions
5. Types Definitions
   - Upstream (+ sharing field)
   - Route (+ sharing field)
   - Auth Plugin
   - Guard Plugin
   - Transform Plugin
   - Built-in plugin configs (retry strategy, circuit breaker strategy)
6. Hierarchical Configuration (sharing modes, merge algorithm)
7. Alias Resolution (shadowing, pooling, compatibility)
8. API Endpoints (with examples)
9. Error Handling (+ error source distinction, reference ADR)
10. Rate Limiting (algorithms: token bucket, sliding window)
11. Future Considerations (deferred items from "To Be Covered")
```

## Decision Log

| Decision                          | Rationale                          | Status   |
|-----------------------------------|------------------------------------|----------|
| Adopt 3-type plugin taxonomy      | Clearer separation of concerns     | Proposed |
| Add hierarchical config to DESIGN | Missing implementation detail      | Proposed |
| Add alias resolution to PRD       | PRD lacks discovery/routing detail | Proposed |
| Defer cache management            | Optimization, not MVP              | Proposed |
| Defer plugin versioning           | Operational complexity             | Proposed |
| Add mTLS to PRD                   | Security capability                | Proposed |
| Keep rate_limit as config field   | Plugin model adds complexity       | Proposed |

## Next Steps

1. **Review this plan** - Confirm decisions and priorities
2. **Phase 1 execution** - Resolve plugin taxonomy
3. **Phase 2 execution** - Sync hierarchical configuration
4. **Incremental updates** - Address phases 3-6 iteratively
5. **Final review** - Ensure both docs consistent and complete
