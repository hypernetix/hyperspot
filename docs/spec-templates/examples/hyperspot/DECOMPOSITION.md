# Decomposition: HyperSpot Server

## 1. Overview

This document decomposes the HyperSpot Server DESIGN into implementable work packages (features) aligned with the existing library structure. The decomposition strategy follows the modular architecture defined in the DESIGN, mapping each major system component to a corresponding library or module that will implement its functionality.

The decomposition prioritizes foundation libraries (modkit, modkit-db, modkit-security) that provide core capabilities required by higher-level modules, followed by API and integration features. Each feature entry maps to specific DESIGN components, sequences, and data models while maintaining traceability to PRD requirements.

Key decomposition decisions:
- **Foundation-first**: Core libraries (modkit, security, database) are prioritized to enable independent development of business modules
- **Library alignment**: Each feature corresponds to an existing library in `libs/` to leverage current project structure
- **Clear boundaries**: Features have minimal overlap with explicit dependency chains
- **Incremental delivery**: Features can be implemented and tested independently given their dependencies

## 2. Entries

**Overall implementation status:**
- [ ] `p1` - **ID**: `spd-hyperspot-examples-status-overall`

### 1. [ModKit Core Framework](features/0001-spd-hyperspot-examples-feature-modkit-core.md) - HIGH

- [ ] `p1` - **ID**: `spd-hyperspot-examples-feature-modkit-core`

- **Purpose**: Provides the foundational module lifecycle management, dependency injection, and client resolution abstractions that enable all other modules to discover each other, communicate across transports, and participate in the application lifecycle.

- **Depends On**: None

- **Scope**:
  - Module trait with lifecycle hooks (init, configure, startup, shutdown, health)
  - Compile-time module discovery via Rust inventory crate
  - Dependency resolution and initialization ordering
  - ClientHub abstraction for transport-agnostic module communication
  - Module metadata and registration system
  - Base configuration loading infrastructure

- **Out of scope**:
  - Security context enforcement (handled by modkit-security)
  - Database abstraction (handled by modkit-db)
  - Specific transport implementations beyond local function calls

- **Requirements Covered**:
  - [ ] `p1` - `spd-hyperspot-fr-module-lifecycle`
  - [ ] `p1` - `spd-hyperspot-fr-module-communication`
  - [ ] `p1` - `spd-hyperspot-fr-configuration`

- **Design Principles Covered**:
  - [ ] `p1` - `spd-hyperspot-principle-modularity`
  - [ ] `p1` - `spd-hyperspot-principle-explicitness`

- **Design Constraints Covered**:
  - [ ] `p1` - `spd-hyperspot-constraint-rust-stable`
  - [ ] `p1` - `spd-hyperspot-constraint-monorepo`

- **Domain Model Entities**:
  - Module
  - ClientHub
  - ModuleRegistry

- **Design Components**:
  - [ ] `p1` - Component: ModKit (from DESIGN §3.2)
  - [ ] `p1` - Component: ClientHub (from DESIGN §3.2)

- **API**:
  - Trait-based module API (no REST endpoints)
  - Internal API for module registration and lifecycle management

- **Sequences**:
  - [ ] `p1` - `spd-hyperspot-seq-module-init`

- **Data**:
  - [ ] `p1` - `spd-hyperspot-dbtable-module-config`

### 2. [ModKit Security Framework](features/0002-spd-hyperspot-examples-feature-modkit-security.md) - HIGH

- [ ] `p1` - **ID**: `spd-hyperspot-examples-feature-modkit-security`

- **Purpose**: Enforces multi-tenant isolation and access control through compile-time verified SecurityCtx propagation, ensuring tenant data cannot leak across boundaries and providing the foundation for all authorization decisions.

- **Depends On**: None

- **Scope**:
  - SecurityCtx type with tenant_id and user_id
  - Request-scoped context propagation across module boundaries
  - Compile-time enforcement of tenant context in database queries
  - Role-based access control (RBAC) data structures
  - Security middleware integration points
  - Audit logging hooks for security events

- **Out of scope**:
  - Authentication token validation (handled by modkit-auth)
  - Database query execution (handled by modkit-db)
  - API gateway request handling (handled by API gateway feature)

