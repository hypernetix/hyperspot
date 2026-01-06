# New Module Scaffold Spec

Overview:
- Goal: Provide a minimal, production-ready scaffold for a new Hyperspot module following guidelines/NEW_MODULE.md.
- Input: module name in snake_case (e.g., users_info, types_registry). The tool derives PascalCase for types and kebab-case for REST paths when needed.
- Output: Two crates under modules/: <name>-sdk and <name>, with DDD-light layout, REST-ready, inventory-registered, and ClientHub-local client wiring.

Naming conventions:
- Module snake_case: input (e.g., users_info).
- PascalCase type base: UsersInfo.
- SDK crate: modules/<name>-sdk, crate name "<name>-sdk".
- Module crate: modules/<name>, crate name "<name>".
- REST base path: "/<kebab-name>/v1" (snake_case converted to kebab-case, users_info -> users-info).
- Operation IDs: "<module>.<resource>.<action>".

Scaffold contents:
1) SDK crate (<name>-sdk)
- Cargo.toml: workspace-inherited version/edition/license/authors; deps: async-trait, thiserror, uuid, chrono, modkit-security, modkit-odata (optional).
- src/lib.rs: re-exports api/errors/models.
- src/api.rs: trait <PascalCase>Api: include a minimal "health" method signature:
  async fn health(&self, ctx: &modkit_security::SecurityCtx) -> Result<Health, <PascalCase>Error>;
- src/models.rs: Health struct (Debug, Clone, Eq, fields: status: String, version: Option<String>). No serde derives.
- src/errors.rs: <PascalCase>Error enum with variants: NotFound, Validation { message }, Internal.

2) Module crate (<name>)
- Cargo.toml: depends on <name>-sdk; workspace deps: anyhow, async-trait, tokio, tracing, inventory, serde, serde_json, utoipa, axum (macros), tower-http (timeout), futures, chrono (serde), uuid, arc-swap, thiserror; local deps: modkit, modkit-auth (axum_ext), modkit-odata (optional).
- src/lib.rs: re-export SDK types and module struct; expose module, local_client; mark api/config/domain/infra as doc(hidden).
- src/module.rs:
  - #[modkit::module(name = "<snake>", capabilities = [rest])]
  - struct <PascalCase> { service: arc_swap::ArcSwapOption<Service> }
  - impl Default; impl Module::init to build Service, store in ArcSwapOption; create local client implementing SDK trait; register in ClientHub: ctx.client_hub().register::<dyn <PascalCase>Api>(api);
  - impl RestfulModule::register_rest: fail if service not initialized; call routes::register_routes and layer Extension(service).
- src/config.rs: typed config with serde(deny_unknown_fields); defaults for future use.
- src/local_client.rs: adapter implements <name>-sdk::api::<PascalCase>Api and delegates to domain Service.
- src/domain/
  - error.rs: DomainError enum; impl From<DomainError> for <PascalCase>Error.
  - service.rs: Service with health(&SecurityCtx) -> Result<Health, DomainError>.
  - ports.rs: EventPublisher<E> trait placeholder (optional).
  - repo.rs: repository trait placeholder (optional, empty for minimal template).
- src/api/rest/
  - dto.rs: HealthDto (serde + ToSchema) and From conversions.
  - handlers.rs: GET handler for /<base>/health using modkit::api::prelude::*.
  - routes.rs: registers GET /<kebab>/v1/health via OperationBuilder; adds standard errors; attaches Extension(service).
  - error.rs: impl From<DomainError> for modkit::api::problem::Problem (RFC-9457 mapping).

3) Tests (minimal)
- Module crate tests/: basic handler smoke test with Router::oneshot for GET /health (StatusCode::OK).

4) Server wiring (instructions only; NOT applied by the tool):
- apps/hyperspot-server/Cargo.toml: add dependency line: <snake> = { path = "../../modules/<snake>" }.
- apps/hyperspot-server/src/registered_modules.rs: add use <snake> as _; entry to ensure inventory linking.

5) Workspace membership (instructions only; NOT applied by the tool):
- Root Cargo.toml [workspace].members: add modules/<name>-sdk and modules/<name>.

Non-goals for minimal scaffold:
- Database layer and migrations (SeaORM) — excluded in the minimal template.
- SSE, plugins, gRPC/OoP — excluded.

Acceptance expectations after implementation:
- cargo check --workspace compiles both crates.
- Route GET /<kebab>/v1/health appears in OpenAPI and returns JSON HealthDto.
- ClientHub resolves local client via hub.get::<dyn <PascalCase>Api>().
