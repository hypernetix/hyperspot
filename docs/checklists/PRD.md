# PRD Expert Checklist

**Artifact**: Product Requirements Document (PRD)  
**Version**: 1.0  
**Purpose**: Comprehensive quality checklist for PRD artifacts

---

## Prerequisites

Before starting the review, confirm:

- [ ] I understand this checklist validates PRD artifacts
- [ ] I will follow the Applicability Context rules below
- [ ] I will check ALL items in MUST HAVE sections
- [ ] I will verify ALL items in MUST NOT HAVE sections
- [ ] I will document any violations found
- [ ] I will provide specific feedback for each failed check
- [ ] I will complete the Final Checklist and provide a review report

---

## Applicability Context

Before evaluating each checklist item, the expert MUST:

1. **Understand the product's domain** — What kind of product is this PRD for? (e.g., consumer app, enterprise platform, developer tool, internal system)

2. **Determine applicability for each requirement** — Not all checklist items apply to all PRDs:
   - An internal tool PRD may not need market positioning analysis
   - A developer framework PRD may not need end-user personas
   - A methodology PRD may not need regulatory compliance analysis

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

## BUSINESS Expertise (BIZ)

### BIZ-PRD-001: Vision Clarity
**Severity**: CRITICAL

- [ ] Purpose statement explains WHY the product exists
- [ ] Target users clearly identified with specificity (not just "users")
- [ ] Key problems solved are concrete and measurable
- [ ] Success criteria are quantifiable (numbers, percentages, timeframes)
- [ ] Capabilities list covers core value propositions
- [ ] Business context is clear without requiring insider knowledge

### BIZ-PRD-002: Stakeholder Coverage
**Severity**: HIGH

- [ ] All relevant user personas represented as actors
- [ ] Business sponsors' needs reflected in requirements
- [ ] End-user needs clearly articulated
- [ ] Organizational constraints acknowledged
- [ ] Market positioning context provided (if applicable)

### BIZ-PRD-003: Requirements Completeness
**Severity**: CRITICAL

- [ ] All business-critical capabilities have corresponding functional requirements
- [ ] Requirements trace back to stated problems
- [ ] No capability is mentioned without a supporting requirement
- [ ] Requirements are prioritized (implicit or explicit)
- [ ] Dependencies between requirements are identified

### BIZ-PRD-004: Use Case Coverage
**Severity**: HIGH

- [ ] All primary user journeys represented as use cases
- [ ] Critical business workflows documented
- [ ] Edge cases and exception flows considered
- [ ] Use cases cover the "happy path" and error scenarios
- [ ] Use cases are realistic and actionable

### BIZ-PRD-005: Success Metrics
**Severity**: HIGH

- [ ] Success criteria are SMART (Specific, Measurable, Achievable, Relevant, Time-bound)
- [ ] Metrics can actually be measured with available data
- [ ] Baseline values established where possible
- [ ] Target values are realistic
- [ ] Timeframes for achieving targets specified

### BIZ-PRD-006: Terminology & Definitions
**Severity**: MEDIUM

- [ ] Key domain terms are defined (glossary or inline)
- [ ] Acronyms are expanded on first use
- [ ] Terms are used consistently (no synonyms that change meaning)

### BIZ-PRD-007: Assumptions & Open Questions
**Severity**: HIGH

- [ ] Key assumptions are explicitly stated
- [ ] Open questions are listed with owners and desired resolution time
- [ ] Dependencies on external teams/vendors are called out

### BIZ-PRD-008: Risks & Non-Goals
**Severity**: MEDIUM

- [ ] Major risks/uncertainties are listed
- [ ] Explicit non-goals/out-of-scope items are documented

---

## ARCHITECTURE Expertise (ARCH)

### ARCH-PRD-001: Scope Boundaries
**Severity**: CRITICAL

