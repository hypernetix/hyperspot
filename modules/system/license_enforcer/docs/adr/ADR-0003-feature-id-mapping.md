# ADR-0003: Map Platform Feature IDs in Plugins

**Date**: 2026-01-28

**Status**: Accepted

**ID**: `fdd-license-enforcer-adr-feature-id-mapping`

## Context and Problem Statement

Platforms may use arbitrary feature identifier formats, while HyperSpot uses GTS feature IDs. We need a mapping strategy that keeps modules independent from Platform identifiers.

## Decision Drivers

* Allow arbitrary Platform feature ID formats
* Keep HyperSpot modules on stable GTS IDs
* Avoid leaking Platform-specific identifiers into core logic

## Considered Options

* Require Platforms to adopt HyperSpot GTS IDs
* Perform ID mapping in the gateway core
* Delegate ID mapping to the Platform plugin

## Decision Outcome

Chosen option: "Delegate ID mapping to the Platform plugin", because it isolates Platform-specific identifiers and keeps the gateway Platform-agnostic.

### Consequences

* Good, because core modules only use HyperSpot GTS IDs
* Good, because each plugin can implement Platform-specific mappings
* Good, because platform identifiers are isolated in plugins and do not leak into core modules
* Bad, because mapping logic must be maintained per plugin

## Related Design Elements

**Requirements**:
* `fdd-license-enforcer-fr-feature-id-mapping` - Feature ID translation
