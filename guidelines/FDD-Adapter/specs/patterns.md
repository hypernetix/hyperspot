# Architecture Patterns Specification

**Source**: Discovered from code architecture, docs/ARCHITECTURE_MANIFEST.md, and modkit system

---

## Module Architecture Pattern

**Pattern**: Modkit Plugin System with Strict Layer Separation

**When to use**: All feature modules in the project

**Structure**:
```
module-name/
├── module-name-contracts/    # Contract layer (pure domain types)
│   └── src/lib.rs            # GTS types, no HTTP/serde
├── module-name-sdk/          # SDK layer (client interfaces)
│   └── src/lib.rs            # Re-exports contracts, client traits
└── module-name/              # Main module
    └── src/
        ├── domain/           # Domain logic
        ├── api/              # API implementations
        │   ├── rest/         # REST endpoints + DTOs
        │   └── grpc/         # gRPC services + DTOs
        ├── core/             # Business logic
        └── infra/            # Infrastructure
```

**Rules**:
- Contract layer: Pure domain types, GTS structs only, NO serde, NO HTTP types
- Domain layer: Business logic, operates on contracts
- API layer: DTOs with serde, HTTP types allowed
- Infra layer: Database, external services

**Enforcement**: Custom dylint lints (de01_contract_layer, de02_api_layer)

---

## GTS Type Pattern

**Pattern**: Global Type System for Domain Model Validation

**When to use**: All domain types that cross module boundaries

**Implementation**:
```rust
// Contract layer - Pure domain type
pub struct UserProfile {
    pub id: String,
    pub name: String,
}

// GTS registration in types-registry
pub const USER_PROFILE_GTS_ID: &str = "gts.ainetx.hyperspot.users.user_profile.v1";
```

**Rules**:
- All identifiers MUST be lowercase with underscores
- Format: `gts.vendor.package.namespace.type.vMAJOR[.MINOR]`
- Register in `modules/types-registry/`
- Validate with GTS library

**Source**: docs/MODKIT_UNIFIED_SYSTEM.md, guidelines/GTS/

---

## DTO Isolation Pattern

**Pattern**: Data Transfer Objects isolated to API layer

**When to use**: REST and gRPC API implementations

**Implementation**:
```rust
// api/rest/dto.rs or api/grpc/dto.rs
#[derive(Serialize, Deserialize, ToSchema)]
pub struct UserProfileDto {
    pub id: String,
    pub name: String,
}

impl From<UserProfile> for UserProfileDto {
    fn from(profile: UserProfile) -> Self {
        UserProfileDto {
            id: profile.id,
            name: profile.name,
        }
    }
}
```

**Rules**:
- DTOs ONLY in `api/rest/dto.rs` or `api/grpc/dto.rs`
- DTOs NOT referenced outside api/ directory
- DTOs MUST have serde derives
- Convert between domain types and DTOs at API boundary

**Enforcement**: dylint lints (de0201, de0202, de0203)

---

## Problem Details Error Pattern

**Pattern**: RFC 7807 Problem Details for HTTP errors

**When to use**: All API error responses

**Implementation**:
```rust
use modkit::Problem;

// In API handler
fn handle_request() -> Result<Response, Problem> {
    if invalid {
        return Err(Problem::bad_request()
            .with_title("Invalid Request")
            .with_detail("Field 'name' is required"));
    }
    Ok(response)
}
```

**Rules**:
- Use `Problem` type from modkit
- Include type, title, status, detail
- GTS-based error type identifiers
- Never expose internal errors to clients

**Source**: docs/MODKIT_UNIFIED_SYSTEM.md

---

## OData Query Pattern

**Pattern**: OData query parameters for filtering and pagination

**When to use**: List endpoints that need filtering/paging

**Implementation**:
```rust
use modkit_odata::{ODataQuery, Page, PageInfo};

// In API handler
async fn list_users(Query(odata): Query<ODataQuery>) -> Result<Json<Page<UserDto>>, Problem> {
    let users = db.query()
        .select(&odata.select)     // $select=id,name
        .filter(&odata.filter)      // $filter=age gt 18
        .order_by(&odata.orderby)   // $orderby=name asc
        .skip(odata.skip)           // $skip=10
        .top(odata.top)             // $top=20
        .execute()
        .await?;
    
    Ok(Json(Page::new(users, PageInfo { ... })))
}
```

**Rules**:
- Use modkit-odata for parsing
- Support `$select`, `$filter`, `$orderby`, `$top`, `$skip`
- Return Page<T> with PageInfo
- Validate field names against domain model

**Source**: docs/ODATA_SELECT.md, docs/MODKIT_UNIFIED_SYSTEM.md

---

## REST Module Integration Pattern (MANDATORY)

**Pattern**: ModKit RestfulModule + OperationBuilder

**When to use**: ALL REST modules (REQUIRED)

**⚠️ CRITICAL**: See `specs/modkit-rest-integration.md` for complete specification.

**Key requirements**:
- MUST implement `RestfulModule` trait
- MUST use `OperationBuilder` for ALL endpoints
- MUST extend passed router (NOT create new)
- MUST register through `api_ingress`
- FORBIDDEN: Direct axum routes, custom middleware, Router::new()

