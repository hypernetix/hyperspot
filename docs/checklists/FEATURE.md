# Spec Expert Checklist

**Artifact**: Spec (SPEC)
**Version**: 2.0
**Last Updated**: 2026-02-03
**Purpose**: Comprehensive quality checklist for specs

---

## Referenced Standards

This checklist validates specs based on the following international standards:

| Standard | Domain | Description |
|----------|--------|-------------|
| [IEEE 1016-2009](https://standards.ieee.org/ieee/1016/4502/) | **Design Description** | Software Design Descriptions — detailed design viewpoint, design entities |
| [ISO/IEC/IEEE 29148:2018](https://www.iso.org/standard/72089.html) | **Requirements Notation** | Requirements engineering — behavioral requirements, shall notation, traceability |
| [ISO/IEC 25010:2011](https://www.iso.org/standard/35733.html) | **Quality Model** | SQuaRE — 8 quality characteristics: performance, security, reliability, maintainability |
| [ISO/IEC/IEEE 29119-3:2021](https://www.iso.org/standard/79429.html) | **Test Documentation** | Software testing — test specification, acceptance criteria |
| [OWASP ASVS 5.0](https://owasp.org/www-project-application-security-verification-standard/) | **Security Verification** | Application security requirements — authentication, authorization, input validation |
| [WCAG 2.2](https://www.w3.org/TR/WCAG22/) | **Accessibility** | Web Content Accessibility Guidelines — POUR principles, Level AA |

---

## Table of Contents

1. [Review Scope Selection](#review-scope-selection)
2. [Prerequisites](#prerequisites)
3. [Applicability Context](#applicability-context)
4. [Severity Dictionary](#severity-dictionary)
5. [MUST HAVE](#must-have)
   - [ARCHITECTURE Expertise (ARCH)](#️-architecture-expertise-arch)
   - [PERFORMANCE Expertise (PERF)](#-performance-expertise-perf)
   - [SECURITY Expertise (SEC)](#-security-expertise-sec)
   - [RELIABILITY Expertise (REL)](#️-reliability-expertise-rel)
   - [DATA Expertise (DATA)](#-data-expertise-data)
   - [INTEGRATION Expertise (INT)](#-integration-expertise-int)
   - [OPERATIONS Expertise (OPS)](#️-operations-expertise-ops)
   - [MAINTAINABILITY Expertise (MAINT)](#-maintainability-expertise-maint)
   - [TESTING Expertise (TEST)](#-testing-expertise-test)
   - [COMPLIANCE Expertise (COMPL)](#-compliance-expertise-compl)
   - [USABILITY Expertise (UX)](#-usability-expertise-ux)
   - [BUSINESS Expertise (BIZ)](#-business-expertise-biz)
   - [Semantic Alignment (SEM)](#semantic-alignment-sem)
   - [Deliberate Omissions](#deliberate-omissions)
6. [MUST NOT HAVE](#must-not-have)
7. [Validation Summary](#validation-summary)
   - [Final Checklist](#final-checklist)
   - [Reporting Readiness Checklist](#reporting-readiness-checklist)
   - [Reporting](#reporting)

---

## Review Scope Selection

**Choose review mode based on feature complexity and risk**:

| Review Mode | When to Use | Domains to Check |
|-------------|-------------|------------------|
| **Quick** | Simple CRUD, minor updates | ARCH (core) + BIZ + changed domains |
| **Standard** | New spec, moderate complexity | All applicable domains |
| **Full** | Security-sensitive, complex logic | All 12 domains with evidence |

### Quick Review (Core Items Only)

**MUST CHECK** (blocking):
- [ ] ARCH-FDESIGN-001: Spec Context Completeness
- [ ] ARCH-FDESIGN-003: Actor Flow Completeness
- [ ] BIZ-FDESIGN-001: Requirements Alignment
- [ ] DOC-FDESIGN-001: Explicit Non-Applicability

**Changed sections** — also check relevant domain items for any sections modified.

### Domain Prioritization by Spec Type

| Spec Type | Priority Domains (check first) | Secondary Domains | Often N/A |
|--------------|-------------------------------|-------------------|-----------|
| **User-facing UI** | ARCH, UX, SEC, TEST | PERF, REL, DATA | OPS, INT, COMPL |
| **Backend API** | ARCH, SEC, DATA, INT | PERF, REL, TEST | UX, COMPL |
| **Data Processing** | ARCH, DATA, PERF, REL | INT, TEST | SEC, UX, OPS, COMPL |
| **CLI Command** | ARCH, MAINT, TEST | DATA, INT | SEC, PERF, UX, OPS, COMPL |
| **Integration/Webhook** | ARCH, INT, SEC, REL | DATA, TEST | UX, PERF, OPS, COMPL |
| **Auth/Security** | SEC, ARCH, DATA, REL | TEST, COMPL | UX, PERF, OPS, INT |

**Applicability Rule**: Domains in "Often N/A" column still require explicit "Not applicable because..." statement in document if skipped.

---

## Prerequisites

Before starting the review, confirm:

- [ ] I understand this checklist validates FEATURE artifacts
- [ ] I will follow the Applicability Context rules below
- [ ] I will check ALL items in MUST HAVE sections
- [ ] I will verify ALL items in MUST NOT HAVE sections
- [ ] I will document any violations found
- [ ] I will provide specific feedback for each failed check
- [ ] I will complete the Final Checklist and provide a review report

---

## Applicability Context

Before evaluating each checklist item, the expert MUST:

1. **Understand the spec's domain** — What kind of feature is this? (e.g., user-facing UI spec, backend API, data processing pipeline, CLI command)

2. **Determine applicability for each requirement** — Not all checklist items apply to all specs:
   - A simple CRUD spec may not need complex State Management analysis
   - A read-only spec may not need Data Integrity analysis
   - A CLI spec may not need UI/UX analysis

3. **Require explicit handling** — For each checklist item:
   - If applicable: The document MUST address it (present and complete)
   - If not applicable: The document MUST explicitly state "Not applicable because..." with reasoning
   - If missing without explanation: Report as violation

4. **Never skip silently** — The expert MUST NOT skip a requirement just because it's not mentioned. Either:
   - The requirement is met (document addresses it), OR
   - The requirement is explicitly marked not applicable (document explains why), OR
   - The requirement is violated (report it with applicability justification)

**Key principle**: The reviewer must be able to distinguish "author considered and excluded" from "author forgot"

---

## Severity Dictionary

- **CRITICAL**: Unsafe/misleading/unverifiable; blocks downstream work.
- **HIGH**: Major ambiguity/risk; should be fixed before approval.
- **MEDIUM**: Meaningful improvement; fix when feasible.
- **LOW**: Minor improvement; optional.

---

# MUST HAVE

---

## 🏗️ ARCHITECTURE Expertise (ARCH)

> **Standard**: [IEEE 1016-2009](https://standards.ieee.org/ieee/1016/4502/) — Software Design Descriptions
>
> Design entities require: identification, type, purpose, function, subordinates, dependencies, resources, processing, data (§5.4)

### ARCH-FDESIGN-001: Spec Context Completeness
**Severity**: CRITICAL
**Ref**: IEEE 1016-2009 §5.4.1 (Design entity attributes)

- [ ] Spec identifier is present and stable (unique within the project)
- [ ] Spec status documented
- [ ] Overall Design reference present
- [ ] Requirements source reference present
- [ ] Actors/user roles are defined and referenced consistently
- [ ] Spec scope clearly stated
- [ ] Spec boundaries explicit
- [ ] Out-of-scope items documented

### ARCH-FDESIGN-002: Overall Design Alignment
**Severity**: CRITICAL

- [ ] Any shared types/schemas are referenced from a canonical source (architecture doc, schema repo, API contract)
- [ ] Any shared APIs/contracts are referenced from a canonical source (API documentation/spec)
- [ ] Architectural decisions are consistent with the architecture and design baseline (if it exists)
- [ ] Domain concepts are referenced consistently with the canonical domain model (if it exists)
- [ ] API endpoints/contracts are referenced consistently with the canonical API documentation (if it exists)
- [ ] Principles compliance documented

### ARCH-FDESIGN-003: Actor Flow Completeness
**Severity**: CRITICAL

- [ ] A flows/user-journeys section exists and is sufficiently detailed
- [ ] All user-facing functionality has actor flows
- [ ] Each flow has a unique name/identifier within the document
- [ ] Flows cover happy path
- [ ] Flows cover error paths
- [ ] Flows cover edge cases
- [ ] Actor/user roles are defined consistently with the requirements document

### ARCH-FDESIGN-004: Algorithm Completeness
**Severity**: CRITICAL

- [ ] A algorithms/business-rules section exists and is sufficiently detailed
- [ ] All business logic has algorithms
- [ ] Each algorithm has a unique name/identifier within the document
- [ ] Algorithms are deterministic and testable
- [ ] Input/output clearly defined
- [ ] Error handling documented
- [ ] Edge cases addressed

### ARCH-FDESIGN-005: State Management
**Severity**: HIGH

- [ ] A states/state-machine section exists when stateful behavior is present (can be minimal)
- [ ] Stateful components have state definitions
- [ ] State transitions define explicit triggers/conditions
- [ ] Valid states enumerated
- [ ] Transition guards documented
- [ ] Invalid state transitions documented
- [ ] State persistence documented (if applicable)

### ARCH-FDESIGN-006: Component Interaction
**Severity**: HIGH

- [ ] Inter-component interactions documented
- [ ] Service calls documented
- [ ] Event emissions documented
- [ ] Data flow between components clear
- [ ] Async operations documented
- [ ] Timeout handling documented

### ARCH-FDESIGN-007: Extension Points
**Severity**: MEDIUM

- [ ] Customization points identified
- [ ] Plugin/hook opportunities documented
- [ ] Configuration options documented
- [ ] Spec flags integration documented
- [ ] Versioning considerations documented

---

## Semantic Alignment (SEM)

> **Standard**: [ISO/IEC/IEEE 29148:2018](https://www.iso.org/standard/72089.html) — Requirements Engineering
>
> "Each requirement shall be traceable bidirectionally... uniquely identified" (§5.2.8, §6.5)

### SEM-FDESIGN-001: PRD Coverage Integrity
**Severity**: CRITICAL
**Ref**: ISO/IEC/IEEE 29148:2018 §6.5 (Traceability)

- [ ] All referenced PRD FR/NFR IDs are valid and correctly cited
- [ ] Spec requirements do not contradict PRD scope, priorities, or constraints
- [ ] Spec outcomes preserve PRD intent and success criteria
- [ ] Any PRD trade-offs are explicitly documented and approved

### SEM-FDESIGN-002: Design Principles and Constraints
**Severity**: CRITICAL

- [ ] Spec design adheres to design principles referenced in the Overall Design
- [ ] Spec design respects all design constraints and does not bypass them
- [ ] Any constraint exception is explicitly documented with rationale

### SEM-FDESIGN-003: Architecture and Component Consistency
**Severity**: HIGH

- [ ] Spec responsibilities align with component boundaries in the Overall Design
- [ ] Interactions and sequences match the system interaction design
- [ ] Data models and entities conform to the Overall Design domain model
- [ ] API contracts and integration boundaries match the Overall Design

### SEM-FDESIGN-004: Spec Semantics Completeness
**Severity**: HIGH

- [ ] Actor flows, algorithms, and state machines are consistent with the design context
- [ ] Definition of Done mappings cover required design references (principles, constraints, components, sequences, tables)
- [ ] Any semantic deviation from design is documented and approved

### SEM-FDESIGN-005: Design Decomposition Consistency
**Severity**: HIGH

- [ ] Spec ID matches the entry in the DECOMPOSITION
- [ ] Purpose, scope, and out-of-scope items align with the DECOMPOSITION entry
- [ ] Dependencies in the feature design match the DECOMPOSITION dependency list
- [ ] Requirements covered (FR/NFR) match the DECOMPOSITION mapping
- [ ] Design principles and constraints covered match the DECOMPOSITION mapping
- [ ] Domain entities, components, APIs, sequences, and data tables match the DECOMPOSITION entry

---

## ⚡ PERFORMANCE Expertise (PERF)

> **Standard**: [ISO/IEC 25010:2011](https://www.iso.org/standard/35733.html) — Performance Efficiency
>
> Sub-characteristics: time behavior, resource utilization, capacity under defined conditions

### PERF-FDESIGN-001: Performance-Critical Paths
**Severity**: HIGH
**Ref**: ISO/IEC 25010:2011 §4.2.2 (Performance efficiency)

- [ ] Hot paths identified
- [ ] Latency-sensitive operations marked
- [ ] Caching strategy documented
- [ ] Batch processing opportunities identified
- [ ] N+1 query prevention addressed
- [ ] Database query optimization documented

### PERF-FDESIGN-002: Resource Management
**Severity**: HIGH

- [ ] Memory allocation patterns documented
- [ ] Connection pooling documented
- [ ] Resource cleanup documented
- [ ] Large data handling documented
- [ ] Streaming approaches documented (if applicable)
- [ ] Pagination documented (if applicable)

### PERF-FDESIGN-003: Scalability Considerations
**Severity**: MEDIUM

- [ ] Concurrent access handling documented
- [ ] Lock contention minimized
- [ ] Stateless patterns used where possible
- [ ] Horizontal scaling support documented
- [ ] Rate limiting handled
- [ ] Throttling documented

### PERF-FDESIGN-004: Performance Acceptance Criteria
**Severity**: MEDIUM

- [ ] Response time targets stated
- [ ] Throughput targets stated
- [ ] Resource usage limits stated
- [ ] Performance test requirements documented
- [ ] Baseline metrics identified

---

## 🔒 SECURITY Expertise (SEC)

> **Standards**:
> - [ISO/IEC 25010:2011](https://www.iso.org/standard/35733.html) — Security: confidentiality, integrity, non-repudiation, accountability, authenticity
> - [OWASP ASVS 5.0](https://owasp.org/www-project-application-security-verification-standard/) — Application Security Verification Standard

### SEC-FDESIGN-001: Authentication Integration
**Severity**: CRITICAL
**Ref**: OWASP ASVS V2 (Authentication), ISO 25010 §4.2.6 (Authenticity)

- [ ] Authentication requirements documented
- [ ] Session handling documented
- [ ] Token validation documented
- [ ] Authentication failure handling documented
- [ ] Multi-factor requirements documented (if applicable)
- [ ] Service-to-service auth documented (if applicable)

### SEC-FDESIGN-002: Authorization Implementation
**Severity**: CRITICAL

- [ ] Permission checks documented in flows
- [ ] Role-based access documented
- [ ] Resource-level authorization documented
- [ ] Authorization failure handling documented
- [ ] Privilege escalation prevention documented
- [ ] Cross-tenant access prevention documented (if applicable)

### SEC-FDESIGN-003: Input Validation
**Severity**: CRITICAL
**Ref**: OWASP ASVS V5 (Validation, Sanitization), ISO 25010 §4.2.6 (Integrity)

- [ ] All inputs validated
- [ ] Validation rules documented
- [ ] Validation failure handling documented
- [ ] SQL injection prevention documented
- [ ] XSS prevention documented
- [ ] Command injection prevention documented
- [ ] Path traversal prevention documented

### SEC-FDESIGN-004: Data Protection
**Severity**: CRITICAL

- [ ] Sensitive data handling documented
- [ ] PII handling documented
- [ ] Encryption requirements documented
- [ ] Data masking documented (if applicable)
- [ ] Secure data transmission documented
- [ ] Data sanitization documented

### SEC-FDESIGN-005: Audit Trail
**Severity**: HIGH

- [ ] Auditable actions identified
- [ ] Audit logging documented
- [ ] User attribution documented
- [ ] Timestamp handling documented
- [ ] Audit data retention documented
- [ ] Non-repudiation requirements documented

### SEC-FDESIGN-006: Security Error Handling
**Severity**: HIGH

- [ ] Security errors don't leak information
- [ ] Error messages are safe
- [ ] Stack traces hidden from users
- [ ] Timing attacks mitigated
- [ ] Rate limiting on security operations documented

---

## 🛡️ RELIABILITY Expertise (REL)

> **Standard**: [ISO/IEC 25010:2011](https://www.iso.org/standard/35733.html) — Reliability
>
> Sub-characteristics: maturity, availability, fault tolerance, recoverability

### REL-FDESIGN-001: Error Handling Completeness
**Severity**: CRITICAL
**Ref**: ISO/IEC 25010:2011 §4.2.5 (Fault tolerance, Recoverability)

- [ ] All error conditions identified
- [ ] Error classification documented
- [ ] Recovery actions documented
- [ ] Error propagation documented
- [ ] User-facing error messages documented
- [ ] Logging requirements documented

### REL-FDESIGN-002: Fault Tolerance
**Severity**: HIGH

- [ ] External dependency failures handled
- [ ] Timeout handling documented
- [ ] Retry logic documented
- [ ] Circuit breaker integration documented
- [ ] Fallback behavior documented
- [ ] Graceful degradation documented

### REL-FDESIGN-003: Data Integrity
**Severity**: CRITICAL
**Ref**: ISO/IEC 25010:2011 §4.2.6.2 (Integrity)

- [ ] Transaction boundaries documented
- [ ] Consistency guarantees documented
- [ ] Concurrent modification handling documented
- [ ] Idempotency documented (where applicable)
- [ ] Data validation before persistence documented
- [ ] Rollback scenarios documented

### REL-FDESIGN-004: Resilience Patterns
**Severity**: MEDIUM

- [ ] Bulkhead patterns documented (if applicable)
- [ ] Backpressure handling documented
- [ ] Queue overflow handling documented
- [ ] Resource exhaustion handling documented
- [ ] Deadlock prevention documented

### REL-FDESIGN-005: Recovery Procedures
**Severity**: MEDIUM

- [ ] Recovery from partial failure documented
- [ ] Data reconciliation documented
- [ ] Manual intervention procedures documented
- [ ] Compensating transactions documented (if applicable)
- [ ] State recovery documented

---

## 📊 DATA Expertise (DATA)

### DATA-FDESIGN-001: Data Access Patterns
**Severity**: HIGH

- [ ] Read patterns documented
- [ ] Write patterns documented
- [ ] Query patterns documented
- [ ] Index usage documented
- [ ] Join patterns documented
- [ ] Aggregation patterns documented

### DATA-FDESIGN-002: Data Validation
**Severity**: CRITICAL

- [ ] Business rule validation documented
- [ ] Format validation documented
- [ ] Range validation documented
- [ ] Referential integrity validation documented
- [ ] Uniqueness validation documented
- [ ] Validation error messages documented

### DATA-FDESIGN-003: Data Transformation
**Severity**: HIGH

- [ ] Input transformation documented
- [ ] Output transformation documented
- [ ] Data mapping documented
- [ ] Format conversion documented
- [ ] Null handling documented
- [ ] Default value handling documented

### DATA-FDESIGN-004: Data Lifecycle
**Severity**: MEDIUM

- [ ] Data creation documented
- [ ] Data update documented
- [ ] Data deletion documented
- [ ] Data archival documented (if applicable)
- [ ] Data retention compliance documented
- [ ] Data migration considerations documented

### DATA-FDESIGN-005: Data Privacy
**Severity**: HIGH (if applicable)

- [ ] PII handling documented
- [ ] Data minimization applied
- [ ] Consent handling documented
- [ ] Data subject rights support documented
- [ ] Cross-border transfer handling documented
- [ ] Anonymization/pseudonymization documented

---

## 🔌 INTEGRATION Expertise (INT)

### INT-FDESIGN-001: API Interactions
**Severity**: HIGH

- [ ] API calls documented with method + path
- [ ] Request construction documented
- [ ] Response handling documented
- [ ] Error response handling documented
- [ ] Rate limiting handling documented
- [ ] Retry behavior documented

### INT-FDESIGN-002: Database Operations
**Severity**: HIGH

- [ ] DB operations documented with operation + table
- [ ] Query patterns documented
- [ ] Transaction usage documented
- [ ] Connection management documented
- [ ] Query parameterization documented
- [ ] Result set handling documented

### INT-FDESIGN-003: External Integrations
**Severity**: HIGH (if applicable)

- [ ] External system calls documented
- [ ] Integration authentication documented
- [ ] Timeout configuration documented
- [ ] Failure handling documented
- [ ] Data format translation documented
- [ ] Version compatibility documented

### INT-FDESIGN-004: Event/Message Handling
**Severity**: MEDIUM (if applicable)

- [ ] Event publishing documented
- [ ] Event consumption documented
- [ ] Message format documented
- [ ] Ordering guarantees documented
- [ ] Delivery guarantees documented
- [ ] Dead letter handling documented

### INT-FDESIGN-005: Cache Integration
**Severity**: MEDIUM (if applicable)

- [ ] Cache read patterns documented
- [ ] Cache write patterns documented
- [ ] Cache invalidation documented
- [ ] Cache miss handling documented
- [ ] Cache TTL documented
- [ ] Cache consistency documented

---

## 🖥️ OPERATIONS Expertise (OPS)

### OPS-FDESIGN-001: Observability
**Severity**: HIGH

- [ ] Logging points documented
- [ ] Log levels documented
- [ ] Metrics collection documented
- [ ] Tracing integration documented
- [ ] Correlation ID handling documented
- [ ] Debug information documented

### OPS-FDESIGN-002: Configuration
**Severity**: MEDIUM

- [ ] Configuration parameters documented
- [ ] Default values documented
- [ ] Configuration validation documented
- [ ] Runtime configuration documented
- [ ] Environment-specific configuration documented
- [ ] Spec flags documented

### OPS-FDESIGN-003: Health & Diagnostics
**Severity**: MEDIUM

- [ ] Health check contributions documented
- [ ] Diagnostic endpoints documented
- [ ] Self-healing behavior documented
- [ ] Troubleshooting guidance documented
- [ ] Common issues documented

### OPS-FDESIGN-004: Rollout & Rollback
**Severity**: HIGH

- [ ] Rollout strategy is documented (phased rollout, feature flag, etc.) when applicable
- [ ] Rollback strategy is documented
- [ ] Data migration/backward compatibility considerations are addressed when applicable

---

## 🔧 MAINTAINABILITY Expertise (MAINT)

> **Standard**: [ISO/IEC 25010:2011](https://www.iso.org/standard/35733.html) — Maintainability
>
> Sub-characteristics: modularity, reusability, analysability, modifiability, testability

### MAINT-FDESIGN-001: Code Organization
**Severity**: MEDIUM
**Ref**: ISO/IEC 25010:2011 §4.2.7 (Modularity, Modifiability)

- [ ] Module structure implied
- [ ] Separation of concerns evident
- [ ] Single responsibility evident
- [ ] Dependency injection opportunities identified
- [ ] Interface boundaries clear

### MAINT-FDESIGN-002: Documentation Quality
**Severity**: MEDIUM

- [ ] Flows self-documenting
- [ ] Complex logic explained
- [ ] Business rules documented
- [ ] Assumptions documented
- [ ] Edge cases documented
- [ ] Examples provided where helpful

### MAINT-FDESIGN-003: Technical Debt Awareness
**Severity**: MEDIUM

- [ ] Known limitations documented
- [ ] Workarounds documented
- [ ] Future improvement opportunities noted
- [ ] Deprecation plans documented (if applicable)
- [ ] Migration considerations documented

---

## 🧪 TESTING Expertise (TEST)

> **Standards**:
> - [ISO/IEC/IEEE 29119-3:2021](https://www.iso.org/standard/79429.html) — Test documentation templates
> - [ISO/IEC 25010:2011](https://www.iso.org/standard/35733.html) §4.2.7.5 — Testability sub-characteristic

### TEST-FDESIGN-001: Testability
**Severity**: HIGH
**Ref**: ISO/IEC 25010:2011 §4.2.7.5 (Testability), ISO/IEC/IEEE 29119-3:2021

- [ ] Flows are testable (deterministic, observable)
- [ ] Algorithms are testable (clear inputs/outputs)
- [ ] States are testable (verifiable transitions)
- [ ] Mock boundaries clear
- [ ] Test data requirements documented
- [ ] Test isolation achievable

### TEST-FDESIGN-002: Test Coverage Guidance
**Severity**: MEDIUM

- [ ] Unit test targets identified
- [ ] Integration test targets identified
- [ ] E2E test scenarios documented
- [ ] Edge case tests identified
- [ ] Error path tests identified
- [ ] Performance test targets identified

### TEST-FDESIGN-003: Acceptance Criteria
**Severity**: HIGH

- [ ] Each requirement has verifiable criteria
- [ ] Criteria are unambiguous
- [ ] Criteria are measurable
- [ ] Criteria cover happy path
- [ ] Criteria cover error paths
- [ ] Criteria testable automatically

---

## 📜 COMPLIANCE Expertise (COMPL)

### COMPL-FDESIGN-001: Regulatory Compliance
**Severity**: HIGH (if applicable)

- [ ] Compliance requirements addressed
- [ ] Audit trail requirements met
- [ ] Data handling compliant
- [ ] Consent handling compliant
- [ ] Retention requirements met
- [ ] Reporting requirements addressed

### COMPL-FDESIGN-002: Privacy Compliance
**Severity**: HIGH (if applicable)

- [ ] Privacy by design evident
- [ ] Data minimization applied
- [ ] Purpose limitation documented
- [ ] Consent handling documented
- [ ] Data subject rights supported
- [ ] Cross-border considerations addressed

---

## 👤 USABILITY Expertise (UX)

> **Standards**:
> - [ISO/IEC 25010:2011](https://www.iso.org/standard/35733.html) — Usability: learnability, operability, user error protection, accessibility
> - [WCAG 2.2](https://www.w3.org/TR/WCAG22/) — Web Content Accessibility Guidelines (Level AA)

### UX-FDESIGN-001: User Experience Flows
**Severity**: MEDIUM
**Ref**: ISO/IEC 25010:2011 §4.2.4 (Usability)

- [ ] User journey clear
- [ ] Feedback points documented
- [ ] Error messages user-friendly
- [ ] Loading states documented
- [ ] Progress indication documented
- [ ] Confirmation flows documented

### UX-FDESIGN-002: Accessibility
**Severity**: MEDIUM (if applicable)
**Ref**: [WCAG 2.2](https://www.w3.org/TR/WCAG22/) Level AA, ISO/IEC 25010:2011 §4.2.4.6 (Accessibility)

- [ ] Accessibility requirements addressed
- [ ] Keyboard navigation supported
- [ ] Screen reader support documented
- [ ] Color contrast considered
- [ ] Focus management documented

---

## 🏢 BUSINESS Expertise (BIZ)

> **Standard**: [ISO/IEC/IEEE 29148:2018](https://www.iso.org/standard/72089.html) — Requirements Engineering
>
> "Requirements shall be necessary, implementation-free, unambiguous, consistent, complete, singular, feasible, traceable, verifiable" (§5.2)

### BIZ-FDESIGN-001: Requirements Alignment
**Severity**: CRITICAL
**Ref**: ISO/IEC/IEEE 29148:2018 §5.2 (Characteristics of requirements)

- [ ] All feature requirements (Section E) documented
- [ ] Requirements trace to PRD
- [ ] Requirements trace to a roadmap/backlog item (if used)
- [ ] Business rules accurately captured
- [ ] Edge cases reflect business reality
- [ ] Acceptance criteria business-verifiable

### BIZ-FDESIGN-002: Value Delivery
**Severity**: HIGH

- [ ] Spec delivers stated value
- [ ] User needs addressed
- [ ] Business process supported
- [ ] Success metrics achievable
- [ ] ROI evident

---

## Deliberate Omissions

### DOC-FDESIGN-001: Explicit Non-Applicability
**Severity**: CRITICAL

- [ ] If a section or requirement is intentionally omitted, it is explicitly stated in the document (e.g., "Not applicable because...")
- [ ] No silent omissions — every major checklist area is either present or has a documented reason for absence
- [ ] Reviewer can distinguish "author considered and excluded" from "author forgot"

---

# MUST NOT HAVE

---

## ❌ ARCH-FDESIGN-NO-001: No System-Level Type Redefinitions
**Severity**: CRITICAL

**What to check**:
- [ ] No new system-wide entity/type definitions (define once in a canonical place)
- [ ] No new value object definitions
- [ ] No domain model changes
- [ ] No schema definitions
- [ ] No type aliases

**Where it belongs**: Central domain model / schema documentation

---

## ❌ ARCH-FDESIGN-NO-002: No New API Endpoints
**Severity**: CRITICAL

**What to check**:
- [ ] No new endpoint definitions
- [ ] No new API contracts
- [ ] No request/response schema definitions
- [ ] No new HTTP methods on existing endpoints
- [ ] Reference existing endpoints by ID only

**Where it belongs**: API contract documentation (e.g., OpenAPI)

---

## ❌ ARCH-FDESIGN-NO-003: No Architectural Decisions
**Severity**: HIGH

**What to check**:
- [ ] No "we chose X over Y" discussions
- [ ] No pattern selection justifications
- [ ] No technology choice explanations
- [ ] No pros/cons analysis
- [ ] No decision debates

**Where it belongs**: `ADR`

---

## ❌ BIZ-FDESIGN-NO-001: No Product Requirements
**Severity**: HIGH

**What to check**:
- [ ] No actor definitions (reference PRD)
- [ ] No functional requirement definitions (reference PRD)
- [ ] No use case definitions (reference PRD)
- [ ] No NFR definitions (reference PRD)
- [ ] No business vision

**Where it belongs**: `PRD`

---

## ❌ BIZ-FDESIGN-NO-002: No Sprint/Task Breakdowns
**Severity**: HIGH

**What to check**:
- [ ] No sprint assignments
- [ ] No task lists beyond phases
- [ ] No effort estimates
- [ ] No developer assignments
- [ ] No timeline estimates
- [ ] No Jira/Linear ticket references

**Where it belongs**: Project management tools

---

## ❌ MAINT-FDESIGN-NO-001: No Code Snippets
**Severity**: HIGH

**What to check**:
- [ ] No production code
- [ ] No code diffs
- [ ] No implementation code
- [ ] No configuration file contents
- [ ] No SQL queries (describe operations instead)
- [ ] No API request/response JSON

**Where it belongs**: Source code repository

---

## ❌ TEST-FDESIGN-NO-001: No Test Implementation
**Severity**: MEDIUM

**What to check**:
- [ ] No test code
- [ ] No test scripts
- [ ] No test data files
- [ ] No assertion implementations
- [ ] No mock implementations

**Where it belongs**: Test directories in source code

---

## ❌ SEC-FDESIGN-NO-001: No Security Secrets
**Severity**: CRITICAL

**What to check**:
- [ ] No API keys
- [ ] No passwords
- [ ] No certificates
- [ ] No encryption keys
- [ ] No connection strings with credentials
- [ ] No tokens

**Where it belongs**: Secret management system

---

## ❌ OPS-FDESIGN-NO-001: No Infrastructure Code
**Severity**: MEDIUM

**What to check**:
- [ ] No Terraform/CloudFormation
- [ ] No Kubernetes manifests
- [ ] No Docker configurations
- [ ] No CI/CD pipeline definitions
- [ ] No deployment scripts

**Where it belongs**: Infrastructure code repository

---

# Validation Summary

## Final Checklist

Confirm before reporting results:

- [ ] I checked ALL items in MUST HAVE sections
- [ ] I verified ALL items in MUST NOT HAVE sections
- [ ] I documented all violations found
- [ ] I provided specific feedback for each failed check
- [ ] All critical issues have been reported

### Explicit Handling Verification

For each major checklist category (ARCH, PERF, SEC, REL, DATA, INT, OPS, MAINT, TEST, COMPL, UX, BIZ), confirm:

- [ ] Category is addressed in the document, OR
- [ ] Category is explicitly marked "Not applicable" with reasoning in the document, OR
- [ ] Category absence is reported as a violation (with applicability justification)

**No silent omissions allowed** — every category must have explicit disposition

---

## Reporting Readiness Checklist

- [ ] I will report every identified issue (no omissions)
- [ ] I will report only issues (no "everything looks good" sections)
- [ ] I will use the exact report format defined below (no deviations)
- [ ] Each reported issue will include Why Applicable (applicability justification)
- [ ] Each reported issue will include Evidence (quote/location)
- [ ] Each reported issue will include Why it matters (impact)
- [ ] Each reported issue will include a Proposal (concrete fix + acceptance criteria)
- [ ] I will avoid vague statements and use precise, verifiable language

---

## Reporting

Report **only** problems (do not list what is OK).

For each issue include:

- **Why Applicable**: Explain why this requirement applies to this specific spec's context (e.g., "This feature handles user authentication, therefore security analysis is required")
- **Issue**: What is wrong (requirement missing or incomplete)
- **Evidence**: Quote the exact text or describe the exact location in the artifact (or note "No mention found")
- **Why it matters**: Impact (risk, cost, user harm, compliance)
- **Proposal**: Concrete fix (what to change/add/remove) with clear acceptance criteria

### Full Report Format (Standard/Full Reviews)

```markdown
## Review Report (Issues Only)

### 1. {Short issue title}

**Checklist Item**: `{CHECKLIST-ID}` — {Checklist item title}

**Severity**: CRITICAL|HIGH|MEDIUM|LOW

#### Why Applicable

{Explain why this requirement applies to this spec's context. E.g., "This feature processes user data, therefore data integrity analysis is required."}

#### Issue

{What is wrong — requirement is missing, incomplete, or not explicitly marked as not applicable}

#### Evidence

{Quote the exact text or describe the exact location in the artifact. If requirement is missing entirely, state "No mention of [requirement] found in the document"}

#### Why It Matters

{Impact: risk, cost, user harm, compliance}

#### Proposal

{Concrete fix: what to change/add/remove, with clear acceptance criteria}

---

### 2. {Short issue title}
...
```

### Compact Report Format (Quick Reviews)

For quick reviews, use this condensed table format:

```markdown
## FEATURE Review Summary

| ID | Severity | Issue | Proposal |
|----|----------|-------|----------|
| ARCH-FDESIGN-001 | HIGH | Missing feature scope | Add scope statement to Section A |
| BIZ-FDESIGN-001 | MEDIUM | No PRD traceability | Add requirement references |

**Applicability**: {Spec type} — checked {N} priority domains, {M} marked N/A
```

---

## Reporting Commitment

- [ ] I reported all issues I found
- [ ] I used the exact report format defined in this checklist (no deviations)
- [ ] I included Why Applicable justification for each issue
- [ ] I included evidence and impact for each issue
- [ ] I proposed concrete fixes for each issue
- [ ] I did not hide or omit known problems
- [ ] I verified explicit handling for all major checklist categories
- [ ] I am ready to iterate on the proposals and re-review after changes
