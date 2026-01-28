# ADR-0005: Phase 1 Read-Only Scope

**Date**: 2026-01-28

**Status**: Accepted

**ID**: `fdd-license-enforcer-adr-phase1-scope`

## Context and Problem Statement

Phase 1 must deliver minimal feature gating without expanding into quotas, usage tracking, or subscription management. We need to constrain scope to the smallest viable set of capabilities.

## Decision Drivers

* Deliver feature gating quickly and safely
* Avoid duplicating Platform licensing and billing logic
* Limit complexity for phase 1

## Considered Options

* Implement full licensing, quotas, and usage reporting
* Combine global feature checks and usage reporting
* Provide read-only global feature checks only
* Add quota checks without reporting

## Decision Outcome

Chosen option: "Provide read-only global feature checks only", because it meets the phase 1 goal while keeping licensing and billing on the Platform side.

### Consequences

* Good, because phase 1 is focused and low-risk
* Good, because usage concerns are separated from feature gating
* Bad, because more complex licensing scenarios are not available in phase 1

## Related Design Elements

**Requirements**:
* `fdd-license-enforcer-constraint-phase1-readonly` - Phase 1 scope constraint
