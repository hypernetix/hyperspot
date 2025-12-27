## ADDED Requirements

### Requirement: model_provider module responsibility
The system SHALL provide a `model_provider` gateway module with **pluggable co-loaded model provider plugins** responsible for managing GenAI model providers, their endpoints, and tenant provisioning state.

The `model_provider` module SHALL:
- Own persistence of anonymous objects identified by UUID (provider instances, endpoints, join tables).
- Treat global taxonomies (vendor/kind/capability/feature/endpoint_type) as well-known GTS instances stored in `types_registry`.
- Expose:
  - A Rust-native gateway API (ClientHub) for other modules.
  - A REST API for external clients.
  - A plugin interface for provider connectivity and capabilities.

#### Scenario: Module boundaries
- **WHEN** a caller needs to list known provider vendors
- **THEN** the caller retrieves vendor instances from `types_registry`
- **AND** the `model_provider` database is not used to store vendor taxonomy

### Requirement: model_provider GTS schemas and instances registration
The system SHALL register all `model_provider`-owned GTS schemas and well-known instances during module startup.

The gateway module SHALL register the following schema IDs (types) in `types_registry`:
- `gts.x.genai.provider.vendor.v1~`
- `gts.x.genai.provider.kind.v1~`
- `gts.x.genai.provider.api_style.v1~`
- `gts.x.genai.provider.cap.v1~`
- `gts.x.genai.provider.feature.v1~`

Each schema SHALL define an object with:
- `id: GtsInstanceId`
- `display_name: string`
- `description?: string`

Well-known instances SHALL be stored as JSON arrays under `modules/model_provider/.../gts/` and registered during gateway `init()` (configuration phase), for example:
- `gts/gts.x.genai.provider.kind.v1~.instances.json`

#### Scenario: Startup registration
- **GIVEN** `types_registry` is in configuration mode
- **WHEN** `model_provider` gateway `init()` runs
- **THEN** it registers all schemas listed above
- **AND** it loads and registers all well-known instance lists from its `gts/` folder

### Requirement: model_provider persistence model
The system SHALL persist the following anonymous objects (UUID IDs) in the `model_provider` database.

#### Database tables (logical)
- `genai_model_provider`
  - `id uuid pk`
  - `tenant_id uuid not null` (always tenant-owned)
  - `available_to_children bool not null` (default: false, given provider is available to tenant children)
  - `vendor_gts_id text not null` (schema: `gts.x.genai.provider.vendor.v1~` instance id)
  - `kind_gts_id text not null` (schema: `gts.x.genai.provider.kind.v1~` instance id)
  - `base_url text not null`
  - `config_params jsonb not null` (gateway config overrides)
  - `parameters jsonb null` (provider-specific parameters)
  - `description text null`
  - `created_at timestamptz not null`
  - `updated_at timestamptz not null`

- `genai_model_provider_tag`
  - `id uuid pk`
  - `tenant_id uuid not null`
  - `provider_id uuid not null fk genai_model_provider(id)`
  - `tag text not null`

- `genai_model_provider_endpoint`
  - `id uuid pk`
  - `provider_id uuid not null fk genai_model_provider(id)`
  - `endpoint_type_gts_id text not null` (schema: `gts.x.genai.provider.api_style.v1~` instance id)
  - `base_url text not null`
  - `healthcheck_url text null`

- `genai_model_provider_endpoint_capability`
  - `id uuid pk`
  - `endpoint_id uuid not null fk genai_model_provider_endpoint(id)`
  - `capability_gts_id text not null` (schema: `gts.x.genai.provider.cap.v1~` instance id)

- `genai_model_provider_endpoint_feature`
  - `id uuid pk`
  - `endpoint_id uuid not null fk genai_model_provider_endpoint(id)`
  - `feature_gts_id text not null` (schema: `gts.x.genai.provider.feature.v1~` instance id)

