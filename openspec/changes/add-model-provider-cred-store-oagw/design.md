## Context
This change proposes specifications for three new modules:

- `model_provider`: manages GenAI model providers and their endpoints, plus tenant provisioning.
- `cred_store`: a generic secret storage gateway with a single active plugin (highest priority).
- `oagw`: outbound API gateway for invoking remote calls with pluggable protocol/auth/streaming implementations.

The system MUST use existing ModKit patterns:

- `types_registry` as authoritative registry for GTS schemas and well-known instances.
- Gateway+plugin architecture with scoped `ClientHub` registration.
- `types_registry` two-phase lifecycle: module `init()` registration in configuration mode, then `types_registry` `post_init()` switches to ready mode.

## Goals / Non-Goals
- Goals:
  - Provide complete domain model definitions and storage responsibilities.
  - Provide both Rust-native and REST gateway contracts.
  - Provide plugin contracts and selection algorithms.
  - Provide module startup procedure for schema + instance registration.
  - Provide OpenAPI 3.1 specifications (module-local) for code generation.

- Non-Goals:
  - Implement the modules.
  - Decide concrete authentication/authorization policy and RBAC model.
  - Standardize global tag taxonomy beyond what is required for the described relationships.

## Decisions
- Decision: Use TypesRegistry as the only storage for GTS schemas and well-known instances.
  - Gateway modules register schemas and well-known instances during `init()`.
  - Plugin modules register their plugin instance objects during `init()`.

- Decision: Keep plugin discovery lazy.
  - Rationale: matches `tenant_resolver` example and avoids race with `types_registry` ready switch.

- Decision: Persist only anonymous objects (UUID IDs) and tenant-scoped configuration in module DB tables.
  - Rationale: aligns with the request and keeps global taxonomies as GTS instances.

## Recommended building blocks (Rust crates)
- `modkit`, `modkit-security`, `modkit-db` (Secure ORM), `modkit-odata` (pagination), `modkit::TracedClient`.
- `types_registry_sdk::TypesRegistryApi` for GTS registration and discovery.
- `gts` / `gts_macros` for schema definitions (`struct_to_gts_schema`).
- `secrecy` for secret material handling in memory.
- `age` (or KMS/Vault SDKs) for envelope encryption in plugins.
- `reqwest` via `modkit::TracedClient` for outbound HTTP.
- `tonic` for gRPC client generation and streaming.
- `oauth2` for OAuth2 client credentials and token exchange.
- `jsonwebtoken` for JWT validation and parsing.
- `metrics` + `metrics-exporter-prometheus` for Prometheus metrics.
- `tracing` + `tracing-opentelemetry` for distributed tracing.
- `governor` for rate limiting.
- `tower` for retry middleware and service composition.
- `failsafe` for circuit breaker pattern.
- `moka` for high-performance in-memory caching (token cache, response cache).
- `ring` or `aws-lc-rs` for FIPS 140-2 validated cryptography (HIPAA compliance).
- Optional (if adopted): `rig` for provider integrations and tool calling abstractions. No `rig` dependency exists in the repo yet; adding it is a follow-up implementation decision.

## OpenAPI
HyperSpot generates a single OpenAPI document at runtime (owned by `api_ingress`), but each module contributes operations. For code generation, the following module-local OpenAPI 3.1 documents are specified.

## Module Startup Procedure (shared pattern)
All three modules MUST follow `types_registry` lifecycle constraints:

1. `types_registry` module `init()` runs first, registers core GTS base types.
2. Plugin modules `init()` run and register their plugin instances and scoped clients (no `list()` calls).
3. Gateway modules `init()` run and register:
   - their module-owned GTS schemas via `XxxV1::gts_schema_with_refs_as_string()`.
   - module-owned well-known instances loaded from `gts/<schema-id>.instances.json`.
4. After *all* modules finish `init()`, `types_registry` `post_init()` runs and switches to ready mode, validating all registered entities.
5. Gateways MUST perform plugin discovery lazily on first request (after ready mode), mirroring `tenant_resolver`.
