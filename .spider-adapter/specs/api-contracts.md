# API Contracts (REST, OpenAPI, OData, gRPC)

**Version**: 1.0  
**Purpose**: Capture how Hyperspot defines and validates API surfaces (REST/OpenAPI and gRPC), including conventions enforced by lints.  
**Scope**: HTTP APIs exposed by the gateway and gRPC services/clients.  

## REST + OpenAPI

- OpenAPI is generated and stored as `docs/api/api.json` (see `make openapi`).
- REST uses Axum and ModKit patterns; errors should use `Problem` (RFC-9457).

## OData query options

- `$select` is supported for field projection; prefer using existing helpers to apply projection consistently.

## gRPC

- gRPC uses `tonic` + `prost`.
- CI installs `protoc` as part of the build/test pipeline.

## Lint-enforced conventions

- REST endpoints should be versioned (Dylint rule category `DE08xx`).
- Prefer OData extension methods where applicable (Dylint `DE0802`).

## Validation Criteria

- [ ] Public REST endpoints have clear versioning and are represented in OpenAPI.
- [ ] `$select` projection behavior matches `docs/ODATA_SELECT.md`.
- [ ] gRPC changes compile in CI (requires protoc).
- [ ] REST conventions lints remain passing.

## Examples

✅ Valid:
- Add an endpoint under a versioned path and update OpenAPI annotations.
- Use `$select` helpers for projection rather than ad-hoc JSON filtering.

❌ Invalid:
- Add an unversioned REST route.
- Hand-roll projection logic that diverges from shared helpers.

---

**Source**: `Makefile` (`openapi`), `docs/api/api.json`, `docs/ODATA_SELECT.md`, `Cargo.toml` (axum/utoipa/tonic), `dylint_lints/README.md`.  
**Last Updated**: 2026-02-05

