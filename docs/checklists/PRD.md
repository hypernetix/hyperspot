# PRD Expert Checklist

**Artifact**: Product Requirements Document (PRD)
**Version**: 1.2
**Last Updated**: 2026-02-03
**Purpose**: Comprehensive quality checklist for PRD artifacts

---

## Referenced Standards

This checklist incorporates requirements and best practices from the following international standards:

| Standard | Domain | Description |
|----------|--------|-------------|
| [ISO/IEC/IEEE 29148:2018](https://www.iso.org/standard/72089.html) | Requirements Engineering | Life cycle processes for requirements engineering (supersedes IEEE 830) |
| [ISO/IEC 25010:2023](https://www.iso.org/standard/78176.html) | Software Quality | Product quality model with 9 characteristics |
| [ISO/IEC 27001:2022](https://www.iso.org/standard/27001) | Information Security | ISMS requirements |
| [ISO 22301:2019](https://www.iso.org/standard/75106.html) | Business Continuity | BCMS requirements |
| [ISO 9241-11:2018](https://www.iso.org/standard/63500.html) | Usability | Usability definitions and framework |
| [ISO 9241-210:2019](https://www.iso.org/standard/77520.html) | Human-Centred Design | HCD for interactive systems |
| [WCAG 2.2](https://www.w3.org/WAI/standards-guidelines/wcag/) | Accessibility | Web Content Accessibility Guidelines |
| [OWASP ASVS 5.0](https://owasp.org/www-project-application-security-verification-standard/) | Application Security | Security verification requirements |
| [NIST SP 800-53 Rev.5](https://csrc.nist.gov/pubs/sp/800/53/r5/upd1/final) | Security Controls | Security and privacy controls catalog |
| [RFC 6749](https://datatracker.ietf.org/doc/html/rfc6749) | Authentication | OAuth 2.0 Authorization Framework |
| [GDPR Art. 25](https://gdpr-info.eu/art-25-gdpr/) | Privacy | Data protection by design and default |
| [HIPAA](https://www.hhs.gov/hipaa/for-professionals/security/laws-regulations/index.html) | Healthcare Privacy | Health information privacy and security |
| [PCI DSS 4.0.1](https://blog.pcisecuritystandards.org/pci-dss-v4-0-resource-hub) | Payment Security | Payment card data security |
| [SOC 2 TSC](https://www.aicpa-cima.com/resources/download/2017-trust-services-criteria-with-revised-points-of-focus-2022) | Trust Services | Security, availability, confidentiality, processing integrity, privacy |

---

## Table of Contents

1. [Referenced Standards](#referenced-standards)
2. [Prerequisites](#prerequisites)
3. [Applicability Context](#applicability-context)
4. [Severity Dictionary](#severity-dictionary)
5. [Applicability Determination](#applicability-determination) — criteria for "(if applicable)" items
6. [Checkpointing](#checkpointing-long-reviews) — for long reviews / context limits
7. **MUST HAVE** (check in priority order):
   - [BIZ: Business](#business-expertise-biz) — Vision, Stakeholders, Requirements, Use Cases ⭐ Start here
   - [ARCH: Architecture](#architecture-expertise-arch) — Scope, Modularity, Scalability, Compatibility
   - [SEC: Security](#-security-expertise-sec) — Auth, Authorization, Data Classification, Privacy by Design
   - [SAFE: Safety](#-safety-expertise-safe) — Operational Safety, Hazard Prevention *(ISO 25010:2023)*
   - [TEST: Testing](#-testing-expertise-test) — Acceptance Criteria, Testability
   - [PERF: Performance](#-performance-expertise-perf) — Response Time, Throughput
   - [REL: Reliability](#️-reliability-expertise-rel) — Availability, Recovery
   - [UX: Usability](#-usability-expertise-ux) — UX Goals, Accessibility, Inclusivity
   - [DATA: Data](#-data-expertise-data) — Ownership, Quality, Lifecycle
   - [INT: Integration](#-integration-expertise-int) — External Systems, APIs
   - [COMPL: Compliance](#-compliance-expertise-compl) — Regulatory, Legal
   - [MAINT: Maintainability](#-maintainability-expertise-maint) — Documentation, Support
   - [OPS: Operations](#️-operations-expertise-ops) — Deployment, Monitoring
   - [Deliberate Omissions](#deliberate-omissions) — DOC-PRD-001
8. **MUST NOT HAVE**:
   - [No Technical Implementation](#-arch-prd-no-001-no-technical-implementation-details)
   - [No Architectural Decisions](#-arch-prd-no-002-no-architectural-decisions)
   - [No Implementation Tasks](#-biz-prd-no-001-no-implementation-tasks)
   - [No Spec-Level Design](#-biz-prd-no-002-no-spec-level-design)
   - [No Data Schema](#-data-prd-no-001-no-data-schema-definitions)
   - [No API Specs](#-int-prd-no-001-no-api-specifications)
   - [No Test Cases](#-test-prd-no-001-no-test-cases)
   - [No Infrastructure Specs](#-ops-prd-no-001-no-infrastructure-specifications)
   - [No Security Implementation](#-sec-prd-no-001-no-security-implementation-details)
   - [No Code-Level Docs](#-maint-prd-no-001-no-code-level-documentation)
8. [Validation Summary](#validation-summary)
9. [Reporting](#reporting)

**Review Priority**: BIZ → ARCH → SEC → TEST → (others as applicable)

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
- [ ] I will use the [Reporting](#reporting) format for output (see end of document)

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

## Applicability Determination

**For items marked "(if applicable)"**, determine applicability using these criteria:

| Domain | Applicable When | Not Applicable When |
|--------|-----------------|---------------------|
| Market positioning (BIZ-PRD-002) | External product, competitive market | Internal tool, no competitors |
| SSO/federation (SEC-PRD-001) | Enterprise product, multi-tenant | Single-user tool, local-only |
| Privacy by Design (SEC-PRD-005) | Handles EU personal data, PII | No personal data processing |
| Safety (SAFE-PRD-001/002) | Could harm people/property/environment, medical devices, vehicles, industrial | Pure information system, no physical interaction |
| Regulatory (COMPL-PRD-001) | Handles PII, financial data, healthcare | Internal dev tool, no user data |
| Accessibility (UX-PRD-002) | Public-facing, government, enterprise | Internal tool with known user base |
| Inclusivity (UX-PRD-005) | Diverse user base, public-facing | Narrow technical audience, internal tool |
| Internationalization (UX-PRD-003) | Multi-region deployment planned | Single-locale deployment |
| Offline capability (UX-PRD-004) | Mobile app, unreliable network | Server-side tool, always-connected |

**When uncertain**: Mark as applicable and let the PRD author explicitly exclude with reasoning.

---

## Checkpointing (Long Reviews)

This checklist is 700+ lines. For reviews that may exceed context limits:

### Checkpoint After Each Domain

After completing each expertise domain (BIZ, ARCH, SEC, etc.), output:
```
✓ {DOMAIN} complete: {N} items checked, {M} issues found
Issues: {list issue IDs}
Remaining: {list unchecked domains}
```

### If Context Runs Low

1. **Save progress**: List completed domains and issues found so far
2. **Note position**: "Stopped at {DOMAIN}-{ID}"
3. **Resume instruction**: "Continue from {DOMAIN}-{ID}, issues so far: {list}"

### Minimum Viable Review

If full review impossible, prioritize in this order:
1. **BIZ** (CRITICAL) — Vision, Requirements, Use Cases
2. **ARCH-PRD-001** (CRITICAL) — Scope Boundaries
3. **SEC-PRD-001/002** (CRITICAL) — Auth/Authorization
4. **DOC-PRD-001** (CRITICAL) — Deliberate Omissions
5. **MUST NOT HAVE** (all CRITICAL/HIGH items)

Mark review as "PARTIAL" if not all domains completed.

---

# MUST HAVE

---

## BUSINESS Expertise (BIZ)

> **Standards**: [ISO/IEC/IEEE 29148:2018](https://www.iso.org/standard/72089.html) §6.2 (StRS content), §6.4 (SRS content)

### BIZ-PRD-001: Vision Clarity
**Severity**: CRITICAL
**Ref**: ISO/IEC/IEEE 29148 §5.2.5 (Stakeholder requirements definition)

- [ ] Purpose statement explains WHY the product exists
- [ ] Target users clearly identified with specificity (not just "users")
- [ ] Key problems solved are concrete and measurable
- [ ] Success criteria are quantifiable (numbers, percentages, timeframes)
- [ ] Capabilities list covers core value propositions
- [ ] Business context is clear without requiring insider knowledge

### BIZ-PRD-002: Stakeholder Coverage
**Severity**: HIGH
**Ref**: ISO/IEC/IEEE 29148 §6.2.2 (Stakeholders), ISO 9241-210 §4 (HCD principles)

- [ ] All relevant user personas represented as actors
- [ ] Business sponsors' needs reflected in requirements
- [ ] End-user needs clearly articulated
- [ ] Organizational constraints acknowledged
- [ ] Market positioning context provided (if applicable)

### BIZ-PRD-003: Requirements Completeness
**Severity**: CRITICAL
**Ref**: ISO/IEC/IEEE 29148 §5.2.6 (Requirements analysis), §6.4.3 (Specific requirements)

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
**Ref**: ISO/IEC/IEEE 29148 §6.2.4 (Operational concept), ISO 9241-11 §5 (Measures of usability)

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

> **Standards**: [ISO/IEC 25010:2023](https://www.iso.org/standard/78176.html) (Maintainability, Flexibility), [ISO/IEC/IEEE 29148](https://www.iso.org/standard/72089.html) §6.3 (SyRS)

### ARCH-PRD-001: Scope Boundaries
**Severity**: CRITICAL
**Ref**: ISO/IEC/IEEE 29148 §6.3.2 (System overview), §6.3.4 (System interfaces)

- [ ] System boundaries are clear (what's in vs out of scope)
- [ ] Integration points with external systems identified
- [ ] Organizational boundaries respected
- [ ] Technology constraints acknowledged at high level
- [ ] No implementation decisions embedded in requirements

### ARCH-PRD-002: Modularity Enablement
**Severity**: MEDIUM
**Ref**: ISO/IEC 25010:2023 §4.2.7.2 (Modularity subcharacteristic)

- [ ] Requirements are decomposable into specs
- [ ] No monolithic "do everything" requirements
- [ ] Clear separation of concerns in requirement grouping
- [ ] Requirements support incremental delivery
- [ ] Dependencies don't create circular coupling

### ARCH-PRD-003: Scalability Considerations
**Severity**: MEDIUM
**Ref**: ISO/IEC 25010:2023 §4.2.8.4 (Scalability subcharacteristic of Flexibility)

- [ ] User volume expectations stated (current and projected)
- [ ] Data volume expectations stated (current and projected)
- [ ] Geographic distribution requirements captured
- [ ] Growth scenarios considered in requirements
- [ ] Performance expectations stated at business level

### ARCH-PRD-004: System Actor Clarity
**Severity**: HIGH
**Ref**: ISO/IEC/IEEE 29148 §6.3.4 (System interfaces)

- [ ] System actors represent real external systems
- [ ] System actor interfaces are clear
- [ ] Integration direction specified (inbound/outbound/bidirectional)
- [ ] System actor availability requirements stated
- [ ] Data exchange expectations documented

### ARCH-PRD-005: Compatibility Requirements
**Severity**: MEDIUM
**Ref**: [ISO/IEC 25010:2023](https://www.iso.org/standard/78176.html) §4.2.3 (Compatibility characteristic)

> **New in v1.2**: Added per ISO/IEC 25010:2023 which defines Compatibility as a distinct quality characteristic covering co-existence and interoperability.

- [ ] Co-existence requirements documented (operation alongside other products without adverse impact)
- [ ] Interoperability requirements stated (ability to exchange information with other systems)
- [ ] Data format compatibility requirements captured (file formats, protocols)
- [ ] Hardware/software environment compatibility stated
- [ ] Backward compatibility requirements documented (if applicable)

---

## 🔒 SECURITY Expertise (SEC)

> **Standards**: [ISO/IEC 25010:2023](https://www.iso.org/standard/78176.html) §4.2.6 (Security), [OWASP ASVS 5.0](https://owasp.org/www-project-application-security-verification-standard/), [NIST SP 800-53 Rev.5](https://csrc.nist.gov/pubs/sp/800/53/r5/upd1/final), [ISO/IEC 27001:2022](https://www.iso.org/standard/27001)

### SEC-PRD-001: Authentication Requirements
**Severity**: CRITICAL
**Ref**: OWASP ASVS V2 (Authentication), [RFC 6749](https://datatracker.ietf.org/doc/html/rfc6749) (OAuth 2.0), NIST 800-53 IA family

- [ ] User authentication needs stated
- [ ] Multi-factor requirements captured (if applicable)
- [ ] SSO/federation requirements documented
- [ ] Session management expectations stated
- [ ] Password/credential policies referenced

### SEC-PRD-002: Authorization Requirements
**Severity**: CRITICAL
**Ref**: OWASP ASVS V4 (Access Control), NIST 800-53 AC family, ISO 27001 A.9

- [ ] Role-based access clearly defined through actors
- [ ] Permission levels distinguished between actors
- [ ] Data access boundaries specified per actor
- [ ] Administrative vs user roles separated
- [ ] Delegation/impersonation needs captured

### SEC-PRD-003: Data Classification
**Severity**: HIGH
**Ref**: ISO/IEC 25010:2023 §4.2.6.2 (Confidentiality), NIST 800-53 SC family, [GDPR Art. 9](https://gdpr-info.eu/art-9-gdpr/)

- [ ] Sensitive data types identified
- [ ] PII handling requirements stated
- [ ] Data retention expectations documented
- [ ] Data deletion/anonymization needs captured
- [ ] Cross-border data transfer considerations noted

### SEC-PRD-004: Audit Requirements
**Severity**: MEDIUM
**Ref**: ISO/IEC 25010:2023 §4.2.6.5 (Accountability), NIST 800-53 AU family, SOC 2 CC6/CC7

- [ ] Audit logging needs identified
- [ ] User action tracking requirements stated
- [ ] Compliance reporting needs captured
- [ ] Forensic investigation support requirements noted
- [ ] Non-repudiation requirements documented (ISO 25010 §4.2.6.6)

### SEC-PRD-005: Privacy by Design
**Severity**: HIGH (if applicable)
**Ref**: [GDPR Article 25](https://gdpr-info.eu/art-25-gdpr/), [EDPB Guidelines 4/2019](https://www.edpb.europa.eu/sites/default/files/files/file1/edpb_guidelines_201904_dataprotection_by_design_and_by_default_v2.0_en.pdf)

> **New in v1.2**: Added per GDPR Article 25 requirement for data protection by design and by default. Applicable when processing personal data of EU residents or when building products that will handle PII.

- [ ] Privacy requirements embedded from project inception (not retrofitted)
- [ ] Data minimization principle stated (collect only what is necessary)
- [ ] Purpose limitation documented (data used only for stated purposes)
- [ ] Storage limitation requirements captured (retention periods defined)
- [ ] Privacy by default requirements stated (most privacy-protective settings as default)
- [ ] Pseudonymization/anonymization requirements documented where applicable

---

## 🛡️ SAFETY Expertise (SAFE)

> **Standards**: [ISO/IEC 25010:2023](https://www.iso.org/standard/78176.html) §4.2.9 (Safety characteristic) — **NEW in 2023 edition**

> **New in v1.2**: Safety was added as a distinct quality characteristic in ISO/IEC 25010:2023. Applicable for systems that could cause harm to people, property, or the environment.

### SAFE-PRD-001: Operational Safety Requirements
**Severity**: CRITICAL (if applicable)
**Ref**: ISO/IEC 25010:2023 §4.2.9.1 (Operational constraint), §4.2.9.2 (Risk identification)

- [ ] Safety-critical operations identified
- [ ] Operational constraints for safe operation documented
- [ ] Potential hazards identified and documented
- [ ] Risk levels assessed for identified hazards
- [ ] User actions that could lead to harm documented

### SAFE-PRD-002: Fail-Safe and Hazard Prevention
**Severity**: CRITICAL (if applicable)
**Ref**: ISO/IEC 25010:2023 §4.2.9.3 (Fail safe), §4.2.9.4 (Hazard warning), §4.2.9.5 (Safe integration)

- [ ] Fail-safe behavior requirements documented (safe state on failure)
- [ ] Hazard warning requirements stated (alerts for dangerous conditions)
- [ ] Emergency shutdown/stop requirements captured (if applicable)
- [ ] Safe integration requirements with other systems documented
- [ ] Human override capabilities defined where needed

---

## ⚡ PERFORMANCE Expertise (PERF)

> **Standards**: [ISO/IEC 25010:2023](https://www.iso.org/standard/78176.html) §4.2.2 (Performance Efficiency)

### PERF-PRD-001: Response Time Expectations
**Severity**: HIGH
**Ref**: ISO/IEC 25010:2023 §4.2.2.2 (Time behaviour)

- [ ] User-facing response time expectations stated
- [ ] Batch processing time expectations stated
- [ ] Report generation time expectations stated
- [ ] Search/query performance expectations stated
- [ ] Expectations are realistic for the problem domain

### PERF-PRD-002: Throughput Requirements
**Severity**: MEDIUM
**Ref**: ISO/IEC 25010:2023 §4.2.2.3 (Resource utilization), §4.2.2.4 (Capacity)

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

> **Standards**: [ISO/IEC 25010:2023](https://www.iso.org/standard/78176.html) §4.2.5 (Reliability), [ISO 22301:2019](https://www.iso.org/standard/75106.html) (Business Continuity), SOC 2 Availability TSC

### REL-PRD-001: Availability Requirements
**Severity**: HIGH
**Ref**: ISO/IEC 25010:2023 §4.2.5.2 (Availability), SOC 2 A1.1

- [ ] Uptime expectations stated (e.g., 99.9%)
- [ ] Maintenance window expectations documented
- [ ] Business hours vs 24/7 requirements clear
- [ ] Geographic availability requirements stated
- [ ] Degraded mode expectations documented

### REL-PRD-002: Recovery Requirements
**Severity**: HIGH
**Ref**: ISO/IEC 25010:2023 §4.2.5.4 (Recoverability), [ISO 22301:2019](https://www.iso.org/standard/75106.html) §8.4 (Business continuity plans), NIST 800-53 CP family

- [ ] Data loss tolerance stated (RPO — Recovery Point Objective)
- [ ] Downtime tolerance stated (RTO — Recovery Time Objective)
- [ ] Backup requirements documented
- [ ] Disaster recovery expectations stated
- [ ] Business continuity requirements captured

### REL-PRD-003: Error Handling Expectations
**Severity**: MEDIUM
**Ref**: ISO/IEC 25010:2023 §4.2.5.3 (Fault tolerance)

- [ ] User error handling expectations stated
- [ ] System error communication requirements documented
- [ ] Graceful degradation expectations captured
- [ ] Retry/recovery user experience documented
- [ ] Support escalation paths identified

---

## 👤 USABILITY Expertise (UX)

> **Standards**: [ISO/IEC 25010:2023](https://www.iso.org/standard/78176.html) §4.2.4 (Interaction Capability), [ISO 9241-11:2018](https://www.iso.org/standard/63500.html), [ISO 9241-210:2019](https://www.iso.org/standard/77520.html), [WCAG 2.2](https://www.w3.org/WAI/standards-guidelines/wcag/)

### UX-PRD-001: User Experience Goals
**Severity**: HIGH
**Ref**: ISO 9241-11 §5 (Framework for usability), ISO/IEC 25010:2023 §4.2.4 (Interaction Capability)

- [ ] Target user skill level defined
- [ ] Learning curve expectations stated (ISO 9241-11: efficiency)
- [ ] Efficiency goals for expert users documented
- [ ] Discoverability requirements for new users stated (ISO 25010 §4.2.4.3 Learnability)
- [ ] User satisfaction targets defined (ISO 9241-11: satisfaction)

### UX-PRD-002: Accessibility Requirements
**Severity**: HIGH
**Ref**: [WCAG 2.2](https://www.w3.org/WAI/standards-guidelines/wcag/) (A/AA/AAA levels), ISO/IEC 25010:2023 §4.2.4.7 (Accessibility), [EN 301 549](https://www.etsi.org/standards/en-301-549)

- [ ] Accessibility standards referenced (WCAG 2.2 level — typically AA)
- [ ] Assistive technology support requirements stated
- [ ] Keyboard navigation requirements documented (WCAG 2.1.1)
- [ ] Screen reader compatibility requirements stated (WCAG 4.1.2)
- [ ] Color/contrast requirements noted (WCAG 1.4.3, 1.4.11)

### UX-PRD-003: Internationalization Requirements
**Severity**: MEDIUM

- [ ] Supported languages listed
- [ ] Localization requirements documented
- [ ] Regional format requirements stated (dates, numbers, currency)
- [ ] RTL language support requirements noted
- [ ] Cultural considerations documented

### UX-PRD-004: Device/Platform Requirements
**Severity**: MEDIUM
**Ref**: ISO/IEC 25010:2023 §4.2.8 (Flexibility — Installability, Adaptability)

- [ ] Supported platforms listed (web, mobile, desktop)
- [ ] Browser requirements stated
- [ ] Mobile device requirements documented
- [ ] Offline capability requirements stated
- [ ] Responsive design requirements documented

### UX-PRD-005: Inclusivity Requirements
**Severity**: MEDIUM (if applicable)
**Ref**: [ISO/IEC 25010:2023](https://www.iso.org/standard/78176.html) §4.2.4.8 (Inclusivity) — **NEW subcharacteristic in 2023 edition**

> **New in v1.2**: Inclusivity was added as a subcharacteristic of Interaction Capability in ISO/IEC 25010:2023. It addresses the widest possible range of users, including those with different backgrounds, abilities, and characteristics.

- [ ] Diverse user populations considered (age, culture, language, ability)
- [ ] Cognitive accessibility requirements documented (beyond WCAG)
- [ ] Support for users with temporary situational limitations considered
- [ ] Cultural sensitivity requirements stated (if applicable)
- [ ] Design for neurodiverse users considered (if applicable)

---

## 🔧 MAINTAINABILITY Expertise (MAINT)

> **Standards**: [ISO/IEC 25010:2023](https://www.iso.org/standard/78176.html) §4.2.7 (Maintainability)

### MAINT-PRD-001: Documentation Requirements
**Severity**: MEDIUM
**Ref**: ISO/IEC/IEEE 29148 §6.6 (Information items)

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

> **Standards**: [GDPR](https://gdpr-info.eu/), [HIPAA](https://www.hhs.gov/hipaa/for-professionals/security/laws-regulations/index.html), [PCI DSS 4.0.1](https://blog.pcisecuritystandards.org/pci-dss-v4-0-resource-hub), [SOC 2 TSC](https://www.aicpa-cima.com/resources/download/2017-trust-services-criteria-with-revised-points-of-focus-2022), [SOX](https://www.sec.gov/about/laws/soa2002.pdf)

### COMPL-PRD-001: Regulatory Requirements
**Severity**: CRITICAL (if applicable)
**Ref**: GDPR (EU personal data), HIPAA (US healthcare), PCI DSS (payment cards), SOX (financial reporting)

- [ ] Applicable regulations identified (GDPR, HIPAA, SOX, PCI DSS, etc.)
- [ ] Compliance certification requirements stated
- [ ] Audit requirements documented
- [ ] Reporting requirements captured
- [ ] Data sovereignty requirements stated (GDPR Art. 44-49)

### COMPL-PRD-002: Industry Standards
**Severity**: MEDIUM
**Ref**: ISO 27001 (security), ISO 22301 (continuity), SOC 2 (trust services), ISO 9001 (quality)

- [ ] Industry standards referenced (ISO, NIST, OWASP, etc.)
- [ ] Best practice frameworks identified
- [ ] Certification requirements stated (ISO 27001, SOC 2, etc.)
- [ ] Interoperability standards documented
- [ ] Security standards referenced (OWASP ASVS, NIST 800-53)

### COMPL-PRD-003: Legal Requirements
**Severity**: HIGH (if applicable)
**Ref**: GDPR Art. 12-23 (Data subject rights), GDPR Art. 6-7 (Consent)

- [ ] Terms of service requirements stated
- [ ] Privacy policy requirements documented
- [ ] Consent management requirements captured (GDPR Art. 7)
- [ ] Data subject rights requirements stated (access, rectification, erasure, portability)
- [ ] Contractual obligations documented

---

## 📊 DATA Expertise (DATA)

> **Standards**: [GDPR](https://gdpr-info.eu/) (personal data), [ISO/IEC 25012](https://www.iso.org/standard/35736.html) (Data Quality)

### DATA-PRD-001: Data Ownership
**Severity**: HIGH
**Ref**: GDPR Art. 4 (Definitions — controller, processor), GDPR Art. 26 (Joint controllers)

- [ ] Data ownership clearly defined
- [ ] Data stewardship responsibilities identified (controller vs processor)
- [ ] Data sharing expectations documented
- [ ] Third-party data usage requirements stated (GDPR Art. 28)
- [ ] User-generated content ownership defined

### DATA-PRD-002: Data Quality Requirements
**Severity**: MEDIUM
**Ref**: [ISO/IEC 25012](https://www.iso.org/standard/35736.html) (Data Quality model), GDPR Art. 5(1)(d) (Accuracy principle)

- [ ] Data accuracy requirements stated (ISO 25012 §4.2.1)
- [ ] Data completeness requirements documented (ISO 25012 §4.2.2)
- [ ] Data freshness requirements captured (ISO 25012 §4.2.8 Currentness)
- [ ] Data validation requirements stated
- [ ] Data cleansing requirements documented

### DATA-PRD-003: Data Lifecycle
**Severity**: MEDIUM
**Ref**: GDPR Art. 5(1)(e) (Storage limitation), GDPR Art. 17 (Right to erasure)

- [ ] Data retention requirements stated (GDPR storage limitation)
- [ ] Data archival requirements documented
- [ ] Data purging requirements captured (right to erasure)
- [ ] Data migration requirements stated
- [ ] Historical data access requirements documented

---

## 🔌 INTEGRATION Expertise (INT)

> **Standards**: [ISO/IEC 25010:2023](https://www.iso.org/standard/78176.html) §4.2.3 (Compatibility — Interoperability)

### INT-PRD-001: External System Integration
**Severity**: HIGH
**Ref**: ISO/IEC 25010:2023 §4.2.3.2 (Interoperability), ISO/IEC/IEEE 29148 §6.3.4 (System interfaces)

- [ ] Required integrations listed
- [ ] Integration direction specified
- [ ] Data exchange requirements documented
- [ ] Integration availability requirements stated
- [ ] Fallback requirements for integration failures documented

### INT-PRD-002: API Requirements
**Severity**: MEDIUM
**Ref**: [OpenAPI Specification](https://spec.openapis.org/oas/latest.html), RFC 6749 (OAuth for API auth)

- [ ] API exposure requirements stated
- [ ] API consumer requirements documented
- [ ] API versioning requirements stated
- [ ] Rate limiting expectations documented
- [ ] API documentation requirements stated (OpenAPI/Swagger)

---

## 🖥️ OPERATIONS Expertise (OPS)

> **Standards**: [ISO 22301:2019](https://www.iso.org/standard/75106.html) (Business Continuity), NIST 800-53 CM/CP families

### OPS-PRD-001: Deployment Requirements
**Severity**: MEDIUM
**Ref**: NIST 800-53 CM family (Configuration Management)

- [ ] Deployment environment requirements stated
- [ ] Release frequency expectations documented
- [ ] Rollback requirements captured
- [ ] Blue/green or canary requirements stated
- [ ] Environment parity requirements documented

### OPS-PRD-002: Monitoring Requirements
**Severity**: MEDIUM
**Ref**: NIST 800-53 AU family (Audit and Accountability), SI family (System and Information Integrity)

- [ ] Alerting requirements stated
- [ ] Dashboard requirements documented
- [ ] Log retention requirements captured
- [ ] Incident response requirements stated (NIST 800-53 IR family)
- [ ] Capacity monitoring requirements documented

---

## 🧪 TESTING Expertise (TEST)

> **Standards**: [ISO/IEC/IEEE 29119](https://www.iso.org/standard/81291.html) (Software Testing), [ISO/IEC/IEEE 29148](https://www.iso.org/standard/72089.html) §5.2.8 (Verification)

### TEST-PRD-001: Acceptance Criteria
**Severity**: HIGH
**Ref**: ISO/IEC/IEEE 29148 §5.2.8 (Requirements verification), ISO/IEC/IEEE 29119-1 §4 (Test concepts)

- [ ] Each functional requirement has verifiable acceptance criteria
- [ ] Use cases define expected outcomes
- [ ] NFRs have measurable thresholds
- [ ] Edge cases are testable
- [ ] Negative test cases implied

### TEST-PRD-002: Testability
**Severity**: MEDIUM
**Ref**: ISO/IEC/IEEE 29148 §5.2.5 (Characteristics of well-formed requirements), ISO/IEC 25010:2023 §4.2.7.4 (Testability)

- [ ] Requirements are unambiguous enough to test (ISO 29148 §5.2.5)
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

**Where it belongs**: Project management tools (Jira, Linear, etc.) or Spec DESIGN

---

## ❌ BIZ-PRD-NO-002: No Spec-Level Design
**Severity**: HIGH

**What to check**:
- [ ] No detailed user flows
- [ ] No wireframes or UI specifications
- [ ] No algorithm descriptions
- [ ] No state machine definitions
- [ ] No detailed error handling logic

**Where it belongs**: `Spec DESIGN` (Spec Design)

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
