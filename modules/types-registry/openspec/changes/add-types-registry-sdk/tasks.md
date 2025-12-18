# Tasks: Types Registry SDK

## 1. SDK Crate Setup

- [ ] 1.1 Create `types-registry-sdk/Cargo.toml` with minimal dependencies
- [ ] 1.2 Define `GtsEntity` model using `GtsIdSegment` from gts-rust
- [ ] 1.3 Define `TypesRegistryError` enum
- [ ] 1.4 Define `TypesRegistryApi` trait with 3 methods:
  - `register(&SecurityCtx, Vec<serde_json::Value>) -> Result<Vec<GtsEntity>>` (batch registration)
  - `list(&SecurityCtx, ListQuery) -> Result<Vec<GtsEntity>>`
  - `get(&SecurityCtx, &str) -> Result<GtsEntity>`
- [ ] 1.5 Define `ListQuery` struct for filtering (pattern, is_type, vendor, package, namespace)

## 2. Testing

- [ ] 2.1 Unit tests for `GtsEntity` model
- [ ] 2.2 Unit tests for `ListQuery` builder pattern
- [ ] 2.3 Unit tests for error types

## 3. Documentation

- [ ] 3.1 Add rustdoc comments to all public types
- [ ] 3.2 Add crate-level documentation with usage examples
- [ ] 3.3 Create SDK README.md