- **Requirements Covered**:
  - [ ] `p1` - `spd-hyperspot-fr-tenant-isolation`
  - [ ] `p1` - `spd-hyperspot-fr-access-control`
  - [ ] `p1` - `spd-hyperspot-nfr-tenant-security`

- **Design Principles Covered**:
  - [ ] `p1` - `spd-hyperspot-principle-security-construction`
  - [ ] `p1` - `spd-hyperspot-principle-compile-safety`

- **Design Constraints Covered**:
  - [ ] `p1` - `spd-hyperspot-constraint-no-unsafe`

- **Domain Model Entities**:
  - SecurityCtx
  - Tenant
  - User
  - Role

- **Design Components**:
  - [ ] `p1` - Component: SecurityCtx (from DESIGN §3.1)
  - [ ] `p1` - Component: Security Middleware (from DESIGN §3.2)

- **API**:
  - Type-safe API for creating and validating SecurityCtx
  - Middleware integration for extracting tenant/user from requests

- **Sequences**:
  - [ ] `p1` - `spd-hyperspot-seq-request-processing`

- **Data**:
  - [ ] `p1` - `spd-hyperspot-dbtable-tenants`
  - [ ] `p1` - `spd-hyperspot-dbtable-users`

### 3. [ModKit Database Abstraction](features/0003-spd-hyperspot-examples-feature-modkit-db.md) - HIGH

- [ ] `p1` - **ID**: `spd-hyperspot-examples-feature-modkit-db`

- **Purpose**: Provides database-agnostic persistence layer supporting SQLite, PostgreSQL, and MariaDB with automatic tenant-scoped query generation, connection pooling, and migration management to enable modules to work with any database backend.

- **Depends On**: `spd-hyperspot-examples-feature-modkit-security`

- **Scope**:
  - Database abstraction layer using sqlx
  - Automatic tenant_id injection into queries based on SecurityCtx
  - Connection pooling with configurable limits
  - Database-agnostic query builder
  - Migration runner supporting per-module migrations
  - Support for SQLite, PostgreSQL, MariaDB dialects

- **Out of scope**:
  - ORM functionality (handled by modkit-odata for OData support)
  - Caching layer
  - Read replica routing

- **Requirements Covered**:
  - [ ] `p1` - `spd-hyperspot-fr-database-agnostic`
  - [ ] `p1` - `spd-hyperspot-nfr-database-compatibility`

- **Design Principles Covered**:
  - [ ] `p1` - `spd-hyperspot-principle-security-construction`
  - [ ] `p1` - `spd-hyperspot-principle-compile-safety`

- **Design Constraints Covered**:
  - [ ] `p1` - `spd-hyperspot-constraint-db-agnostic`

- **Domain Model Entities**:
  - Database (abstraction)
  - ConnectionPool
  - Migration
  - Query

- **Design Components**:
  - [ ] `p1` - Component: Database Manager (from DESIGN §3.2)

- **API**:
  - Internal API for database operations with SecurityCtx
  - Migration API for module initialization

- **Sequences**:
  - [ ] `p1` - `spd-hyperspot-seq-request-processing`

- **Data**:
  - [ ] `p1` - All module tables require tenant_id column
  - [ ] `p1` - Index strategy: (tenant_id, id) on all tables

### 4. [ModKit Authentication](features/0004-spd-hyperspot-examples-feature-modkit-auth.md) - HIGH

- [ ] `p2` - **ID**: `spd-hyperspot-examples-feature-modkit-auth`

- **Purpose**: Handles JWT token validation, user authentication flows, and credential management to extract tenant and user identities from requests before creating SecurityCtx for authorization.

- **Depends On**: `spd-hyperspot-examples-feature-modkit-security`

- **Scope**:
  - JWT token validation with signature verification
  - Token claims extraction (tenant_id, user_id)
  - Bearer token handling in HTTP headers
  - Token expiration and refresh logic
  - Integration with SecurityCtx creation

- **Out of scope**:
  - User registration and password management (business module responsibility)
  - OAuth/OIDC provider integration (future enhancement)
  - Multi-factor authentication

