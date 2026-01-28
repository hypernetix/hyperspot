# Feature Specification Expert Checklist

**Artifact**: Feature Specification (FEATURE)  
**Version**: 1.0  
**Purpose**: Comprehensive quality checklist for feature specifications

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

1. **Understand the feature's domain** — What kind of feature is this? (e.g., user-facing UI feature, backend API, data processing pipeline, CLI command)

2. **Determine applicability for each requirement** — Not all checklist items apply to all features:
   - A simple CRUD feature may not need complex State Management analysis
   - A read-only feature may not need Data Integrity analysis
   - A CLI feature may not need UI/UX analysis

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

### ARCH-FDESIGN-001: Feature Context Completeness
**Severity**: CRITICAL

- [ ] Feature identifier is present and stable (unique within the project)
- [ ] Feature status documented
- [ ] Overall Design reference present
- [ ] Requirements source reference present
- [ ] Actors/user roles are defined and referenced consistently
- [ ] Feature scope clearly stated
- [ ] Feature boundaries explicit
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
- [ ] Feature flags integration documented
- [ ] Versioning considerations documented

---

## ⚡ PERFORMANCE Expertise (PERF)

### PERF-FDESIGN-001: Performance-Critical Paths
**Severity**: HIGH

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

### SEC-FDESIGN-001: Authentication Integration
**Severity**: CRITICAL

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

### REL-FDESIGN-001: Error Handling Completeness
**Severity**: CRITICAL

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
- [ ] Feature flags documented

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

### MAINT-FDESIGN-001: Code Organization
**Severity**: MEDIUM

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

### TEST-FDESIGN-001: Testability
**Severity**: HIGH

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

### UX-FDESIGN-001: User Experience Flows
**Severity**: MEDIUM

- [ ] User journey clear
- [ ] Feedback points documented
- [ ] Error messages user-friendly
- [ ] Loading states documented
- [ ] Progress indication documented
- [ ] Confirmation flows documented

### UX-FDESIGN-002: Accessibility
**Severity**: MEDIUM (if applicable)

- [ ] Accessibility requirements addressed
- [ ] Keyboard navigation supported
- [ ] Screen reader support documented
- [ ] Color contrast considered
- [ ] Focus management documented

---

## 🏢 BUSINESS Expertise (BIZ)

### BIZ-FDESIGN-001: Requirements Alignment
**Severity**: CRITICAL

- [ ] All feature requirements (Section E) documented
- [ ] Requirements trace to PRD
- [ ] Requirements trace to a roadmap/backlog item (if used)
- [ ] Business rules accurately captured
- [ ] Edge cases reflect business reality
- [ ] Acceptance criteria business-verifiable

### BIZ-FDESIGN-002: Value Delivery
**Severity**: HIGH

- [ ] Feature delivers stated value
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

- **Why Applicable**: Explain why this requirement applies to this specific feature's context (e.g., "This feature handles user authentication, therefore security analysis is required")
- **Issue**: What is wrong (requirement missing or incomplete)
- **Evidence**: Quote the exact text or describe the exact location in the artifact (or note "No mention found")
- **Why it matters**: Impact (risk, cost, user harm, compliance)
- **Proposal**: Concrete fix (what to change/add/remove) with clear acceptance criteria

Recommended output format for chat:

```markdown
## Review Report (Issues Only)

### 1. {Short issue title}

**Checklist Item**: `{CHECKLIST-ID}` — {Checklist item title}

**Severity**: CRITICAL|HIGH|MEDIUM|LOW

#### Why Applicable

{Explain why this requirement applies to this feature's context. E.g., "This feature processes user data, therefore data integrity analysis is required."}

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

**Checklist Item**: `{CHECKLIST-ID}` — {Checklist item title}

**Severity**: CRITICAL|HIGH|MEDIUM|LOW

#### Why Applicable

{...}

#### Issue

{...}

---

...
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
