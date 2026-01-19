# ADR — Identified White Spots And Next ADRs Scope

**Source:** Consistency review of `ADR_DOMAIN_MODEL_AND_APIS.md` against `PRD.md`  
**Date:** 2026-01-21

---

## 1. Unaddressed PRD Requirements

### P0 Requirements Not Fully Addressed

| PRD ID | Requirement                                | Gap Description                                                                      | Severity    |
|--------|--------------------------------------------|--------------------------------------------------------------------------------------|-------------|
| BR-026 | Secure handling of secrets                 | No `secret_ref` type or secret binding model for workflows to reference secrets      | **Blocker** |
| BR-039 | Injection attack prevention                | No input sanitization rules or validation constraints in schema definitions          | **Blocker** |
| BR-040 | Privilege escalation prevention            | No privilege scope constraints or execution identity validation model                | **Blocker** |
| BR-041 | Resource exhaustion protection             | Limits defined but no detection/termination model for spinning loops or memory leaks | High        |
| BR-008 | Runtime capabilities (HTTP, events, audit) | No SDK/capability interface for workflow authors to invoke platform services         | High        |
| BR-012 | Graceful disconnection handling            | No adapter health model or API for rejecting starts when adapter disconnected        | High        |
| BR-014 | Long-running credential refresh            | No model for token refresh or credential lifecycle in security context               | High        |

### P1 Requirements Not Addressed

| PRD ID | Requirement                                     | Gap Description                                                 |
|--------|-------------------------------------------------|-----------------------------------------------------------------|
| BR-101 | Debugging with breakpoints                      | No debugging API or breakpoint model                            |
| BR-102 | Step-through execution                          | No step-through control model                                   |
| BR-104 | Child workflows / modular composition           | No parent-child invocation relationship model                   |
| BR-105 | Parallel execution with concurrency caps        | No parallel execution model or concurrency controls for steps   |
| BR-108 | External signals and manual intervention        | Partial (suspend/resume exists), but no signal delivery model   |
| BR-109 | Alerts and notifications                        | No notification/alert model or subscription mechanism           |
| BR-114 | Dependency management                           | No dependency declaration or compatibility model                |
| BR-115 | Distributed tracing                             | `trace_id` field exists but no integration or propagation model |
| BR-117 | Environment customization (timezone, locale)    | No execution environment configuration model                    |
| BR-119 | Monitoring dashboards                           | No dashboard model (implementation concern)                     |
| BR-120 | Performance profiling                           | No profiling model or data schema                               |
| BR-121 | Blue-green deployment                           | No deployment strategy model                                    |
| BR-122 | Publishing governance                           | No review/approval workflow model                               |
| BR-125 | Workflow visualization                          | No visualization data model or API                              |
| BR-127 | Debugging access control with sensitive masking | No sensitive field annotations in schemas                       |
| BR-129 | Standardized error taxonomy                     | Base error type exists; specific error types not enumerated     |
| BR-130 | Debug call trace (masked secrets)               | Call trace not modeled; masking rules not defined               |

### P2 Requirements Not Addressed

| PRD ID | Requirement                   | Note                         |
|--------|-------------------------------|------------------------------|
| BR-201 | Long-term archival            | Future scope                 |
| BR-202 | Import/export                 | Future scope                 |
| BR-203 | Execution time travel         | Future scope                 |
| BR-204 | A/B testing                   | Future scope                 |
| BR-205 | Canary releases               | Future scope                 |
| BR-206 | Stronger isolation boundaries | Future scope (sandbox model) |

---

## 2. Next ADR Scope (Recommended)

### ADR-2: Security Model (P0 — Blocker)

**Scope:**

- Secret reference model (`secret_ref` type) and secret binding for entrypoints
- Privilege scope constraints and execution identity validation
- Sandbox isolation model and boundaries
- Sensitive field masking annotations for schemas
- Input sanitization rules and injection prevention patterns

**PRD Coverage:** BR-026, BR-039, BR-040, BR-127, BR-130, PRD Risks 

### ADR-3: Runtime Capabilities SDK (P0 — High Priority)

**Scope:**

- Capability interface for workflows (HTTP client, event publisher, audit logger)
- Platform operation invocation model
- Resource exhaustion detection and termination model
- Adapter health and disconnection handling

**PRD Coverage:** BR-008, BR-012, BR-041

### ADR-4: Debugging and Observability (P1)

**Scope:**

- Debugging API (breakpoints, step-through, inspection)
- Debug session model and access control
- Call trace schema with duration and masked I/O
- Performance profiling data model
- Distributed tracing propagation

**PRD Coverage:** BR-101, BR-102, BR-115, BR-120, BR-130

### ADR-5: Advanced Workflow Patterns (P1)

**Scope:**

- Parent-child workflow relationship model
- Parallel execution and concurrency control model
- External signal delivery to suspended workflows
- Dependency declaration and compatibility

**PRD Coverage:** BR-104, BR-105, BR-108, BR-114

### ADR-6: Deployment and Governance (P1)

**Scope:**

- Blue-green deployment strategy model
- Publishing governance (review/approval) workflow
- Alerts and notification model
- Execution environment customization (timezone, locale)

**PRD Coverage:** BR-109, BR-117, BR-121, BR-122

### ADR-7: Error Taxonomy (P1)

**Scope:**

- Enumerate specific error types for all failure categories
- Error code registry and documentation
- Error-to-retry-policy mapping

**PRD Coverage:** BR-129