- **Requirements Covered**:
  - [ ] `p2` - `spd-hyperspot-fr-access-control`

- **Design Principles Covered**:
  - [ ] `p2` - `spd-hyperspot-principle-security-construction`

- **Design Constraints Covered**:
  - [ ] `p2` - `spd-hyperspot-constraint-no-unsafe`

- **Domain Model Entities**:
  - Token
  - Claims

- **Design Components**:
  - [ ] `p2` - Component: Security Middleware (authentication portion, DESIGN §3.2)

- **API**:
  - Middleware API for token validation
  - Internal API for token creation (for testing)

- **Sequences**:
  - [ ] `p2` - `spd-hyperspot-seq-request-processing`

- **Data**:
  - None (stateless token validation)

### 5. [API Gateway and OpenAPI Generation](features/0005-spd-hyperspot-examples-feature-api-gateway.md) - HIGH

- [ ] `p1` - **ID**: `spd-hyperspot-examples-feature-api-gateway`

- **Purpose**: Automatically generates REST APIs from module definitions, produces comprehensive OpenAPI 3.0 documentation, and provides HTTP routing with middleware for CORS, rate limiting, and security enforcement.

- **Depends On**: `spd-hyperspot-examples-feature-modkit-core`, `spd-hyperspot-examples-feature-modkit-auth`

- **Scope**:
  - Axum-based HTTP router
  - Automatic route generation from module API definitions
  - OpenAPI 3.0 documentation via utoipa
  - Swagger UI and Redoc integration at /docs
  - CORS middleware
  - Health check endpoints (/health, /healthz)
  - Request/response validation
  - Error response formatting

- **Out of scope**:
  - GraphQL support (future enhancement)
  - WebSocket handling
  - Rate limiting implementation (basic framework only)

- **Requirements Covered**:
  - [ ] `p1` - `spd-hyperspot-fr-api-generation`
  - [ ] `p1` - `spd-hyperspot-nfr-response-time`

- **Design Principles Covered**:
  - [ ] `p1` - `spd-hyperspot-principle-explicitness`
  - [ ] `p1` - `spd-hyperspot-principle-compile-safety`

- **Design Constraints Covered**:
  - [ ] `p1` - `spd-hyperspot-constraint-rust-stable`

- **Domain Model Entities**:
  - Route
  - Endpoint
  - OpenAPISpec

- **Design Components**:
  - [ ] `p1` - Component: API Gateway Module (from DESIGN §3.2)
  - [ ] `p1` - Component: Router (from DESIGN §3.2)
  - [ ] `p1` - Component: OpenAPI Generator (from DESIGN §3.2)
  - [ ] `p1` - Component: Middleware (from DESIGN §3.2)

