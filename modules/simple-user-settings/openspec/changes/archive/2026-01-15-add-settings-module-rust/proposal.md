# Change: Add User Settings Module in Rust

## Why

The settings module needs to be migrated from Go to Rust to align with HyperSpot's modular architecture. The existing Go implementation provides user-scoped settings (theme, language) with tenant isolation, but needs to be rewritten using ModKit patterns, Secure ORM, and the SDK pattern for proper integration with the HyperSpot ecosystem.

## What Changes

- Create `simple-user-settings-sdk` crate with public API trait, models, and errors (transport-agnostic)
- Create `simple-user-settings` module crate with full implementation following DDD-light structure
- Implement REST API with three endpoints:
  - `GET /settings/v1/settings` - Retrieve user settings
  - `POST /settings/v1/settings` - Full update of settings
  - `PATCH /settings/v1/settings` - Partial update of settings
- Use `SecurityContext` for automatic tenant and user isolation (no explicit tenant_id/user_id in requests)
- Implement Secure ORM patterns with SeaORM for data access
- Support database migrations with SeaORM migration system
- Follow ModKit conventions for module registration and lifecycle
- Add comprehensive unit and integration tests

## Impact

- Affected specs: `user-settings` (new capability)
- Affected code: 
  - New: `modules/settings/simple-user-settings-sdk/` (SDK crate)
  - New: `modules/settings/simple-user-settings/` (implementation crate)
  - Modified: Root `Cargo.toml` (workspace members - added both crates)
  - Modified: `apps/hyperspot-server/Cargo.toml` (module dependency - added simple-user-settings)
  - New: `testing/e2e/modules/settings/test_settings_integration.py` (E2E tests)
- Migration: Go settings code will be deprecated after Rust implementation is complete and validated
