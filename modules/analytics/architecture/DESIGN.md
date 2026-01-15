# Analytics - Technical Design

**Version**: 1.0  
**Date**: 2025-12-31  
**Module**: Analytics

**Business Context**: [BUSINESS.md](BUSINESS.md)

**Architecture Decisions**: [ADR.md](ADR.md)

---

## A. Architecture Overview

### Architectural Vision

The Analytics module follows a **plugin-based, data-agnostic architecture** built on the **Hyperspot Platform's modkit pattern**. The core philosophy is:

1. **Zero Vendor Lock-in**: No built-in data warehouse or ETL. All data access via dynamically registered query plugins.
2. **Type Safety First**: GTS (Global Type System) ensures runtime type validation across all components.
3. **Security by Design**: SecurityCtx enforced at compile-time via Secure ORM, tenant isolation guaranteed.
4. **Horizontal Scalability**: Stateless service design enables unlimited scaling, with query result caching for performance.

The architecture separates **contract** (SDK) from **implementation** (service), enabling independent evolution and testing. All datasources are plugins that implement standardized query interfaces, supporting OData v4, native queries, and REST APIs.

### Architecture layers

![Architecture Diagram](diagrams/architecture.drawio.svg)

The architecture consists of four distinct layers:

**PRESENTATION** (HAI3 - UI Application):
- Dashboards, Reports, Widgets
- Datasources & Templates
- REST API consumption with JWT authentication

**APPLICATION** (Analytics Service):
- Plugin Gateway - Dynamic datasource registration
- Query Execution Engine - Multi-datasource queries with caching
- Dashboard Management - CRUD operations for dashboards/widgets
- Report Generation - Scheduled reports via platform
- SecurityCtx propagation throughout all operations

**DOMAIN** (Analytics SDK - Contract Layer):
- GTS Type Definitions (26 schema files)
- Contract Traits (Query API, Plugin API)
- Business Logic (isolated from infrastructure)
- No HTTP, no database, no serialization

**INFRASTRUCTURE**:
- **Database**: PostgreSQL via Secure ORM (metadata only: GTS types, instances, configuration)
- **External Data Sources**: Query plugins access external DWH/OLAP/APIs with JWT propagation
- **Platform Services**: Event management, tenancy, authentication, scheduling, email
- **Observability**: OpenTelemetry tracing, structured logging, Prometheus metrics

**Key Architectural Patterns**:
- **Plugin Architecture**: Dynamic registration without service restart
- **SDK Pattern**: Contract/implementation separation via traits
- **Secure ORM**: Compile-time tenant isolation enforcement
- **GTS Native**: All plugin communication via GTS for type safety

---

## B. Requirements & Principles

### 1. System Requirements & Constraints

**Performance Requirements**:  
**ID**: `fdd-analytics-req-performance`  
**Capabilities**: `fdd-analytics-capability-performance`, `fdd-analytics-capability-query-execution`  
**Actors**: `fdd-analytics-actor-platform-admin`, `fdd-analytics-actor-query-plugin`
- Query execution: p95 < 1s, p99 < 3s (depends on external data sources)
- Dashboard load: < 2s for typical dashboard
- API response: p95 < 200ms
- Concurrent users: 100+ per tenant
- Plugin registration: < 5s

**Scalability**:  
**ID**: `fdd-analytics-req-scalability`  
**Capabilities**: `fdd-analytics-capability-performance`, `fdd-analytics-capability-dashboard-mgmt`  
**Actors**: `fdd-analytics-actor-platform-admin`, `fdd-analytics-actor-dashboard-designer`
- 1000+ dashboards per tenant
- 100+ widgets per dashboard
- 10M+ rows per query result (limited by external sources)
- 50+ concurrent queries per tenant
- Unlimited datasource plugins

**Security Requirements**:  
**ID**: `fdd-analytics-req-security`  
**Capabilities**: `fdd-analytics-capability-security`  
**Actors**: `fdd-analytics-actor-platform-admin`, `fdd-analytics-actor-tenant-admin`, `fdd-analytics-actor-platform`  
**ADRs**: `fdd-analytics-adr-security-ctx-secure-orm`
- Multi-tenant isolation (mandatory, **provided by Hyperspot Platform**)
- JWT signature validation
- Automatic tenant_id injection
- SecurityCtx checks at all layers
- Audit logging for all queries (**platform-level**)
- Row-level security in data access (enforced by external sources)