- **API**:
  - GET /health - Health check with module status
  - GET /healthz - Kubernetes liveness probe
  - GET /docs - OpenAPI UI
  - GET /api/v1/{module}/* - Auto-generated module endpoints

- **Sequences**:
  - [ ] `p1` - `spd-hyperspot-seq-request-processing`

- **Data**:
  - None (stateless routing)

### 6. [GTS Registry and Gateway-Plugin Pattern](features/0006-spd-hyperspot-examples-feature-gts-registry.md) - MEDIUM

- [ ] `p2` - **ID**: `spd-hyperspot-examples-feature-gts-registry`

- **Purpose**: Enables runtime type discovery and plugin registration to support the Gateway-Plugin pattern, allowing third-party extensions to implement gateway contracts without modifying core platform code while maintaining type safety.

- **Depends On**: `spd-hyperspot-examples-feature-modkit-core`

- **Scope**:
  - Global Type System (GTS) registry for type definitions
  - Plugin contract registration and discovery
  - Gateway module support for plugin resolution
  - Type-safe plugin interface definitions
  - Runtime plugin validation against contracts
  - Plugin instance ID management

- **Out of scope**:
  - Specific plugin implementations (business module responsibility)
  - Plugin sandboxing or isolation (future WASM support)
  - Plugin versioning and compatibility checking

- **Requirements Covered**:
  - [ ] `p2` - `spd-hyperspot-fr-gateway-plugin`

- **Design Principles Covered**:
  - [ ] `p2` - `spd-hyperspot-principle-gateway-plugin`
  - [ ] `p2` - `spd-hyperspot-principle-modularity`

- **Design Constraints Covered**:
  - [ ] `p2` - `spd-hyperspot-constraint-rust-stable`

- **Domain Model Entities**:
  - GTSRegistry
  - PluginContract
  - PluginInstance

- **Design Components**:
  - [ ] `p2` - Component: GTS Registry (from DESIGN §3.2)
  - [ ] `p2` - Component: Gateway Modules (from DESIGN §3.2)
  - [ ] `p2` - Component: Plugin Modules (from DESIGN §3.2)

- **API**:
  - Internal API for plugin registration and discovery
  - Trait-based plugin interfaces

- **Sequences**:
  - [ ] `p2` - `spd-hyperspot-seq-gateway-plugin`
  - [ ] `p2` - `spd-hyperspot-seq-module-init`

- **Data**:
  - None (in-memory registry)

### 7. [Observability Infrastructure](features/0007-spd-hyperspot-examples-feature-observability.md) - MEDIUM

- [ ] `p2` - **ID**: `spd-hyperspot-examples-feature-observability`

- **Purpose**: Provides production-grade visibility into system behavior through structured logging with correlation IDs, distributed tracing with OpenTelemetry integration, and metrics collection for performance monitoring and troubleshooting.

- **Depends On**: `spd-hyperspot-examples-feature-modkit-core`, `spd-hyperspot-examples-feature-modkit-security`

- **Scope**:
  - Structured logging via tracing crate with JSON formatting
  - Distributed tracing with OpenTelemetry spans
  - Correlation ID (trace_id) propagation across modules
  - Tenant context in all log entries
  - Configurable log levels and outputs (console, file)
  - Log rotation with configurable retention
  - Metrics collection (request count, latency, error rate)
  - Health check metrics

- **Out of scope**:
  - Metrics backend integration (Prometheus, Grafana)
  - Alerting and notification
  - Log aggregation and analysis tools

- **Requirements Covered**:
  - [ ] `p2` - `spd-hyperspot-fr-observability`
  - [ ] `p2` - `spd-hyperspot-nfr-observability-retention`

- **Design Principles Covered**:
  - [ ] `p2` - `spd-hyperspot-principle-explicitness`

- **Design Constraints Covered**:
  - [ ] `p2` - `spd-hyperspot-constraint-rust-stable`

- **Domain Model Entities**:
  - TraceContext
  - LogEntry
  - Metric
  - Span

- **Design Components**:
  - [ ] `p2` - Component: Observability System (from DESIGN §3.2)

- **API**:
  - Internal API for logging, tracing, and metrics
  - GET /health endpoint includes observability status

- **Sequences**:
  - [ ] `p2` - `spd-hyperspot-seq-request-processing`

- **Data**:
  - None (logs to files/streams, metrics to external systems)

### 8. [Testing Infrastructure](features/0008-spd-hyperspot-examples-feature-testing.md) - HIGH

- [ ] `p1` - **ID**: `spd-hyperspot-examples-feature-testing`

- **Purpose**: Establishes comprehensive testing support including unit test patterns, integration test infrastructure with testcontainers for real databases, and end-to-end test frameworks to achieve and maintain 90%+ test coverage.

- **Depends On**: `spd-hyperspot-examples-feature-modkit-core`, `spd-hyperspot-examples-feature-modkit-db`

- **Scope**:
  - Unit test patterns for pure domain logic
  - Integration test setup with testcontainers
  - Mock database support for fast tests
  - Test fixtures and data builders
  - E2E test framework using pytest
  - Code coverage reporting with cargo-tarpaulin
  - CI integration for test execution
  - Performance test infrastructure
  - Security test patterns for tenant isolation

- **Out of scope**:
  - Specific test cases for business modules (module responsibility)
  - Load testing tools and scenarios
  - UI/frontend testing

- **Requirements Covered**:
  - [ ] `p1` - `spd-hyperspot-fr-testing`
  - [ ] `p1` - `spd-hyperspot-nfr-test-coverage`

- **Design Principles Covered**:
  - [ ] `p1` - `spd-hyperspot-principle-compile-safety`

- **Design Constraints Covered**:
  - [ ] `p1` - `spd-hyperspot-constraint-rust-stable`

- **Domain Model Entities**:
  - TestContext
  - MockDatabase
  - TestFixture

- **Design Components**:
  - None (testing infrastructure)

- **API**:
  - Testing utilities and helpers
  - Mock implementations of core traits

- **Sequences**:
  - None (testing infrastructure)

- **Data**:
  - Test data fixtures

### 9. [Static Analysis and Linting](features/0009-spd-hyperspot-examples-feature-static-analysis.md) - MEDIUM

- [ ] `p2` - **ID**: `spd-hyperspot-examples-feature-static-analysis`

- **Purpose**: Enforces architectural patterns, security policies, and code quality standards at compile time through custom dylint lints and strict Rust compiler settings, catching anti-patterns before code review.

- **Depends On**: None

- **Scope**:
  - Custom dylint lints for HyperSpot-specific patterns
  - Security-focused lints (no raw queries, SecurityCtx enforcement)
  - Architecture lints (module boundaries, dependency rules)
  - Code quality lints (error handling, documentation)
  - CI integration with zero tolerance for warnings
  - RUSTFLAGS="-D warnings" enforcement
  - cargo clippy strict mode configuration

- **Out of scope**:
  - Runtime checks or dynamic analysis
  - Performance profiling
  - Dependency auditing (separate tool)

- **Requirements Covered**:
  - [ ] `p2` - `spd-hyperspot-fr-static-analysis`
  - [ ] `p2` - `spd-hyperspot-nfr-compilation-safety`

- **Design Principles Covered**:
  - [ ] `p2` - `spd-hyperspot-principle-compile-safety`
  - [ ] `p2` - `spd-hyperspot-principle-security-construction`

- **Design Constraints Covered**:
  - [ ] `p2` - `spd-hyperspot-constraint-rust-stable`
  - [ ] `p2` - `spd-hyperspot-constraint-no-unsafe`

- **Domain Model Entities**:
  - Lint
  - LintRule
  - CompilerError

- **Design Components**:
  - None (development tooling)

- **API**:
  - CLI for lint execution

- **Sequences**:
  - None (compile-time tooling)

- **Data**:
  - None

### 10. [Configuration Management](features/0010-spd-hyperspot-examples-feature-configuration.md) - MEDIUM

- [ ] `p2` - **ID**: `spd-hyperspot-examples-feature-configuration`

- **Purpose**: Provides YAML-based configuration with environment variable overrides following HYPERSPOT_ prefix convention, enabling operators to configure database connections, module settings, and logging without code changes.

- **Depends On**: `spd-hyperspot-examples-feature-modkit-core`

- **Scope**:
  - YAML configuration file loading via config-rs
  - Environment variable overrides with HYPERSPOT_ prefix
  - Typed configuration structs with serde
  - Configuration validation at startup
  - Global settings (server, database, logging)
  - Per-module configuration sections
  - Secrets handling via environment variables
  - Configuration hot-reload (optional)

- **Out of scope**:
  - Dynamic configuration updates at runtime
  - Configuration UI or API
  - Encrypted configuration storage

- **Requirements Covered**:
  - [ ] `p2` - `spd-hyperspot-fr-configuration`

- **Design Principles Covered**:
  - [ ] `p2` - `spd-hyperspot-principle-explicitness`

- **Design Constraints Covered**:
  - [ ] `p2` - `spd-hyperspot-constraint-rust-stable`

- **Domain Model Entities**:
  - Configuration
  - ConfigSection
  - EnvironmentVariable

- **Design Components**:
  - None (configuration infrastructure)

- **API**:
  - Internal API for accessing configuration values
  - CLI flags for config file path

- **Sequences**:
  - [ ] `p2` - `spd-hyperspot-seq-module-init`

- **Data**:
  - [ ] `p2` - `spd-hyperspot-dbtable-module-config`

### 11. [Transport Layer Abstractions](features/0011-spd-hyperspot-examples-feature-transports.md) - MEDIUM

- [ ] `p3` - **ID**: `spd-hyperspot-examples-feature-transports`

- **Purpose**: Implements pluggable transport mechanisms (local function calls, gRPC, HTTP/REST) behind ClientHub abstraction, enabling modules to communicate without knowing underlying protocols and supporting flexible deployment models.

- **Depends On**: `spd-hyperspot-examples-feature-modkit-core`

- **Scope**:
  - Local in-process transport for function calls
  - gRPC transport layer via tonic
  - HTTP/REST transport layer
  - Automatic transport selection based on deployment config
  - SecurityCtx propagation across transports
  - Serialization/deserialization for wire protocols
  - Connection management and retries

- **Out of scope**:
  - Custom binary protocols
  - WebSocket transport
  - Message queue integration

- **Requirements Covered**:
  - [ ] `p3` - `spd-hyperspot-fr-module-communication`
  - [ ] `p3` - `spd-hyperspot-fr-universal-deployment`
  - [ ] `p3` - `spd-hyperspot-nfr-deployment-flexibility`

- **Design Principles Covered**:
  - [ ] `p3` - `spd-hyperspot-principle-explicitness`

- **Design Constraints Covered**:
  - [ ] `p3` - `spd-hyperspot-constraint-rust-stable`

- **Domain Model Entities**:
  - Transport
  - LocalTransport
  - GrpcTransport
  - HttpTransport

- **Design Components**:
  - [ ] `p3` - Component: ClientHub (transport portion, from DESIGN §3.2)

- **API**:
  - Internal transport APIs
  - gRPC service definitions

- **Sequences**:
  - [ ] `p3` - `spd-hyperspot-seq-request-processing`

- **Data**:
  - None

### 12. [Build and Deployment Pipeline](features/0012-spd-hyperspot-examples-feature-build-deploy.md) - MEDIUM

- [ ] `p2` - **ID**: `spd-hyperspot-examples-feature-build-deploy`

- **Purpose**: Establishes CI/CD pipeline for automated testing, building, and releasing, supporting universal deployment to cloud containers, on-premises binaries, and mobile-integrated services from a single codebase.

- **Depends On**: `spd-hyperspot-examples-feature-testing`, `spd-hyperspot-examples-feature-static-analysis`

- **Scope**:
  - Makefile targets for check, test, build, release
  - CI pipeline configuration (GitHub Actions or equivalent)
  - Docker multi-stage builds
  - Cross-compilation for Windows, Linux, macOS
  - Binary release packaging
  - Container image publishing
  - Dependency caching (sccache)
  - Incremental compilation optimization

- **Out of scope**:
  - Kubernetes manifests and cluster configuration
  - Cloud provider-specific IaC (Terraform, CloudFormation)
  - Release automation and versioning strategy

- **Requirements Covered**:
  - [ ] `p2` - `spd-hyperspot-fr-universal-deployment`
  - [ ] `p2` - `spd-hyperspot-nfr-build-performance`
  - [ ] `p2` - `spd-hyperspot-nfr-deployment-flexibility`

- **Design Principles Covered**:
  - [ ] `p2` - `spd-hyperspot-principle-explicitness`

- **Design Constraints Covered**:
  - [ ] `p2` - `spd-hyperspot-constraint-rust-stable`
  - [ ] `p2` - `spd-hyperspot-constraint-monorepo`

- **Domain Model Entities**:
  - BuildArtifact
  - DeploymentTarget

- **Design Components**:
  - None (build infrastructure)

- **API**:
  - None (CLI and scripts)

- **Sequences**:
  - None (build-time tooling)

- **Data**:
  - None

### 13. [ModKit Error Handling](features/0013-spd-hyperspot-examples-feature-modkit-errors.md) - MEDIUM

- [ ] `p2` - **ID**: `spd-hyperspot-examples-feature-modkit-errors`

- **Purpose**: Provides standardized error types and error handling patterns with LLM-friendly error messages, enabling consistent error propagation across module boundaries and helpful debugging information.

- **Depends On**: None

- **Scope**:
  - Core error types and error trait implementations
  - Error conversion utilities
  - Error context and backtrace support
  - LLM-friendly error message formatting
  - Error categorization (client error, server error, not found, etc.)
  - Error serialization for API responses
  - Error macros for ergonomic error creation

- **Out of scope**:
  - Error reporting to external services
  - Crash reporting and analysis
  - Error recovery strategies (module responsibility)

- **Requirements Covered**:
  - None explicitly (supports all features)

- **Design Principles Covered**:
  - [ ] `p2` - `spd-hyperspot-principle-compile-safety`
  - [ ] `p2` - `spd-hyperspot-principle-explicitness`

- **Design Constraints Covered**:
  - [ ] `p2` - `spd-hyperspot-constraint-rust-stable`

- **Domain Model Entities**:
  - Error
  - ErrorContext
  - ErrorCode

- **Design Components**:
  - None (error infrastructure used by all components)

- **API**:
  - Error trait implementations
  - Error conversion functions
  - Error response formatting

- **Sequences**:
  - None (error handling is cross-cutting)

- **Data**:
  - None

### 14. [ModKit OData Support](features/0014-spd-hyperspot-examples-feature-modkit-odata.md) - LOW

- [ ] `p3` - **ID**: `spd-hyperspot-examples-feature-modkit-odata`

- **Purpose**: Enables automatic OData endpoint generation for module entities, providing powerful query capabilities (filtering, ordering, pagination) with minimal module code.

- **Depends On**: `spd-hyperspot-examples-feature-modkit-db`, `spd-hyperspot-examples-feature-api-gateway`

- **Scope**:
  - OData v4 protocol implementation
  - Query string parsing for $filter, $orderby, $top, $skip
  - Entity set exposure via OData endpoints
  - Metadata document generation ($metadata)
  - Integration with modkit-db for query translation
  - Tenant-scoped OData queries
  - OData macros for entity annotation

- **Out of scope**:
  - OData batch operations
  - OData actions and functions
  - Complex type support beyond basic entities

- **Requirements Covered**:
  - [ ] `p3` - `spd-hyperspot-fr-api-generation`

- **Design Principles Covered**:
  - [ ] `p3` - `spd-hyperspot-principle-modularity`

- **Design Constraints Covered**:
  - [ ] `p3` - `spd-hyperspot-constraint-db-agnostic`

- **Domain Model Entities**:
  - ODataQuery
  - EntitySet
  - Metadata

- **Design Components**:
  - [ ] `p3` - Component: API Gateway Module (OData portion, from DESIGN §3.2)

- **API**:
  - GET /odata/{entity}?$filter=...&$orderby=...&$top=N&$skip=N
  - GET /odata/$metadata

- **Sequences**:
  - [ ] `p3` - `spd-hyperspot-seq-request-processing`

- **Data**:
  - None (queries existing module tables)

### 15. [ModKit Utilities](features/0015-spd-hyperspot-examples-feature-modkit-utils.md) - LOW

- [ ] `p3` - **ID**: `spd-hyperspot-examples-feature-modkit-utils`

- **Purpose**: Provides common utility functions and types used across multiple modules, reducing code duplication and establishing consistent patterns for common operations.

- **Depends On**: None

- **Scope**:
  - Date/time utilities
  - String manipulation helpers
  - Validation utilities
  - ID generation (UUIDs, etc.)
  - Serialization helpers
  - Common trait implementations
  - Type conversion utilities

- **Out of scope**:
  - Business logic utilities (module responsibility)
  - Heavy computational libraries
  - External service integrations

- **Requirements Covered**:
  - None explicitly (supports all features)

- **Design Principles Covered**:
  - [ ] `p3` - `spd-hyperspot-principle-explicitness`

- **Design Constraints Covered**:
  - [ ] `p3` - `spd-hyperspot-constraint-rust-stable`

- **Domain Model Entities**:
  - None (utilities)

- **Design Components**:
  - None (utilities)

- **API**:
  - Utility functions and helper traits

- **Sequences**:
  - None

- **Data**:
  - None
