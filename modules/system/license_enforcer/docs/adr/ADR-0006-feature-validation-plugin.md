# ADR-0006: Keep Feature ID Validation in Plugins

**Date**: 2026-01-30

**Status**: Accepted

**ID**: `fdd-license-enforcer-adr-plugin-feature-validation`

## Context and Problem Statement

HyperSpot feature IDs are represented as GTS identifiers, but Platform licensing systems use their own identifiers. The gateway could require feature IDs to be registered in the types-registry and validate against that set, yet the platform plugin is the only component that knows which HyperSpot feature IDs can be mapped to Platform feature identifiers. Types-registry stores GTS entities and does not provide feature ID mapping semantics.

## Decision Drivers

* Avoid coupling license enforcement to types-registry feature lists
* Keep feature mapping logic in the Platform plugin where the external contract is known
* Allow plugins to reject unmappable feature IDs even if other modules registered them
* Minimize unnecessary registry requirements for license checks

## Considered Options

* Require all license feature IDs to be registered in types-registry and validated by the gateway
* Maintain a hardcoded allow-list in the gateway
* Delegate validation/mapping to the platform plugin (no gateway registry requirement)

## Decision Outcome

Chosen option: "Delegate validation/mapping to the platform plugin (no gateway registry requirement)", because the plugin owns the mapping contract and types-registry does not encode mapping semantics.

### Consequences

* Good, because the gateway remains platform-agnostic and does not enforce an incomplete registry allow-list
* Good, because plugins can reject unmappable feature IDs even if they are registered elsewhere
* Bad, because feature ID validity is enforced at runtime by plugins instead of central registry validation

## Related Design Elements

**Requirements**:
* `fdd-license-enforcer-fr-feature-id-mapping` - Feature ID translation