- `genai_model_provider_provisioned_endpoint`
  - `id uuid pk`
  - `tenant_id uuid not null`
  - `available_to_children bool not null` (default: false, given endpoint is available to tenant children)
  - `endpoint_id uuid not null fk genai_model_provider_endpoint(id)`
  - `enabled bool not null`
  - `created_at timestamptz not null`
  - `updated_at timestamptz not null`

- `genai_model_provider_provisioned_endpoint_tag`
  - `id uuid pk`
  - `tenant_id uuid not null`
  - `available_to_children bool not null` (default: false, given endpoint is available to tenant children)
  - `provisioned_endpoint_id uuid not null fk genai_model_provider_provisioned_endpoint(id)`
  - `tag text not null`

#### Scenario: Persist provider and endpoint
- **WHEN** a tenant creates a provider instance and endpoint
- **THEN** the gateway stores rows in `genai_model_provider` and `genai_model_provider_endpoint`
- **AND** capabilities/features are stored in join tables

### Requirement: model_provider gateway API (Rust-native)
The system SHALL expose a Rust-native API (ClientHub) for managing providers and endpoints.

The gateway SDK SHALL define a `ModelProviderApi` trait where all methods accept `&SecurityCtx`.

The API SHALL support at least:
- Create/get/list/delete provider instances.
- Create/get/list/delete endpoints bound to provider instances.
- Upsert tenant provisioning (enable/disable endpoint for tenant).

Also, some of the providers to be created and registered using static JSON files with GTS identifiers and provider charatecteristics. THe following known providers must be defined
statically: openai, anthropic, gemini, deepseek, qwen, ollama, local lm studio, vllm, huggingface, openrouter.

The following GTS schemas and instances must be defined, and the JSON files must be placed in the `gts/` folder:
- `gts.x.genai.provider.vendor.v1~`
- `gts.x.genai.provider.kind.v1~`
- `gts.x.genai.provider.api_style.v1~`
- `gts.x.genai.provider.cap.v1~`
- `gts.x.genai.provider.feature.v1~`

#### Scenario: Rust-native call path
- **WHEN** another module calls `ModelProviderApi::list_endpoints(ctx, ...)`
- **THEN** the gateway enforces tenant isolation using `SecurityCtx`
- **AND** returns transport-agnostic SDK models

### Requirement: model_provider REST API
The system SHALL expose a REST API for managing providers and endpoints.

The REST API SHALL include:
- `GET/POST /model-provider/v1/providers`
- `GET/PATCH/DELETE /model-provider/v1/providers/{providerId}`
- `GET/POST /model-provider/v1/endpoints`
- `GET/PATCH/DELETE /model-provider/v1/endpoints/{endpointId}`
- `GET/PUT /model-provider/v1/provisioned-endpoints`

All endpoints SHALL use RFC-9457 Problem Details for errors.

#### Scenario: REST create provider
- **WHEN** a client POSTs to `/model-provider/v1/providers`
- **THEN** the gateway validates referenced GTS IDs exist in `types_registry`
- **AND** responds `201` with the created provider

### Requirement: model_provider plugin interface
The system SHALL define a plugin interface for provider connectivity.

Each plugin instance SHALL be represented in `types_registry` as a `BaseModkitPluginV1<...>` instance with:
- `id` (GTS instance id)
- `vendor` string
- `priority` integer (lower wins)
- `properties` describing supported `vendor_gts_id`, `endpoint_type_gts_id`, and supported capabilities/features.

The gateway SHALL select an eligible plugin on demand based on:
- Provider vendor/kind
- Endpoint type
- Capability/feature requirements
- Priority

#### Scenario: Plugin selection
- **GIVEN** multiple eligible plugins are registered
- **WHEN** the gateway resolves a plugin for an endpoint
- **THEN** it selects the eligible plugin with the lowest priority

### Requirement: model_provider client resolution API
The system SHALL provide a client resolution API for retrieving provider client instances by GTS ID.