- [ ] System boundaries are clear (what's in vs out of scope)
- [ ] Integration points with external systems identified
- [ ] Organizational boundaries respected
- [ ] Technology constraints acknowledged at high level
- [ ] No implementation decisions embedded in requirements

### ARCH-PRD-002: Modularity Enablement
**Severity**: MEDIUM

- [ ] Requirements are decomposable into features
- [ ] No monolithic "do everything" requirements
- [ ] Clear separation of concerns in requirement grouping
- [ ] Requirements support incremental delivery
- [ ] Dependencies don't create circular coupling

### ARCH-PRD-003: Scalability Considerations
**Severity**: MEDIUM

- [ ] User volume expectations stated (current and projected)
- [ ] Data volume expectations stated (current and projected)
- [ ] Geographic distribution requirements captured
- [ ] Growth scenarios considered in requirements
- [ ] Performance expectations stated at business level

### ARCH-PRD-004: System Actor Clarity
**Severity**: HIGH

- [ ] System actors represent real external systems
- [ ] System actor interfaces are clear
- [ ] Integration direction specified (inbound/outbound/bidirectional)
- [ ] System actor availability requirements stated
- [ ] Data exchange expectations documented

---

## 🔒 SECURITY Expertise (SEC)

### SEC-PRD-001: Authentication Requirements
**Severity**: CRITICAL

- [ ] User authentication needs stated
- [ ] Multi-factor requirements captured (if applicable)
- [ ] SSO/federation requirements documented
- [ ] Session management expectations stated
- [ ] Password/credential policies referenced

### SEC-PRD-002: Authorization Requirements
**Severity**: CRITICAL

- [ ] Role-based access clearly defined through actors
- [ ] Permission levels distinguished between actors
- [ ] Data access boundaries specified per actor
- [ ] Administrative vs user roles separated
- [ ] Delegation/impersonation needs captured

### SEC-PRD-003: Data Classification
**Severity**: HIGH

- [ ] Sensitive data types identified
- [ ] PII handling requirements stated
- [ ] Data retention expectations documented
- [ ] Data deletion/anonymization needs captured
- [ ] Cross-border data transfer considerations noted

### SEC-PRD-004: Audit Requirements
**Severity**: MEDIUM

- [ ] Audit logging needs identified
- [ ] User action tracking requirements stated
- [ ] Compliance reporting needs captured
- [ ] Forensic investigation support requirements noted
- [ ] Non-repudiation requirements documented

---

## ⚡ PERFORMANCE Expertise (PERF)

### PERF-PRD-001: Response Time Expectations
**Severity**: HIGH

- [ ] User-facing response time expectations stated
- [ ] Batch processing time expectations stated
- [ ] Report generation time expectations stated
- [ ] Search/query performance expectations stated
- [ ] Expectations are realistic for the problem domain

### PERF-PRD-002: Throughput Requirements
**Severity**: MEDIUM

- [ ] Concurrent user expectations documented
- [ ] Transaction volume expectations stated
- [ ] Peak load scenarios identified
- [ ] Sustained load expectations documented
- [ ] Growth projections factored in

### PERF-PRD-003: Capacity Planning Inputs
**Severity**: MEDIUM

- [ ] Data volume projections provided
- [ ] User base growth projections provided
- [ ] Seasonal/cyclical patterns identified
- [ ] Burst scenarios documented
- [ ] Historical growth data referenced (if available)

---

## 🛡️ RELIABILITY Expertise (REL)

### REL-PRD-001: Availability Requirements
**Severity**: HIGH

- [ ] Uptime expectations stated (e.g., 99.9%)
- [ ] Maintenance window expectations documented
- [ ] Business hours vs 24/7 requirements clear
- [ ] Geographic availability requirements stated
- [ ] Degraded mode expectations documented

### REL-PRD-002: Recovery Requirements
**Severity**: HIGH

- [ ] Data loss tolerance stated (RPO)
- [ ] Downtime tolerance stated (RTO)
- [ ] Backup requirements documented
- [ ] Disaster recovery expectations stated
- [ ] Business continuity requirements captured

### REL-PRD-003: Error Handling Expectations
**Severity**: MEDIUM

- [ ] User error handling expectations stated
- [ ] System error communication requirements documented
- [ ] Graceful degradation expectations captured
- [ ] Retry/recovery user experience documented
- [ ] Support escalation paths identified

---

## 👤 USABILITY Expertise (UX)

### UX-PRD-001: User Experience Goals
**Severity**: HIGH

- [ ] Target user skill level defined
- [ ] Learning curve expectations stated
- [ ] Efficiency goals for expert users documented
- [ ] Discoverability requirements for new users stated
- [ ] User satisfaction targets defined

### UX-PRD-002: Accessibility Requirements
**Severity**: HIGH

- [ ] Accessibility standards referenced (WCAG level)
- [ ] Assistive technology support requirements stated
- [ ] Keyboard navigation requirements documented
- [ ] Screen reader compatibility requirements stated
- [ ] Color/contrast requirements noted

### UX-PRD-003: Internationalization Requirements
**Severity**: MEDIUM

- [ ] Supported languages listed
- [ ] Localization requirements documented
- [ ] Regional format requirements stated (dates, numbers, currency)
- [ ] RTL language support requirements noted
- [ ] Cultural considerations documented

### UX-PRD-004: Device/Platform Requirements
**Severity**: MEDIUM

- [ ] Supported platforms listed (web, mobile, desktop)
- [ ] Browser requirements stated
- [ ] Mobile device requirements documented
- [ ] Offline capability requirements stated
- [ ] Responsive design requirements documented

---

## 🔧 MAINTAINABILITY Expertise (MAINT)

### MAINT-PRD-001: Documentation Requirements
**Severity**: MEDIUM

- [ ] User documentation requirements stated
- [ ] Admin documentation requirements stated
- [ ] API documentation requirements stated
- [ ] Training material requirements documented
- [ ] Help system requirements captured

### MAINT-PRD-002: Support Requirements
**Severity**: MEDIUM

- [ ] Support tier expectations documented
- [ ] SLA requirements stated
- [ ] Self-service support requirements captured
- [ ] Diagnostic capability requirements stated
- [ ] Troubleshooting support requirements documented

---

## 📜 COMPLIANCE Expertise (COMPL)

### COMPL-PRD-001: Regulatory Requirements
**Severity**: CRITICAL (if applicable)

- [ ] Applicable regulations identified (GDPR, HIPAA, SOX, etc.)
- [ ] Compliance certification requirements stated
- [ ] Audit requirements documented
- [ ] Reporting requirements captured
- [ ] Data sovereignty requirements stated

### COMPL-PRD-002: Industry Standards
**Severity**: MEDIUM

- [ ] Industry standards referenced
- [ ] Best practice frameworks identified
- [ ] Certification requirements stated
- [ ] Interoperability standards documented
- [ ] Security standards referenced

### COMPL-PRD-003: Legal Requirements
**Severity**: HIGH (if applicable)

- [ ] Terms of service requirements stated
- [ ] Privacy policy requirements documented
- [ ] Consent management requirements captured
- [ ] Data subject rights requirements stated
- [ ] Contractual obligations documented

---

## 📊 DATA Expertise (DATA)

### DATA-PRD-001: Data Ownership
**Severity**: HIGH

- [ ] Data ownership clearly defined
- [ ] Data stewardship responsibilities identified
- [ ] Data sharing expectations documented
- [ ] Third-party data usage requirements stated
- [ ] User-generated content ownership defined

### DATA-PRD-002: Data Quality Requirements
**Severity**: MEDIUM

- [ ] Data accuracy requirements stated
- [ ] Data completeness requirements documented
- [ ] Data freshness requirements captured
- [ ] Data validation requirements stated
- [ ] Data cleansing requirements documented

### DATA-PRD-003: Data Lifecycle
**Severity**: MEDIUM

- [ ] Data retention requirements stated
- [ ] Data archival requirements documented
- [ ] Data purging requirements captured
- [ ] Data migration requirements stated
- [ ] Historical data access requirements documented

---

## 🔌 INTEGRATION Expertise (INT)

### INT-PRD-001: External System Integration
**Severity**: HIGH

- [ ] Required integrations listed
- [ ] Integration direction specified
- [ ] Data exchange requirements documented
- [ ] Integration availability requirements stated
- [ ] Fallback requirements for integration failures documented

### INT-PRD-002: API Requirements
**Severity**: MEDIUM

- [ ] API exposure requirements stated
- [ ] API consumer requirements documented
- [ ] API versioning requirements stated
- [ ] Rate limiting expectations documented
- [ ] API documentation requirements stated

---

## 🖥️ OPERATIONS Expertise (OPS)

### OPS-PRD-001: Deployment Requirements
**Severity**: MEDIUM

- [ ] Deployment environment requirements stated
- [ ] Release frequency expectations documented
- [ ] Rollback requirements captured
- [ ] Blue/green or canary requirements stated
- [ ] Environment parity requirements documented

### OPS-PRD-002: Monitoring Requirements
**Severity**: MEDIUM

- [ ] Alerting requirements stated
- [ ] Dashboard requirements documented
- [ ] Log retention requirements captured
- [ ] Incident response requirements stated
- [ ] Capacity monitoring requirements documented

---

## 🧪 TESTING Expertise (TEST)

### TEST-PRD-001: Acceptance Criteria
**Severity**: HIGH

- [ ] Each functional requirement has verifiable acceptance criteria
- [ ] Use cases define expected outcomes
- [ ] NFRs have measurable thresholds
- [ ] Edge cases are testable
- [ ] Negative test cases implied

### TEST-PRD-002: Testability
**Severity**: MEDIUM

- [ ] Requirements are unambiguous enough to test
- [ ] Requirements don't use vague terms ("fast", "easy", "intuitive")
- [ ] Requirements specify concrete behaviors
- [ ] Requirements avoid compound statements (multiple "and"s)
- [ ] Requirements can be independently verified

---

## Deliberate Omissions

### DOC-PRD-001: Explicit Non-Applicability
**Severity**: CRITICAL

- [ ] If a section or requirement is intentionally omitted, it is explicitly stated in the document (e.g., "Not applicable because...")
- [ ] No silent omissions — every major checklist area is either present or has a documented reason for absence
- [ ] Reviewer can distinguish "author considered and excluded" from "author forgot"

---

# MUST NOT HAVE

---

## ❌ ARCH-PRD-NO-001: No Technical Implementation Details
**Severity**: CRITICAL

**What to check**:
- [ ] No database schema definitions
- [ ] No API endpoint specifications
- [ ] No technology stack decisions
- [ ] No code snippets or pseudocode
- [ ] No infrastructure specifications
- [ ] No framework/library choices

**Where it belongs**: `DESIGN` (Overall Design)

---

## ❌ ARCH-PRD-NO-002: No Architectural Decisions
**Severity**: CRITICAL

**What to check**:
- [ ] No microservices vs monolith decisions
- [ ] No database choice justifications
- [ ] No cloud provider selections
- [ ] No architectural pattern discussions
- [ ] No component decomposition

**Where it belongs**: `ADR` (Architecture Decision Records)

---

## ❌ BIZ-PRD-NO-001: No Implementation Tasks
**Severity**: HIGH

**What to check**:
- [ ] No sprint/iteration plans
- [ ] No task breakdowns
- [ ] No effort estimates
- [ ] No developer assignments
- [ ] No implementation timelines

**Where it belongs**: Project management tools (Jira, Linear, etc.) or Feature DESIGN

---

## ❌ BIZ-PRD-NO-002: No Feature-Level Design
**Severity**: HIGH

**What to check**:
- [ ] No detailed user flows
- [ ] No wireframes or UI specifications
- [ ] No algorithm descriptions
- [ ] No state machine definitions
- [ ] No detailed error handling logic

**Where it belongs**: `Feature DESIGN` (Feature Design)

---

## ❌ DATA-PRD-NO-001: No Data Schema Definitions
**Severity**: HIGH

**What to check**:
- [ ] No entity-relationship diagrams
- [ ] No table definitions
- [ ] No JSON schema specifications
- [ ] No data type specifications
- [ ] No field-level constraints

**Where it belongs**: Architecture and design documentation (domain model and schemas)

---

## ❌ INT-PRD-NO-001: No API Specifications
**Severity**: HIGH

**What to check**:
- [ ] No REST endpoint definitions
- [ ] No request/response schemas
- [ ] No HTTP method specifications
- [ ] No authentication header specifications
- [ ] No error response formats

**Where it belongs**: API contract documentation (e.g., OpenAPI) or architecture and design documentation

---

## ❌ TEST-PRD-NO-001: No Test Cases
**Severity**: MEDIUM

**What to check**:
- [ ] No detailed test scripts
- [ ] No test data specifications
- [ ] No automation code
- [ ] No test environment specifications

**Where it belongs**: Test plans, test suites, or QA documentation

---

## ❌ OPS-PRD-NO-001: No Infrastructure Specifications
**Severity**: MEDIUM

**What to check**:
- [ ] No server specifications
- [ ] No Kubernetes manifests
- [ ] No Docker configurations
- [ ] No CI/CD pipeline definitions
- [ ] No monitoring tool configurations

**Where it belongs**: Infrastructure-as-code repositories or operations/infrastructure documentation

---

## ❌ SEC-PRD-NO-001: No Security Implementation Details
**Severity**: HIGH

**What to check**:
- [ ] No encryption algorithm specifications
- [ ] No key management procedures
- [ ] No firewall rules
- [ ] No security tool configurations
- [ ] No penetration test results

**Where it belongs**: Security architecture documentation or ADRs

---

## ❌ MAINT-PRD-NO-001: No Code-Level Documentation
**Severity**: MEDIUM

**What to check**:
- [ ] No code comments
- [ ] No function/class documentation
- [ ] No inline code examples
- [ ] No debugging instructions

**Where it belongs**: Source code, README files, or developer documentation

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

For each major checklist category (BIZ, ARCH, SEC, TEST, MAINT), confirm:

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

- **Why Applicable**: Explain why this requirement applies to this specific PRD's context (e.g., "This PRD describes a user-facing product, therefore stakeholder coverage is required")
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

{Explain why this requirement applies to this PRD's context. E.g., "This PRD describes a regulated industry product, therefore compliance requirements are required."}

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
