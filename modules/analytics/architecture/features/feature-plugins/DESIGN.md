# Feature: Plugins

**Status**: NOT_STARTED  
**Feature Slug**: `feature-plugins`

---

## A. Feature Context

### Overview

Plugin management system for query adapters and datasource plugins. Provides runtime plugin registration, lifecycle management, and contract format adapters for integrating with external data sources.

**Purpose**: Extend Analytics platform with pluggable query adapters and custom processing logic.

**Scope**:
- Plugin registration and discovery
- Plugin metadata storage (DB tables)
- Query adapter management (OData, REST, Prometheus, Elasticsearch, etc.)
- Datasource plugin lifecycle (enable/disable/update)
- Plugin configuration storage
- Plugin security and sandboxing
- Contract format adapters (3rd-party to native)
- Plugin health monitoring
- Plugin API endpoints
- Auto-registration of queries and datasources

**Out of Scope**:
- Query execution logic - handled by feature-query-execution
- Query type registration - handled by feature-query-definitions
- Datasource configuration - handled by feature-datasources

### GTS Types

This feature **does not own GTS types** - plugins auto-register queries/datasources as GTS entities.

**Uses types from**:
- `gts://gts.hypernetix.hyperspot.ax.query.v1~*` - Auto-registered query definitions
- `gts://gts.hypernetix.hyperspot.ax.datasource.v1~*` - Auto-registered datasources

### OpenAPI Endpoints

From `architecture/openapi/v1/api.yaml`:
- `GET /api/analytics/v1/plugins` - List all plugins
- `GET /api/analytics/v1/plugins/{id}` - Get plugin details
- `GET /api/analytics/v1/plugins/{id}/health` - Plugin health check
- `POST /api/analytics/v1/plugins/{id}/reload` - Reload plugin config
- `PUT /api/analytics/v1/plugins/{id}/enable` - Enable plugin
- `PUT /api/analytics/v1/plugins/{id}/disable` - Disable plugin
- `DELETE /api/analytics/v1/plugins/{id}` - Unregister plugin

### Actors

**Human Actors** (from Overall Design):
- **Platform Admin** - Manages plugin lifecycle
- **Plugin Developer** - Develops and deploys plugins

**System Actors**:
- **Plugin Manager** - Loads and manages plugins
- **Plugin Loader** - Discovers and initializes plugins
- **Health Monitor** - Monitors plugin health
- **Query Executor** - Invokes plugins for query execution

**Service Roles** (from OpenAPI):
- `analytics:plugins:read` - View plugins
- `analytics:plugins:write` - Manage plugins
- `analytics:plugins:admin` - Full plugin control

---

## B. Actor Flows

### Flow 1: Plugin Developer Deploys Plugin

**Actor**: Plugin Developer  
**Trigger**: New plugin ready for deployment  
**Goal**: Load plugin into platform

**Steps**:
1. Write plugin code (implement QueryPlugin trait)
2. Place in plugins directory (e.g., `/plugins/prometheus-adapter/`)
3. Update service configuration with plugin metadata
4. Reload service config (restart or hot-reload)
5. Plugin loads and auto-registers queries as GTS entities
6. Queries become available via `/gts` endpoint

**Configuration Example**:
```yaml
plugins:
  - id: prometheus-adapter
    enabled: true
    path: /plugins/prometheus-adapter
    config:
      base_url: https://prometheus.example.com
      timeout: 30s
    auto_register:
      queries: true
      datasources: true
```

---

### Flow 2: Platform Admin Monitors Plugin Health

**Actor**: Platform Admin  
**Trigger**: Need to verify plugin operational status  
**Goal**: Check plugin health and performance

**API Interaction**:
```
GET /api/analytics/v1/plugins/{plugin-id}/health

→ Returns:
{
  "status": "healthy",
  "plugin_id": "prometheus-adapter",
  "version": "1.0.0",
  "uptime": "72h15m",
  "queries_executed": 15234,
  "error_rate": 0.02,
  "dependencies": [
    {"name": "Prometheus API", "status": "healthy"}
  ]
}
```