The gateway SDK SHALL define methods:
- `resolve_provider_client(ctx: &SecurityCtx, provider_gts_id: &str) -> Result<Box<dyn ProviderClient>, Error>`
- `resolve_provider_client_by_id(ctx: &SecurityCtx, provider_id: Uuid) -> Result<Box<dyn ProviderClient>, Error>`

The API SHALL:
- Resolve the appropriate plugin based on provider vendor/kind/endpoint type
- Return a typed client instance for invoking provider APIs
- Enforce tenant isolation via SecurityCtx
- Support caching of client instances for performance

#### Scenario: Resolve client by GTS ID
- **WHEN** caller invokes `resolve_provider_client(ctx, "gts.x.genai.provider.vendor.v1~x.openai._.vendor.v1")`
- **THEN** gateway looks up provider configuration for tenant
- **AND** selects appropriate plugin (OpenAI plugin)
- **AND** returns typed client instance implementing `ProviderClient` trait

#### Scenario: Resolve client by provider UUID
- **WHEN** caller invokes `resolve_provider_client_by_id(ctx, provider_uuid)`
- **THEN** gateway loads provider from database
- **AND** resolves plugin based on vendor_gts_id and kind_gts_id
- **AND** returns configured client instance

#### Scenario: Client resolution failure
- **WHEN** no eligible plugin found for requested provider
- **THEN** returns error with type `urn:<module GTS type>:error:<error GTS type>`
- **AND** includes available plugins in error detail

### Requirement: model_provider hierarchical tenant access
The system SHALL support hierarchical tenant access using `available_to_children` flag.

When `available_to_children = true`:
- Provider is accessible to tenant and all descendant tenants
- Provisioned endpoints are accessible to descendant tenants
- Tags are inherited by descendant tenants

#### Scenario: Parent tenant creates provider available to children
- **GIVEN** parent tenant P creates provider with `available_to_children = true`
- **WHEN** child tenant C lists available providers
- **THEN** provider created by P appears in C's list
- **AND** C can provision endpoints from that provider

#### Scenario: Child tenant cannot modify parent's provider
- **GIVEN** parent tenant P created provider with `available_to_children = true`
- **WHEN** child tenant C attempts to update that provider
- **THEN** gateway returns 403 Forbidden
- **AND** only P can modify the provider

#### Scenario: Provisioned endpoint inheritance
- **GIVEN** parent tenant P provisions endpoint E with `available_to_children = true`
- **WHEN** child tenant C queries provisioned endpoints
- **THEN** endpoint E is available to C
- **AND** C can use the endpoint without re-provisioning

#### Scenario: Tag inheritance
- **GIVEN** parent tenant P tags provisioned endpoint with `available_to_children = true`
- **WHEN** child tenant C searches by tag
- **THEN** C discovers endpoints tagged by P
- **AND** tags are read-only for C

#### Scenario: Revoke child access
- **WHEN** parent tenant P sets `available_to_children = false` on provider
- **THEN** child tenants lose access immediately
- **AND** existing child usage is terminated gracefully

### Requirement: model_provider audit logging
The system SHALL maintain an audit log for all provider operations.

#### Database table: model_provider_audit_log
- `id uuid pk`
- `tenant_id uuid not null`
- `user_id uuid null`
- `operation text not null` (enum: 'create_provider', 'update_provider', 'delete_provider', 'provision_endpoint', 'invoke_model', 'resolve_client')
- `provider_id uuid null`
- `endpoint_id uuid null`
- `status text not null` (enum: 'success', 'failure')
- `error_code text null`
- `trace_id text null`
- `context_metadata jsonb null`
- `timestamp timestamptz not null`

**Indexes:**
- `(tenant_id, timestamp desc)` for tenant audit queries
- `(provider_id, timestamp desc)` for provider-specific audit trail
- `(operation, timestamp desc)` for operation-specific queries

