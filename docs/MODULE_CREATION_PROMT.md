
# Prompt v4 — Generate Hyperspot/ModKit module from OpenAPI (strict, DB + Client + Repo)

```
ROLE
You are an expert Rust code generator for Hyperspot modules built on ModKit.

PRIMARY GOAL
From an OpenAPI 3.x spec, generate a Hyperspot module that:
(1) STRICTLY matches the style of the given reference module,
(2) Implements ModKit "rest" + "db" capabilities with DbModule::migrate(),
(3) Wires typed config, Service, Repository (SeaORM) and native Client trait,
(4) Registers ALL routes in ONE function via OperationBuilder,
(5) Uses Problem (implements IntoResponse) + centralized error mapping,
(6) Keeps contract models clean (no serde).

INPUTS
OPENAPI_PATH    = modules/ecommerce/openapi.yaml
REF_MODULE_PATH = modules/ecommerce
OUT_MODULE_PATH = modules/ecommerce
OUT_CRATE_NAME  = ecommerce

ABSOLUTE RULES — HYPERSPOT/MODKIT
- Hyperspot already has the single REST host: `api_ingress`. DO NOT implement rest_host or server startup.
- Your module declares: capabilities = [db, rest].
- Module macro MUST include: deps = ["db"], client = "<path to native trait in contract::client>".
- Use OperationBuilder for every route; centralized OpenAPI registry is provided by api_ingress.
- Handlers are THIN; business logic is in domain::service; repositories in infra::storage.
- contract models are transport-agnostic: NO serde derives; only `#[derive(Debug, Clone, PartialEq, Eq)]`.

STEP 0 — REFERENCE STYLE SNAPSHOT (MANDATORY, APPLY VERBATIM)
Scan REF_MODULE_PATH and mirror EXACTLY:
- DI in handlers: use `axum::Extension(std::sync::Arc<Service>)` and, in routes, attach it once:
  `router = router.layer(Extension(service.clone()));`
- OperationBuilder call order (match reference): describe params → `.handler(...)` → success/error responses → `.register(...)`.
- Handler returns:
  - GET/200 with body → `Result<Json<T>, Problem>`
  - POST/201 with body → `Result<(StatusCode, Json<T>), Problem>`
  - 204 → `Result<StatusCode, Problem>` (NO body)
- Imports/visibilities: public exports in lib.rs exactly like reference; internal modules `#[doc(hidden)]`.
- Cargo deps/feature flags (serde/uuid/chrono features etc.) — mirror the reference.
- operation_id naming style — replicate reference (e.g., `<crate>.<resource>.<action>`).
- Test pattern — `Router::oneshot` smoke tests.

STEP 1 — PROJECT LAYOUT (UNDER OUT_MODULE_PATH)
src/
  lib.rs
  module.rs
  contract/
    mod.rs
    client.rs
    model.rs
    error.rs
  domain/
    mod.rs
    model.rs
    error.rs
    service.rs
    repository.rs       # TRAITS live here
  infra/
    storage/
      entity.rs
      mapper.rs
      repositories.rs   # SeaORM implementations live here
      migrations/       # SeaORM migrator skeleton
  api/
    rest/
      dto.rs
      handlers.rs
      routes.rs
      error.rs
Cargo.toml

STEP 2 — CONTRACT LAYER (clean, no serde)
- In `contract/model.rs`, define public models used BETWEEN modules. Derive:
  `#[derive(Debug, Clone, PartialEq, Eq)]` ONLY (no serde, no utoipa).
- In `contract/error.rs`, define domain error enum for inter-module use (similar to):
```

\#\[derive(Debug, Clone, PartialEq, Eq)]
pub enum <Domain>Error {
NotFound{ ... },
Conflict{ ... },
Validation{ message: String },
Internal,
}

```
- In `contract/client.rs`, define the NATIVE client trait that other modules will call (NOT a REST client).
Name it `<PascalCaseModule>Api` and declare async methods mirroring domain service operations (accept/return contract models).
NO HTTP. NO serde.
Example:
```

\#\[async\_trait::async\_trait]
pub trait EcommerceApi: Send + Sync {
async fn get\_product(\&self, id: uuid::Uuid) -> Result\<model::Product, error::EcommerceError>;
// ...
}

```

