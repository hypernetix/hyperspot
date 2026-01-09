# Implementation Tasks

## 1. Project Setup
- [ ] 1.1 Create `modules/settings/settings-sdk` directory structure
- [ ] 1.2 Create `modules/settings/settings` directory structure
- [ ] 1.3 Add workspace members to root `Cargo.toml`
- [ ] 1.4 Create `settings-sdk/Cargo.toml` with minimal dependencies
- [ ] 1.5 Create `settings/Cargo.toml` with full dependencies

## 2. SDK Crate Implementation
- [ ] 2.1 Implement `settings-sdk/src/models.rs` (Settings, SettingsPatch)
- [ ] 2.2 Implement `settings-sdk/src/errors.rs` (SettingsError enum)
- [ ] 2.3 Implement `settings-sdk/src/api.rs` (SettingsApi trait with SecurityContext)
- [ ] 2.4 Implement `settings-sdk/src/lib.rs` (re-exports)

## 3. Domain Layer
- [ ] 3.1 Implement `src/domain/error.rs` (DomainError enum)
- [ ] 3.2 Add `From<DomainError> for SettingsError` conversion
- [ ] 3.3 Implement `src/domain/repo.rs` (SettingsRepository trait with SecurityContext)
- [ ] 3.4 Implement `src/domain/service.rs` (Service struct with business logic)

## 4. Infrastructure Layer
- [ ] 4.1 Implement `src/infra/storage/entity.rs` (SeaORM entity with Scopable derive)
- [ ] 4.2 Implement `src/infra/storage/mapper.rs` (Entity <-> Model conversions)
- [ ] 4.3 Implement `src/infra/storage/sea_orm_repo.rs` (SettingsRepository with SecureConn)
- [ ] 4.4 Create `src/infra/storage/migrations/initial_001.rs` (create settings table)
- [ ] 4.5 Implement `src/infra/storage/migrations/mod.rs` (migration registration)

## 5. REST API Layer
- [ ] 5.1 Implement `src/api/rest/dto.rs` (request/response DTOs with serde + utoipa)
- [ ] 5.2 Implement `src/api/rest/error.rs` (`From<DomainError> for Problem`)
- [ ] 5.3 Implement `src/api/rest/handlers.rs` (get_settings, update_settings, patch_settings)
- [ ] 5.4 Implement `src/api/rest/routes.rs` (OperationBuilder registration)

## 6. Module Infrastructure
- [ ] 6.1 Implement `src/config.rs` (typed module configuration)
- [ ] 6.2 Implement `src/local_client.rs` (LocalClient implementing SettingsApi)
- [ ] 6.3 Implement `src/module.rs` (Module struct with #[modkit::module])
- [ ] 6.4 Implement `src/lib.rs` (re-exports and module organization)

## 7. Testing
- [ ] 7.1 Write unit tests for domain service logic
- [ ] 7.2 Write repository tests with SecureConn isolation
- [ ] 7.3 Write handler integration tests (GET, POST, PATCH)
- [ ] 7.4 Write tests for tenant and user isolation
- [ ] 7.5 Write tests for first-time user scenarios (defaults)
- [ ] 7.6 Test composite primary key uniqueness

## 8. Integration
- [ ] 8.1 Add settings module to `apps/hyperspot-server/Cargo.toml`
- [ ] 8.2 Verify module loads via inventory mechanism
- [ ] 8.3 Test REST endpoints via api_gateway
- [ ] 8.4 Verify OpenAPI documentation generation
- [ ] 8.5 Test ClientHub integration for inter-module access

## 9. Documentation
- [ ] 9.1 Add module-level rustdoc comments
- [ ] 9.2 Document API trait methods in SDK
- [ ] 9.3 Add usage examples in SDK crate docs
- [ ] 9.4 Update module README if needed

## 10. Validation & Quality
- [ ] 10.1 Run `cargo fmt --all`
- [ ] 10.2 Run `cargo clippy --workspace --all-targets`
- [ ] 10.3 Run `cargo test --workspace`
- [ ] 10.4 Verify 90%+ test coverage
- [ ] 10.5 Run E2E tests if applicable
