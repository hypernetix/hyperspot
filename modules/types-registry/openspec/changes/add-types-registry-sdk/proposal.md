# Change: Add Types Registry SDK

## Why

HyperSpot needs a lightweight SDK crate (`types-registry-sdk`) that defines the public API contracts for the Types Registry module. This SDK enables other modules to depend on the API without pulling in the full implementation, following the established SDK pattern used by other HyperSpot modules.

## What Changes

**New Crate: `types-registry-sdk`**
- `TypesRegistryApi` trait with 3 core methods: `register`, `list`, `get`
- `GtsEntity` model using `GtsIdSegment` from gts-rust
- `TypesRegistryError` enum for error handling
- `ListQuery` struct for filtering entities
- Re-exports of gts-rust types needed by consumers

## Impact

- **Affected specs**: `types-registry-sdk` (new capability)
- **Affected code**: `modules/types-registry-sdk/` (new crate)
- **Dependencies**: gts-rust, serde, async-trait