STEP 3 — DOMAIN LAYER
- `domain/model.rs`: internal rich domain types if needed (can reuse contract models where appropriate).
- `domain/error.rs`: domain error (map to/from contract error as needed).
- `domain/repository.rs`: define repository TRAITS (one or several) consumed by Service.
Example:
```

\#\[async\_trait::async\_trait]
pub trait ProductsRepository: Send + Sync {
async fn find\_by\_id(\&self, id: uuid::Uuid) -> anyhow::Result\<Option[contract::model::Product](contract::model::Product)>;
async fn list(\&self, filter: ProductsFilter) -> anyhow::Result\<Vec[contract::model::Product](contract::model::Product)>;
}

```
- `domain/service.rs`: struct `Service` with generic repo deps, constructed in module `init()`.
Service methods return `Result<contract::model::..., contract::error::<Domain>Error>`.

STEP 4 — INFRA/STORAGE (SeaORM Repository Implementations)
- `infra/storage/repositories.rs`: implement SeaORM repos with generic connection `C: sea_orm::ConnectionTrait + Send + Sync`.
```

pub struct SeaOrmProductsRepository<C> { conn: C }
impl<C> SeaOrmProductsRepository<C> where C: ConnectionTrait + Send + Sync {
pub fn new(conn: C) -> Self { Self{ conn } }
}
\#\[async\_trait::async\_trait]
impl<C> ProductsRepository for SeaOrmProductsRepository<C>
where C: ConnectionTrait + Send + Sync + 'static {
async fn find\_by\_id(\&self, id: Uuid) -> anyhow::Result\<Option<Product>> {
let found = Entity::find\_by\_id(id).one(\&self.conn).await.context("find\_by\_id failed")?;
Ok(found.map(Into::into))
}
// ...
}

```
- `infra/storage/entity.rs`, `mapper.rs` — mappers (map DB ↔ contract models) with `From`/`Into`.
- `infra/storage/migrations/` — create a SeaORM migrator skeleton.

STEP 5 — CONFIG + INIT + DB CAPABILITY
- `src/config.rs`: define a typed config with safe defaults:
```

\#\[derive(serde::Deserialize, Debug, Clone)]
\#\[serde(deny\_unknown\_fields)]
pub struct Config {
pub feature\_flags: Option\<Vec<String>>,
// add module-specific fields later
}
impl Default for Config { fn default() -> Self { Self { feature\_flags: None } } }

```
- In `module.rs`, macro MUST include:
```

\#\[modkit::module(
name = "OUT\_CRATE\_NAME",
deps = \["db"],
capabilities = \[db, rest],
client = "contract::client::<PascalCaseModule>Api",
lifecycle(entry = "serve", stop\_timeout = "30s", await\_ready)
)]

```
- `impl Module::init` MUST:
1) Read typed config: `let cfg = ctx.module_config::<crate::config::Config>()?;`
2) Check DB presence: `let db = ctx.db().ok_or_else(|| anyhow::anyhow!("Database required"))?;`
3) Build repos with SeaORM conn: `let conn = db.sea(); let products_repo = Arc::new(SeaOrmProductsRepository::new(conn));`
4) Build `Service::new(...)` with repos, store it in `OnceCell<Arc<Service>>`.
5) Publish NATIVE client to ClientHub (NOT REST): convert `Arc<Service>` into `Arc<dyn contract::client::<...>Api>` via a small `gateways::local` adapter, then call the auto-generated `expose_<module>_client(ctx, &api)?;`

- `impl DbModule for ModuleType` MUST implement `migrate()`:
```

async fn migrate(\&self, db: \&db::DbHandle) -> anyhow::Result<()> {
infra::storage::migrations::Migrator::up(db.sea(), None).await?;
Ok(())
}

```

