# Implementation Tasks

## 1. Project Setup
- [x] 1.1 Create `modules/simple-user-settings/simple-user-settings-sdk` directory structure
- [x] 1.2 Create `modules/simple-user-settings/simple-user-settings` directory structure
- [x] 1.3 Add workspace members to root `Cargo.toml`
- [x] 1.4 Create `simple-user-settings-sdk/Cargo.toml` with minimal dependencies
- [x] 1.5 Create `simple-user-settings/Cargo.toml` with full dependencies

## 2. SDK Crate Implementation
- [x] 2.1 Implement `simple-user-settings-sdk/src/models.rs` (Settings, SettingsPatch)
- [x] 2.2 Implement `simple-user-settings-sdk/src/errors.rs` (SettingsError enum)
- [x] 2.3 Implement `simple-user-settings-sdk/src/api.rs` (SimpleUserSettingsApi trait with SecurityContext)
- [x] 2.4 Implement `simple-user-settings-sdk/src/lib.rs` (re-exports)

## 3. Domain Layer
- [x] 3.1 Implement `src/domain/error.rs` (DomainError enum)
- [x] 3.2 Add `From<DomainError> for SettingsError` conversion
- [x] 3.3 Implement `src/domain/repo.rs` (SettingsRepository trait with SecurityContext)
- [x] 3.4 Implement `src/domain/service.rs` (Service struct with business logic)
- [x] 3.5 Implement `src/domain/fields.rs` (domain field types)
- [x] 3.6 Write `src/domain/service_test.rs` (unit tests for service logic)

## 4. Infrastructure Layer
- [x] 4.1 Implement `src/infra/storage/entity.rs` (SeaORM entity with Scopable derive)
- [x] 4.2 Implement `src/infra/storage/mapper.rs` (Entity <-> Model conversions)
- [x] 4.3 Implement `src/infra/storage/sea_orm_repo.rs` (SettingsRepository with SecureConn)
- [x] 4.4 Create `src/infra/storage/migrations/initial_001.rs` (create settings table)
- [x] 4.5 Implement `src/infra/storage/migrations/mod.rs` (migration registration)
- [x] 4.6 Write `src/infra/storage/mapper_test.rs` (unit tests for mapper)

## 5. REST API Layer
- [x] 5.1 Implement `src/api/rest/dto.rs` (request/response DTOs with serde + utoipa)
- [x] 5.2 Implement `src/api/rest/error.rs` (`From<DomainError> for Problem`)
- [x] 5.3 Implement `src/api/rest/handlers.rs` (get_settings, update_settings, patch_settings)
- [x] 5.4 Implement `src/api/rest/routes.rs` (OperationBuilder registration)
- [x] 5.5 Write `src/api/rest/dto_test.rs` (unit tests for DTOs)

## 6. Module Infrastructure
- [x] 6.1 Implement `src/config.rs` (typed module configuration)
- [x] 6.2 Implement `src/local_client.rs` (LocalClient implementing SimpleUserSettingsApi)
- [x] 6.3 Implement `src/module.rs` (Module struct with #[modkit::module])
- [x] 6.4 Implement `src/lib.rs` (re-exports and module organization)
- [x] 6.5 Implement `src/errors.rs` (module-level error types)

## 7. Testing
- [x] 7.1 Write unit tests for domain service logic (`src/domain/service_test.rs`)
- [x] 7.2 Write mapper tests (`src/infra/storage/mapper_test.rs`)
- [x] 7.3 Write DTO tests (`src/api/rest/dto_test.rs`)
- [x] 7.4 Write E2E integration tests (`testing/e2e/modules/settings/test_settings_integration.py`)
- [x] 7.5 Test GET, POST, PATCH endpoints with full workflow scenarios
- [x] 7.6 Test idempotency and consistency across methods

## 8. Integration
- [x] 8.1 Add simple-user-settings module to `apps/hyperspot-server/Cargo.toml`
- [x] 8.2 Verify module loads via inventory mechanism
- [x] 8.3 Test REST endpoints via api_gateway (E2E tests confirm)
- [x] 8.4 Verify OpenAPI documentation generation (routes use utoipa)
- [x] 8.5 Test ClientHub integration for inter-module access (LocalClient implements API trait)

## 9. Documentation
- [x] 9.1 Add module-level rustdoc comments
- [x] 9.2 Document API trait methods in SDK
- [x] 9.3 Add usage examples in SDK crate docs
- [x] 9.4 Update module README if needed (no README currently exists)

## 10. Validation & Quality
- [x] 10.1 Run `cargo fmt --all`
- [x] 10.2 Run `cargo clippy --workspace --all-targets`
- [x] 10.3 Run `cargo test --workspace`
- [x] 10.4 Verify 90%+ test coverage
  - Added tests for `routes.rs`, `handlers.rs`, `module.rs`, `error.rs`
  - All 34 unit tests passing
- [x] 10.5 Run E2E tests if applicable