**Compliance**:  
**ID**: `fdd-analytics-req-compliance`  
**Capabilities**: `fdd-analytics-capability-security`  
**Actors**: `fdd-analytics-actor-platform-admin`, `fdd-analytics-actor-tenant-admin`
- GDPR compliant (data retention, deletion) - **managed by platform**
- SOC 2 Type II requirements
- Audit trail for all data access (**platform-level**)
- Data encryption at rest and in transit

**Technology Constraints**:  
**ID**: `fdd-analytics-req-tech-constraints`  
**Capabilities**: `fdd-analytics-capability-data-access`, `fdd-analytics-capability-extensibility`  
**Actors**: `fdd-analytics-actor-plugin-developer`, `fdd-analytics-actor-query-plugin`  
**ADRs**: `fdd-analytics-adr-initial-architecture`, `fdd-analytics-adr-odata-protocol`
- Rust for core services
- **PostgreSQL for OLTP** (GTS metadata, types, instances, configuration)
- **No built-in DWH** - data agnostic, all sources via query registration
- GTS for all type definitions
- JWT for authentication (**provided by Hyperspot Platform**)

**Platform Dependencies**:  
**ID**: `fdd-analytics-req-platform-deps`  
**Capabilities**: `fdd-analytics-capability-security`, `fdd-analytics-capability-reporting`  
**Actors**: `fdd-analytics-actor-platform`
- **Hyperspot Platform** provides:
  - Event management system
  - Tenancy management and isolation
  - User authentication and authorization
  - Access control framework
  - UI configuration and settings management
  - **Scheduling service** - For report scheduling and periodic tasks
  - **Email delivery** - For report delivery and notifications

---

### 1a. Security Requirements

**Secure ORM (REQUIRED)**:  
**ID**: `fdd-analytics-req-secure-orm`  
**Capabilities**: `fdd-analytics-capability-security`  
**Actors**: `fdd-analytics-actor-platform-admin`  
**ADRs**: `fdd-analytics-adr-security-ctx-secure-orm`
- All database queries MUST use `SecureConn` with `SecurityCtx`
- Entities MUST derive `#[derive(Scopable)]` with explicit scope dimensions
- Compile-time enforcement: unscoped queries cannot execute
- Tenant isolation automatic when tenant_ids provided

**SecurityCtx Propagation**:  
**ID**: `fdd-analytics-req-security-ctx`  
**Capabilities**: `fdd-analytics-capability-security`  
**Actors**: `fdd-analytics-actor-platform-admin`  
**ADRs**: `fdd-analytics-adr-security-ctx-secure-orm`
- All service methods accept `&SecurityCtx` as first parameter
- All repository methods accept `&SecurityCtx` for scope enforcement
- SecurityCtx created from request auth (per-operation, not stored)

**Input Validation**:  
**ID**: `fdd-analytics-req-input-validation`  
**Capabilities**: `fdd-analytics-capability-security`  
**Actors**: `fdd-analytics-actor-api-consumer`, `fdd-analytics-actor-ui-app`
- Use `validator` crate for DTO validation
- Field-level constraints (length, email, custom validators)
- Return 422 with structured validation errors

**Secrets Management**:  
**ID**: `fdd-analytics-req-secrets-mgmt`  
**Capabilities**: `fdd-analytics-capability-security`  
**Actors**: `fdd-analytics-actor-platform-admin`
- Never commit secrets to version control
- Use platform secret storage
- Rotate secrets regularly
- Use secure random generation for tokens

**References**: [SECURITY](../../../guidelines/SECURITY.md), [SECURE-ORM](../../../docs/SECURE-ORM.md)

---

### 1b. Observability Requirements

**Distributed Tracing (OpenTelemetry)**:  
**ID**: `fdd-analytics-req-tracing`  
**Capabilities**: `fdd-analytics-capability-performance`  
**Actors**: `fdd-analytics-actor-platform-admin`
- Accept/propagate `traceparent` header (W3C Trace Context)
- Emit `traceId` header on all responses
- Auto-instrument: HTTP requests, DB queries, inter-module calls
- Export to Jaeger/Uptrace via OTLP

