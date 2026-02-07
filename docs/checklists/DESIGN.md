# DESIGN (Overall Design) Expert Checklist

**Artifact**: Overall System Design (DESIGN)
**Version**: 2.0
**Last Updated**: 2026-02-03
**Purpose**: Comprehensive quality checklist for Overall Design artifacts

---

## Referenced Standards

This checklist validates system design artifacts based on the following international standards:

| Standard | Domain | Description |
|----------|--------|-------------|
| [IEEE 1016-2009](https://standards.ieee.org/ieee/1016/4502/) | **Design Description** | Software Design Descriptions — context, composition, logical, dependency viewpoints |
| [ISO/IEC/IEEE 42010:2022](https://www.iso.org/standard/74393.html) | **Architecture Description** | Architecture viewpoints, stakeholders, concerns, model correspondences |
| [ISO/IEC 25010:2011](https://www.iso.org/standard/35733.html) | **Quality Model** | SQuaRE — 8 quality characteristics: performance, security, reliability, maintainability |
| [ISO/IEC/IEEE 29148:2018](https://www.iso.org/standard/72089.html) | **Requirements Traceability** | Bidirectional traceability, requirements-to-design mapping |
| [OWASP ASVS 5.0](https://owasp.org/www-project-application-security-verification-standard/) | **Security Architecture** | Authentication, authorization, cryptography, data protection |
| [ISO/IEC 27001:2022](https://www.iso.org/standard/27001) | **Information Security** | ISMS controls, security management framework |

---

## Table of Contents

1. [Review Scope Selection](#review-scope-selection)
2. [Prerequisites](#prerequisites)
3. [Evidence Requirements (STRICT mode)](#evidence-requirements-strict-mode)
4. [Applicability Context](#applicability-context)
5. [Severity Dictionary](#severity-dictionary)
6. [MUST HAVE](#must-have)
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
7. [MUST NOT HAVE](#must-not-have)
8. [Validation Summary](#validation-summary)
   - [Final Checklist](#final-checklist)
   - [Reporting Readiness Checklist](#reporting-readiness-checklist)
   - [Reporting](#reporting)

---

## Review Scope Selection

**Choose review mode based on DESIGN scope and change type**:

| Review Mode | When to Use | Domains to Check |
|-------------|-------------|------------------|
| **Quick** | Minor updates, <3 sections changed | ARCH (core only) + changed domains |
| **Standard** | New DESIGN, moderate changes | All applicable domains |
| **Full** | Major architectural changes, compliance-critical | All 12 domains with evidence |

### Quick Review (ARCH Core Only)

**MUST CHECK** (blocking):
- [ ] ARCH-DESIGN-001: Architecture Overview Completeness
- [ ] ARCH-DESIGN-004: Component Model Quality
- [ ] ARCH-DESIGN-005: Domain Model Authority
- [ ] DOC-DESIGN-001: Explicit Non-Applicability

**Changed sections** — also check relevant domain items for any sections modified.

### Domain Prioritization by System Type

| System Type | Priority Domains (check first) | Secondary Domains | Often N/A |
|-------------|-------------------------------|-------------------|-----------|
| **Web Service** | ARCH, SEC, REL, DATA, INT | PERF, OPS, TEST | UX, COMPL |
| **CLI Tool** | ARCH, MAINT, TEST | DATA, INT | SEC, PERF, OPS, COMPL, UX |
| **Data Pipeline** | ARCH, DATA, REL, PERF | INT, OPS | SEC, UX, COMPL |
| **Methodology/Framework** | ARCH, MAINT | TEST | SEC, PERF, REL, DATA, INT, OPS, COMPL, UX |
| **Mobile App** | ARCH, UX, SEC, PERF | DATA, TEST | OPS, INT, COMPL |

**Applicability Rule**: Domains in "Often N/A" column still require explicit "Not applicable because..." statement in document if skipped.

---

## Prerequisites

Before starting the review, confirm:

- [ ] I understand this checklist validates DESIGN artifacts
- [ ] I will follow the Applicability Context rules below
- [ ] I will check ALL items in MUST HAVE sections
- [ ] I will verify ALL items in MUST NOT HAVE sections
- [ ] I will document any violations found
- [ ] I will provide specific feedback for each failed check
- [ ] I will complete the Final Checklist and provide a review report

---

## Evidence Requirements (STRICT mode)

**When Rules Mode = STRICT** (per `agent-compliance.md`):

### For Each Checklist Category

Agent MUST output evidence in this format:

| Category ID | Status | Evidence |
|-------------|--------|----------|
| ARCH-DESIGN-001 | PASS | Lines 45-67: "System purpose is to provide..." |
| ARCH-DESIGN-002 | PASS | Section B.1 contains 9 principles with unique IDs |
| PERF-DESIGN-001 | N/A | Line 698: "Performance architecture not applicable..." |

### Evidence Standards

**For PASS**:
- Quote 2-5 sentences from document proving requirement is met
- Include line numbers or section references
- Evidence must directly demonstrate compliance

**For N/A (Not Applicable)**:
- Quote the **explicit** "Not applicable because..." statement from document
- If no explicit statement exists → report as VIOLATION, not N/A
- Agent CANNOT decide N/A on author's behalf — document must state it

**For FAIL**:
- State what's missing or incorrect
- Provide location where it should be
- Quote surrounding context

### Why Evidence Matters

Without evidence requirements, agents exhibit anti-pattern AP-004 (BULK_PASS):
- Report "all checks pass" without actually verifying each item
- Skip tedious checklist review
- Produce invalid validation that misses real issues

**Real example of this failure**:
> Agent reported "DESIGN validation PASS" without reading the file, because deterministic gate passed and agent assumed semantic review was optional.

---

## Applicability Context

Before evaluating each checklist item, the expert MUST:

1. **Understand the artifact's domain** — What kind of system/project is this DESIGN for? (e.g., web service, CLI tool, data pipeline, methodology framework)

2. **Determine applicability for each requirement** — Not all checklist items apply to all designs:
   - A CLI tool design may not need Security Architecture analysis
   - A methodology framework design may not need Performance Architecture analysis
   - A local development tool design may not need Operations Architecture analysis

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

> **Standards**:
> - [IEEE 1016-2009](https://standards.ieee.org/ieee/1016/4502/) — Context (§5.2), Composition (§5.3), Logical (§5.4), Dependency (§5.5) viewpoints
> - [ISO/IEC/IEEE 42010:2022](https://www.iso.org/standard/74393.html) — Stakeholders, concerns, architecture viewpoints (§5-6)

### ARCH-DESIGN-001: Architecture Overview Completeness
**Severity**: CRITICAL
**Ref**: ISO/IEC/IEEE 42010:2022 §5.3 (Stakeholders and concerns), IEEE 1016-2009 §5.2 (Context viewpoint)

- [ ] System purpose clearly stated
- [ ] High-level architecture described
- [ ] Key architectural decisions summarized
- [ ] Architecture drivers documented (Section A)
- [ ] Key product/business requirements mapped to architectural drivers (links or references)
- [ ] System context diagram present or described
- [ ] External system boundaries identified
- [ ] ADR references provided for significant constraints

### ARCH-DESIGN-002: Principles Coherence
**Severity**: CRITICAL

- [ ] Each principle has a stable reference (name/identifier) and is unique
- [ ] Principles are actionable (can guide decisions)
- [ ] Principles don't contradict each other
- [ ] Principles are prioritized when they conflict
- [ ] Principles trace to business drivers
- [ ] ADR references provided for major principles (if ADRs exist)

### ARCH-DESIGN-003: Constraints Documentation
**Severity**: CRITICAL

- [ ] Each constraint has a stable reference (name/identifier) and is unique
- [ ] Regulatory constraints documented
- [ ] Platform/technology constraints documented
- [ ] Vendor/licensing constraints documented
- [ ] Legacy system integration constraints documented
- [ ] Data residency constraints documented
- [ ] Resource constraints documented (budget, team, time)
- [ ] ADR references provided for significant constraints

### ARCH-DESIGN-004: Component Model Quality
**Severity**: CRITICAL
**Ref**: IEEE 1016-2009 §5.3 (Composition viewpoint), ISO/IEC/IEEE 42010:2022 §6 (Architecture views)

- [ ] At least one architecture diagram present (image, Mermaid, or ASCII)
- [ ] All major components/services identified
- [ ] Component responsibilities clearly defined
- [ ] Component boundaries explicit
- [ ] Component interactions documented
- [ ] Data flow between components described
- [ ] Control flow between components described
- [ ] Component naming is consistent and meaningful

### ARCH-DESIGN-005: Domain Model Authority
**Severity**: CRITICAL
**Ref**: IEEE 1016-2009 §5.4 (Logical viewpoint), §5.6 (Information viewpoint)

- [ ] Domain model section exists
- [ ] Core entities/aggregates defined
- [ ] Value objects identified
- [ ] Entity relationships documented
- [ ] Core invariants stated
- [ ] Links to machine-readable schemas/types are provided (when available)
- [ ] Schema location in repo specified
- [ ] Schema/type format specified (JSON Schema, TypeScript, OpenAPI, etc.)

### ARCH-DESIGN-006: API Contracts Authority
**Severity**: CRITICAL

- [ ] API/Interface contracts section exists (if applicable)
- [ ] External APIs documented
- [ ] Internal APIs documented (if applicable)
- [ ] Links to machine-readable contracts are provided (when available)
- [ ] Contract format specified (OpenAPI, GraphQL, proto)
- [ ] Key endpoints/operations described
- [ ] Request/response shapes outlined
- [ ] Error handling expectations documented
- [ ] AuthN/AuthZ entry points documented
- [ ] Versioning strategy documented (if applicable)

### ARCH-DESIGN-007: Interaction Sequences
**Severity**: HIGH

- [ ] Key interaction flows documented
- [ ] Sequence diagrams for critical paths
- [ ] Actors/use cases referenced consistently with the requirements document
- [ ] Happy path sequences documented
- [ ] Error path sequences documented
- [ ] Async flows documented (if applicable)
- [ ] Long-running operations documented

### ARCH-DESIGN-008: Modularity & Extensibility
**Severity**: HIGH

- [ ] Extension points identified
- [ ] Plugin/module boundaries defined
- [ ] API stability zones identified
- [ ] Internal vs external interfaces distinguished
- [ ] Coupling between components minimized
- [ ] Cohesion within components maximized

---

## Semantic Alignment (SEM)

> **Standard**: [ISO/IEC/IEEE 29148:2018](https://www.iso.org/standard/72089.html) — Requirements Engineering
>
> "Bidirectional traceability between requirements and design" (§6.5)

### SEM-DESIGN-001: PRD Intent Preservation
**Severity**: CRITICAL
**Ref**: ISO/IEC/IEEE 29148:2018 §6.5 (Traceability)

- [ ] Design addresses all PRD FR and NFR IDs referenced in the design
- [ ] Architecture drivers align with PRD problems, capabilities, and success criteria
- [ ] Actors and use cases referenced in design match the PRD actor definitions
- [ ] Non-goals and risks in PRD are respected and not contradicted

### SEM-DESIGN-002: PRD Scope Consistency
**Severity**: HIGH

- [ ] No design scope extends beyond PRD boundaries without explicit approval
- [ ] Assumptions and open questions are consistent with PRD assumptions
- [ ] Any PRD trade-offs are explicitly documented in design context

### SEM-DESIGN-003: ADR Consistency and Coverage
**Severity**: CRITICAL

- [ ] Each referenced ADR decision is reflected in architecture choices and rationale
- [ ] ADR status is respected (Rejected/Deprecated decisions are not implemented)
- [ ] ADR decision drivers are reflected in design principles and constraints
- [ ] ADR consequences are incorporated into design risks or constraints

### SEM-DESIGN-004: ADR/PRD Link Integrity
**Severity**: HIGH

- [ ] ADR links include a clear trace to PRD context or requirement IDs
- [ ] Design references to ADRs are complete and do not omit critical decisions
- [ ] Any deviation from ADR decisions is explicitly justified and approved

### ARCH-DESIGN-009: Technology Stack Alignment
**Severity**: MEDIUM

- [ ] Technology choices documented (if applicable)
- [ ] Choices align with constraints
- [ ] Choices align with team capabilities
- [ ] Choices support NFRs
- [ ] Choices are maintainable long-term
- [ ] Technology risks identified

### ARCH-DESIGN-010: Capacity and Cost Budgets
**Severity**: HIGH

- [ ] Capacity planning approach documented
- [ ] Cost estimation approach documented
- [ ] Budget allocation strategy documented
- [ ] Cost optimization patterns documented

---

## ⚡ PERFORMANCE Expertise (PERF)

> **Standard**: [ISO/IEC 25010:2011](https://www.iso.org/standard/35733.html) — Performance Efficiency
>
> Sub-characteristics: time behavior, resource utilization, capacity

### PERF-DESIGN-001: Performance Architecture
**Severity**: HIGH
**Ref**: ISO/IEC 25010:2011 §4.2.2 (Performance efficiency)

- [ ] Caching strategy documented
- [ ] Database access patterns optimized
- [ ] N+1 query prevention addressed
- [ ] Batch processing patterns documented
- [ ] Async processing patterns documented
- [ ] Resource pooling strategies documented
- [ ] Memory management considerations documented

### PERF-DESIGN-002: Scalability Architecture
**Severity**: HIGH

- [ ] Horizontal scaling approach documented
- [ ] Vertical scaling limits identified
- [ ] Stateless design patterns used where possible
- [ ] Session management strategy documented
- [ ] Load balancing approach documented
- [ ] Database scaling strategy documented
- [ ] Queue/message broker strategy documented

### PERF-DESIGN-003: Latency Optimization
**Severity**: MEDIUM

- [ ] Critical path latency identified
- [ ] Latency budget allocated to components
- [ ] Network hop minimization addressed
- [ ] Data locality considerations documented
- [ ] CDN strategy documented (if applicable)
- [ ] Edge computing considerations (if applicable)

### PERF-DESIGN-004: Resource Efficiency
**Severity**: MEDIUM

- [ ] CPU efficiency considerations documented
- [ ] Memory efficiency considerations documented
- [ ] Storage efficiency considerations documented
- [ ] Network bandwidth considerations documented
- [ ] Cost optimization patterns documented

---

## 🔒 SECURITY Expertise (SEC)

> **Standards**:
> - [ISO/IEC 25010:2011](https://www.iso.org/standard/35733.html) — Security: confidentiality, integrity, non-repudiation, accountability, authenticity
> - [OWASP ASVS 5.0](https://owasp.org/www-project-application-security-verification-standard/) — Architecture requirements (V1)
> - [ISO/IEC 27001:2022](https://www.iso.org/standard/27001) — Information security controls

### SEC-DESIGN-001: Authentication Architecture
**Severity**: CRITICAL
**Ref**: OWASP ASVS V1.2 (Authentication Architecture), ISO 25010 §4.2.6

- [ ] Authentication mechanism documented
- [ ] Token/session management described
- [ ] Multi-factor authentication support documented
- [ ] SSO/federation integration documented
- [ ] Service-to-service authentication documented
- [ ] Credential storage approach documented
- [ ] Session timeout/renewal strategy documented

### SEC-DESIGN-002: Authorization Architecture
**Severity**: CRITICAL

- [ ] Authorization model documented (RBAC, ABAC, etc.)
- [ ] Role definitions documented
- [ ] Permission matrix documented
- [ ] Resource-level access control documented
- [ ] API endpoint authorization documented
- [ ] Least privilege principle applied
- [ ] Privilege escalation prevention documented

### SEC-DESIGN-003: Data Protection
**Severity**: CRITICAL

- [ ] Data encryption at rest documented
- [ ] Data encryption in transit documented
- [ ] Encryption key management documented
- [ ] Sensitive data classification documented
- [ ] PII handling procedures documented
- [ ] Data masking/anonymization documented
- [ ] Secure data disposal documented

### SEC-DESIGN-004: Security Boundaries
**Severity**: HIGH

- [ ] Trust boundaries identified
- [ ] Network segmentation documented
- [ ] DMZ architecture documented (if applicable)
- [ ] Firewall rules documented
- [ ] Input validation strategy documented
- [ ] Output encoding strategy documented
- [ ] CORS policy documented (if applicable)

### SEC-DESIGN-005: Threat Modeling
**Severity**: HIGH

- [ ] Major threats identified
- [ ] Attack vectors documented
- [ ] Mitigation strategies documented
- [ ] Security assumptions stated
- [ ] Third-party security risks documented
- [ ] Supply chain security considerations documented

### SEC-DESIGN-006: Audit & Compliance
**Severity**: HIGH

- [ ] Audit logging architecture documented
- [ ] Log retention policy documented
- [ ] Tamper-proof logging documented
- [ ] Compliance controls documented
- [ ] Security monitoring integration documented
- [ ] Incident response hooks documented

---

## 🛡️ RELIABILITY Expertise (REL)

> **Standard**: [ISO/IEC 25010:2011](https://www.iso.org/standard/35733.html) — Reliability
>
> Sub-characteristics: maturity, availability, fault tolerance, recoverability

### REL-DESIGN-001: Fault Tolerance
**Severity**: HIGH
**Ref**: ISO/IEC 25010:2011 §4.2.5 (Fault tolerance)

- [ ] Single points of failure identified and mitigated
- [ ] Redundancy strategies documented
- [ ] Failover mechanisms documented
- [ ] Circuit breaker patterns documented
- [ ] Retry policies documented
- [ ] Timeout policies documented
- [ ] Bulkhead patterns documented

### REL-DESIGN-002: Error Handling Architecture
**Severity**: HIGH

- [ ] Error classification documented
- [ ] Error propagation strategy documented
- [ ] Error recovery procedures documented
- [ ] Dead letter queue strategy documented
- [ ] Poison message handling documented
- [ ] Compensating transaction patterns documented

### REL-DESIGN-003: Data Consistency
**Severity**: CRITICAL

- [ ] Consistency model documented (strong, eventual, etc.)
- [ ] Transaction boundaries documented
- [ ] Distributed transaction strategy documented
- [ ] Saga patterns documented (if applicable)
- [ ] Conflict resolution strategies documented
- [ ] Idempotency patterns documented

### REL-DESIGN-004: Recovery Architecture
**Severity**: HIGH

- [ ] Backup strategy documented
- [ ] Recovery procedures documented
- [ ] Point-in-time recovery capability documented
- [ ] Disaster recovery architecture documented
- [ ] Business continuity procedures documented
- [ ] Data replication strategy documented

### REL-DESIGN-005: Resilience Patterns
**Severity**: MEDIUM

- [ ] Graceful degradation patterns documented
- [ ] Spec flags architecture documented
- [ ] Canary deployment support documented
- [ ] Blue/green deployment support documented
- [ ] Rollback procedures documented
- [ ] Health check mechanisms documented

---

## 📊 DATA Expertise (DATA)

> **Standard**: [IEEE 1016-2009](https://standards.ieee.org/ieee/1016/4502/) — Information Viewpoint (§5.6)
>
> Data entities, persistent data stores, data flow

### DATA-DESIGN-001: Data Architecture
**Severity**: CRITICAL
**Ref**: IEEE 1016-2009 §5.6 (Information viewpoint)

- [ ] Data stores identified
- [ ] Data partitioning strategy documented
- [ ] Data replication strategy documented
- [ ] Data sharding strategy documented (if applicable)
- [ ] Hot/warm/cold data strategy documented
- [ ] Data archival strategy documented

### DATA-DESIGN-002: Data Integrity
**Severity**: CRITICAL

- [ ] Referential integrity approach documented
- [ ] Constraint enforcement documented
- [ ] Validation rules documented
- [ ] Data versioning strategy documented
- [ ] Concurrent modification handling documented
- [ ] Orphan data prevention documented

### DATA-DESIGN-003: Data Governance
**Severity**: HIGH

- [ ] Data ownership documented
- [ ] Data lineage documented
- [ ] Data catalog integration documented
- [ ] Master data management documented
- [ ] Data quality monitoring documented
- [ ] Data dictionary/glossary linked

### DATA-DESIGN-004: Database Design Quality
**Severity**: HIGH (if database schemas are documented)

- [ ] Tables/collections have stable identifiers (names/IDs) and are uniquely defined
- [ ] Normalization level appropriate
- [ ] Indexes documented
- [ ] Query patterns documented
- [ ] Migration strategy documented
- [ ] Schema versioning documented

---

## 🔌 INTEGRATION Expertise (INT)

### INT-DESIGN-001: Integration Architecture
**Severity**: HIGH

- [ ] Integration patterns documented (sync, async, event-driven)
- [ ] Message formats documented
- [ ] Protocol choices documented
- [ ] Integration middleware documented (if applicable)
- [ ] API gateway strategy documented (if applicable)
- [ ] Service mesh strategy documented (if applicable)

### INT-DESIGN-002: External System Integration
**Severity**: HIGH

- [ ] External system dependencies documented
- [ ] Integration contracts documented
- [ ] SLA expectations documented
- [ ] Fallback strategies documented
- [ ] Circuit breaker implementations documented
- [ ] Rate limiting handling documented

### INT-DESIGN-003: Event Architecture
**Severity**: MEDIUM (if applicable)

- [ ] Event catalog documented
- [ ] Event schemas documented
- [ ] Event sourcing patterns documented (if applicable)
- [ ] Event replay capability documented
- [ ] Event ordering guarantees documented
- [ ] Dead letter queue handling documented

### INT-DESIGN-004: API Versioning & Evolution
**Severity**: MEDIUM

- [ ] API versioning strategy documented
- [ ] Breaking change policy documented
- [ ] Deprecation policy documented
- [ ] Backward compatibility approach documented
- [ ] API lifecycle management documented

---

## 🖥️ OPERATIONS Expertise (OPS)

### OPS-DESIGN-001: Deployment Architecture
**Severity**: HIGH

- [ ] Deployment topology documented (if applicable)
- [ ] Container/VM strategy documented
- [ ] Orchestration approach documented
- [ ] Environment promotion strategy documented
- [ ] Configuration management documented
- [ ] Secret management documented

### OPS-DESIGN-002: Observability Architecture
**Severity**: HIGH

- [ ] Logging architecture documented
- [ ] Log aggregation documented
- [ ] Metrics collection documented
- [ ] Distributed tracing documented
- [ ] Health check endpoints documented
- [ ] Alerting strategy documented
- [ ] Dashboard strategy documented

### OPS-DESIGN-003: Infrastructure as Code
**Severity**: MEDIUM

- [ ] IaC approach documented
- [ ] Environment parity documented
- [ ] Immutable infrastructure approach documented
- [ ] Auto-scaling configuration documented
- [ ] Resource tagging strategy documented

### OPS-DESIGN-004: SLO / Observability Targets
**Severity**: HIGH

- [ ] Key user-facing reliability targets are defined (SLO/SLI or equivalent)
- [ ] Alerting thresholds are aligned with those targets
- [ ] Error budgets (or an equivalent decision mechanism) are defined when applicable

---

## 🔧 MAINTAINABILITY Expertise (MAINT)

> **Standard**: [ISO/IEC 25010:2011](https://www.iso.org/standard/35733.html) — Maintainability
>
> Sub-characteristics: modularity, reusability, analysability, modifiability, testability

### MAINT-DESIGN-001: Code Organization
**Severity**: HIGH
**Ref**: ISO/IEC 25010:2011 §4.2.7 (Modularity)

- [ ] Module structure documented
- [ ] Package/namespace conventions documented
- [ ] Layering strategy documented
- [ ] Dependency injection approach documented
- [ ] Interface definitions documented

### MAINT-DESIGN-002: Technical Debt Management
**Severity**: MEDIUM

- [ ] Known technical debt documented
- [ ] Debt remediation roadmap documented
- [ ] Deprecation timeline documented
- [ ] Migration paths documented

### MAINT-DESIGN-003: Documentation Strategy
**Severity**: MEDIUM

- [ ] Documentation structure documented
- [ ] API documentation approach documented
- [ ] Architecture documentation approach documented
- [ ] Runbook approach documented
- [ ] Knowledge base approach documented

---

## 🧪 TESTING Expertise (TEST)

> **Standard**: [ISO/IEC 25010:2011](https://www.iso.org/standard/35733.html) §4.2.7.5 — Testability

### TEST-DESIGN-001: Testability Architecture
**Severity**: HIGH
**Ref**: ISO/IEC 25010:2011 §4.2.7.5 (Testability)

- [ ] Dependency injection for testability documented
- [ ] Mock/stub boundaries documented
- [ ] Test data management documented
- [ ] Test environment strategy documented
- [ ] Test isolation approach documented

### TEST-DESIGN-002: Testing Strategy
**Severity**: MEDIUM

- [ ] Unit test approach documented
- [ ] Integration test approach documented
- [ ] E2E test approach documented
- [ ] Performance test approach documented
- [ ] Security test approach documented
- [ ] Contract test approach documented

---

## 📜 COMPLIANCE Expertise (COMPL)

### COMPL-DESIGN-001: Compliance Architecture
**Severity**: HIGH (if applicable)

- [ ] Compliance requirements mapped to architecture
- [ ] Control implementations documented
- [ ] Audit trail architecture documented
- [ ] Evidence collection approach documented
- [ ] Compliance monitoring documented

### COMPL-DESIGN-002: Privacy Architecture
**Severity**: HIGH (if applicable)

- [ ] Privacy by design documented
- [ ] Consent management architecture documented
- [ ] Data subject rights implementation documented
- [ ] Privacy impact assessment documented
- [ ] Cross-border transfer controls documented

---

## 👤 USABILITY Expertise (UX)

### UX-DESIGN-001: User-Facing Architecture
**Severity**: MEDIUM

- [ ] Frontend architecture documented
- [ ] State management approach documented
- [ ] Responsive design approach documented
- [ ] Progressive enhancement approach documented
- [ ] Offline support architecture documented (if applicable)

---

## 🏢 BUSINESS Expertise (BIZ)

> **Standard**: [ISO/IEC/IEEE 29148:2018](https://www.iso.org/standard/72089.html) — Requirements Engineering
>
> Requirements-to-design allocation and traceability (§6.5)

### BIZ-DESIGN-001: Business Alignment
**Severity**: HIGH
**Ref**: ISO/IEC/IEEE 29148:2018 §6.5 (Requirements-design traceability)

- [ ] All functional requirements are addressed
- [ ] All non-functional requirements are addressed
- [ ] Business capability mapping documented
- [ ] Time-to-market considerations documented
- [ ] Cost implications documented

---

## Deliberate Omissions

### DOC-DESIGN-001: Explicit Non-Applicability
**Severity**: CRITICAL

- [ ] If a section or requirement is intentionally omitted, it is explicitly stated in the document (e.g., "Not applicable because...")
- [ ] No silent omissions — every major checklist area is either present or has a documented reason for absence
- [ ] Reviewer can distinguish "author considered and excluded" from "author forgot"

---

# MUST NOT HAVE

---

## ❌ ARCH-DESIGN-NO-001: No Spec-Level Details
**Severity**: CRITICAL

**What to check**:
- [ ] No spec-specific user flows
- [ ] No spec-specific algorithms
- [ ] No spec-specific state machines
- [ ] No spec-specific error handling details
- [ ] No spec implementation steps

**Where it belongs**: `Spec DESIGN`

---

## ❌ ARCH-DESIGN-NO-002: No Decision Debates
**Severity**: HIGH
**What to check**:
- [ ] No "we considered X vs Y" discussions
- [ ] No pros/cons analysis of alternatives
- [ ] No decision justification narratives
- [ ] No "why we didn't choose X" explanations
- [ ] No historical context of decisions

**Where it belongs**: `ADR` (Architecture Decision Records)

---

## ❌ BIZ-DESIGN-NO-003: No Product Requirements
**Severity**: HIGH

**What to check**:
- [ ] No business vision statements
- [ ] No actor definitions (reference PRD instead)
- [ ] No use case definitions (reference PRD instead)
- [ ] No functional requirement definitions (reference PRD instead)
- [ ] No success criteria definitions

**Where it belongs**: `PRD`

---

## ❌ BIZ-DESIGN-NO-004: No Implementation Tasks
**Severity**: HIGH

**What to check**:
- [ ] No sprint/iteration plans
- [ ] No task breakdowns
- [ ] No effort estimates
- [ ] No developer assignments
- [ ] No implementation timelines
- [ ] No TODO lists

**Where it belongs**: Project management tools or Spec DESIGN

---

## ❌ DATA-DESIGN-NO-001: No Code-Level Schema Definitions
**Severity**: MEDIUM

**What to check**:
- [ ] No inline SQL CREATE TABLE statements
- [ ] No complete JSON Schema definitions (link to files instead)
- [ ] No TypeScript interface definitions (link to files instead)
- [ ] No migration scripts

**Where it belongs**: Source code repository and/or schema repository, referenced from the design documentation


---

## ❌ INT-DESIGN-NO-001: No Complete API Specifications
**Severity**: MEDIUM

**What to check**:
- [ ] No complete OpenAPI specifications (link to files instead)
- [ ] No complete GraphQL schemas (link to files instead)
- [ ] No request/response JSON examples (keep in API spec files)
- [ ] No curl examples

**Where it belongs**: API contract files (e.g., OpenAPI/GraphQL/proto), referenced from the design documentation

---

## ❌ OPS-DESIGN-NO-001: No Infrastructure Code
**Severity**: MEDIUM

**What to check**:
- [ ] No Terraform/CloudFormation templates
- [ ] No Kubernetes manifests
- [ ] No Docker Compose files
- [ ] No CI/CD pipeline YAML
- [ ] No shell scripts

**Where it belongs**: Infrastructure code repository or `infra/` directory

---

## ❌ TEST-DESIGN-NO-001: No Test Code
**Severity**: MEDIUM

**What to check**:
- [ ] No test case implementations
- [ ] No test data files
- [ ] No assertion logic
- [ ] No mock implementations

**Where it belongs**: Test directories in source code

---

## ❌ MAINT-DESIGN-NO-001: No Code Snippets
**Severity**: HIGH

**What to check**:
- [ ] No production code examples
- [ ] No implementation snippets
- [ ] No debugging code
- [ ] No configuration file contents (link instead)

**Where it belongs**: Source code, with links from documentation

---

## ❌ SEC-DESIGN-NO-001: No Security Secrets
**Severity**: CRITICAL

**What to check**:
- [ ] No API keys
- [ ] No passwords
- [ ] No certificates
- [ ] No private keys
- [ ] No connection strings with credentials
- [ ] No encryption keys

**Where it belongs**: Secret management system (Vault, AWS Secrets Manager, etc.)

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

- **Why Applicable**: Explain why this requirement applies to this specific DESIGN's context (e.g., "This DESIGN describes a web service architecture, therefore security architecture is required")
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

{Explain why this requirement applies to this DESIGN's context. E.g., "This DESIGN describes a distributed system architecture, therefore reliability architecture is required."}

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
## DESIGN Review Summary

| ID | Severity | Issue | Proposal |
|----|----------|-------|----------|
| ARCH-DESIGN-001 | HIGH | Missing context diagram | Add system context diagram to Section A |
| ARCH-DESIGN-005 | MEDIUM | No schema location | Add path to domain types in Section C.1 |

**Applicability**: {System type} — checked {N} priority domains, {M} marked N/A
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
