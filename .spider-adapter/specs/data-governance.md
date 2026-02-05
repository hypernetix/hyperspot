# Data Governance (Tenancy, Persistence, Migrations)

**Version**: 1.0  
**Purpose**: Keep data boundaries (tenant/resource), migrations, and persistence patterns consistent across modules.  
**Scope**: Modules that store or access persisted data.  

## Tenancy & access scoping

- Assume multi-tenancy concerns apply by default for user/tenant-facing data.
- Apply tenant/resource scoping through the secure ORM layer.

## Migrations

- Use `sea-orm-migration` in module infra layers (`infra/storage/migrations`) as per module guidelines.

## Data serialization

- Transport DTOs (serde) belong in REST layer; contract/domain models should remain transport-agnostic per layering rules.

## Validation Criteria

- [ ] Entities and queries have explicit tenant/resource scope behavior (or explicit “unrestricted” justification).
- [ ] Migrations are versioned and live with the owning module.
- [ ] Contract/domain layers remain free of REST-only concerns.

## Examples

✅ Valid:
- Create migrations under `modules/<name>/<name>/src/infra/storage/migrations/`.

❌ Invalid:
- Add a migration at repo root not associated with a module, without a clear shared-DB rationale.

---

**Source**: `docs/SECURE-ORM.md`, `guidelines/NEW_MODULE.md`, `docs/MODKIT_UNIFIED_SYSTEM.md`, `Cargo.toml` (sea-orm + migration), `clippy.toml`, `dylint_lints/README.md`.  
**Last Updated**: 2026-02-05