**Structured Logging**:  
**ID**: `fdd-analytics-req-logging`  
**Capabilities**: `fdd-analytics-capability-performance`  
**Actors**: `fdd-analytics-actor-platform-admin`
- JSON logs per request: `traceId`, `requestId`, `userId`, `path`, `status`, `durationMs`
- Use `tracing` crate with contextual fields
- Log levels configurable per-module

**Metrics (Prometheus)**:  
**ID**: `fdd-analytics-req-metrics`  
**Capabilities**: `fdd-analytics-capability-performance`  
**Actors**: `fdd-analytics-actor-platform-admin`
- Health check endpoint: `/health`
- RED metrics: Rate, Errors, Duration (per route)
- USE metrics: Utilization, Saturation, Errors
- Performance: p50/p90/p99 latencies
- Resource: memory, connection pools, queue depths

**Health Checks**:  
**ID**: `fdd-analytics-req-health-checks`  
**Capabilities**: `fdd-analytics-capability-performance`  
**Actors**: `fdd-analytics-actor-platform-admin`
- Liveness probe: service is running
- Readiness probe: service can handle traffic
- Kubernetes-compatible health endpoints

**References**: [TRACING_SETUP](../../../docs/TRACING_SETUP.md), [ARCHITECTURE_MANIFEST](../../../docs/ARCHITECTURE_MANIFEST.md)

---

### 1c. Functional Requirements

**Data Visualization**:  
**ID**: `fdd-analytics-req-data-visualization`  
**Capabilities**: `fdd-analytics-capability-data-visualization`  
**Actors**: `fdd-analytics-actor-dashboard-designer`, `fdd-analytics-actor-business-analyst`, `fdd-analytics-actor-end-user`, `fdd-analytics-actor-template-developer`
- Support rich chart types (line, bar, pie, scatter, heatmap, maps)
- Interactive tables with sorting and filtering
- Custom widget templates via JavaScript bundles
- Values selectors (dropdowns, autocomplete, pickers) for filters and parameters

**Datasource Management**:  
**ID**: `fdd-analytics-req-datasource-mgmt`  
**Capabilities**: `fdd-analytics-capability-datasource-mgmt`  
**Actors**: `fdd-analytics-actor-plugin-developer`, `fdd-analytics-actor-dashboard-designer`, `fdd-analytics-actor-tenant-admin`
- Datasource configuration (query + parameters + UI controls)
- Parameter binding and validation with GTS type checking
- Values selector integration for parameter inputs
- Datasource reusability across widgets and dashboards
- Runtime parameter injection with security context

**Export & Sharing**:  
**ID**: `fdd-analytics-req-export-sharing`  
**Capabilities**: `fdd-analytics-capability-export-sharing`  
**Actors**: `fdd-analytics-actor-dashboard-designer`, `fdd-analytics-actor-business-analyst`, `fdd-analytics-actor-system-integrator`
- Dashboard export to multiple formats (PDF, PNG, CSV, Excel)
- Dashboard sharing with tenant-scoped permissions
- Widget embedding in external applications
- Public/private dashboard URLs with security tokens

**Organization & Libraries**:  
**ID**: `fdd-analytics-req-organization`  
**Capabilities**: `fdd-analytics-capability-organization`  
**Actors**: `fdd-analytics-actor-dashboard-designer`, `fdd-analytics-actor-template-developer`, `fdd-analytics-actor-plugin-developer`
- Hierarchical categories for all GTS types and instances
- Widget libraries for reusable component collections
- Template libraries (visualization marketplace)
- Datasource libraries (preconfigured connectors)
- Query libraries (shareable query definitions)

---

### 2. Principles

#### 1. Security First

**ID**: `fdd-analytics-principle-security-first`

SecurityCtx enforced at every level. No query execution without tenant context.

**Implementation**: SecurityCtx as first parameter in all service methods

#### 2. Plugin-Based Extensibility

**ID**: `fdd-analytics-principle-plugin-extensibility`

Datasources as dynamically registered plugins. No service restart required.

**Implementation**: Plugin registry with runtime registration

#### 3. GTS Native

**ID**: `fdd-analytics-principle-gts-native`

All plugin communication via GTS. Type safety at runtime.

**Implementation**: GTS Schema Registry for all data structures

#### 4. Strongly Typed

**ID**: `fdd-analytics-principle-strongly-typed`

All configuration validated with schemas. No runtime errors from invalid config.

**Implementation**: JSON Schema validation + GTS type checking