NOTE: table parititioning and archiving and retention policies are not specified in this document. It's a subject for future improvements.

#### Scenario: Audit provider creation
- **WHEN** tenant creates a new provider
- **THEN** logs (tenant_id, user_id, 'create_provider', provider_id, 'success', timestamp)
- **AND** includes OpenTelemetry trace_id

#### Scenario: Audit model invocation
- **WHEN** client invokes model via resolved provider client
- **THEN** logs invocation with (operation='invoke_model', provider_id, status, duration)
- **AND** captures error details on failure

### Requirement: model_provider RBAC model
The system SHALL enforce role-based access control for provider operations.

#### Database table: model_provider_role_assignment
- `id uuid pk`
- `tenant_id uuid not null`
- `principal_id uuid not null`
- `principal_type text not null` (enum: 'user', 'service_account')
- `role text not null` (enum: 'provider.admin', 'provider.writer', 'provider.reader', 'provider.invoker')
- `scope text null` (optional scope to specific provider or vendor)
- `created_at timestamptz not null`

**Indexes:**
- `(tenant_id, principal_id)` for fast role lookup

NOTE: `model_provider_role_assignment` is out of scope for this document. It's a subject for external access control module.

#### Role definitions:
- `gts.x.core.idp.role.v1~x.model_provider.provider.admin.v1`: Full CRUD on providers, endpoints, provisioning
- `gts.x.core.idp.role.v1~x.model_provider.provider.writer.v1`: Create/update providers and endpoints (no delete)
- `gts.x.core.idp.role.v1~x.model_provider.provider.reader.v1`: Read-only access to providers and endpoints
- `gts.x.core.idp.role.v1~x.model_provider.provider.invoker.v1`: Can invoke models but not manage providers

#### Scenario: Enforce role at gateway
- **GIVEN** user U has role `gts.x.core.idp.role.v1~x.model_provider.provider.reader.v1`
- **WHEN** U attempts `DELETE /model-provider/v1/providers/{id}`
- **THEN** gateway checks SecurityCtx roles
- **AND** returns 403 Forbidden with RFC 9457 Problem Detail

#### Scenario: Scoped role to vendor
- **GIVEN** user U has role `gts.x.core.idp.role.v1~x.model_provider.provider.writer.v1` scoped to vendor='gts.x.genai.provider.vendor.v1~x.openai._.vendor.v1'
- **WHEN** U attempts to create an Anthropic provider
- **THEN** gateway denies with 403 Forbidden

### Requirement: model_provider observability
The system SHALL expose Prometheus-compatible metrics.

Required metrics (using `metrics` crate):
- `model_provider.providers.created` (counter, labels: tenant_id, vendor)
- `model_provider.providers.total` (gauge, labels: tenant_id, vendor)
- `model_provider.endpoints.provisioned` (counter, labels: tenant_id, endpoint_type)
- `model_provider.invocations` (counter, labels: tenant_id, provider_id, status)
- `model_provider.invocation.duration` (histogram, labels: provider_id, capability)
- `model_provider.errors` (counter, labels: error_type, provider_id)
- `model_provider.client_cache.hit_rate` (gauge)
- `model_provider.plugin.resolution.duration` (histogram, labels: plugin_id)

Implementation SHALL use `metrics` and `metrics-exporter-prometheus` crates.

#### Scenario: Metrics collection
- **WHEN** provider operation completes
- **THEN** system increments appropriate counters
- **AND** records latency histogram
- **AND** metrics are exposed at GET /metrics endpoint

### Requirement: model_provider health checks
The system SHALL expose health check endpoints.

REST endpoints:
- `GET /model-provider/v1/health` - Liveness probe
- `GET /model-provider/v1/ready` - Readiness probe (checks plugin availability)
- `GET /model-provider/v1/providers/{providerId}/health` - Provider-specific health check