---

### Flow 3: Query Executor Invokes Plugin

**Actor**: Query Executor (System)  
**Trigger**: Query execution request  
**Goal**: Execute query via plugin adapter

**Steps**:
1. Resolve query definition (contains `adapter_id`)
2. Load plugin by adapter_id
3. Call `plugin.before_jwt_sign()` hook
4. Call `plugin.execute_query()` with SecurityCtx and ODataParams
5. Plugin translates to target format (e.g., PromQL)
6. Plugin executes external API call
7. Plugin converts response to OData format
8. Return standardized result

---

### Flow 4: Platform Admin Disables Plugin

**Actor**: Platform Admin  
**Trigger**: Plugin maintenance or issues  
**Goal**: Gracefully disable plugin

**API Interaction**:
```
PUT /api/analytics/v1/plugins/{plugin-id}/disable

→ Plugin marked inactive
→ Existing queries continue (graceful degradation)
→ No new queries accepted
→ Cleanup performed
```

---

## C. Algorithms

### Service Algorithm 1: Plugin Loading and Discovery

**Purpose**: Discover and load plugins from filesystem

**Steps**:

1. Initialize empty plugins list
2. Scan plugins directory for .so files
3. **FOR EACH** .so file:
   1. Load plugin metadata
   2. Validate plugin interface
   3. **IF** valid:
      1. Load plugin into memory
      2. Initialize with config
      3. Add to plugins list
4. **RETURN** loaded plugins
        // 4. Initialize plugin
        let mut plugin = create_plugin_instance(&metadata)?;
        plugin.init(config.clone())?;
        
        // 5. Auto-register entities
        if config.auto_register_queries {
            auto_register_queries(&plugin, &metadata)?;
        }
        
        // 6. Health check
        if plugin.health_check().is_healthy() {
            plugins.push(plugin);
        }
    }
    
    Ok(plugins)
}
```

---

### Service Algorithm 2: Contract Format Translation

**Purpose**: Translate OData queries to target format

**Example: Prometheus Adapter**:

1. Parse OData filter conditions
2. Extract metric name from filters
3. Build PromQL label matchers
4. Extract time range
5. Construct PromQL query: `metric{labels}[timerange]`
6. **RETURN** PromQL string
    }
    
    fn convert_response(&self, response: PrometheusResponse) -> Result<QueryResult> {
        // Transform Prometheus timeseries to OData format
        let odata_result = ODataResponse {
            context: "$metadata#Metrics",
            count: response.data.result.len(),
            value: response.data.result.into_iter()
                .map(|ts| convert_timeseries(ts))
                .collect()
        };
        Ok(odata_result)
    }
}
```

---

## D. States

### Plugin Lifecycle States

```
[Unloaded] → (Load) → [Loaded]
[Loaded] → (Init) → [Initialized]
[Initialized] → (Enable) → [Active]
[Active] → (Disable) → [Inactive]
[Inactive] → (Enable) → [Active]
[Active] → (Unload) → [Unloaded]
```

**State Descriptions**:
- **Unloaded**: Plugin not yet loaded from filesystem
- **Loaded**: Plugin code loaded, not yet initialized
- **Initialized**: Plugin initialized with config
- **Active**: Plugin accepting and executing queries
- **Inactive**: Plugin disabled, graceful degradation mode

---

## E. Technical Details

### Plugin Architecture

Plugins are optional extensions that run inside the platform. They are **independent** from datasource registration.

**Plugin Capabilities:**

### 1. Local Datasource Implementation
- Implement datasource logic directly in platform
- No external API calls needed
- Used when datasource is tightly coupled to platform

