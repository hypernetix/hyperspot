# Feature: {Feature Name}

## 1. Feature Context

**Feature ID**: `fdd-{module-name}-feature-{feature-slug}`

**Status**: NOT_STARTED | IN_PROGRESS | IMPLEMENTED

### 1.1 Overview

{Brief overview of what this feature does}

### 1.2 Purpose

{Why this feature exists, what problem it solves}

### 1.3 Actors

- `fdd-{module-name}-actor-{actor-slug}` - {Role in this feature}

### 1.4 References

- Overall Design: [DESIGN.md](../../DESIGN.md)
- Dependencies: {List feature dependencies or "None"}

## 2. Actor Flows (FDL)

User-facing interactions that start with an actor (human or external system) and describe the end-to-end flow of a use case. Each flow has a triggering actor and shows how the system responds to actor actions.

> **FDL pseudo-code is optional.** Use detailed steps for early-stage projects, complex domains, or when you need to clearly communicate expected behavior. Skip for mature teams or simple features to avoid documentation overhead.

### {Flow Name}

- [ ] **ID**: `fdd-{module-name}-feature-{feature-slug}-flow-{flow-slug}`

<!-- fdd-id-content -->
**Actor**: `fdd-{module-name}-actor-{actor-slug}`

**Success Scenarios**:
- {Scenario 1}

**Error Scenarios**:
- {Error scenario 1}

**Steps**:
1. [ ] - `ph-1` - {Actor action} - `inst-{step-id}`
2. [ ] - `ph-1` - {API: METHOD /path (request/response summary)} - `inst-{step-id}`
3. [ ] - `ph-1` - {DB: OPERATION table(s) (key columns/filters)} - `inst-{step-id}`
4. [ ] - `ph-1` - **IF** {condition} - `inst-{step-id}`
   1. [ ] - `ph-1` - {Action if true (include API/DB/Integration details)} - `inst-{step-id}`
5. [ ] - `ph-1` - **ELSE** - `inst-{step-id}`
   1. [ ] - `ph-1` - {Action if false (include API/DB/Integration details)} - `inst-{step-id}`
6. [ ] - `ph-1` - **RETURN** {result} - `inst-{step-id}`
<!-- fdd-id-content -->

<!-- TODO: Add more flows as needed -->

## 3. Algorithms (FDL)

Internal system functions and procedures that do not interact with actors directly. Examples: database layer operations, authorization logic, middleware, validation routines, library functions, background jobs. These are reusable building blocks called by Actor Flows or other Algorithms.

> **FDL pseudo-code is optional.** Same guidance as Actor Flows — use when clarity matters, skip when it becomes overhead.

### {Algorithm Name}

- [ ] **ID**: `fdd-{module-name}-feature-{feature-slug}-algo-{algo-slug}`

<!-- fdd-id-content -->
**Input**: {Input description}

**Output**: {Output description}

**Steps**:
1. [ ] - `ph-1` - {Parse/normalize input} - `inst-{step-id}`
2. [ ] - `ph-1` - {DB: OPERATION table(s) (key columns/filters)} - `inst-{step-id}`
3. [ ] - `ph-1` - {API: METHOD /path (request/response summary)} - `inst-{step-id}`
4. [ ] - `ph-1` - **FOR EACH** {item} in {collection} - `inst-{step-id}`
   1. [ ] - `ph-1` - {Process item (include API/DB/Integration details)} - `inst-{step-id}`
5. [ ] - `ph-1` - **TRY** - `inst-{step-id}`
   1. [ ] - `ph-1` - {Risky operation (include API/DB/Integration details)} - `inst-{step-id}`
6. [ ] - `ph-1` - **CATCH** {error} - `inst-{step-id}`
   1. [ ] - `ph-1` - {Handle error} - `inst-{step-id}`
7. [ ] - `ph-1` - **RETURN** {result} - `inst-{step-id}`
<!-- fdd-id-content -->

<!-- TODO: Add more algorithms as needed -->

## 4. States (FDL)

### {Entity Name} State Machine

- [ ] **ID**: `fdd-{module-name}-feature-{feature-slug}-state-{entity-slug}`

<!-- fdd-id-content -->
**States**: {State1}, {State2}, {State3}

**Initial State**: {State1}

**Transitions**:
1. [ ] - `ph-1` - **FROM** {State1} **TO** {State2} **WHEN** {condition} - `inst-{step-id}`
2. [ ] - `ph-1` - **FROM** {State2} **TO** {State3} **WHEN** {condition} - `inst-{step-id}`
<!-- fdd-id-content -->

<!-- TODO: Add more state machines as needed -->
<!-- Note: This section is optional if feature has no state management -->

<!-- TODO: What should be done, a list of requirements to be implemented -->
## 5. Requirements

### {Requirement Title}

- [ ] **ID**: `fdd-{module-name}-feature-{feature-slug}-req-{req-slug}`

<!-- fdd-id-content -->
**Status**: NOT_STARTED | IN_PROGRESS | COMPLETED

**Description**: {Clear description with SHALL/MUST statements}

**Implementation details**:
- {If this requirement touches API: specify endpoint/method + request/response}
- {If this requirement touches DB: specify exact query shape and tables}
- {If this requirement touches domain entities: list entity names and identifiers}

<!-- Algorithms, flows, states which should be implemented -->
**Implements**:
- `fdd-{module-name}-feature-{feature-slug}-flow-{flow-slug}`
- `fdd-{module-name}-feature-{feature-slug}-algo-{algo-slug}`
- `fdd-{module-name}-feature-{feature-slug}-state-{entity-slug}`

**Phases**:
- [ ] `ph-1`: {What is implemented in this phase}
<!-- fdd-id-content -->

<!-- TODO: Add more requirements as needed -->

## 6. Additional Context (optional)

### {Context Item Title}

**ID**: `fdd-{module-name}-feature-{feature-slug}-context-{context-slug}`

<!-- fdd-id-content -->
{Additional notes and context that inform implementation.}
<!-- fdd-id-content -->
