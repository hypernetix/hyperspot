# Architecture Patterns (ModKit + Modules)

**Version**: 1.0  
**Purpose**: Provide the “shape” rules for modules, layering, client contracts, and lifecycle so new code stays consistent.  
**Scope**: Modules (`modules/*`), shared libs (`libs/*`), and gateway patterns.  

## Module architecture (DDD-light)

- Prefer the canonical ModKit module layout (REST adapter, contract surface, domain logic, infra/storage).
- Use the SDK pattern: consumers depend on `*-sdk` (traits + transport-agnostic models/errors); the module crate implements the SDK trait and exposes clients via ClientHub.

## Gateway + plugins pattern

- A gateway module owns the HTTP server/router and OpenAPI.
- Plugins register themselves in discovery systems for runtime lookup; keep plugin boundaries strict.

## Validation Criteria

- [ ] New modules follow the `guidelines/NEW_MODULE.md` structure.
- [ ] Public cross-module APIs are defined in SDK crates, not in REST DTOs.
- [ ] REST types stay in REST layer; contract/domain stay transport-agnostic.
- [ ] Lifecycle is explicit for long-running tasks (cancellation-aware).

## Examples

✅ Valid:
- Implement `MyModuleClient` in a local client adapter and expose via ClientHub.

❌ Invalid:
- Use REST DTOs as the “contract” between modules.

---

**Source**: `docs/MODKIT_UNIFIED_SYSTEM.md`, `guidelines/NEW_MODULE.md`, `docs/ARCHITECTURE_MANIFEST.md`, `dylint_lints/README.md`.  
**Last Updated**: 2026-02-05