**Quick example**:
```rust
impl RestfulModule for MyModule {
    fn register_rest(&self, _ctx: &ModuleCtx, router: Router, openapi: &dyn OpenApiRegistry) -> Result<Router> {
        let router = OperationBuilder::get("/my-module/v1/resource")
            .operation_id("my_module.get")
            .require_auth(&Resource, &Action)
            .handler(handlers::get_resource)
            .json_response_with_schema::<Dto>(openapi, StatusCode::OK, "Success")
            .register(router, openapi);
        Ok(router)
    }
}
```

**Source**: 
- `specs/modkit-rest-integration.md` (MANDATORY reading)
- `examples/modkit/type_safe_api_builder.rs`
- `modules/file_parser/` (working example)

---

## Security Context Pattern

**Pattern**: SecurityCtx for authentication and authorization

**When to use**: Protected API endpoints

**Implementation**:
```rust
use modkit_security::SecurityCtx;
use modkit_auth::AuthDispatcher;

async fn protected_handler(
    Extension(sec_ctx): Extension<SecurityCtx>,
) -> Result<Response, Problem> {
    sec_ctx.require_permission("users.read")?;
    // ... handle request
}
```

**Rules**:
- Use AuthDispatcher for JWT validation
- Use SecurityCtx for permission checks
- Never manually parse JWT
- Fail closed on auth errors

**Source**: libs/modkit-auth/, libs/modkit-security/

---

## Modular Plugin Pattern

**Pattern**: Hot-reloadable modules with lifecycle management

**When to use**: All application modules

**Implementation**:
```rust
use modkit::{Module, ModuleContext};

pub struct MyModule;

impl Module for MyModule {
    fn init(&mut self, ctx: &ModuleContext) -> Result<()> {
        // Initialize module
    }
    
    fn shutdown(&mut self) -> Result<()> {
        // Cleanup
    }
}
```

**Rules**:
- Implement Module trait
- Register in module registry
- Support hot reload
- Isolated dependencies

**Source**: docs/MODKIT_UNIFIED_SYSTEM.md, examples/modkit/

---

## API Endpoint Versioning Pattern

**Pattern**: Version prefix in REST endpoints

**When to use**: All REST API endpoints

**Implementation**:
```rust
// Correct
router.route("/api/v1/users", get(list_users));

// Incorrect - will fail lint
router.route("/users", get(list_users));
```

**Rules**:
- Format: `/api/v<N>/<resource>`
- Version in path, not query/header
- Enforcement via de0801_api_endpoint_version lint

**Source**: dylint_lints/de08_rest_api_conventions/

---

## Mock Mode Pattern

**Pattern**: Comprehensive mock mode for development and testing

**When to use**: Development, testing, demos without infrastructure dependencies

**Implementation**:
```rust
// Service-side mock mode
// Run with: cargo run -- --mock-mode
// Or env: MOCK_MODE=true

// Mock datasource implementation
pub struct MockDatasource;

impl DatasourcePlugin for MockDatasource {
    fn execute_query(&self, query: &Query) -> Result<QueryResult> {
        // Return realistic mock data matching GTS schemas
        Ok(QueryResult {
            data: mock_data_for_query(query),
            metadata: mock_metadata(),
        })
    }
}
```

**Rules**:
- Service mock mode via `--mock-mode` flag or `MOCK_MODE=true` env var
- UI mock mode via `VITE_MOCK_MODE=true` build-time config
- Mock responses MUST follow same GTS contracts as real implementations
- Mock data should be realistic and match production schemas
- All services and UI components should support mock mode

**Benefits**:
- Faster development cycles without database/plugin dependencies
- Reliable E2E tests with deterministic mock data
- Demo environments without production data access
- Offline development capability

**Source**: modules/analytics/architecture/ADR.md (ADR-0004)

---

## Module Category Pattern

**Pattern**: Three-tier module architecture (Generic/Gateway/Worker)

**When to use**: Organizing modules based on their role in the system

**Module Types**:

### 1. Generic Module
- Independent module with own public API
- Responsible for specific domain
- Example: `api_ingress`, `directory_service`

**Structure**:
```
generic-module/
├── api/              # REST/gRPC endpoints
├── domain/           # Business logic
├── infra/            # Database, external APIs
└── gateways/         # Optional: clients to other modules
```

### 2. Gateway Module
- Exposes unified public API
- Routes requests to worker modules
- Example: `file_parser`, `search_gateway`

**Structure**:
```
gateway-module/
├── api/              # Public REST API
├── domain/           # Routing logic
└── gateways/         # Worker connectors
```

### 3. Worker Module
- Implements specific functionality
- Called by gateway modules
- No public REST API
- Example: `file_parser_tika`, `qdrant_search`

**Structure**:
```
worker-module/
├── domain/           # Implementation logic
├── infra/            # External service integration
└── gateways/         # Clients to external services
```

**Rules**:
- Gateway modules route based on context (tenant ID, request params)
- Worker modules implement contracts defined by their gateway
- Workers do not expose their own public REST API
- All module communication through versioned interfaces

**Source**: docs/ARCHITECTURE_MANIFEST.md

---

## Source References

- docs/ARCHITECTURE_MANIFEST.md
- docs/MODKIT_UNIFIED_SYSTEM.md
- docs/ODATA_SELECT.md
- examples/modkit/
- dylint_lints/ (architectural lint rules)
- modules/analytics/architecture/ADR.md
