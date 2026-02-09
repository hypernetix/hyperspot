# Feature: {Feature Name}

<!--
=============================================================================
FEATURE SPECIFICATION
=============================================================================
PURPOSE: Define detailed implementation behavior — flows, algorithms, states,
and implementation requirements that bridge PRD and DESIGN to code.

SCOPE:
  ✓ Actor flows (user-facing interactions, step by step)
  ✓ Processes / Business Logic (incl. internal logic, validation, async jobs, etc)
  ✓ State machines (entity lifecycle)
  ✓ Implementation requirements (what to build)
  ✓ Acceptance criteria (how to verify)

NOT IN THIS DOCUMENT (see other templates):
  ✗ Requirements → PRD.md
  ✗ Architecture, components, APIs → DESIGN.md
  ✗ Why a specific approach was chosen → ADR/

CDSL PSEUDO-CODE:
  Optional. Use for complex flows or when precise behavior must be
  communicated. Skip for simple features to avoid overhead.
=============================================================================
-->

## 1. Feature Context

<!-- reference to DECOMPOSITION entry -->
- [ ] `p2` `cpt-{system}-feature-{slug}`

### 1.1 Overview

{Brief overview of what this feature does — 1-2 sentences.}

### 1.2 Purpose

{Why this feature exists, what PRD requirements or DESIGN element it addresses.}

### 1.3 Actors

| Actor | Role in Feature |
|-------|-----------------|
| `cpt-{system}-actor-{slug}` | {What this actor does in this feature} |

### 1.4 References

- **PRD**: [PRD.md](../PRD.md)
- **Design**: [DESIGN.md](../DESIGN.md)
- **Dependencies**: {List feature dependencies or "None"}

## 2. Actor Flows (CDSL)

User-facing interactions that start with an actor (human or external system) and describe the end-to-end flow of a use case. Each flow has a triggering actor and shows how the system responds to actor actions.

> **CDSL pseudo-code is optional.** Use detailed steps for early-stage projects, complex domains, or when you need to clearly communicate expected behavior. Skip for mature teams or simple features to avoid documentation overhead.

### {Flow Name}

- [ ] `p1` - **ID**: `cpt-{system}-flow-{slug}`

**Actor**: `cpt-{system}-actor-{slug}`

**Success Scenarios**:
- {Scenario 1}

**Error Scenarios**:
- {Error scenario 1}

**Steps**:
1. [ ] - `p1` - {Actor action} - `inst-{step-id}`
2. [ ] - `p1` - {API: METHOD /path (request/response summary)} - `inst-{step-id}`
3. [ ] - `p1` - {DB: OPERATION table(s) (key columns/filters)} - `inst-{step-id}`
4. [ ] - `p1` - **IF** {condition} - `inst-{step-id}`
   1. [ ] - `p1` - {Action if true} - `inst-{step-id}`
5. [ ] - `p1` - **ELSE** - `inst-{step-id}`
   1. [ ] - `p1` - {Action if false} - `inst-{step-id}`
6. [ ] - `p1` - **RETURN** {result} - `inst-{step-id}`

## 3. Processes / Business Logic (CDSL)

Internal system functions and procedures that do not interact with actors directly. Examples: database layer operations, authorization logic, middleware, validation routines, library functions, background jobs. These are reusable building blocks called by Actor Flows or other processes.

> **CDSL pseudo-code is optional.** Same guidance as Actor Flows — use when clarity matters, skip when it becomes overhead.

### {Process Name}

- [ ] `p2` - **ID**: `cpt-{system}-algo-{slug}`

**Input**: {Input description}

**Output**: {Output description}

**Steps**:
1. [ ] - `p1` - {Parse/normalize input} - `inst-{step-id}`
2. [ ] - `p1` - {DB: OPERATION table(s) (key columns/filters)} - `inst-{step-id}`
3. [ ] - `p1` - {API: METHOD /path (request/response summary)} - `inst-{step-id}`
4. [ ] - `p1` - **FOR EACH** {item} in {collection} - `inst-{step-id}`
   1. [ ] - `p1` - {Process item} - `inst-{step-id}`
5. [ ] - `p1` - **TRY** - `inst-{step-id}`
   1. [ ] - `p1` - {Risky operation} - `inst-{step-id}`
6. [ ] - `p1` - **CATCH** {error} - `inst-{step-id}`
   1. [ ] - `p1` - {Handle error} - `inst-{step-id}`
7. [ ] - `p1` - **RETURN** {result} - `inst-{step-id}`

## 4. States (CDSL)

Optional: Include when entities have explicit lifecycle states.

### {Entity Name} State Machine

- [ ] `p2` - **ID**: `cpt-{system}-state-{slug}`

**States**: {State1}, {State2}, {State3}

**Initial State**: {State1}

**Transitions**:
1. [ ] - `p1` - **FROM** {State1} **TO** {State2} **WHEN** {condition} - `inst-{step-id}`
2. [ ] - `p1` - **FROM** {State2} **TO** {State3} **WHEN** {condition} - `inst-{step-id}`

## 5. Definitions of Done

Specific implementation tasks derived from flows/algorithms above.

### {Requirement Title}

- [ ] `p1` - **ID**: `cpt-{system}-dod-{slug}`

The system **MUST** {clear description of what to implement}.

**Implements**:
- `cpt-{system}-flow-{slug}`

**Touches**:
- API: `{METHOD} {/path}`
- DB: `{table}`
- Entities: `{EntityName}`

## 6. Acceptance Criteria

- [ ] {Testable criterion for this feature}
- [ ] {Another testable criterion}