### 2. Contract Adapters
- Convert 3rd-party API formats to native contract
- Reusable across multiple datasources
- Examples: Prometheus adapter, Elasticsearch adapter, OpenTelemetry adapter

### 3. Custom Processing
- Add preprocessing/postprocessing logic
- Handle complex authentication flows
- Implement caching strategies

---

## Plugin Loading Flow

Plugins are loaded via filesystem and configuration, not API calls:

1. **Write plugin code** - Implement plugin interface in Rust
2. **Place in plugins directory** - e.g., `/plugins/prometheus-adapter/`
3. **Enable in service config** - Update configuration with plugin metadata
4. **Reload service config** - Restart service or hot-reload configuration
5. **Plugin loads** - Queries auto-register as full GTS entities, endpoints become available

---

## Auto-registration

Plugin can optionally auto-register GTS entities on load:

### Queries (`auto_register_queries`)

- Plugin creates **one or more** full GTS Query entities
- Each Query gets proper **GTS identifier** (e.g., `gts.hypernetix.hyperspot.ax.query.v1~prometheus.monitoring._.server_metrics.v1`)
- Query registered with all required fields:
  - `category`, `name`, `description`
  - `api_endpoint` (plugin-internal endpoint)
  - `capabilities_id` (defines OData query capabilities)
  - `returns_schema_id` (defines response schema)
  - `contract_format: "custom"` (since plugin uses adapter)
  - `adapter_id` (references this plugin's adapter)
- Queries are **discoverable via `/gts` endpoint** like any other registered query
- Queries can be **used in dashboards** like manually registered queries

### Datasources (`auto_register_datasources`)

- Plugin can optionally create **one or more** Datasource entities
- Each Datasource links to a Query via `query_id`
- Datasource includes `params` (OData parameters) and `render_options` (UI controls)
- Useful for providing pre-configured datasources with specific filters/parameters

### Validation

- Plugin declares entities it provides in config
- Service validates and registers them as GTS entities on startup
- If entity with same ID exists, plugin registration fails (conflict)
- Config format defined in plugin configuration YAML (see Plugin Configuration section)

---

## Plugin Interface

**Plugin Interface**: Plugins implement QueryPlugin trait with:
- `metadata()` - Return plugin information
- `init(config)` - Initialize with configuration
- `before_jwt_sign(ctx, claims)` - Modify JWT claims before signing
- `execute_query(...)` - Execute query and return results
- `health_check()` - Return plugin health status 
        ctx: &SecurityCtx, 
        query_id: &str, 
        params: &ODataParams
    ) -> Result<QueryResult>;
    
    /// Health check
    fn health_check(&self) -> HealthStatus;
    
    /// Cleanup on plugin unload
    fn shutdown(&mut self) -> Result<()>;
}

struct PluginMetadata {
    id: String,
    name: String,
    version: String,
    author: String,
    description: String,
    contract_format: ContractFormat,
}

enum ContractFormat {
    Native,           // Analytics native contract
    OData,           // OData v4
    Prometheus,      // Prometheus query format
    Elasticsearch,   // Elasticsearch DSL
    GraphQL,         // GraphQL queries
    Custom(String),  // Custom adapter
}

struct PluginConfig {
    settings: HashMap<String, Value>,
    auto_register_queries: bool,
    auto_register_datasources: bool,
}
```

---

## Contract Format Adapters

Adapters translate between external API formats and Analytics native contract.

### Built-in Adapters

**1. OData Adapter**
- Translates Analytics queries to OData v4
- Handles standard OData operations
- No custom translation needed

**2. Prometheus Adapter**
- Translates OData filters to PromQL
- Converts time ranges and aggregations
- Maps metric names and labels

**3. Elasticsearch Adapter**
- Translates OData to Elasticsearch DSL
- Handles full-text search mapping
- Converts aggregations and filters

**4. REST Adapter**
- Generic REST API adapter
- Configurable URL templates
- Parameter mapping rules

### Custom Adapter Development

**Example: Prometheus Adapter** implementation:
- Stores base_url for Prometheus API
- Translates OData $filter to PromQL
- Executes PromQL against Prometheus HTTP API
- Converts Prometheus response to OData format
        let url = format!("{}/api/v1/query", self.base_url);
        let response = http_client.post(&url)
            .query(&[("query", query)])
            .send()?;
        
        // Convert Prometheus response to Analytics format
        self.convert_response(response)
    }
    
    fn convert_response(&self, response: PrometheusResponse) -> Result<QueryResult> {
        // Transform Prometheus data to OData format
        // Apply schema validation
        // Return standardized result
    }
}
```

