# ADR-0003: Browser Support Policy

**Date**: 2026-02-07

**Status**: Accepted

**ID**: `cpt-examples-todo-app-adr-browser-support`

## Context and Problem Statement

The Todo App is a browser-based application and relies on modern platform capabilities (IndexedDB, Service Worker, WebSocket). We need a clear browser support policy to balance compatibility with development complexity.

## Decision Drivers

- Ensure offline-first features work reliably
- Keep implementation complexity reasonable
- Align with typical user environments for a personal productivity app

## Considered Options

- Support only Chrome (fastest development)
- Support latest 2 versions of major browsers
- Support long-tail legacy browsers

## Decision Outcome

Chosen option: **Support latest 2 versions of Chrome, Firefox, Safari, and Edge**.

### Consequences

- Good, because offline-first primitives (IndexedDB) are available and stable
- Good, because aligns with modern browser update cadence
- Bad, because we may need small compatibility shims across engines

### Confirmation

Confirmed via:

- Browser test matrix execution (latest 2 versions policy) in CI
- Manual smoke test of offline-first flows across supported browsers

## Traceability

- **PRD**: [PRD.md](../PRD.md)
- **DESIGN**: [DESIGN.md](../DESIGN.md)

This decision directly addresses the following requirements or design elements:

- `cpt-examples-todo-app-constraint-browser-compat`
- `cpt-examples-todo-app-nfr-offline-support`
- `cpt-examples-todo-app-interface-indexeddb`
- `cpt-examples-todo-app-actor-user`
