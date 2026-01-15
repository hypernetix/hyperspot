# Analytics - Architecture Decision Records

**Module**: Analytics  
**Version**: 1.0  
**Last Updated**: 2025-12-31

This document tracks all significant architectural decisions for the Analytics module.

---

## ADR-0001: Initial Analytics Architecture

**ID**: `fdd-analytics-adr-initial-architecture`  
**Date**: 2025-12-31  
**Status**: Accepted  
**Deciders**: Hyperspot Team  
**Technical Story**: Analytics module initialization

### Context and Problem Statement

The Hyperspot platform required a comprehensive analytics and reporting solution that could:
- Support multiple data sources without vendor lock-in
- Provide type-safe data visualization and querying
- Ensure complete tenant isolation in multi-tenant environment
- Scale horizontally while maintaining performance
- Enable extensibility through plugins

The challenge was to design an architecture that balances flexibility, security, type safety, and performance while avoiding coupling to specific data warehouse technologies.

### Decision Drivers

- **Type Safety**: Need compile-time and runtime type validation across distributed system
- **Multi-Tenancy**: Complete tenant isolation is mandatory for SaaS platform
- **Data Agnostic**: No vendor lock-in to specific DWH or data source technologies
- **Extensibility**: Plugin architecture for datasources, queries, and visualizations
- **Performance**: Sub-second query response times for typical dashboards
- **Security**: JWT-based authentication with automatic tenant context propagation
- **Modularity**: Reusable components (dashboards, widgets, templates)

### Considered Options

1. **GTS + Plugin Architecture** (chosen)
   - GTS (Global Type System) for type-safe cross-module communication
   - Plugin-based datasource architecture with dynamic registration
   - No built-in DWH, all data via external query plugins
   - SecurityCtx enforcement at every layer
   - Modkit pattern with SDK separation

2. **Monolithic with Built-in DWH**
   - Integrated PostgreSQL-based data warehouse
   - Direct SQL queries from analytics service
   - Simpler deployment but vendor lock-in
   - ETL pipelines built into analytics module

3. **Microservices with Dedicated Query Service**
   - Separate query execution service
   - REST API between analytics and query service
   - More network overhead, complex deployment
   - Harder to maintain type safety across services

### Decision Outcome

**Chosen option**: "GTS + Plugin Architecture"

**Rationale**:
- **Type Safety**: GTS provides schema validation at runtime with JSON Schema compliance
- **Flexibility**: Plugin architecture allows adding new datasources without service restart
- **Data Agnostic**: Query plugins abstract data source details, supporting any external system
- **Security**: SecurityCtx enforced at compilation via Secure ORM and at runtime via JWT
- **Performance**: Stateless design enables horizontal scaling, caching reduces query latency
- **Modularity**: SDK pattern separates contracts from implementation

**Positive Consequences**:
- No vendor lock-in - can connect to any data source (OLAP, OLTP, REST APIs)
- Type-safe communication through GTS eliminates entire class of runtime errors
- Plugin registration without service restart improves operational flexibility
- Complete tenant isolation via SecurityCtx prevents data leakage
- Horizontal scalability achieved through stateless service design
- Reusable components (templates, widgets, layouts) reduce development time

**Negative Consequences**:
- More complex architecture than monolithic approach
- Plugin development requires understanding of GTS system
- Query performance depends on external data source capabilities
- Initial setup requires more configuration than integrated solution
- Debugging distributed queries can be challenging

### Related Design Elements

**Actors**:
- `fdd-analytics-actor-plugin-developer` - Develops custom datasource plugins
- `fdd-analytics-actor-data-engineer` - Manages external data infrastructure
- `fdd-analytics-actor-query-plugin` - Executes queries against datasources

**Capabilities**:
- `fdd-analytics-capability-data-access` - Plugin-based datasource architecture
- `fdd-analytics-capability-extensibility` - Dynamic datasource registration
- `fdd-analytics-capability-query-execution` - Plugin-based query adapters

**Requirements**:
- `fdd-analytics-req-performance` - Query execution performance requirements
- `fdd-analytics-req-security` - Multi-tenant isolation and SecurityCtx
- `fdd-analytics-req-tech-constraints` - GTS for type definitions, no built-in DWH

**Principles**:
- `fdd-analytics-principle-plugin-extensibility` - No service restart required
- `fdd-analytics-principle-gts-native` - All plugin communication via GTS
- `fdd-analytics-principle-data-agnostic` - No built-in data sources or DWH
- `fdd-analytics-principle-security-first` - SecurityCtx enforced at every level

---

## ADR-0002: OData v4 Query Protocol Selection

**ID**: `fdd-analytics-adr-odata-protocol`  
**Date**: 2025-12-31  
**Status**: Accepted  
**Deciders**: Hyperspot Team

### Context and Problem Statement

Analytics module needed a standardized query protocol for data retrieval that supports:
- Complex filtering, sorting, and pagination
- Field projection and selection
- Metadata discovery
- Industry-standard compatibility

### Decision Drivers

- Need standardized query language that external systems can understand
- Must support complex filtering and sorting operations
- Require metadata endpoint for schema discovery
- Industry adoption and tooling ecosystem

### Considered Options

1. **OData v4** (chosen)
2. **GraphQL**
3. **Custom REST query DSL**

### Decision Outcome