---

## Plugin Lifecycle

### Registration
1. Plugin placed in filesystem
2. Service scans plugins directory
3. Plugin config validated
4. Plugin loaded and initialized
5. Auto-registered entities created in GTS Registry

### Activation
1. Plugin marked as active in config
2. Health check performed
3. Queries become available via `/queries` endpoint
4. Plugin ready for query execution

### Deactivation
1. Plugin marked as inactive
2. Existing queries continue to work (graceful degradation)
3. No new queries accepted
4. Plugin cleanup performed

### Update
1. New plugin version placed in filesystem
2. Service detects change
3. Old version gracefully shutdown
4. New version loaded and initialized
5. Queries migrated to new version

### Removal
1. Plugin removed from config
2. Existing queries marked as deprecated
3. Plugin shutdown and unloaded
4. Optional: soft-delete registered queries

---

## Plugin Configuration

**Configuration File Example:**

```yaml
plugins:
  - id: prometheus-adapter
    enabled: true
    path: /plugins/prometheus-adapter
    config:
      base_url: https://prometheus.example.com
      timeout: 30s
      retry_count: 3
    auto_register:
      queries: true
      datasources: true
    queries:
      - id: gts.hypernetix.hyperspot.ax.query.v1~prometheus.monitoring._.cpu_usage.v1
        name: CPU Usage Metrics
        category: monitoring
        api_endpoint: /metrics/cpu
        capabilities_id: gts.hypernetix.hyperspot.ax.query_capabilities.v1~prometheus.default.v1
        returns_schema_id: gts.hypernetix.hyperspot.ax.schema.v1~prometheus.timeseries.v1~
```

---

## Plugin Security

### Sandboxing
- Plugins run in isolated contexts
- Limited filesystem access
- Network access controlled
- Resource limits enforced (CPU, memory, execution time)

### Permissions
- Read: Can read query definitions
- Write: Can register queries/datasources
- Execute: Can execute queries
- Admin: Can modify plugin configuration

### API Key Management
- Plugins can store encrypted API keys
- Keys never exposed in API responses
- Rotation support
- Audit logging for key usage

---

## Plugin Health Monitoring

**Health Check Endpoint:**
```
GET /api/analytics/v1/plugins/{plugin-id}/health
```

**Health Status:**
```json
{
  "status": "healthy",
  "plugin_id": "prometheus-adapter",
  "version": "1.0.0",
  "uptime": "72h15m",
  "queries_executed": 15234,
  "error_rate": 0.02,
  "last_error": "2024-01-08T10:15:30Z",
  "dependencies": [
    {
      "name": "Prometheus API",
      "status": "healthy",
      "response_time": "45ms"
    }
  ]
}
```

**Health Status Types:**
- `healthy` - Plugin operational
- `degraded` - Plugin working but with issues
- `unhealthy` - Plugin failing, circuit breaker may open
- `unknown` - Health check failed

---

## Plugin Metadata Storage

**Database Schema:**

