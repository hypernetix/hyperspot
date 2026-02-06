# ADR-0001: Delegate Platform Integration to Plugins

**Date**: 2026-01-28

**Status**: Accepted

**ID**: `fdd-license-enforcer-adr-plugin-boundary`

## Context and Problem Statement

License enforcement must work with different Platforms without embedding Platform-specific licensing logic in HyperSpot modules. We need a boundary that isolates external API contracts and lets deployments select the appropriate integration.

## Decision Drivers

* Keep core license enforcement Platform-agnostic
* Support multiple licensing backends in different deployments
* Enable isolated testing of Platform integrations
* Limit blast radius of Platform API changes

## Considered Options

* Implement Platform-specific calls directly in the gateway
* Compile-time selection of a single Platform client
* Delegate Platform integration to a plugin interface

## Decision Outcome

Chosen option: "Delegate Platform integration to a plugin interface", because it preserves a stable core gateway and allows multiple Platform backends without coupling modules to external APIs.

### Consequences

* Good, because Platform logic is isolated and replaceable
* Good, because multiple plugins can coexist
* Bad, because configuration is required to select the plugin
* Bad, because each plugin must maintain its own API mapping and tests

## Related Design Elements

**Actors**:
* `fdd-license-enforcer-actor-plugin` - Encapsulates Platform API integration
* `fdd-license-enforcer-actor-hs-module` - Consumer of the stable SDK

**Requirements**:
* `fdd-license-enforcer-fr-plugin-integration` - Delegation of Platform calls