**Chosen option**: "OData v4"

**Rationale**: OData v4 provides mature standard for querying data with built-in support for filtering (`$filter`), sorting (`$orderby`), pagination (`$top`, `$skip`), field selection (`$select`), and metadata discovery (`$metadata`). Wide industry adoption ensures compatibility with external tools and services.

**Positive Consequences**:
- Standard protocol understood by many external systems
- Built-in metadata discovery via `$metadata` endpoint
- Rich query capabilities (filter, sort, project, paginate)
- JSON CSDL support for schema definition

**Negative Consequences**:
- OData query syntax can be complex for users
- Additional implementation overhead vs custom DSL
- Some OData features not needed by all use cases

### Related Design Elements

**Actors**:
- `fdd-analytics-actor-api-consumer` - Integrates analytics programmatically and relies on predictable query semantics
- `fdd-analytics-actor-query-plugin` - Executes queries against datasources using OData-compatible query options
- `fdd-analytics-actor-ui-app` - Issues OData queries for dashboards and data exploration

**Capabilities**:
- `fdd-analytics-capability-query-execution` - Multi-datasource queries with OData support
- `fdd-analytics-capability-data-access` - OData v4 query support

**Requirements**:
- `fdd-analytics-req-tech-constraints` - Technology choices including query protocols

**Principles**:
- `fdd-analytics-principle-api-first` - REST API with OpenAPI specification
- `fdd-analytics-principle-gts-native` - Query and metadata semantics expressed via GTS types

---

## ADR-0003: SecurityCtx and Secure ORM for Tenant Isolation

**ID**: `fdd-analytics-adr-security-ctx-secure-orm`  
**Date**: 2025-12-31  
**Status**: Accepted  
**Deciders**: Hyperspot Team

### Context and Problem Statement

Multi-tenant SaaS platform requires absolute guarantee of tenant data isolation. Traditional approach of manual tenant_id filtering in queries is error-prone and can lead to data leakage vulnerabilities.

### Decision Drivers

- Zero-tolerance for tenant data leakage
- Compile-time enforcement of security constraints
- Automatic tenant context propagation
- Audit trail for all data access

### Considered Options

1. **Secure ORM with SecurityCtx** (chosen)
2. **Manual tenant_id filtering**
3. **Database-level row-level security (RLS)**

### Decision Outcome

**Chosen option**: "Secure ORM with SecurityCtx"

**Rationale**: Secure ORM with `#[derive(Scopable)]` macro provides compile-time enforcement of tenant isolation. All database queries must go through `SecureConn` with `SecurityCtx`, making it impossible to execute unscoped queries. This eliminates entire class of security vulnerabilities.

**Positive Consequences**:
- Compile-time enforcement - unscoped queries cannot compile
- Automatic tenant_id injection in all queries
- SecurityCtx propagation through all service layers
- JWT validation and tenant extraction centralized
- Impossible to accidentally query across tenant boundaries

**Negative Consequences**:
- Additional boilerplate - all methods accept `&SecurityCtx`
- Learning curve for developers unfamiliar with pattern
- Cannot opt-out of scoping even when intentional (e.g., admin queries)

### Related Design Elements

**Capabilities**:
- `fdd-analytics-capability-security` - Complete tenant isolation, SecurityCtx enforced everywhere

**Requirements**:
- `fdd-analytics-req-security` - Multi-tenant isolation requirements
- `fdd-analytics-req-secure-orm` - Secure ORM with SecurityCtx (REQUIRED)
- `fdd-analytics-req-security-ctx` - SecurityCtx propagation requirements

**Principles**:
- `fdd-analytics-principle-security-first` - SecurityCtx enforced at every level
- `fdd-analytics-principle-tenant-isolation` - Complete data separation per tenant

---

## ADR-0004: Mock Mode Architecture

**ID**: `fdd-analytics-adr-mock-mode`  
**Date**: 2025-12-31  
**Status**: Accepted  
**Deciders**: Hyperspot Team

### Context and Problem Statement

Development and testing require ability to run analytics module without dependencies on database, external data sources, or query plugins. Need consistent mock data that matches production schemas.

### Decision Drivers

- Faster local development without infrastructure setup
- Reliable E2E testing with predictable data
- Demo environments without production data access
- Offline development capability

### Considered Options

1. **Comprehensive Mock Mode** (chosen)
2. **Test database with seed data**
3. **In-memory database only**

### Decision Outcome

**Chosen option**: "Comprehensive Mock Mode"

**Rationale**: Mock mode at both service and UI levels provides maximum flexibility for development and testing. Mock datasources return realistic data matching GTS schemas, enabling full feature development without infrastructure.

**Positive Consequences**:
- Service mock mode via `--mock-mode` flag or `MOCK_MODE=true` env var
- UI mock mode via `VITE_MOCK_MODE=true` build-time config
- Mock responses follow same GTS contracts as real implementations
- Faster development cycles without database/plugin dependencies
- Reliable E2E tests with deterministic mock data

**Negative Consequences**:
- Additional maintenance - mock implementations must stay in sync with real implementations
- Mock data may not catch all edge cases that occur in production
- Risk of "works in mock but fails in production" scenarios

### Related Design Elements

**Principles**:
- `fdd-analytics-principle-mock-mode` - All services and UI components support mock mode
