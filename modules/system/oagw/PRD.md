# PRD — Outbound API Gateway (OAGW)

<!-- TOC START -->
## Table of Contents

- [Overview](#overview)
  - [Key Concepts](#key-concepts)
  - [Target Users](#target-users)
  - [Problems Solved](#problems-solved)
  - [Success Criteria](#success-criteria)
- [Actors](#actors)
  - [Human](#human)
  - [System](#system)
- [Functional Requirements](#functional-requirements)
  - [Upstream Management](#upstream-management)
  - [Route Management](#route-management)
  - [Request Proxying](#request-proxying)
  - [Authentication Injection](#authentication-injection)
  - [Rate Limiting](#rate-limiting)
  - [Header Transformation](#header-transformation)
  - [Plugin System](#plugin-system)
  - [Streaming Support](#streaming-support)
  - [Configuration Layering](#configuration-layering)
  - [Hierarchical Configuration Override](#hierarchical-configuration-override)
  - [Alias Resolution and Shadowing](#alias-resolution-and-shadowing)
- [Use Cases](#use-cases)
  - [Proxy HTTP Request](#proxy-http-request)
  - [Configure Upstream](#configure-upstream)
  - [Configure Route](#configure-route)
  - [Rate Limit Exceeded](#rate-limit-exceeded)
  - [SSE Streaming](#sse-streaming)
- [Non-Functional Requirements](#non-functional-requirements)
- [Built-in Plugins](#built-in-plugins)
  - [Auth Plugins (`gts.x.core.oagw.plugin.auth.v1~*`)](#auth-plugins-gtsxcoreoagwpluginauthv1)
  - [Guard Plugins (`gts.x.core.oagw.plugin.guard.v1~*`)](#guard-plugins-gtsxcoreoagwpluginguardv1)
  - [Transform Plugins (`gts.x.core.oagw.plugin.transform.v1~*`)](#transform-plugins-gtsxcoreoagwplugintransformv1)
  - [Error Codes](#error-codes)
- [API Endpoints](#api-endpoints)
- [Dependencies](#dependencies)

<!-- TOC END -->

## 1. Overview

### 1.1 Purpose

OAGW manages all outbound API requests from CyberFabric to external services, centralizing credential management, rate limiting, and security policies.

### 1.2 Background / Problem Statement

Applications consuming external APIs (OpenAI, Stripe, etc.) need a secure, centralized way to manage credentials, enforce rate limits, and apply consistent policies. Direct API integration exposes credentials in application code, lacks unified rate limiting, and complicates SSRF protection and audit trails.

OAGW solves this by providing a proxy endpoint with credential injection, configurable routing, plugin-based transformations, and hierarchical configuration inheritance across tenant boundaries.

### 1.3 Goals (Business Outcomes)

- <10ms added latency (p95) for proxied requests
- Zero credential exposure in logs or error responses
- 99.9% availability with circuit breaker protection
- Complete audit trail for all outbound API requests

### 1.4 Glossary

| Term | Definition |
|------|------------|
| Upstream | External service target (scheme/host/port, protocol, auth, headers, rate limits) |
| Route | API path on an upstream. Matches by method/path/query (HTTP), service/method (gRPC) |
| Plugin | Modular processor - Auth (credential injection), Guard (validation), Transform (mutation) |
| Alias | Human-readable identifier for upstream resolution in proxy URLs |
| Shadowing | Descendant tenant upstream overriding ancestor upstream with same alias |

## 2. Actors

### 2.1 Human Actors

#### Platform Operator

- [ ] `p1` - **ID**: `fdd-oagw-actor-platform-operator-v1`

**Role**: Manages global configuration including upstreams, routes, system-wide plugins, and security policies.

**Needs**: Administrative access to configure gateway-wide settings, enforce policies across tenants, monitor system health.

#### Tenant Administrator

- [ ] `p1` - **ID**: `fdd-oagw-actor-tenant-admin-v1`

**Role**: Manages tenant-specific settings including credentials, rate limits, and custom plugins.

**Needs**: Ability to configure tenant-scoped upstreams, override inherited configurations, manage secrets.

#### Application Developer

- [ ] `p2` - **ID**: `fdd-oagw-actor-app-developer-v1`

**Role**: Consumes external APIs via proxy endpoint without managing credentials directly.

**Needs**: Simple proxy interface, reliable request forwarding, clear error messages, no credential handling.

### 2.2 System Actors

#### Credential Store

- [ ] `p1` - **ID**: `fdd-oagw-actor-cred-store-v1`

**Role**: Secure storage and retrieval of secrets by UUID reference, tenant-isolated.

#### Types Registry

- [ ] `p1` - **ID**: `fdd-oagw-actor-types-registry-v1`

**Role**: GTS schema and instance registration, validation of configuration payloads.

#### Upstream Service

- [ ] `p1` - **ID**: `fdd-oagw-actor-upstream-service-v1`

**Role**: External third-party service (OpenAI, Stripe, etc.) receiving proxied requests.

## 3. Scope

### 3.1 In Scope

- HTTP family traffic proxying (HTTP, SSE, WebSocket, WebTransport)
- Upstream and route CRUD operations
- Multi-protocol authentication (API Key, OAuth2, Basic, Bearer)
- Tenant-scoped rate limiting with hierarchical enforcement
- Plugin system (Auth, Guard, Transform)
- Hierarchical configuration with sharing modes
- Alias-based upstream resolution with shadowing
- Streaming support (SSE, WebSocket)

### 3.2 Out of Scope

- gRPC support (planned for later phase, p4)
- Automatic request retries (client-managed)
- Response caching (future consideration)

## 4. Functional Requirements

### 4.1 Upstream Management

#### CRUD Upstream Configurations

- [ ] `p1` - **ID**: `fdd-oagw-fr-upstream-crud-v1`

The system **MUST** provide REST API endpoints for creating, reading, updating, and deleting upstream configurations.

**Rationale**: Platform operators and tenant administrators need to manage external service targets dynamically.

**Actors**: `fdd-oagw-actor-platform-operator-v1`, `fdd-oagw-actor-tenant-admin-v1`

#### Upstream Definition

- [ ] `p1` - **ID**: `fdd-oagw-fr-upstream-schema-v1`

The system **MUST** support upstream definitions including server endpoints (host, port, scheme), protocol, authentication configuration, request/response headers, and rate limits.

**Rationale**: Complete upstream specification enables flexible integration with diverse external services.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

### 4.2 Route Management

#### CRUD Route Configurations

- [ ] `p1` - **ID**: `fdd-oagw-fr-route-crud-v1`

The system **MUST** provide REST API endpoints for creating, reading, updating, and deleting route configurations.

**Rationale**: Routes map API paths to upstreams with granular matching rules.

**Actors**: `fdd-oagw-actor-platform-operator-v1`, `fdd-oagw-actor-tenant-admin-v1`

#### Route Matching Rules

- [ ] `p1` - **ID**: `fdd-oagw-fr-route-matching-v1`

The system **MUST** support route matching by HTTP method, path pattern, and query parameter allowlist.

**Rationale**: Flexible routing enables fine-grained control over which requests reach specific upstreams.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

### 4.3 Request Proxying

#### Proxy Endpoint

- [ ] `p1` - **ID**: `fdd-oagw-fr-proxy-endpoint-v1`

The system **MUST** expose proxy endpoint at `{METHOD} /api/oagw/v1/proxy/{alias}[/{path}][?{query}]`.

**Rationale**: Single entry point for all proxied requests with alias-based upstream resolution.

**Actors**: `fdd-oagw-actor-app-developer-v1`

#### Alias Resolution

- [ ] `p1` - **ID**: `fdd-oagw-fr-alias-resolution-v1`

The system **MUST** resolve upstream by alias from tenant hierarchy (descendant to root), with closest match winning.

**Rationale**: Hierarchical resolution enables tenant shadowing and configuration inheritance.

**Actors**: `fdd-oagw-actor-app-developer-v1`

#### No Automatic Retries

- [ ] `p1` - **ID**: `fdd-oagw-fr-no-retries-v1`

The system **MUST** perform at most one upstream attempt per inbound request without automatic retries.

**Rationale**: Retry behavior is client-managed to avoid idempotency issues and duplicate operations.

**Actors**: `fdd-oagw-actor-app-developer-v1`

### 4.4 Authentication Injection

#### Credential Retrieval

- [ ] `p1` - **ID**: `fdd-oagw-fr-auth-credential-retrieval-v1`

The system **MUST** retrieve credentials from credential store by UUID reference at request time.

**Rationale**: Credentials are never stored in gateway configuration, only secure references.

**Actors**: `fdd-oagw-actor-cred-store-v1`

#### Supported Auth Methods

- [ ] `p1` - **ID**: `fdd-oagw-fr-auth-methods-v1`

The system **MUST** support API Key, Basic Auth, OAuth2 Client Credentials, and Bearer Token authentication methods.

**Rationale**: Covers most common external API authentication schemes.

**Actors**: `fdd-oagw-actor-upstream-service-v1`

### 4.5 Rate Limiting

#### Rate Limit Enforcement

- [ ] `p1` - **ID**: `fdd-oagw-fr-rate-limit-enforce-v1`

The system **MUST** enforce rate limits at upstream and route levels with configurable rate, window, capacity, cost, scope, and strategy.

**Rationale**: Prevents abuse and cost overruns from excessive external API usage.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

#### Rate Limit Scope

- [ ] `p1` - **ID**: `fdd-oagw-fr-rate-limit-scope-v1`

The system **MUST** support rate limiting scopes: global, tenant, user, and IP address.

**Rationale**: Granular scoping enables precise control over different traffic sources.

**Actors**: `fdd-oagw-actor-tenant-admin-v1`

#### Rate Limit Strategies

- [ ] `p1` - **ID**: `fdd-oagw-fr-rate-limit-strategies-v1`

The system **MUST** support rate limit strategies: reject (429 with Retry-After), queue, and degrade.

**Rationale**: Different strategies for graceful degradation versus strict enforcement.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

### 4.6 Header Transformation

#### Header Operations

- [ ] `p1` - **ID**: `fdd-oagw-fr-header-transform-v1`

The system **MUST** support header transformations: set, add, remove, and passthrough control.

**Rationale**: Enables header injection, sanitization, and protocol compliance.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

#### Hop-by-Hop Stripping

- [ ] `p1` - **ID**: `fdd-oagw-fr-header-hop-strip-v1`

The system **MUST** automatically strip hop-by-hop headers (Connection, Keep-Alive, etc.) from proxied requests/responses.

**Rationale**: Protocol compliance and security best practices require removing proxy-specific headers.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

### 4.7 Plugin System

#### Plugin Types

- [ ] `p1` - **ID**: `fdd-oagw-fr-plugin-types-v1`

The system **MUST** support three plugin types: Auth (`gts.x.core.oagw.plugin.auth.v1~*`), Guard (`gts.x.core.oagw.plugin.guard.v1~*`), and Transform (`gts.x.core.oagw.plugin.transform.v1~*`).

**Rationale**: Modular architecture enables credential injection, validation, and request/response mutation.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

#### Plugin Execution Order

- [ ] `p1` - **ID**: `fdd-oagw-fr-plugin-execution-order-v1`

The system **MUST** execute plugins in order: Auth → Guards → Transform(request) → Upstream → Transform(response/error).

**Rationale**: Deterministic execution order ensures predictable behavior and proper credential injection before request forwarding.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

#### Plugin Chain Composition

- [ ] `p1` - **ID**: `fdd-oagw-fr-plugin-chain-v1`

The system **MUST** execute upstream plugins before route plugins in the plugin chain.

**Rationale**: Upstream-level policies apply first, then route-specific overrides.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

#### Plugin Immutability

- [ ] `p1` - **ID**: `fdd-oagw-fr-plugin-immutability-v1`

The system **MUST** enforce plugin definition immutability after creation; updates require creating new plugin version and re-binding references.

**Rationale**: Immutability guarantees deterministic behavior for attached routes/upstreams, improves auditability, and avoids in-place source mutation risks.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

#### Circuit Breaker Policy

- [ ] `p1` - **ID**: `fdd-oagw-fr-circuit-breaker-v1`

The system **MUST** implement circuit breaker as core gateway resilience capability (configured as core policy, not a plugin).

**Rationale**: Circuit breaker is fundamental resilience mechanism preventing cascade failures.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

### 4.8 Streaming Support

#### HTTP Family Streaming

- [ ] `p1` - **ID**: `fdd-oagw-fr-streaming-http-v1`

The system **MUST** support HTTP request/response, SSE, WebSocket, and WebTransport session flows.

**Rationale**: Modern external APIs use streaming protocols for real-time data.

**Actors**: `fdd-oagw-actor-app-developer-v1`

#### SSE Event Forwarding

- [ ] `p1` - **ID**: `fdd-oagw-fr-streaming-sse-v1`

The system **MUST** forward SSE events as received and handle connection lifecycle (open/close/error).

**Rationale**: Low-latency event streaming requires immediate forwarding without buffering.

**Actors**: `fdd-oagw-actor-app-developer-v1`

### 4.9 Configuration Layering

#### Configuration Merge

- [ ] `p1` - **ID**: `fdd-oagw-fr-config-merge-v1`

The system **MUST** merge configurations with priority: Upstream (base) < Route < Tenant (highest priority).

**Rationale**: Hierarchical merging enables sensible defaults with targeted overrides.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

### 4.10 Hierarchical Configuration Override

#### Sharing Modes

- [ ] `p1` - **ID**: `fdd-oagw-fr-sharing-modes-v1`

The system **MUST** support configuration sharing modes: `private` (not visible to descendants), `inherit` (visible, descendant can override), and `enforce` (visible, descendant cannot override).

**Rationale**: Fine-grained control over configuration inheritance across tenant boundaries.

**Actors**: `fdd-oagw-actor-platform-operator-v1`, `fdd-oagw-actor-tenant-admin-v1`

#### Auth Override

- [ ] `p1` - **ID**: `fdd-oagw-fr-auth-override-v1`

The system **MUST** allow descendant tenant with permission to override inherited auth configuration when `sharing: inherit`.

**Rationale**: Enables tenants to use own credentials while inheriting upstream configuration.

**Actors**: `fdd-oagw-actor-tenant-admin-v1`

#### Rate Limit Enforcement Hierarchy

- [ ] `p1` - **ID**: `fdd-oagw-fr-rate-limit-hierarchy-v1`

The system **MUST** calculate effective rate limit as `min(ancestor.enforced, descendant)` when ancestor enforces rate limits.

**Rationale**: Descendants can only be stricter, preventing bypass of parent limits.

**Actors**: `fdd-oagw-actor-tenant-admin-v1`

#### Plugin Layering

- [ ] `p1` - **ID**: `fdd-oagw-fr-plugin-layering-v1`

The system **MUST** append descendant plugins to inherited plugin chain; enforced plugins cannot be removed.

**Rationale**: Ensures mandatory policies from ancestors remain active.

**Actors**: `fdd-oagw-actor-tenant-admin-v1`

#### Tag Merging

- [ ] `p1` - **ID**: `fdd-oagw-fr-tag-merging-v1`

The system **MUST** merge tags using `union(ancestor_tags..., descendant_tags)` with add-only semantics; descendants cannot remove inherited tags.

**Rationale**: Discovery metadata accumulates through hierarchy without deletion.

**Actors**: `fdd-oagw-actor-tenant-admin-v1`

### 4.11 Alias Resolution and Shadowing

#### Alias Defaults Single Host

- [ ] `p1` - **ID**: `fdd-oagw-fr-alias-default-single-v1`

The system **MUST** default alias to `hostname` (without port) for single-endpoint upstreams.

**Rationale**: Automatic alias generation reduces configuration burden for simple cases.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

#### Alias Defaults Multi-Host

- [ ] `p1` - **ID**: `fdd-oagw-fr-alias-default-multi-v1`

The system **MUST** extract common domain suffix as default alias for multi-endpoint upstreams with common suffix.

**Rationale**: Load-balanced upstreams share logical identity via common domain.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

#### Alias Explicit Requirement

- [ ] `p1` - **ID**: `fdd-oagw-fr-alias-explicit-v1`

The system **MUST** require explicit alias for IP-based or heterogeneous host endpoints.

**Rationale**: No logical alias can be inferred from IP addresses or unrelated hostnames.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

#### Shadowing Resolution

- [ ] `p1` - **ID**: `fdd-oagw-fr-shadowing-resolution-v1`

The system **MUST** resolve alias by searching tenant hierarchy from descendant to root, with closest match winning.

**Rationale**: Descendant upstreams shadow ancestor upstreams with same alias.

**Actors**: `fdd-oagw-actor-tenant-admin-v1`

#### Multi-Endpoint Pooling

- [ ] `p1` - **ID**: `fdd-oagw-fr-multi-endpoint-pool-v1`

The system **MUST** treat multiple endpoints within same upstream as load-balance pool, distributing requests across endpoints.

**Rationale**: High availability and load distribution for upstream services.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

#### Endpoint Compatibility

- [ ] `p1` - **ID**: `fdd-oagw-fr-endpoint-compatibility-v1`

The system **MUST** enforce identical protocol, scheme, and port across all endpoints in a pool.

**Rationale**: Mixed protocols/schemes/ports would create routing ambiguity.

**Actors**: `fdd-oagw-actor-platform-operator-v1`

#### Enforced Limits Across Shadowing

- [ ] `p1` - **ID**: `fdd-oagw-fr-enforced-limits-shadowing-v1`

The system **MUST** apply enforced limits from ancestor even when descendant shadows alias.

**Rationale**: Shadowing does not bypass ancestor-enforced policies.

**Actors**: `fdd-oagw-actor-tenant-admin-v1`

## 5. Use Cases

#### UC: Proxy HTTP Request

- [ ] `p1` - **ID**: `fdd-oagw-usecase-proxy-request-v1`

**Actor**: `fdd-oagw-actor-app-developer-v1`

**Preconditions**:
- Upstream and route configured
- Credentials stored in credential store
- Request matches route pattern

**Main Flow**:
1. App sends request to `/api/oagw/v1/proxy/{alias}/{path}`
2. System resolves upstream by alias from tenant hierarchy
3. System matches route by method/path
4. System merges configurations (upstream < route < tenant)
5. System retrieves credentials from credential store
6. System transforms request headers
7. System executes plugin chain (Auth → Guards → Transform)
8. System forwards request to upstream
9. System returns response to app

**Postconditions**:
- Request logged with correlation ID
- Metrics recorded (latency, status, upstream)

#### UC: Configure Upstream

- [ ] `p1` - **ID**: `fdd-oagw-usecase-config-upstream-v1`

**Actor**: `fdd-oagw-actor-platform-operator-v1`, `fdd-oagw-actor-tenant-admin-v1`

**Preconditions**:
- User has upstream configuration permission
- Server endpoints valid and reachable

**Main Flow**:
1. User POSTs to `/api/oagw/v1/upstreams` with server endpoints, protocol, auth config
2. System validates endpoint format and protocol compatibility
3. System generates or validates alias
4. System persists upstream configuration
5. System returns created upstream with ID

**Postconditions**:
- Upstream available for route binding
- Configuration logged to audit trail

#### UC: Configure Route

- [ ] `p1` - **ID**: `fdd-oagw-usecase-config-route-v1`

**Actor**: `fdd-oagw-actor-platform-operator-v1`, `fdd-oagw-actor-tenant-admin-v1`

**Preconditions**:
- Upstream exists and is accessible
- User has route configuration permission

**Main Flow**:
1. User POSTs to `/api/oagw/v1/routes` with upstream_id and match rules
2. System validates upstream reference exists
3. System validates match rules (method, path pattern, query allowlist)
4. System persists route configuration
5. System returns created route with ID

**Postconditions**:
- Route active for request matching
- Configuration logged to audit trail

#### UC: Rate Limit Exceeded

- [ ] `p1` - **ID**: `fdd-oagw-usecase-rate-limit-exceeded-v1`

**Actor**: `fdd-oagw-actor-app-developer-v1`

**Preconditions**:
- Rate limit configured for upstream/route
- Request count exceeds rate limit threshold

**Main Flow**:
1. System detects rate limit exceeded for scope (tenant/user/IP)
2. System applies configured strategy (reject/queue/degrade)

**Alternative Flows**:
- **Strategy: reject**: System returns 429 with Retry-After header
- **Strategy: queue**: System queues request for later processing
- **Strategy: degrade**: System applies degraded service mode

**Postconditions**:
- Rate limit event logged with scope and threshold
- Metrics recorded (rate limit hits by scope)

#### UC: SSE Streaming

- [ ] `p1` - **ID**: `fdd-oagw-usecase-sse-streaming-v1`

**Actor**: `fdd-oagw-actor-app-developer-v1`

**Preconditions**:
- Upstream supports SSE protocol
- Route configured for streaming

**Main Flow**:
1. App opens SSE connection to `/api/oagw/v1/proxy/{alias}/{path}`
2. System establishes upstream SSE connection
3. System forwards events as received without buffering
4. System handles connection lifecycle (open/close/error)
5. System closes downstream connection when upstream closes

**Postconditions**:
- Connection duration logged
- Event count metrics recorded

## 6. Non-Functional Requirements

### 6.1 Module-Specific NFRs

#### Low Latency

- [ ] `p1` - **ID**: `fdd-oagw-nfr-latency-v1`

The system **MUST** add <10ms overhead (p95) for proxied requests.

**Threshold**: p95 latency ≤ 10ms measured from proxy entry to upstream forwarding

**Rationale**: Gateway latency directly impacts user-facing response times; minimal overhead critical for real-time APIs.

**Architecture Allocation**: See DESIGN.md § NFR Allocation for caching and connection pooling strategies

#### High Availability

- [ ] `p1` - **ID**: `fdd-oagw-nfr-availability-v1`

The system **MUST** maintain 99.9% availability with circuit breaker protection.

**Threshold**: 99.9% uptime (43 minutes/month max downtime), circuit breaker trips at 50% error rate

**Rationale**: Gateway is critical path for all external API access; downtime blocks all outbound integrations.

**Architecture Allocation**: See DESIGN.md § Circuit Breaker for failure detection and recovery

#### SSRF Protection

- [ ] `p1` - **ID**: `fdd-oagw-nfr-ssrf-protection-v1`

The system **MUST** validate DNS resolution, pin IP addresses, and strip sensitive headers.

**Threshold**: 100% of requests validated against allowlist/denylist, no internal IP ranges reachable

**Rationale**: Prevents Server-Side Request Forgery attacks targeting internal infrastructure.

**Architecture Allocation**: See DESIGN.md § Security for DNS validation and header filtering

#### Credential Isolation

- [ ] `p1` - **ID**: `fdd-oagw-nfr-credential-isolation-v1`

The system **MUST** never expose credentials in logs or error responses, using UUID references only.

**Threshold**: Zero credential leakage in logs/errors/responses, 100% tenant-isolated credential access

**Rationale**: Credential exposure violates security policy and creates compliance risk.

**Architecture Allocation**: See DESIGN.md § Security for credential store integration

#### Input Validation

- [ ] `p1` - **ID**: `fdd-oagw-nfr-input-validation-v1`

The system **MUST** validate path, query, headers, and body size; reject invalid requests with 400.

**Threshold**: 100% of requests validated before forwarding, max body size 10MB

**Rationale**: Prevents injection attacks and resource exhaustion from malformed requests.

**Architecture Allocation**: See DESIGN.md § Request Validation for validation pipeline

#### Observability

- [ ] `p1` - **ID**: `fdd-oagw-nfr-observability-v1`

The system **MUST** log requests with correlation ID and expose Prometheus metrics.

**Threshold**: 100% request logging, <50ms metric collection overhead, 7-day log retention

**Rationale**: Enables troubleshooting, performance analysis, and SLA monitoring.

**Architecture Allocation**: See DESIGN.md § Observability for logging and metrics architecture

#### Starlark Sandbox

- [ ] `p1` - **ID**: `fdd-oagw-nfr-starlark-sandbox-v1`

The system **MUST** enforce Starlark sandbox restrictions: no network/file I/O, no imports, timeout/memory limits.

**Threshold**: Network/file access blocked, 100ms execution timeout, 10MB memory limit per plugin

**Rationale**: Custom plugins must not access external resources or consume excessive resources.

**Architecture Allocation**: See DESIGN.md § Plugin System for sandbox implementation

#### Multi-tenancy

- [ ] `p1` - **ID**: `fdd-oagw-nfr-multi-tenancy-v1`

The system **MUST** scope all resources by tenant with isolation at data layer.

**Threshold**: 100% tenant-scoped queries, zero cross-tenant data leakage

**Rationale**: Tenant isolation is fundamental security requirement for multi-tenant architecture.

**Architecture Allocation**: See DESIGN.md § Multi-Tenancy for data isolation strategy

## 7. Public Library Interfaces

### 7.1 Public API Surface

#### REST API Endpoints

- [ ] `p1` - **ID**: `fdd-oagw-interface-rest-api-v1`

**Type**: REST API

**Stability**: stable

**Description**: CRUD operations for upstreams, routes, plugins; proxy endpoint for request forwarding.

**Breaking Change Policy**: Major version bump required for endpoint path, method, or request/response schema changes.

**Endpoints**:
- `POST/GET/PUT/DELETE /api/oagw/v1/upstreams[/{id}]`
- `POST/GET/PUT/DELETE /api/oagw/v1/routes[/{id}]`
- `POST/GET/DELETE /api/oagw/v1/plugins[/{id}]`
- `GET /api/oagw/v1/plugins/{id}/source`
- `{METHOD} /api/oagw/v1/proxy/{alias}[/{path}][?{query}]`

#### Built-in Auth Plugins

- [ ] `p1` - **ID**: `fdd-oagw-interface-auth-plugins-v1`

**Type**: Plugin Interface

**Stability**: stable

**Description**: GTS-identified auth plugins for credential injection.

**Breaking Change Policy**: Plugin GTS ID immutable; config schema changes require new version.

**Plugins**:
- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.noop.v1` - No authentication
- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.apikey.v1` - API key injection (header/query)
- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.basic.v1` - HTTP Basic authentication
- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.oauth2.client_cred.v1` - OAuth2 client credentials flow
- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.oauth2.client_cred_basic.v1` - OAuth2 with Basic auth
- `gts.x.core.oagw.plugin.auth.v1~x.core.oagw.bearer.v1` - Bearer token injection

#### Built-in Guard Plugins

- [ ] `p1` - **ID**: `fdd-oagw-interface-guard-plugins-v1`

**Type**: Plugin Interface

**Stability**: stable

**Description**: GTS-identified guard plugins for validation and policy enforcement.

**Breaking Change Policy**: Plugin GTS ID immutable; config schema changes require new version.

**Plugins**:
- `gts.x.core.oagw.plugin.guard.v1~x.core.oagw.timeout.v1` - Request timeout enforcement
- `gts.x.core.oagw.plugin.guard.v1~x.core.oagw.cors.v1` - CORS preflight validation

#### Built-in Transform Plugins

- [ ] `p1` - **ID**: `fdd-oagw-interface-transform-plugins-v1`

**Type**: Plugin Interface

**Stability**: stable

**Description**: GTS-identified transform plugins for request/response mutation.

**Breaking Change Policy**: Plugin GTS ID immutable; config schema changes require new version.

**Plugins**:
- `gts.x.core.oagw.plugin.transform.v1~x.core.oagw.logging.v1` - Request/response logging
- `gts.x.core.oagw.plugin.transform.v1~x.core.oagw.metrics.v1` - Prometheus metrics collection
- `gts.x.core.oagw.plugin.transform.v1~x.core.oagw.request_id.v1` - X-Request-ID propagation

## 8. Error Codes

| HTTP | Error                | Retriable | Description |
|------|----------------------|-----------|-------------|
| 400  | ValidationError      | No        | Invalid request format, path, query, or headers |
| 401  | AuthenticationFailed | No        | Credential retrieval failed or invalid |
| 404  | RouteNotFound        | No        | No route matches request pattern |
| 413  | PayloadTooLarge      | No        | Request body exceeds size limit |
| 429  | RateLimitExceeded    | Yes       | Rate limit threshold exceeded for scope |
| 500  | SecretNotFound       | No        | Credential UUID reference not found in store |
| 502  | DownstreamError      | Depends   | Upstream service returned error |
| 503  | CircuitBreakerOpen   | Yes       | Circuit breaker protecting upstream |
| 504  | Timeout              | Yes       | Upstream request timeout exceeded |

## 9. Dependencies

| Dependency | Description | Criticality |
|------------|-------------|-------------|
| `types-registry` | GTS schema/instance registration and validation | p1 |
| `cred_store` | Secure secret retrieval by UUID reference | p1 |
| `api_ingress` | REST API hosting and routing | p1 |
| `modkit-db` | Database persistence for upstreams/routes/plugins | p1 |
| `modkit-auth` | Authorization and tenant context | p1 |

## 10. Assumptions

- Upstream services support HTTP family protocols (HTTP, SSE, WebSocket)
- Credential store provides tenant-isolated secret access
- DNS resolution is reliable and SSRF-protected
- Plugin execution completes within timeout (100ms default)

## 11. Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Upstream service downtime | Gateway cannot forward requests | Circuit breaker prevents cascade failures |
| Plugin timeout or infinite loop | Request delays or gateway hangs | Strict timeout enforcement (100ms), memory limits |
| Credential store unavailability | Cannot inject auth credentials | Fail-safe: reject requests with 500 SecretNotFound |
| DNS rebinding attack | SSRF access to internal services | IP pinning, allowlist/denylist validation |

## 12. Traceability

- **Design**: [DESIGN.md](./DESIGN.md)
- **ADRs**: [docs/adr-resource-identification.md](./docs/adr-resource-identification.md)