STEP 6 — REST (ONE entry function + safety checks)
- `api/rest/routes.rs`: define a SINGLE entry:
```

pub fn register\_routes(
mut router: axum::Router,
openapi: \&dyn modkit::api::OpenApiRegistry,
service: std::sync::Arc[domain::service::Service](domain::service::Service),
) -> anyhow::Result[axum::Router](axum::Router) { ... }

````
- In `impl RestfulModule::register_rest`:
- Check the service is initialized:
  ```
  let svc = self.service.get().ok_or_else(|| anyhow::anyhow!("Service not initialized"))?.clone();
  ```
- Call `register_routes(router, openapi, svc)` and return its result.

STEP 7 — OPENAPI → DTOs (api/rest/dto.rs)
- Map `components.schemas` to REST DTOs. Derive: `Serialize`, `Deserialize`, `Clone`, `Debug`, `utoipa::ToSchema`.
- Formats:
- string uuid → `uuid::Uuid`
- string date-time → `chrono::DateTime<chrono::Utc>`
- string date → `chrono::NaiveDate`
- integer int32 → `i32`; int64 → `i64`
- number float → `f32`; double → `f64`
- Provide `From` conversions between REST DTOs and `contract::model` (by value and by &ref). REST layer handles these conversions; domain & contract never depend on REST DTOs.

STEP 8 — HANDLERS (api/rest/handlers.rs)
- DI: use `Extension(Arc<Service>)` (NO Router state).
- Extractors:
- Path → `Path<T>` (struct if multiple).
- Query → `Query<TParams>`.
- Body (application/json) → `Json<ReqDto>`.
- Returns:
- GET/200 with body → `Result<Json<RespDto>, Problem>`
- POST/201 with body → `Result<(StatusCode, Json<RespDto>), Problem>`
- 204 → `Result<StatusCode, Problem>`
- Map domain errors via centralized `api/rest/error.rs`:
````

pub fn map\_domain\_error(e: \&contract::error::<...>Error, instance: \&str) -> Problem

```
- NEVER invent response DTOs when OpenAPI response has NO `content`. Return `StatusCode` only.

STEP 9 — ROUTES (OperationBuilder, one place)
- Register ALL endpoints inside `register_routes(...)`.
- Call order: describe → `.handler(handlers::...)` → success `.json_response(_with_schema)` or `.json_response` (no schema) → `.problem_response(...)` → `.register(router, openapi)`.
- operation_id: follow reference (e.g., `ecommerce.products.list`, `ecommerce.cart.add_item`, `ecommerce.auth.login`).
- For secured operations in OpenAPI, add `.problem_response(openapi, 401, "Unauthorized")`.
- After all routes: `router = router.layer(Extension(service.clone()));` → `Ok(router)`.

STEP 10 — ERROR MAPPING (api/rest/error.rs)
- Centralize RFC-9457 mapping with helpers:
```

\#\[derive(thiserror::Error, Debug, Clone)]
pub enum RestError { ... } // optional, if reference has it
pub fn from\_parts(status: StatusCode, code: \&str, title: \&str, detail: impl Into<String>, instance: \&str) -> Problem
pub fn map\_domain\_error(e: \&contract::error::<...>Error, instance: \&str) -> Problem

```
- Use `instance` = the request path; include meaningful `title/detail`.

STEP 11 — CARGO + LIB EXPORTS
- Clone deps & features from REF_MODULE_PATH/Cargo.toml (serde with derive; chrono+serde; uuid+serde; utoipa; axum; http; anyhow; async-trait; tokio; modkit; tracing; sea-orm).
- lib.rs:
```

pub mod contract;
pub use contract::{client, error, model};

pub mod module;
pub use module::<TypeName>;

\#\[doc(hidden)]
pub mod api;
\#\[doc(hidden)]
pub mod domain;
\#\[doc(hidden)]
pub mod infra;
\#\[doc(hidden)]
pub mod config;
\#\[doc(hidden)]
pub mod gateways; // if you place local client adapter here

```

PITFALL GUARDS (MUST PASS)
- (1.1) capabilities include `db`; implement `DbModule::migrate()`.
- (1.2) read typed config in `init()`; provide defaults.
- (1.3) check DB presence in `init()` (error if missing).
- (1.4) build domain Service in `init()`; pass SeaORM connection into repositories.
- (1.5) in `register_rest`, fail if service not initialized.
- (1.6) module macro must include `client = "contract::client::<...>Api"`.
- (1.7) contract/client.rs defines NATIVE trait (no REST); expose it in `init()` to ClientHub via local adapter.
- (1.8) ALL routes are built in ONE `register_routes()` function.
- (1.9) domain error enum follows the explicit pattern; use `thiserror::Error` derive for REST error equivalents where needed.
- (1.10) repository pattern: TRAIT in `domain::repository`, SeaORM implementation in `infra::storage::repositories` with generic `C: ConnectionTrait + Send + Sync`.
- (1.11) NO serde in `contract` models or errors.

OUTPUT
- Overwrite/create files under OUT_MODULE_PATH.
- Print:
- endpoints generated,
- which endpoints returned only `StatusCode` (no content in spec),
- TODOs for ambiguous oneOf/anyOf or missing DB entities/migrations.
- Code MUST compile against reference’s deps/features and match its style.
```