```sql
CREATE TABLE plugins (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    author VARCHAR(255),
    description TEXT,
    contract_format VARCHAR(50),
    enabled BOOLEAN DEFAULT true,
    config JSONB,
    registered_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ,
    registered_by VARCHAR(255),
    updated_by VARCHAR(255)
);

CREATE TABLE plugin_health (
    plugin_id VARCHAR(255) REFERENCES plugins(id),
    checked_at TIMESTAMPTZ NOT NULL,
    status VARCHAR(50),
    response_time_ms INTEGER,
    error_message TEXT,
    metrics JSONB
);

CREATE INDEX idx_plugins_enabled ON plugins(enabled);
CREATE INDEX idx_plugin_health_plugin_id ON plugin_health(plugin_id);
CREATE INDEX idx_plugin_health_checked_at ON plugin_health(checked_at DESC);
```

---

## Key Differences

**Datasource registration vs Plugin loading:**

| Aspect | Datasource Registration | Plugin Loading |
|--------|------------------------|----------------|
| **Method** | API call to register external/plugin endpoint | Config-based code extension (filesystem + config reload) |
| **Purpose** | Register data query endpoints | Extend platform capabilities |
| **Runtime** | Dynamic via API | Static via config + restart |
| **Scope** | Single query/datasource | Multiple queries + adapters |
| **Use Case** | External APIs, SaaS integrations | Platform extensions, protocol adapters |

---

### Access Control

**SecurityCtx Enforcement**:
- All plugin operations require authenticated admin
- Plugin metadata access restricted to authorized users
- Plugin configuration contains encrypted secrets
- Plugin execution inherits SecurityCtx from query

**Permission Checks**:
- Plugin management: Requires `analytics:plugins:admin`
- Plugin health check: Requires `analytics:plugins:read`
- Plugin enable/disable: Requires `analytics:plugins:write`

---

### Database Operations

**Tables**:
- `plugins` - Plugin metadata and configuration
- `plugin_health` - Health check history

**Indexes**:
- `idx_plugins_enabled` - Fast active plugin lookup
- `idx_plugin_health_plugin_id` - Health history by plugin
- `idx_plugin_health_checked_at` - Recent health checks

**Queries**:
```sql
-- List active plugins
SELECT * FROM plugins WHERE enabled = true;

-- Get plugin with recent health
SELECT p.*, ph.status, ph.checked_at
FROM plugins p
LEFT JOIN plugin_health ph ON p.id = ph.plugin_id
WHERE p.id = $1
ORDER BY ph.checked_at DESC LIMIT 1;
```

---

### Error Handling

**Common Errors**:
- **404 Not Found**: Plugin not found
- **400 Bad Request**: Invalid plugin configuration
- **503 Service Unavailable**: Plugin unhealthy
- **409 Conflict**: Plugin ID already exists
- **500 Internal Server Error**: Plugin execution failure

**Error Response Format (RFC 7807)**:
```json
{
  "type": "https://example.com/problems/plugin-unhealthy",
  "title": "Plugin Unhealthy",
  "status": 503,
  "detail": "Plugin 'prometheus-adapter' health check failed",
  "instance": "/api/analytics/v1/plugins/prometheus-adapter"
}
```

---

## F. Validation & Implementation

### Testing Scenarios

**Unit Tests**:
- Contract format translation (OData → PromQL, Elasticsearch DSL)
- Plugin metadata validation
- Health status aggregation
- Config encryption/decryption
- Error handling and fallbacks

**Integration Tests**:
- Plugin loading from filesystem
- Auto-registration of queries
- Plugin lifecycle (enable/disable/reload)
- Health monitoring
- Multi-plugin coordination

**Security Tests**:
- Plugin sandboxing verification
- API key encryption
- Resource limit enforcement
- Permission validation
- Audit logging

**Performance Tests**:
- Plugin initialization time (< 1s)
- Query translation overhead (< 10ms)
- Health check response time (< 100ms)
- Concurrent plugin execution

**Edge Cases**:
1. Plugin with circular dependencies
2. Plugin crash during execution
3. External API timeout
4. Malformed plugin configuration
5. Plugin version conflict
6. Auto-registration conflict (duplicate IDs)

