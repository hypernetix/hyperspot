# ADR (Architecture Decision Record) Expert Checklist

**Artifact**: Architecture Decision Record (ADR)
**Version**: 2.0
**Purpose**: Comprehensive quality checklist for ADR artifacts

---

## Referenced Standards

This checklist incorporates requirements and best practices from:

| Standard | Scope | Key Sections Used |
|----------|-------|-------------------|
| **Michael Nygard's ADR Template (2011)** | De facto standard for ADR format | Title, Status, Context, Decision, Consequences structure |
| **ISO/IEC/IEEE 42010:2022** | Architecture Description | §5.7 AD elements, §6.7 Architecture decisions and rationale |
| **ISO/IEC 25010:2011** | SQuaRE Software Quality Model | §4.2 Quality characteristics (performance, security, reliability, maintainability) |
| **ISO/IEC/IEEE 29148:2018** | Requirements Engineering | §6.5 Behavioral requirements, traceability |
| **OWASP ASVS 5.0** | Application Security Verification | V1.2 Architecture, V2 Authentication, V5 Validation |
| **ISO/IEC 27001:2022** | Information Security Management | Annex A controls, risk assessment |
| **ISO/IEC/IEEE 29119-3:2021** | Test Documentation | Test specification, acceptance criteria |

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Applicability Context](#applicability-context)
3. [Severity Dictionary](#severity-dictionary)
4. [Review Scope Selection](#review-scope-selection)
5. **MUST HAVE**
   - [Architecture (ARCH)](#️-architecture-expertise-arch) — Core ADR quality
   - [Performance (PERF)](#-performance-expertise-perf) — If applicable
   - [Security (SEC)](#-security-expertise-sec) — If applicable
   - [Reliability (REL)](#️-reliability-expertise-rel) — If applicable
   - [Data (DATA)](#-data-expertise-data) — If applicable
   - [Integration (INT)](#-integration-expertise-int) — If applicable
   - [Operations (OPS)](#️-operations-expertise-ops) — If applicable
   - [Maintainability (MAINT)](#-maintainability-expertise-maint) — If applicable
   - [Testing (TEST)](#-testing-expertise-test) — If applicable
   - [Compliance (COMPL)](#-compliance-expertise-compl) — If applicable
   - [Usability (UX)](#-usability-expertise-ux) — If applicable
   - [Business (BIZ)](#-business-expertise-biz) — If applicable
6. **MUST NOT HAVE**
7. [ADR-Specific Quality Checks](#adr-specific-quality-checks)
8. [Validation Summary](#validation-summary)
9. [Reporting](#reporting)

---

## Prerequisites

Before starting the review, confirm:

- [ ] I understand this checklist validates ADR artifacts
- [ ] I will follow the Applicability Context rules below
- [ ] I will check ALL items in MUST HAVE sections
- [ ] I will verify ALL items in MUST NOT HAVE sections
- [ ] I will document any violations found
- [ ] I will provide specific feedback for each failed check
- [ ] I will complete the Final Checklist and provide a review report

---

## Applicability Context

Before evaluating each checklist item, the expert MUST:

1. **Understand the artifact's domain** — What kind of system/project is this ADR for? (e.g., CLI tool, web service, data pipeline, methodology framework)

2. **Determine applicability for each requirement** — Not all checklist items apply to all ADRs:
   - A CLI tool ADR may not need Security Impact analysis
   - A methodology framework ADR may not need Performance Impact analysis
   - A local development tool ADR may not need Operational Readiness analysis

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

## Review Scope Selection

Select review depth based on ADR complexity and impact:

### Review Modes

| ADR Type | Review Mode | Domains to Check |
|----------|-------------|------------------|
| Simple (single component, low risk) | **Quick** | ARCH only |
| Standard (multi-component, moderate risk) | **Standard** | ARCH + relevant domains |
| Complex (architectural, high risk) | **Full** | All applicable domains |

### Quick Review (ARCH Only)

For simple, low-risk decisions, check only core architecture items:

**MUST CHECK**:
- [ ] ARCH-ADR-001: Decision Significance
- [ ] ARCH-ADR-002: Context Completeness
- [ ] ARCH-ADR-003: Options Quality
- [ ] ARCH-ADR-004: Decision Rationale
- [ ] ARCH-ADR-006: ADR Metadata Quality
- [ ] QUALITY-002: Clarity
- [ ] QUALITY-003: Actionability

**SKIP**: All domain-specific sections (PERF, SEC, REL, etc.)

Note: `Quick review: checked ARCH core items only`

### Standard Review (ARCH + Relevant Domains)

Select applicable domains based on ADR subject:

| ADR Subject | Required Domains |
|-------------|------------------|
| Technology choice | ARCH, MAINT, OPS |
| Security mechanism | ARCH, SEC, COMPL |
| Database/storage | ARCH, DATA, PERF |
| API/integration | ARCH, INT, SEC |
| Infrastructure | ARCH, OPS, REL, PERF |
| User-facing spec | ARCH, UX, BIZ |

### Full Review

For architectural decisions with broad impact, check ALL applicable domains.

### Domain Applicability Quick Reference

| Domain | When Required | When N/A |
|--------|--------------|----------|
| **PERF** | Performance-sensitive systems | Methodology, documentation |
| **SEC** | User data, network, auth | Local-only tools |
| **REL** | Production systems, SLAs | Dev tools, prototypes |
| **DATA** | Persistent storage, migrations | Stateless components |
| **INT** | External APIs, contracts | Self-contained systems |
| **OPS** | Deployed services | Libraries, frameworks |
| **MAINT** | Long-lived code | Throwaway prototypes |
| **TEST** | Quality-critical systems | Exploratory work |
| **COMPL** | Regulated industries | Internal tools |
| **UX** | End-user impact | Backend infrastructure |
| **BIZ** | Business alignment needed | Technical decisions |

---

# MUST HAVE

---

## 🏗️ ARCHITECTURE Expertise (ARCH)

**Standards**: Michael Nygard's ADR Template (2011), ISO/IEC/IEEE 42010:2022 §6.7

### ARCH-ADR-001: Decision Significance
**Severity**: CRITICAL
**Ref**: ISO 42010 §6.7.1 — Architecture decisions shall be documented when they affect the system's fundamental structure

- [ ] Decision is architecturally significant (not trivial)
- [ ] Decision affects multiple components or teams
- [ ] Decision is difficult to reverse
- [ ] Decision has long-term implications
- [ ] Decision represents a real choice between alternatives
- [ ] Decision is worth documenting for future reference

### ARCH-ADR-002: Context Completeness
**Severity**: CRITICAL
**Ref**: Michael Nygard ADR Template — "Context" section; ISO 42010 §6.7.2 — Decision rationale shall include the context

- [ ] Problem statement is clear and specific
- [ ] Business context explained
- [ ] Technical context explained
- [ ] Constraints identified
- [ ] Assumptions stated
- [ ] Timeline/urgency documented
- [ ] Stakeholders identified
- [ ] ≥2 sentences describing the problem

### ARCH-ADR-003: Options Quality
**Severity**: CRITICAL
**Ref**: ISO 42010 §6.7.3 — Decision rationale shall document considered alternatives

- [ ] ≥2 distinct options considered
- [ ] Options are genuinely viable
- [ ] Options are meaningfully different
- [ ] Chosen option clearly marked
- [ ] Option descriptions are comparable
- [ ] No strawman options (obviously inferior just for comparison)
- [ ] All options could realistically be implemented

### ARCH-ADR-004: Decision Rationale
**Severity**: CRITICAL
**Ref**: Michael Nygard ADR Template — "Decision" & "Consequences" sections; ISO 42010 §6.7.2 — rationale documentation

- [ ] Chosen option clearly stated
- [ ] Rationale explains WHY this option was chosen
- [ ] Rationale connects to context and constraints
- [ ] Trade-offs acknowledged
- [ ] Consequences documented (good and bad)
- [ ] Risks of chosen option acknowledged
- [ ] Mitigation strategies for risks documented

### ARCH-ADR-005: Traceability
**Severity**: HIGH
**Ref**: ISO 29148 §5.2.8 — Requirements traceability; ISO 42010 §5.7 — AD element relationships

- [ ] Links to related requirements, risks, or constraints are provided
- [ ] Links to impacted architecture and design documents are provided (when applicable)
- [ ] Links to impacted feature specifications are provided (when applicable)
- [ ] Each link has a short explanation of relevance
- [ ] Scope of impact is explicitly stated (what changes, what does not)

### ARCH-ADR-006: ADR Metadata Quality
**Severity**: CRITICAL
**Ref**: Michael Nygard ADR Template — Title, Status, Date fields

- [ ] Title is descriptive and action-oriented
- [ ] Date is present and unambiguous
- [ ] Status is present and uses a consistent vocabulary (e.g., Proposed, Accepted, Rejected, Deprecated, Superseded)
- [ ] Decision owner/approver is identified (person/team)
- [ ] Scope / affected systems are stated
- [ ] If this record supersedes another decision record, the superseded record is linked

### ARCH-ADR-007: Decision Drivers (if present)
**Severity**: MEDIUM

- [ ] Drivers are specific and measurable where possible
- [ ] Drivers are prioritized
- [ ] Drivers trace to business or technical requirements
- [ ] Drivers are used to evaluate options
- [ ] No vague drivers ("good", "better", "fast")

### ARCH-ADR-008: Supersession Handling
**Severity**: HIGH (if applicable)
**Ref**: Michael Nygard ADR Template — Status values include "Superseded by [ADR-XXX]"

- [ ] Superseding ADR referenced
- [ ] Reason for supersession explained
- [ ] Migration guidance provided
- [ ] Deprecated specs identified
- [ ] Timeline for transition documented

### ARCH-ADR-009: Review Cadence
**Severity**: MEDIUM

- [ ] A review date or trigger is defined (when the decision should be revisited)
- [ ] Conditions that would invalidate this decision are documented

### ARCH-ADR-010: Decision Scope
**Severity**: MEDIUM

- [ ] Decision scope is clearly defined
- [ ] Boundaries of the decision are explicitly stated
- [ ] Assumptions about the scope are documented

---

## ⚡ PERFORMANCE Expertise (PERF)

**Standards**: ISO/IEC 25010:2011 §4.2.2 (Performance Efficiency)

### PERF-ADR-001: Performance Impact
**Severity**: HIGH (if applicable)
**Ref**: ISO 25010 §4.2.2 — Time behavior, resource utilization, capacity sub-characteristics

- [ ] Performance requirements referenced
- [ ] Performance trade-offs documented
- [ ] Latency impact analyzed
- [ ] Throughput impact analyzed
- [ ] Resource consumption impact analyzed
- [ ] Scalability impact analyzed
- [ ] Benchmarks or estimates provided where applicable

### PERF-ADR-002: Performance Testing
**Severity**: MEDIUM (if applicable)

- [ ] How to verify performance claims documented
- [ ] Performance acceptance criteria stated
- [ ] Load testing approach outlined
- [ ] Performance monitoring approach outlined

---

## 🔒 SECURITY Expertise (SEC)

**Standards**: OWASP ASVS 5.0 V1.2 (Architecture), ISO/IEC 27001:2022 (ISMS)

### SEC-ADR-001: Security Impact
**Severity**: CRITICAL (if applicable)
**Ref**: OWASP ASVS V1.2 — Security architecture requirements; ISO 27001 Annex A.8 — Asset management

- [ ] Security requirements referenced
- [ ] Security trade-offs documented
- [ ] Threat model impact analyzed
- [ ] Attack surface changes documented
- [ ] Security risks of each option analyzed
- [ ] Compliance impact analyzed
- [ ] Data protection impact analyzed

### SEC-ADR-002: Security Review
**Severity**: HIGH (if applicable)
**Ref**: ISO 27001 §9.2 — Internal audit; OWASP ASVS V1.2.4 — Security architecture review

- [ ] Security review conducted
- [ ] Security reviewer identified
- [ ] Security concerns addressed
- [ ] Penetration testing requirements documented
- [ ] Security monitoring requirements documented

### SEC-ADR-003: Authentication/Authorization Impact
**Severity**: HIGH (if applicable)
**Ref**: OWASP ASVS V2 — Authentication, V4 — Access Control

- [ ] AuthN mechanism changes documented
- [ ] AuthZ model changes documented
- [ ] Session management changes documented
- [ ] Token/credential handling changes documented
- [ ] Backward compatibility for auth documented

---

## 🛡️ RELIABILITY Expertise (REL)

**Standards**: ISO/IEC 25010:2011 §4.2.5 (Reliability)

### REL-ADR-001: Reliability Impact
**Severity**: HIGH (if applicable)
**Ref**: ISO 25010 §4.2.5 — Maturity, availability, fault tolerance, recoverability

- [ ] Availability impact analyzed
- [ ] Failure mode changes documented
- [ ] Recovery impact analyzed
- [ ] Single point of failure analysis
- [ ] Resilience pattern changes documented
- [ ] SLA impact documented

### REL-ADR-002: Operational Readiness
**Severity**: MEDIUM

- [ ] Deployment complexity analyzed
- [ ] Rollback strategy documented
- [ ] Monitoring requirements documented
- [ ] Alerting requirements documented
- [ ] Runbook updates required documented

---

## 📊 DATA Expertise (DATA)

**Standards**: IEEE 1016-2009 §5.6 (Information Viewpoint)

### DATA-ADR-001: Data Impact
**Severity**: HIGH (if applicable)
**Ref**: IEEE 1016 §5.6 — Information viewpoint: data entities, relationships, integrity constraints

- [ ] Data model changes documented
- [ ] Migration requirements documented
- [ ] Backward compatibility analyzed
- [ ] Data integrity impact analyzed
- [ ] Data consistency impact analyzed
- [ ] Data volume impact analyzed

### DATA-ADR-002: Data Governance
**Severity**: MEDIUM (if applicable)
**Ref**: ISO 27001 Annex A.5.9-5.14 — Information classification, handling

- [ ] Data ownership impact documented
- [ ] Data classification impact documented
- [ ] Data retention impact documented
- [ ] Privacy impact analyzed
- [ ] Compliance impact documented

---

## 🔌 INTEGRATION Expertise (INT)

**Standards**: IEEE 1016-2009 §5.3 (Interface Viewpoint)

### INT-ADR-001: Integration Impact
**Severity**: HIGH (if applicable)
**Ref**: IEEE 1016 §5.3 — Interface viewpoint: services, protocols, data formats

- [ ] API breaking changes documented
- [ ] Protocol changes documented
- [ ] Integration partner impact analyzed
- [ ] Version compatibility documented
- [ ] Migration path documented
- [ ] Deprecation timeline documented

### INT-ADR-002: Contract Changes
**Severity**: HIGH (if applicable)

- [ ] Contract changes documented
- [ ] Backward compatibility analyzed
- [ ] Consumer notification requirements documented
- [ ] Testing requirements for consumers documented

---

## 🖥️ OPERATIONS Expertise (OPS)

### OPS-ADR-001: Operational Impact
**Severity**: HIGH

- [ ] Deployment impact analyzed
- [ ] Infrastructure changes documented
- [ ] Configuration changes documented
- [ ] Monitoring changes documented
- [ ] Logging changes documented
- [ ] Cost impact analyzed

### OPS-ADR-002: Transition Plan
**Severity**: MEDIUM

- [ ] Rollout strategy documented
- [ ] Spec flag requirements documented
- [ ] Canary/gradual rollout requirements documented
- [ ] Rollback triggers documented
- [ ] Success criteria documented

---

## 🔧 MAINTAINABILITY Expertise (MAINT)

**Standards**: ISO/IEC 25010:2011 §4.2.7 (Maintainability)

### MAINT-ADR-001: Maintainability Impact
**Severity**: MEDIUM
**Ref**: ISO 25010 §4.2.7 — Modularity, reusability, analysability, modifiability, testability

- [ ] Code complexity impact analyzed
- [ ] Technical debt impact documented
- [ ] Learning curve for team documented
- [ ] Documentation requirements documented
- [ ] Long-term maintenance burden analyzed

### MAINT-ADR-002: Evolution Path
**Severity**: MEDIUM

- [ ] Future evolution considerations documented
- [ ] Extension points preserved or documented
- [ ] Deprecation path documented
- [ ] Migration to future solutions documented

---

## 🧪 TESTING Expertise (TEST)

**Standards**: ISO/IEC/IEEE 29119-3:2021 (Test Documentation)

### TEST-ADR-001: Testing Impact
**Severity**: MEDIUM
**Ref**: ISO 29119-3 §8 — Test design specification; ISO 25010 §4.2.7.5 — Testability

- [ ] Test strategy changes documented
- [ ] Test coverage requirements documented
- [ ] Test automation impact analyzed
- [ ] Integration test requirements documented
- [ ] Performance test requirements documented

### TEST-ADR-002: Validation Plan
**Severity**: MEDIUM
**Ref**: ISO 29119-3 §9 — Test case specification; acceptance criteria

- [ ] How to validate decision documented
- [ ] Acceptance criteria stated
- [ ] Success metrics defined
- [ ] Timeframe for validation stated

---

## 📜 COMPLIANCE Expertise (COMPL)

**Standards**: ISO/IEC 27001:2022 (ISMS), domain-specific regulations (GDPR, HIPAA, SOC 2)

### COMPL-ADR-001: Compliance Impact
**Severity**: CRITICAL (if applicable)
**Ref**: ISO 27001 §4.2 — Interested parties; §6.1 — Risk assessment; Annex A — Controls

- [ ] Regulatory impact analyzed
- [ ] Certification impact documented
- [ ] Audit requirements documented
- [ ] Legal review requirements documented
- [ ] Privacy impact assessment requirements documented

---

## 👤 USABILITY Expertise (UX)

### UX-ADR-001: User Impact
**Severity**: MEDIUM (if applicable)

- [ ] User experience impact documented
- [ ] User migration requirements documented
- [ ] User communication requirements documented
- [ ] Training requirements documented
- [ ] Documentation updates required documented

---

## 🏢 BUSINESS Expertise (BIZ)

**Standards**: ISO/IEC/IEEE 29148:2018 §5.2 (Stakeholder requirements)

### BIZ-ADR-001: Business Alignment
**Severity**: HIGH
**Ref**: ISO 29148 §5.2 — Stakeholder requirements definition; business value traceability

- [ ] Business requirements addressed
- [ ] Business value of decision explained
- [ ] Time-to-market impact documented
- [ ] Cost implications documented
- [ ] Resource requirements documented
- [ ] Stakeholder buy-in documented

### BIZ-ADR-002: Risk Assessment
**Severity**: HIGH

- [ ] Business risks identified
- [ ] Risk mitigation strategies documented
- [ ] Risk acceptance documented
- [ ] Contingency plans documented

---

# MUST NOT HAVE

---

## ❌ ARCH-ADR-NO-001: No Complete Architecture Description
**Severity**: CRITICAL

**What to check**:
- [ ] No full system architecture restatement
- [ ] No complete component model
- [ ] No full domain model
- [ ] No comprehensive API specification
- [ ] No full infrastructure description

**Where it belongs**: System/Architecture design documentation

---

## ❌ ARCH-ADR-NO-002: No Spec Implementation Details
**Severity**: HIGH

**What to check**:
- [ ] No feature user flows
- [ ] No feature algorithms
- [ ] No feature state machines
- [ ] No step-by-step implementation guides
- [ ] No low-level implementation pseudo-code

**Where it belongs**: Spec specification / implementation design documentation

---

## ❌ BIZ-ADR-NO-001: No Product Requirements
**Severity**: HIGH

**What to check**:
- [ ] No business vision statements
- [ ] No actor definitions
- [ ] No functional requirement definitions
- [ ] No use case definitions
- [ ] No NFR definitions

**Where it belongs**: Requirements / Product specification document

---

## ❌ BIZ-ADR-NO-002: No Implementation Tasks
**Severity**: HIGH

**What to check**:
- [ ] No sprint/iteration plans
- [ ] No detailed task breakdowns
- [ ] No effort estimates
- [ ] No developer assignments
- [ ] No project timelines

**Where it belongs**: Project management tools

---

## ❌ DATA-ADR-NO-001: No Complete Schema Definitions
**Severity**: MEDIUM

**What to check**:
- [ ] No full database schemas
- [ ] No complete JSON schemas
- [ ] No full API specifications
- [ ] No migration scripts

**Where it belongs**: Source code repository or architecture documentation

---

## ❌ MAINT-ADR-NO-001: No Code Implementation
**Severity**: HIGH

**What to check**:
- [ ] No production code
- [ ] No complete code examples
- [ ] No library implementations
- [ ] No configuration files
- [ ] No infrastructure code

**Where it belongs**: Source code repository

---

## ❌ SEC-ADR-NO-001: No Security Secrets
**Severity**: CRITICAL

**What to check**:
- [ ] No API keys
- [ ] No passwords
- [ ] No certificates
- [ ] No private keys
- [ ] No connection strings with credentials

**Where it belongs**: Secret management system

---

## ❌ TEST-ADR-NO-001: No Test Implementation
**Severity**: MEDIUM

**What to check**:
- [ ] No test case code
- [ ] No test data
- [ ] No test scripts
- [ ] No complete test plans

**Where it belongs**: Test documentation or test code

---

## ❌ OPS-ADR-NO-001: No Operational Procedures
**Severity**: MEDIUM

**What to check**:
- [ ] No complete runbooks
- [ ] No incident response procedures
- [ ] No monitoring configurations
- [ ] No alerting configurations

**Where it belongs**: Operations documentation or runbooks

---

## ❌ ARCH-ADR-NO-003: No Trivial Decisions
**Severity**: MEDIUM

**What to check**:
- [ ] No variable naming decisions
- [ ] No code formatting decisions
- [ ] No obvious technology choices (no alternatives)
- [ ] No easily reversible decisions
- [ ] No team-local decisions with no broader impact

**Where it belongs**: Team conventions, coding standards, or not documented at all

---

## ❌ ARCH-ADR-NO-004: No Incomplete Decisions
**Severity**: HIGH

**What to check**:
- [ ] No "TBD" in critical sections
- [ ] No missing context
- [ ] No missing options analysis
- [ ] No missing rationale
- [ ] No missing consequences

**Where it belongs**: Complete the ADR before publishing, or use "Proposed" status

---

# ADR-Specific Quality Checks

---

## ADR Writing Quality

**Standards**: Michael Nygard ADR Template — writing style guidance

### QUALITY-001: Neutrality
**Severity**: MEDIUM
**Ref**: Michael Nygard — "Options should be presented neutrally"

- [ ] Options described neutrally (no leading language)
- [ ] Pros and cons balanced for all options
- [ ] No strawman arguments
- [ ] Honest about chosen option's weaknesses
- [ ] Fair comparison of alternatives

### QUALITY-002: Clarity
**Severity**: HIGH
**Ref**: ISO 29148 §5.2.5 — Requirements shall be unambiguous; IEEE 1016 §4.2 — SDD comprehensibility

- [ ] Decision can be understood without insider knowledge
- [ ] Acronyms expanded on first use
- [ ] Technical terms defined if unusual
- [ ] No ambiguous language
- [ ] Clear, concrete statements

### QUALITY-003: Actionability
**Severity**: HIGH
**Ref**: Michael Nygard — "Decision" section specifies what to do

- [ ] Clear what action to take based on decision
- [ ] Implementation guidance provided
- [ ] Scope of application clear
- [ ] Exceptions documented
- [ ] Expiration/review date set (if applicable)

### QUALITY-004: Reviewability
**Severity**: MEDIUM
**Ref**: ISO 42010 §6.7 — AD rationale shall be verifiable

- [ ] Can be reviewed in a reasonable time
- [ ] Evidence and references provided
- [ ] Assumptions verifiable
- [ ] Consequences measurable
- [ ] Success criteria verifiable

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

### Format Selection

| Review Mode | Report Format |
|-------------|---------------|
| Quick | Compact (table) |
| Standard | Compact or Full |
| Full | Full (detailed) |

### Compact Format (for Quick/Standard reviews)

```markdown
## ADR Review: {title}

| # | ID | Sev | Issue | Fix |
|---|-----|-----|-------|-----|
| 1 | ARCH-002 | CRIT | Missing problem statement | Add 2+ sentences describing the problem |
| 2 | ARCH-003 | HIGH | Only 1 option listed | Add at least 1 viable alternative |

**Review mode**: Quick (ARCH core only)
**Domains checked**: ARCH
**Domains N/A**: PERF, SEC, REL, DATA, INT, OPS (methodology ADR)
```

### Full Format (for detailed reviews)

For each issue include:

- **Why Applicable**: Explain why this requirement applies to this specific ADR's context (e.g., "This ADR describes a web service architecture, therefore security impact analysis is required")
- **Issue**: What is wrong (requirement missing or incomplete)
- **Evidence**: Quote the exact text or describe the exact location in the artifact (or note "No mention found")
- **Why it matters**: Impact (risk, cost, user harm, compliance)
- **Proposal**: Concrete fix (what to change/add/remove) with clear acceptance criteria

Full output format:

```markdown
## Review Report (Issues Only)

### 1. {Short issue title}

**Checklist Item**: `{CHECKLIST-ID}` — {Checklist item title}

**Severity**: CRITICAL|HIGH|MEDIUM|LOW

#### Why Applicable

{Explain why this requirement applies to this ADR's context. E.g., "This ADR describes a distributed system architecture, therefore reliability impact analysis is required."}

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
