# Change: Add Types Registry Module

## Why

With the `types-registry-sdk` crate providing the public API contracts, we need the actual module implementation that provides GTS entity registration, storage, validation, and REST API endpoints. This module integrates with gts-rust for all GTS operations and follows the HyperSpot module patterns.

**Depends on**: `add-types-registry-sdk` (must be implemented first)

## What Changes

**New Crate: `types-registry`**
- Module declaration with `#[modkit::module(name = "types_registry", capabilities = [system, rest])]`
- Two-phase storage (configuration + production) using gts-rust `GtsOps`
- Domain service with business logic
- Local client implementing `TypesRegistryApi` trait
- REST API handlers, routes, and DTOs
- ClientHub registration for inter-module access
- Full gts-rust integration (OP#1-OP#11)

## Impact

- **Affected specs**: `types-registry` (new capability)
- **Affected code**: `modules/types-registry/` (new crate)
- **Dependencies**: types-registry-sdk, gts-rust, modkit, axum