Health check endpoints access MUST be protected by role `gts.x.core.idp.role.v1~x.model_provider.provider.health_check.v1` assigned.

#### Scenario: Provider health check
- **WHEN** `GET /model-provider/v1/providers/{providerId}/health` is called
- **AND** access is protected by role `gts.x.core.idp.role.v1~x.model_provider.provider.health_check.v1`
- **THEN** system invokes provider's healthcheck_url endpoint
- **AND** returns aggregated health status

#### Scenario: Readiness check
- **WHEN** `GET /model-provider/v1/ready` is called
- **AND** access is protected by role `gts.x.core.idp.role.v1~x.model_provider.provider.health_check.v1`
- **THEN** checks that at least one plugin is available
- **AND** returns 200 OK if system is ready

### Requirement: model_provider tracing integration
The system SHALL integrate with OpenTelemetry for distributed tracing.

All operations SHALL create spans with attributes:
- `tenant_id`
- `user_id` (if available)
- `provider_id`
- `endpoint_id`
- `vendor`
- `operation`
- `plugin_id`

Implementation SHALL use `tracing` and `tracing-opentelemetry` crates.

#### Scenario: Trace model invocation
- **WHEN** model invocation is performed
- **THEN** system creates OpenTelemetry span with all relevant attributes
- **AND** span is linked to parent trace context
- **AND** errors are recorded as span events

### Requirement: model_provider error taxonomy
The system SHALL define comprehensive error types using RFC 9457 Problem Details.

Error types SHALL include:
- `PluginUnavailable`: No plugin available for provider
- `ProviderNotFound`: Provider UUID does not exist
- `EndpointNotProvisioned`: Endpoint not enabled for tenant
- `QuotaExceeded`: Tenant provider limit reached
- `InvalidVendor`: Unknown vendor GTS ID
- `InvocationFailed`: Model invocation failed
- `RateLimitExceeded`: Invocation rate limit hit
- `Forbidden`: Access denied

#### Scenario: Plugin unavailable error
- **WHEN** no plugin available for requested provider vendor
- **THEN** gateway returns 503 Service Unavailable
- **AND** includes Problem Detail with type `urn:<model provider GTS id>:error:<error GTS id>`
- **AND** lists required plugin characteristics

#### Scenario: Quota exceeded error
- **WHEN** tenant attempts to create provider beyond quota
- **THEN** gateway returns 429 Too Many Requests
- **AND** includes current quota and usage in error detail

### Requirement: configurable per-tenant defaults

There must be per-module defaults configurable via global config file and also per-tenant defaults
stored in database. If per-tenant defaults are not set, then global per-module defaults are used.

### Requirement: model_provider resource limits
The system SHALL enforce resource limits per tenant.

#### Database table: model_provider_tenant_quota
- `id uuid pk`
- `tenant_id uuid not null unique`
- `max_providers int not null default 50`
- `max_endpoints_per_provider int not null default 10`
- `max_provisioned_endpoints int not null default 100`
- `max_invocations_per_min int not null default 1000`

#### Scenario: Enforce provider count quota
- **GIVEN** tenant T has max_providers = 50 and current count = 50
- **WHEN** T attempts to create provider 51
- **THEN** gateway returns 429 Too Many Requests
- **AND** includes Problem Detail with quota information

#### Scenario: Enforce invocation rate limit
- **GIVEN** tenant T has max_invocations_per_min = 1000
- **WHEN** T exceeds 1000 invocations in current minute
- **THEN** gateway returns 429 Too Many Requests
- **AND** includes Retry-After header

### Requirement: model_provider rate limiting
The system SHALL enforce rate limits per tenant using token bucket algorithm.

Implementation SHOULD use `governor` crate.

Rate limits SHALL be configurable per tenant (default: 1000 invocations/min).

#### Scenario: Rate limit enforcement
- **GIVEN** tenant T has rate limit 1000 invocations/min
- **WHEN** T exceeds limit
- **THEN** gateway returns 429 Too Many Requests
- **AND** includes Retry-After header with seconds until reset

