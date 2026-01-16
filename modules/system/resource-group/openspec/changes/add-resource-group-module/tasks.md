# Tasks: Resource Group Module

## 1. SDK Crate Setup

- [ ] 1.1 Create `resource-group-sdk/Cargo.toml` with minimal dependencies
- [ ] 1.2 Create SDK structure: `api.rs`, `models.rs`, `errors.rs`, `lib.rs`
- [ ] 1.3 Define `ResourceGroupApi` trait with all methods taking `&SecurityCtx`
- [ ] 1.4 Define transport-agnostic models (NO serde): `ResourceGroupType`, `ResourceGroupEntity`, `ResourceGroupReference`
- [ ] 1.5 Define `ResourceGroupError` enum with convenience constructors

## 2. Module Crate Setup

- [ ] 2.1 Create `resource-group/Cargo.toml` with SDK and ModKit dependencies
- [ ] 2.2 Create module structure following DDD-light pattern
- [ ] 2.3 Implement `#[modkit::module]` declaration with `capabilities = [db, rest]`
- [ ] 2.4 Implement `Module::init` with config loading and ClientHub registration
- [ ] 2.5 Create typed config struct with defaults

## 3. Domain Layer

- [ ] 3.1 Define domain error types (`DomainError` enum)
- [ ] 3.2 Define repository traits for types, entities, references, and closure table
- [ ] 3.3 Implement domain service with business logic:
  - [ ] 3.3.1 Type management (create, list, get, update, delete)
  - [ ] 3.3.2 Entity CRUD operations
  - [ ] 3.3.3 Hierarchy operations (ancestors, descendants, move subtree)
  - [ ] 3.3.4 Reference management (create, delete, reference counting)
  - [ ] 3.3.5 Cycle detection for hierarchy operations
- [ ] 3.4 Define domain events (optional, for SSE)

## 4. Infrastructure Layer (Storage)

- [ ] 4.1 Create SeaORM entities:
  - [ ] 4.1.1 `ResourceGroupType` entity with `#[derive(Scopable)]`
  - [ ] 4.1.2 `ResourceGroupEntity` entity with `#[derive(Scopable)]`
  - [ ] 4.1.3 `ResourceGroupClosure` entity for closure table
  - [ ] 4.1.4 `ResourceGroupReference` entity with `#[derive(Scopable)]`
- [ ] 4.2 Create SeaORM migrations:
  - [ ] 4.2.1 Initial migration for all tables
  - [ ] 4.2.2 Indexes for performance (closure table, references)
- [ ] 4.3 Implement repository traits:
  - [ ] 4.3.1 `ResourceGroupTypesRepository` with SecureConn
  - [ ] 4.3.2 `ResourceGroupEntitiesRepository` with SecureConn
  - [ ] 4.3.3 `ResourceGroupClosureRepository` for hierarchy operations
  - [ ] 4.3.4 `ResourceGroupReferencesRepository` with SecureConn
- [ ] 4.4 Implement closure table operations:
  - [ ] 4.4.1 Create closure entries on entity creation
  - [ ] 4.4.2 Rebuild closure entries on subtree move
  - [ ] 4.4.3 Delete closure entries on entity deletion
  - [ ] 4.4.4 Query ancestors and descendants efficiently

## 5. Local Client

- [ ] 5.1 Implement `ResourceGroupApi` trait for local client
- [ ] 5.2 Wire local client to ClientHub in `init()`
- [ ] 5.3 Convert `DomainError` to `ResourceGroupError` via `From` impl

## 6. REST API Layer

- [ ] 6.1 Create REST DTOs (request/response models with serde + ToSchema):
  - [ ] 6.1.1 Type DTOs (CreateTypeReq, TypeDto, UpdateTypeReq)
  - [ ] 6.1.2 Entity DTOs (CreateEntityReq, EntityDto, UpdateEntityReq)
  - [ ] 6.1.3 Reference DTOs (CreateReferenceReq, ReferenceDto)
  - [ ] 6.1.4 Hierarchy DTOs (AncestorsDto, DescendantsDto)
- [ ] 6.2 Implement REST handlers:
  - [ ] 6.2.1 Type handlers (create, list, get, update, delete)
  - [ ] 6.2.2 Entity handlers (create, get, update, delete)
  - [ ] 6.2.3 Hierarchy handlers (ancestors, descendants)
  - [ ] 6.2.4 Reference handlers (create, delete)
- [ ] 6.3 Define REST routes and register with `api_gateway`:
  - [ ] 6.3.1 Type routes (`/resource-group/v1/types`)
  - [ ] 6.3.2 Entity routes (`/resource-group/v1/groups`)
  - [ ] 6.3.3 Hierarchy routes (`/resource-group/v1/groups/{id}/ancestors`, `/descendants`)
  - [ ] 6.3.4 Reference routes (`/resource-group/v1/groups/{id}/references`)
- [ ] 6.4 Implement error mapping (`impl From<DomainError> for Problem`)
- [ ] 6.5 Add OpenAPI documentation for all endpoints
- [ ] 6.6 Add OData filtering support for list endpoints (if needed)

## 7. Authorization & Security

- [ ] 7.1 Implement application-based authorization checks
- [ ] 7.2 Validate type permissions (owner vs. allowed applications)
- [ ] 7.3 Ensure all repository methods use SecureConn with SecurityCtx
- [ ] 7.4 Add tenant isolation via Secure ORM
- [ ] 7.5 Validate input data (code format, name length, etc.)

## 8. Testing (Target: 90%+ coverage)

- [ ] 8.1 Unit tests for domain service:
  - [ ] 8.1.1 Type management tests
  - [ ] 8.1.2 Entity CRUD tests
  - [ ] 8.1.3 Hierarchy operation tests (move, cycle detection)
  - [ ] 8.1.4 Reference management tests
- [ ] 8.2 Unit tests for repositories:
  - [ ] 8.2.1 Closure table operations
  - [ ] 8.2.2 SecureConn integration
- [ ] 8.3 Integration tests:
  - [ ] 8.3.1 REST API endpoint tests
  - [ ] 8.3.2 Multi-tenant isolation tests
  - [ ] 8.3.3 Hierarchy operation integration tests
- [ ] 8.4 Verify code coverage (90%+ target)

## 9. Documentation

- [ ] 9.1 Add rustdoc comments to public API
- [ ] 9.2 Update module README with usage examples
- [ ] 9.3 Document closure table pattern usage

## 10. Migration from Old Implementation

- [ ] 10.1 Review old `resource-group/spec/` documentation
- [ ] 10.2 Extract business logic requirements
- [ ] 10.3 Plan data migration (if needed)
- [ ] 10.4 Remove old symlink after new module is complete
