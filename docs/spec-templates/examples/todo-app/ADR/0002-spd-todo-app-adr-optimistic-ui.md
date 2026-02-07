# ADR-0002: Optimistic UI Updates

**Date**: 2026-02-07

**Status**: Accepted

**ID**: `cpt-todo-app-adr-optimistic-ui`

## Context and Problem Statement

The Todo App should feel fast and responsive even when network conditions are poor or intermittent. If the UI waits for server confirmation for every action, perceived latency increases and offline-first behavior becomes inconsistent.

## Decision Drivers

- Maintain a fast perceived response time for core actions
- Preserve offline-first behavior without blocking on network
- Provide a consistent UX for create/complete/delete actions
- Allow safe rollback on server-side rejections or conflicts

## Considered Options

- Always wait for server confirmation before updating UI
- Optimistically update UI and reconcile in background
- Hybrid approach per action type

## Decision Outcome

Chosen option: **Optimistically update UI and reconcile in background**.

### Consequences

- Good, because UI stays responsive and consistent with offline-first flow
- Good, because actions work without network connectivity
- Bad, because we need reconciliation/rollback logic for rare rejection/conflict cases

## Related Design Elements

**Principles**:
- `cpt-todo-app-design-principle-optimistic-updates`
- `cpt-todo-app-design-principle-offline-first`

**Requirements**:
- `cpt-todo-app-nfr-response-time`
- `cpt-todo-app-nfr-offline-support`

**Features / Flows**:
- `cpt-todo-app-flow-core-create-task`
- `cpt-todo-app-flow-core-delete-task`

**Actors**:
- `cpt-todo-app-actor-user`
- `cpt-todo-app-actor-sync-service`