#### 5. Metadata Storage

**ID**: `fdd-analytics-principle-metadata-storage`

OLTP database for storing GTS types, instances, and configuration.

**Implementation**: PostgreSQL for metadata, GTS Registry for CRUD operations

#### 6. Data Agnostic Architecture

**ID**: `fdd-analytics-principle-data-agnostic`

No built-in data sources or DWH. All data access via registered queries to external systems.

**Implementation**: Query plugins with JWT propagation to external APIs/DWH

#### 7. Modular Design

**ID**: `fdd-analytics-principle-modular-design`  
**ADRs**: `fdd-analytics-adr-initial-architecture`

Reusable layouts, items, widgets, templates.

**Implementation**: GTS-based composable components

#### 8. API-First

**ID**: `fdd-analytics-principle-api-first`

REST API with OpenAPI specification. All features accessible via API.

**Implementation**: OpenAPI 3.x spec with code generation

#### 9. Horizontal Scalability

**ID**: `fdd-analytics-principle-horizontal-scalability`

Stateless services, distributed architecture.

**Implementation**: Kubernetes deployment, Redis for caching

#### 10. Tenant Isolation

**ID**: `fdd-analytics-principle-tenant-isolation`

Complete data separation per tenant. Cryptographic JWT integrity.

**Implementation**: Automatic tenant_id injection + JWT validation

#### 11. Mock Mode Support

**ID**: `fdd-analytics-principle-mock-mode`  
**ADRs**: `fdd-analytics-adr-mock-mode`

All services and UI components support mock mode for development and testing.

**Implementation**:
- **Service Mock Mode**: Analytics service can run without database or query plugins
  - Mock datasources provide realistic test data
  - Mock queries return sample responses matching real schemas
  - Enabled via `--mock-mode` flag or `MOCK_MODE=true` environment variable
  - Mock responses follow same GTS contracts as real implementations
  
- **UI Mock Mode**: Analytics UI can run without backend
  - Mock API client provides realistic dashboard data
  - All endpoints have mock implementations
  - Enabled via build-time configuration (`VITE_MOCK_MODE=true`)
  - Mock data includes complete GTS instances

**Benefits**: Faster local development, reliable E2E testing, demo environments, offline capability

---

### SDK Pattern

**Purpose**: Analytics module follows SDK pattern for clean separation

**Structure**:
```
modules/analytics/
├── analytics-sdk/    # Public API (transport-agnostic)
└── analytics/        # Implementation
```

**Rules**:
- SDK types MUST NOT have `serde` or transport-specific derives
- All API methods MUST accept `&SecurityCtx` as first parameter
- Consumers depend only on SDK crate
- Local client in module crate implements SDK trait

**Note**: Analytics currently has a flat structure and needs refactoring to follow SDK pattern.

**References**: [examples/modkit/users_info](../../../examples/modkit/users_info/), [guidelines/NEW_MODULE.md](../../../guidelines/NEW_MODULE.md)

---

### Plugin Architecture

**Implementation**: Analytics uses Gateway + Plugin pattern for datasources

**Gateway + Plugin Pattern**:
- **Analytics Gateway** - exposes query API, routes to selected datasource plugins
- **Datasource Plugins** - implement query execution for specific data sources (PostgreSQL, MySQL, REST APIs, etc.)
- **GTS-based Registration** - plugins register via types-registry with GTS instance IDs

**Plugin Discovery**:
- Plugins register GTS instances in types-registry
- Gateway queries types-registry for available datasource plugins
- Selection by: query configuration, tenant preferences

**Plugin Types**:
- Database plugins (PostgreSQL, MySQL, SQLite)
- API plugins (REST, GraphQL)
- File plugins (CSV, JSON, Parquet)
- Custom plugins (tenant-specific data sources)

**Key Concepts**:
- Query API trait: Public interface for consumers
- Datasource Plugin API trait: Interface for plugin implementations
- GTS Instance IDs: Each datasource identified by GTS ID
- Dynamic registration: Add plugins without service restart

**References**: [docs/MODKIT_PLUGINS.md](../../../docs/MODKIT_PLUGINS.md)

---

## C. Technical Architecture

#### C.1: Component Architecture

**PRESENTATION** (HAI3 - UI Application):
- Dashboards, Reports, Widgets
- Datasources & Templates
- Interactive features (drilldowns, tooltips, filtering)
- Admin Panel