#### Scenario: Per-provider rate limiting
- **GIVEN** specific provider has per-provider limit configured
- **WHEN** tenant exceeds provider-specific limit
- **THEN** only that provider is rate limited
- **AND** other providers continue normally

### Requirement: model_provider circuit breaker
The system SHALL implement circuit breaker per provider to prevent cascade failures.

#### Extended table: genai_model_provider
- Add: `circuit_breaker_threshold int not null default 5`
- Add: `circuit_breaker_timeout_sec int not null default 30`
- Add: `circuit_breaker_success_threshold int not null default 2`

Circuit breaker configuration:
- **Failure threshold:** 5 consecutive failures (configurable per provider)
- **Timeout:** 30 seconds (half-open state)
- **Success threshold:** 2 successes to close
- **Configurable:** module config file parameters can override defaults

Implementation SHOULD use `failsafe` or `tower` crate.

#### Scenario: Circuit breaker opens
- **GIVEN** provider P fails 5 consecutive invocations
- **WHEN** circuit breaker opens
- **THEN** subsequent invocations to P fail-fast (503)
- **AND** after 30s, circuit enters half-open state

#### Scenario: Circuit breaker recovery
- **GIVEN** circuit breaker is half-open
- **WHEN** 2 consecutive invocations succeed
- **THEN** circuit breaker closes
- **AND** normal operation resumes

### Requirement: model_provider retry logic
The system SHALL implement exponential backoff retry for transient failures.

Retry policy:
- **Transient errors:** Network timeout, 502/503/504 from provider
- **Max retries:** 3
- **Backoff:** 100ms, 200ms, 400ms
- **Non-retryable:** 400 Bad Request, 401 Unauthorized, 403 Forbidden, 404 Not Found

Implementation SHOULD use `tower` crate retry middleware.

#### Scenario: Retry transient provider error
- **WHEN** provider returns 503 Service Unavailable
- **THEN** gateway retries after 100ms
- **AND** if still failing, retries after 200ms, then 400ms
- **AND** if all retries exhausted, returns 503 to client

### Requirement: model_provider client caching
The system SHALL cache resolved provider client instances for performance.

Implementation SHOULD use in-memory cache with:
- **Max capacity:** 1,000 client instances
- **TTL:** 300 seconds (5 minutes)
- **Cache key:** `(tenant_id, provider_id, plugin_id)`

#### Scenario: Client cache hit
- **WHEN** client resolution requested for cached provider
- **THEN** returns cached client instance (no plugin resolution)
- **AND** records cache hit metric

#### Scenario: Client cache eviction
- **WHEN** provider configuration is updated
- **THEN** invalidate cached client for that provider
- **AND** next resolution creates fresh client

### Requirement: model_provider OData support
The system SHALL implement OData query capabilities per ModKit conventions.

REST endpoints SHALL support:
- `$skip` and `$top` for pagination
- `$filter` for filtering
- `$orderby` for sorting
- `$select` for field projection

Implementation SHALL use `modkit::api::OperationBuilder` OData support.

#### Scenario: Paginated provider list
- **WHEN** `GET /model-provider/v1/providers?$skip=20&$top=10`
- **THEN** returns providers 21-30
- **AND** includes `@odata.nextLink` for pagination

#### Scenario: Filter by vendor
- **WHEN** `GET /model-provider/v1/providers?$filter=vendor_gts_id eq 'gts.x.genai.provider.vendor.v1~openai'`
- **THEN** returns only OpenAI providers

#### Scenario: Filter inherited providers
- **WHEN** `GET /model-provider/v1/providers?$filter=available_to_children eq true`
- **THEN** returns providers shared by parent tenants

### Requirement: model_provider timeout configuration
The system SHALL support configurable timeouts for provider invocations.

