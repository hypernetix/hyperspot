# ADR (Architecture Decision Record) Expert Checklist

**Artifact**: Architecture Decision Record (ADR)  
**Version**: 1.0  
**Purpose**: Comprehensive quality checklist for ADR artifacts

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

# MUST HAVE

---

## 🏗️ ARCHITECTURE Expertise (ARCH)

### ARCH-ADR-001: Decision Significance
**Severity**: CRITICAL

- [ ] Decision is architecturally significant (not trivial)
- [ ] Decision affects multiple components or teams
- [ ] Decision is difficult to reverse
- [ ] Decision has long-term implications
- [ ] Decision represents a real choice between alternatives
- [ ] Decision is worth documenting for future reference

### ARCH-ADR-002: Context Completeness
**Severity**: CRITICAL

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

- [ ] ≥2 distinct options considered
- [ ] Options are genuinely viable
- [ ] Options are meaningfully different
- [ ] Chosen option clearly marked
- [ ] Option descriptions are comparable
- [ ] No strawman options (obviously inferior just for comparison)
- [ ] All options could realistically be implemented

### ARCH-ADR-004: Decision Rationale
**Severity**: CRITICAL

- [ ] Chosen option clearly stated
- [ ] Rationale explains WHY this option was chosen
- [ ] Rationale connects to context and constraints
- [ ] Trade-offs acknowledged
- [ ] Consequences documented (good and bad)
- [ ] Risks of chosen option acknowledged
- [ ] Mitigation strategies for risks documented

### ARCH-ADR-005: Traceability
**Severity**: HIGH

- [ ] Links to related requirements, risks, or constraints are provided
- [ ] Links to impacted architecture and design documents are provided (when applicable)
- [ ] Links to impacted feature specifications are provided (when applicable)
- [ ] Each link has a short explanation of relevance
- [ ] Scope of impact is explicitly stated (what changes, what does not)

### ARCH-ADR-006: ADR Metadata Quality
**Severity**: CRITICAL

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

- [ ] Superseding ADR referenced
- [ ] Reason for supersession explained
- [ ] Migration guidance provided
- [ ] Deprecated features identified
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

### PERF-ADR-001: Performance Impact
**Severity**: HIGH (if applicable)

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

### SEC-ADR-001: Security Impact
**Severity**: CRITICAL (if applicable)

- [ ] Security requirements referenced
- [ ] Security trade-offs documented
- [ ] Threat model impact analyzed
- [ ] Attack surface changes documented
- [ ] Security risks of each option analyzed
- [ ] Compliance impact analyzed
- [ ] Data protection impact analyzed

### SEC-ADR-002: Security Review
**Severity**: HIGH (if applicable)

- [ ] Security review conducted
- [ ] Security reviewer identified
- [ ] Security concerns addressed
- [ ] Penetration testing requirements documented
- [ ] Security monitoring requirements documented

### SEC-ADR-003: Authentication/Authorization Impact
**Severity**: HIGH (if applicable)

- [ ] AuthN mechanism changes documented
- [ ] AuthZ model changes documented
- [ ] Session management changes documented
- [ ] Token/credential handling changes documented
- [ ] Backward compatibility for auth documented

---

## 🛡️ RELIABILITY Expertise (REL)

### REL-ADR-001: Reliability Impact
**Severity**: HIGH (if applicable)

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

### DATA-ADR-001: Data Impact
**Severity**: HIGH (if applicable)

- [ ] Data model changes documented
- [ ] Migration requirements documented
- [ ] Backward compatibility analyzed
- [ ] Data integrity impact analyzed
- [ ] Data consistency impact analyzed
- [ ] Data volume impact analyzed

### DATA-ADR-002: Data Governance
**Severity**: MEDIUM (if applicable)

- [ ] Data ownership impact documented
- [ ] Data classification impact documented
- [ ] Data retention impact documented
- [ ] Privacy impact analyzed
- [ ] Compliance impact documented

---

## 🔌 INTEGRATION Expertise (INT)

### INT-ADR-001: Integration Impact
**Severity**: HIGH (if applicable)

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
- [ ] Feature flag requirements documented
- [ ] Canary/gradual rollout requirements documented
- [ ] Rollback triggers documented
- [ ] Success criteria documented

---

## 🔧 MAINTAINABILITY Expertise (MAINT)

### MAINT-ADR-001: Maintainability Impact
**Severity**: MEDIUM

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

### TEST-ADR-001: Testing Impact
**Severity**: MEDIUM

- [ ] Test strategy changes documented
- [ ] Test coverage requirements documented
- [ ] Test automation impact analyzed
- [ ] Integration test requirements documented
- [ ] Performance test requirements documented

### TEST-ADR-002: Validation Plan
**Severity**: MEDIUM

- [ ] How to validate decision documented
- [ ] Acceptance criteria stated
- [ ] Success metrics defined
- [ ] Timeframe for validation stated

---

## 📜 COMPLIANCE Expertise (COMPL)

### COMPL-ADR-001: Compliance Impact
**Severity**: CRITICAL (if applicable)

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

### BIZ-ADR-001: Business Alignment
**Severity**: HIGH

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

## ❌ ARCH-ADR-NO-002: No Feature Implementation Details
**Severity**: HIGH

**What to check**:
- [ ] No feature user flows
- [ ] No feature algorithms
- [ ] No feature state machines
- [ ] No step-by-step implementation guides
- [ ] No low-level implementation pseudo-code

**Where it belongs**: Feature specification / implementation design documentation

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

### QUALITY-001: Neutrality
**Severity**: MEDIUM

- [ ] Options described neutrally (no leading language)
- [ ] Pros and cons balanced for all options
- [ ] No strawman arguments
- [ ] Honest about chosen option's weaknesses
- [ ] Fair comparison of alternatives

### QUALITY-002: Clarity
**Severity**: HIGH

- [ ] Decision can be understood without insider knowledge
- [ ] Acronyms expanded on first use
- [ ] Technical terms defined if unusual
- [ ] No ambiguous language
- [ ] Clear, concrete statements

### QUALITY-003: Actionability
**Severity**: HIGH

- [ ] Clear what action to take based on decision
- [ ] Implementation guidance provided
- [ ] Scope of application clear
- [ ] Exceptions documented
- [ ] Expiration/review date set (if applicable)

### QUALITY-004: Reviewability
**Severity**: MEDIUM

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

For each issue include:

- **Why Applicable**: Explain why this requirement applies to this specific ADR's context (e.g., "This ADR describes a web service architecture, therefore security impact analysis is required")
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