**API LAYER**:
- REST API with Multi-Tenancy support (via Hyperspot Platform)
- Queries execution engine with JWT propagation
- GTS Registry for type and instance management
- OData v4 compatibility layer

**BUSINESS LOGIC**:
- GTS Types & Instances CRUD
- Query plugins for external data sources
- Adapter plugins for protocol translations (OData, REST, native)
- SecurityCtx enforcement (via Hyperspot Platform)

**STORAGE**:
- **PostgreSQL (OLTP)** - GTS metadata, types, instances, dashboards, configurations
- **No built-in DWH** - Analytics is data agnostic, all data via external queries

**PLATFORM INTEGRATION**:
- **Hyperspot Platform** - events, tenancy, auth, access control, UI settings

#### C.2: Domain Model

**Technology**: GTS (Global Type System) + JSON Schema

**Location**: `../gts/types/`

##### Domain Model Diagram

![Domain Model](diagrams/domain-model.drawio.svg)

Key entity relationships:
- Dashboard → Layout → Item → Widget
- Widget → Template + Datasource
- Query → Plugin → External API
- Schema → GTS Type → Validation

##### Core Domain Types

All domain types are defined using GTS (Global Type System) with JSON Schema format.

**Location**: `../gts/types/*.json`

The domain model consists of interconnected type categories managed through unified GTS Registry:

**Core Entities**:

**Schemas** - Define data structures and validation rules. All schemas inherit from base and require `examples[0]` for validation.

- [`gts://gts.hypernetix.hyperspot.ax.schema.v1~`](../gts/types/schema/v1/base.schema.json) - Base schema type with mandatory examples
- [`gts://gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~`](../gts/types/schema/v1/query_returns.schema.json) - Query result schema (paginated, scalar-only fields)
- [`gts://gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.template_config.v1~`](../gts/types/schema/v1/template_config.schema.json) - Template configuration base
- [`gts://gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.values.v1~`](../gts/types/schema/v1/values.schema.json) - Value lists for UI selectors

**Queries** - Define data retrieval operations using OData v4 protocol.

- [`gts://gts.hypernetix.hyperspot.ax.query.v1~`](../gts/types/query/v1/query.schema.json) - Query registration with OData integration
- [`gts://gts.hypernetix.hyperspot.ax.query_capabilities.v1~`](../gts/types/query/v1/query_capabilities.schema.json) - OData capabilities annotations (FilterRestrictions, SortRestrictions, etc)
- [`gts://gts.hypernetix.hyperspot.ax.query.v1~hypernetix.hyperspot.ax.values.v1~`](../gts/types/query/v1/values.schema.json) - Default OData query options

**Categories** - Organize and group related GTS entities by domain or purpose.

- [`gts://gts.hypernetix.hyperspot.ax.category.v1~`](../gts/types/category/v1/base.schema.json) - Base category (hierarchical classification)
- Category types for: Query, Template, Datasource, Widget, Item, Group, Dashboard, Layout

**Templates** - Define rendering logic and configuration for visual components.

- [`gts://gts.hypernetix.hyperspot.ax.template.v1~`](../gts/types/template/v1/base.schema.json) - Base template (UI component config)
- [`gts://gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~`](../gts/types/template/v1/widget.schema.json) - Widget template (data visualizations)
- [`gts://gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.values_selector.v1~`](../gts/types/template/v1/values_selector.schema.json) - Values selector template (dropdowns, autocomplete)

**Datasources** - Connect query definitions with runtime parameters and UI controls.

- [`gts://gts.hypernetix.hyperspot.ax.datasource.v1~`](../gts/types/datasource/v1/datasource.schema.json) - Datasource (query + params + UI controls)

**Items** - Reusable building blocks for layouts.

- [`gts://gts.hypernetix.hyperspot.ax.item.v1~`](../gts/types/item/v1/base.schema.json) - Base item (name, size, category)
- [`gts://gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~`](../gts/types/item/v1/widget.schema.json) - Widget item (template + datasource)
- [`gts://gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.group.v1~`](../gts/types/item/v1/group.schema.json) - Group item (container for items)

**Layouts** - Organize items into dashboards and reports.