---

### OpenSpec Changes Plan

#### Change 001: Plugin Interface Definition
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/plugins/traits.rs`
- **Description**: Define QueryPlugin trait and PluginMetadata structs
- **Dependencies**: None (foundational)
- **Effort**: 1 hour (AI agent)
- **Validation**: Trait compilation, example plugin

#### Change 002: Database Schema
- **Type**: database
- **Files**: 
  - `modules/analytics/migrations/001_create_plugins.sql`
- **Description**: Create plugins and plugin_health tables
- **Dependencies**: Change 001
- **Effort**: 0.5 hours (AI agent)
- **Validation**: Migration tests, constraint validation

#### Change 003: Plugin Loader
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/plugins/loader.rs`
- **Description**: Filesystem scanning, plugin discovery, initialization
- **Dependencies**: Change 001
- **Effort**: 2 hours (AI agent)
- **Validation**: Unit tests with mock plugins

#### Change 004: Contract Adapters
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/plugins/adapters/prometheus.rs`
  - `modules/analytics/src/domain/plugins/adapters/elasticsearch.rs`
  - `modules/analytics/src/domain/plugins/adapters/odata.rs`
- **Description**: Built-in contract format adapters
- **Dependencies**: Change 001
- **Effort**: 4 hours (AI agent)
- **Validation**: Translation tests, integration tests

#### Change 005: Plugin Management API
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/api/rest/plugins/handlers.rs`
  - `modules/analytics/src/domain/plugins/service.rs`
- **Description**: Plugin CRUD, enable/disable, reload endpoints
- **Dependencies**: Change 002, Change 003
- **Effort**: 2 hours (AI agent)
- **Validation**: API tests, integration tests

#### Change 006: Health Monitoring
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/plugins/health.rs`
- **Description**: Health check scheduler, status aggregation
- **Dependencies**: Change 003
- **Effort**: 1.5 hours (AI agent)
- **Validation**: Health check tests, timing tests

#### Change 007: Auto-Registration Logic
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/plugins/auto_register.rs`
- **Description**: Auto-register queries and datasources as GTS entities
- **Dependencies**: Change 003, feature-gts-core
- **Effort**: 2 hours (AI agent)
- **Validation**: Registration tests, conflict detection

#### Change 008: Plugin Security
- **Type**: rust
- **Files**: 
  - `modules/analytics/src/domain/plugins/security.rs`
- **Description**: Sandboxing, resource limits, API key encryption
- **Dependencies**: Change 001
- **Effort**: 2 hours (AI agent)
- **Validation**: Security tests, penetration tests

#### Change 009: OpenAPI Specification
- **Type**: openapi
- **Files**: 
  - `architecture/openapi/v1/api.yaml`
- **Description**: Document plugin management endpoints
- **Dependencies**: All previous changes
- **Effort**: 0.5 hours (AI agent)
- **Validation**: Swagger validation

#### Change 010: Integration Testing Suite
- **Type**: rust (tests)
- **Files**: 
  - `tests/integration/plugins_test.rs`
- **Description**: End-to-end plugin lifecycle tests
- **Dependencies**: All previous changes
- **Effort**: 2 hours (AI agent)
- **Validation**: 100% scenario coverage

**Total Effort**: 17 hours (AI agent + OpenSpec)

---

## Dependencies

- **Depends On**: 
  - feature-gts-core (GTS entity registration)
- **Blocks**: 
  - feature-query-execution (plugins provide query adapters)

---

## References

- Overall Design: `architecture/DESIGN.md` Section 2 (Actors), Section 3 (System Capabilities)
- GTS Types: Query and datasource schemas (auto-registered by plugins)
- OpenAPI Spec: `architecture/openapi/v1/api.yaml` (plugin endpoints)
- Feature Manifest: `architecture/features/FEATURES.md` (feature-plugins entry)
