# ModKit Architecture & Developer Guide

This folder contains the ModKit developer documentation, split by topic for focused reading and LLM retrieval.

## How to use this folder

- **For humans**: Browse the file that matches your task (e.g., module layout, DB access, OData, plugins).
- **For LLMs**: Start by reading this `README.md` only, then load the smallest set of sections/files needed for your task. Do not load unrelated docs.

## Task → Document routing

| Task / Goal | Primary file(s) to read | Related external docs |
|-------------|------------------------|----------------------|
| Adding a new module | `02_module_layout_and_sdk_pattern.md` | `guidelines/NEW_MODULE.md` |
| DB/persistence, SecureConn | `06_secure_orm_db_access.md` | |
| REST endpoint wiring, OperationBuilder | `04_rest_operation_builder.md` | |
| OData, $select, pagination, filtering | `07_odata_pagination_select_filter.md` | `docs/ODATA_SELECT.md`, `docs/ODATA_MACRO_MIGRATION.md` |
| ClientHub, inter-module clients | `03_clienthub_and_plugins.md` | |
| Plugins, scoped clients, GTS | `03_clienthub_and_plugins.md` | `docs/MODKIT_PLUGINS.md` |
| Errors, RFC-9457 Problem | `05_errors_rfc9457.md` | |
| Lifecycle, background tasks, cancellation | `08_lifecycle_stateful_tasks.md` | |
| Out-of-Process / gRPC / SDK pattern | `09_oop_grpc_sdk_pattern.md` | |
| Domain model macro, DDD enforcement | `02_module_layout_and_sdk_pattern.md` (§ Domain types) | `dylint_lints/de03_domain_layer/de0309_must_have_domain_model/README.md` |
| Quick checklists, templates | `10_checklists_and_templates.md` | |
| HTTP client (TLS, retries, timeouts, concurrency, OTel tracing, auth hook) | | `docs/adrs/modkit/0001-modkit-hyper-tower-http-client.md` |
| Authentication (inbound JWT/OIDC policies, outbound OAuth2 client-credentials) | | `docs/adrs/modkit/0002-modkit-auth-client-with-aliri.md` |

## Core invariants (apply everywhere)

- **SDK pattern is the public API**: Use `<module>-sdk` crate for traits, models, errors. Do not expose internals.
- **Secure-by-default DB access**: Use `SecureConn` + `AccessScope`. Modules cannot access raw database connections.
- **RFC-9457 errors everywhere**: Use `Problem` (implements `IntoResponse`). Do not use `ProblemResponse`.
- **Type-safe REST**: Use `OperationBuilder` with `.require_auth()` and `.standard_errors()`.
- **OData macros are in `modkit-odata-macros`**: Use `modkit_odata_macros::ODataFilterable`.
- **ClientHub registration**: `ctx.client_hub().register::<dyn MyModuleApi>(api)`; `ctx.client_hub().get::<dyn MyModuleApi>()?`.
- **Cancellation**: Pass `CancellationToken` to background tasks for cooperative shutdown.
- **Domain model enforcement**: All `struct`/`enum` in `domain/` must have `#[domain_model]` (`modkit_macros::domain_model`). CI lint DE0309 enforces this.
- **GTS schema**: Use `gts_schema_with_refs_as_string()` for faster, correct schema generation.

## File overview

- `01_overview.md` – What ModKit provides, core concepts, golden path.
- `02_module_layout_and_sdk_pattern.md` – Module directory layout, SDK crate, module crate, re-exports.
- `03_clienthub_and_plugins.md` – Typed ClientHub, in-process vs remote clients, scoped clients, GTS-based plugin discovery.
- `04_rest_operation_builder.md` – OperationBuilder usage, auth, error registration, SSE, content types.
- `05_errors_rfc9457.md` – Problem error type, From impls, handler patterns, OpenAPI error registration.
- `06_secure_orm_db_access.md` – SecureConn, SecurityContext, Scopable derive, raw access rules.
- `07_odata_pagination_select_filter.md` – OData $filter/$orderby/$select, pagination, macro usage, field projection.
- `08_lifecycle_stateful_tasks.md` – WithLifecycle, cancellation tokens, stateful module patterns.
- `09_oop_grpc_sdk_pattern.md` – Out-of-Process modules, gRPC, SDK pattern for OoP, client utilities.
- `10_checklists_and_templates.md` – Quick checklists per task, minimal code templates.

### Related ADRs

- `docs/adrs/modkit/0001-modkit-hyper-tower-http-client.md` – **modkit-http**: Hyper+Tower HTTP client with TLS, retries, timeouts, concurrency limiting, decompression, OTel tracing, and extensible auth layer hook.
- `docs/adrs/modkit/0002-modkit-auth-client-with-aliri.md` – **modkit-auth**: Inbound JWT/OIDC route-level policies and outbound OAuth2 client-credentials flow with automatic token refresh and `Authorization: Bearer` injection.