- [`gts://gts.hypernetix.hyperspot.ax.layout.v1~`](../gts/types/layout/v1/base.schema.json) - Base layout (ordered item array)
- [`gts://gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.dashboard.v1~`](../gts/types/layout/v1/dashboard.schema.json) - Dashboard layout (real-time, auto-refresh)
- [`gts://gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~`](../gts/types/layout/v1/report.schema.json) - Report layout (scheduled, exportable)

**Component Registration**: All types and instances managed via unified `/gts` endpoint with automatic tenant isolation

---

#### C.3: API Contracts

**Technology**: OpenAPI 3.0.3 (REST)

**Location**: `architecture/openapi/v1/api.yaml`

##### OpenAPI Specification

**Design-Time OpenAPI**: [api.yaml](openapi/v1/api.yaml)  
**Runtime OpenAPI (repository)**: [api.json](../../../docs/api/api.json) (generated from code)  
**Runtime OpenAPI (endpoint)**: [openapi.json](http://localhost:8087/openapi.json)

**API Endpoints**:
- `/$metadata` - OData service metadata (JSON CSDL)
- `/gts` - GTS Registry (unified CRUD for all GTS types and instances)
- `/gts/{id}` - Get specific GTS entity by identifier
- `/gts/{id}/enablement` - Tenant enablement configuration for GTS entities
- `/queries/{id}` - Execute query with OData v4 options (GET/POST)
- `/queries/{id}/$metadata` - Query-specific metadata
- `/queries/{id}/$query` - Query with JSON body (POST)
- `/templates/{id}/bundle` - Upload/download template JavaScript bundles (POST/GET)

**Authentication**: JWT Bearer token in `Authorization` header

**Tenant Context**: Automatic `tenant_id` injection from SecurityCtx

**Note**: 
- All entity **metadata** (dashboards, templates, datasources, queries, schemas) registered via unified `/gts` endpoint
- Template **JavaScript bundles** uploaded separately via `/templates/{id}/bundle` endpoint
- Tenant enablement managed via `/gts/{id}/enablement` with automatic dependency enablement

**Future Endpoints** (defined in feature designs):
- Reporting endpoints - Report generation, scheduling, delivery (see `feature-reporting`)
- Sharing endpoints - Dashboard sharing, public URLs, embed tokens (see `feature-export-sharing`)

##### REST API Standards

**Purpose**: Define REST API conventions and standards for Analytics module

**Required Standards** (see [guidelines/DNA/REST/API.md](../../../guidelines/DNA/REST/API.md)):

**Pagination**: OData cursor-based with `$filter`, `$orderby`, `$select`
- Max limit: 200 items per page
- Opaque versioned cursors
- Filter safety: cursors bound to query
- Already implemented in `/gts` endpoint

**Error Handling**: RFC 9457 Problem Details
- All 4xx/5xx return `application/problem+json`
- Include `type`, `title`, `status`, `detail`, `traceId`
- Structured validation errors with field-level details

**Status Codes** (see [guidelines/DNA/REST/STATUS_CODES.md](../../../guidelines/DNA/REST/STATUS_CODES.md)):
- Success: 200 (read), 201 (create + Location), 204 (delete), 202 (async)
- Client errors: 400 (bad request), 401 (auth), 403 (authz), 404, 409 (conflict), 422 (validation), 429 (rate limit)
- Server errors: 500 (internal), 503 (unavailable + Retry-After)

**Concurrency**: ETag + If-Match for optimistic locking
- Used for GTS entity updates to prevent concurrent modifications

**Idempotency**: Idempotency-Key header on POST/PATCH/DELETE
- Query execution is idempotent by design (same query params = same results)

**Rate Limiting**: RateLimit-Policy and RateLimit headers (IETF draft)
- Per-tenant query rate limits
- Per-tenant API call rate limits

 **References**:
 - [REST API Guidelines](../../../guidelines/DNA/REST/API.md)
 - [Querying / Pagination](../../../guidelines/DNA/REST/QUERYING.md)
 - [Status Codes](../../../guidelines/DNA/REST/STATUS_CODES.md)

---

#### C.4: Security Model

**Authentication**: JWT-based authentication provided by Hyperspot Platform

**Token Structure**:
- JWT tokens issued by platform authentication service
- Contains: `tenant_id`, `user_id`, `roles`, `permissions`, `exp` (expiration)
- Signed with platform secret key
- Validated on every API request

**Authorization**: SecurityCtx-based authorization

**SecurityCtx Enforcement**:
- Created from JWT token on each request (not stored)
- Contains validated `tenant_id`, `user_id`, and permission set
- Passed as first parameter to all service methods: `fn method(&SecurityCtx, ...)`
- Enforced at compile-time via Secure ORM

**Secure ORM**:
- All database entities derive `#[derive(Scopable)]`
- Database queries require `SecureConn` with `SecurityCtx`
- Automatic `tenant_id` filtering in all queries
- Compile-time error if query executed without SecurityCtx
- **Cannot bypass** - unscoped queries won't compile

**Data Protection**:
- **Tenant Isolation**: Complete data separation per tenant via SecurityCtx
- **Row-Level Security**: Automatic tenant_id filtering in all database queries
- **JWT Validation**: Signature verification on every request
- **Encryption at Rest**: Database encryption enabled
- **Encryption in Transit**: TLS 1.3 for all API communication

**Security Boundaries**:
- **API Layer**: JWT validation, SecurityCtx creation
- **Service Layer**: SecurityCtx propagation, authorization checks
- **Repository Layer**: Secure ORM enforcement, tenant filtering
- **Plugin Layer**: JWT propagation to external APIs, tenant context maintained

**External Data Access**:
- Analytics propagates JWT to external query plugins
- External APIs responsible for validating JWT and filtering by `tenant_id`
- No direct database access - all data via query plugins

**Audit Logging**: Provided by Hyperspot Platform
- All API requests logged with: `tenant_id`, `user_id`, `endpoint`, `timestamp`, `traceId`
- Query execution logged with: query_id, parameters, execution time
- GTS entity changes logged with: entity_id, operation, before/after state

**References**: 
- [docs/SECURE-ORM.md](../../../docs/SECURE-ORM.md) - Secure ORM implementation details
- [guidelines/SECURITY.md](../../../guidelines/SECURITY.md) - Platform security guidelines
- Section B.1a - Security Requirements

---

#### C.5: Non-Functional Requirements

**Performance Requirements**:  
**ID**: `fdd-analytics-nfr-performance`
- Query execution: p95 < 1s, p99 < 3s (depends on external data sources)
- Dashboard load: < 2s for typical dashboard (10-20 widgets)
- API response: p95 < 200ms for metadata operations
- Plugin registration: < 5s to register new datasource plugin
- Query result caching reduces repeated query latency

**Scalability Requirements**:  
**ID**: `fdd-analytics-nfr-scalability`
- Horizontal scaling: Stateless service design enables adding instances
- Concurrent users: 100+ per tenant without degradation
- Data volume: 10M+ rows per query result (limited by external sources)
- Entity limits: 1000+ dashboards per tenant, 100+ widgets per dashboard
- Plugin capacity: Unlimited datasource plugins via dynamic registration

**Reliability & Availability Requirements**:  
**ID**: `fdd-analytics-nfr-reliability`
- Uptime SLA: 99.9% availability target
- Health checks: Liveness and readiness probes for Kubernetes
- Graceful degradation: Mock mode fallback when dependencies unavailable
- Retry logic: Automatic retry with exponential backoff for transient failures
- Circuit breaker: Prevent cascade failures from external API issues

**Observability Requirements**:  
**ID**: `fdd-analytics-nfr-observability`
- Distributed tracing: OpenTelemetry with W3C Trace Context propagation
- Structured logging: JSON logs with `traceId`, `requestId`, `tenant_id`
- Metrics: Prometheus RED (Rate, Errors, Duration) and USE (Utilization, Saturation, Errors) metrics
- Health endpoints: `/health` (liveness), `/ready` (readiness)
- **References**: [docs/TRACING_SETUP.md](../../../docs/TRACING_SETUP.md)

**Maintainability Requirements**:  
**ID**: `fdd-analytics-nfr-maintainability`
- Modular architecture: Plugin-based extensibility without core changes
- SDK pattern: Clear contract/implementation separation
- Type safety: GTS eliminates runtime type errors
- Testing: Mock mode enables fast local development and E2E testing
- Documentation: OpenAPI specification auto-generated from code

**Deployment Requirements**:  
**ID**: `fdd-analytics-nfr-deployment`
- Container-based: Docker images for consistent deployment
- Kubernetes-ready: Supports horizontal pod autoscaling
- Configuration: Environment variables and config files
- Zero downtime: Rolling updates without service interruption