#### Extended table: genai_model_provider_endpoint
- Add: `connection_timeout_ms int not null default 5000` (5s to establish connection)
- Add: `request_timeout_ms int not null default 30000` (30s total request duration)
- Add: `idle_timeout_ms int not null default 60000` (60s idle connection reuse)

These settings are overriable by OAGW per-tenant, per-endpoint and per-link defaults.

#### Scenario: Connection timeout
- **WHEN** connection to provider takes > 5s (configured in endpoint)
- **THEN** gateway aborts connection attempt
- **AND** returns 504 Gateway Timeout to client

#### Scenario: Request timeout
- **WHEN** model invocation takes > 30s (configured in endpoint)
- **THEN** gateway cancels request
- **AND** returns 504 Gateway Timeout to client

### Requirement: model_provider SLOs (Service Level Objectives)
The system SHOULD target:
- **Availability:** 99.9% (excluding planned maintenance and provider downtime)
- **Latency:**
  - p50: < 100ms for provider resolution
  - p95: < 200ms for provider resolution
  - p99: < 500ms for client resolution with cold cache
- **Error rate:** < 0.1% (excluding provider errors and client errors)

#### Scenario: SLO monitoring
- **WHEN** operations team reviews SLO dashboard
- **THEN** metrics show p95 latency, error rate, and uptime
- **AND** alerts trigger if SLO thresholds are breached

### Requirement: model_provider plugin trait definition
The system SHALL define a Rust trait for provider plugins.

```rust
#[async_trait]
pub trait ModelProviderPlugin: Send + Sync {
    /// Plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Check if plugin can handle this provider configuration
    fn can_handle(&self, vendor_gts_id: &str, kind_gts_id: &str, endpoint_type_gts_id: &str) -> bool;

    /// Create a provider client instance
    async fn create_client(
        &self,
        ctx: &SecurityCtx,
        provider: &ProviderConfig,
    ) -> Result<Box<dyn ProviderClient>, Error>;

    /// Health check
    async fn health_check(&self, ctx: &SecurityCtx) -> Result<HealthStatus, Error>;
}

#[async_trait]
pub trait ProviderClient: Send + Sync {
    /// Invoke model with given parameters
    async fn invoke(
        &self,
        ctx: &SecurityCtx,
        capability: &str,
        request: InvocationRequest,
    ) -> Result<InvocationResponse, Error>;

    /// Stream model response
    async fn invoke_stream(
        &self,
        ctx: &SecurityCtx,
        capability: &str,
        request: InvocationRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<StreamChunk, Error>>>>, Error>;
}
```

#### Scenario: Plugin registration
- **WHEN** plugin module `init()` runs
- **THEN** plugin registers instance in `types_registry` with metadata
- **AND** registers scoped client in `ClientHub` for `ModelProviderPlugin` trait

#### Scenario: Client invocation
- **WHEN** caller invokes `client.invoke(ctx, "chat_completion", request)`
- **THEN** plugin-specific client executes provider API call
- **AND** normalizes response to common format
- **AND** records metrics and tracing spans

### Requirement: model_provider request/response normalization
The system SHALL normalize provider-specific requests and responses to common formats.

Common request format:
```rust
pub struct InvocationRequest {
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: bool,
    pub tools: Option<Vec<ToolDefinition>>,
    pub extra_params: HashMap<String, serde_json::Value>,
}
```

Common response format:
```rust
pub struct InvocationResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
    pub finish_reason: FinishReason,
}
```

#### Scenario: Normalize OpenAI request
- **WHEN** caller invokes OpenAI provider with common request format
- **THEN** plugin transforms to OpenAI-specific format
- **AND** handles OpenAI-specific parameters in `extra_params`

#### Scenario: Normalize Anthropic response
- **WHEN** Anthropic provider returns response
- **THEN** plugin transforms to common response format
- **AND** maps Anthropic-specific fields to common schema
