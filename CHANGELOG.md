# Changelog

All notable changes to this repository are documented in this file.

This file follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

release-plz updates this file in the Release PR.

## [Unreleased]

## [0.1.5](https://github.com/hypernetix/hyperspot/compare/cf-system-sdks-v0.1.4...cf-system-sdks-v0.1.5) - 2026-02-02

### Other

- updated the following local packages: cf-system-sdk-directory

## [0.1.5](https://github.com/hypernetix/hyperspot/compare/cf-system-sdk-directory-v0.1.4...cf-system-sdk-directory-v0.1.5) - 2026-02-02

### Other

- updated the following local packages: cf-modkit-transport-grpc

### Breaking Changes

- **DatabaseCapability refactored**: Modules no longer receive raw database connections.
  - `DatabaseCapability::migrate(&self, db: &DbHandle)` removed
  - New signature: `DatabaseCapability::migrations(&self) -> Vec<Box<dyn MigrationTrait>>`
  - Runtime collects migrations and executes them with privileged connection
  - Each module gets its own migration history table (`modkit_migrations__<prefix>__<hash8>`)

- **Raw database access removed from public API**:
  - `DbHandle::sea()` is now `pub(crate)` (was `pub` with `insecure-escape` feature)
  - `DbHandle::sqlx_postgres()`, `sqlx_mysql()`, `sqlx_sqlite()` are now `pub(crate)`
  - The `insecure-escape` feature has been removed entirely

- **SecureConn no longer exposes raw connection**:
  - `SecureConn::conn()` removed from public API
  - Secure query/ops execute via `&SecureConn` (e.g. `.one(&secure_conn)`, `.all(&secure_conn)`, `.exec(&secure_conn)`)

### Added

- **Migration runner** (`modkit_db::migration_runner`):
  - `run_migrations_for_module(db, module_name, migrations)` - runtime entry point
  - `run_migrations_for_testing(db, migrations)` - test helper
  - `get_pending_migrations(db, module_name, migrations)` - check pending migrations
  - Per-module migration tables prevent cross-module conflicts
  - Deterministic ordering by migration name
  - Idempotent execution (skips already applied migrations)
  - Duplicate migration names rejected
  - Best-effort atomicity (transaction around `up()` + version record)

### Migration Guide for Module Authors

Before (old API):
```rust
#[async_trait]
impl DatabaseCapability for MyModule {
    async fn migrate(&self, db: &modkit_db::DbHandle) -> anyhow::Result<()> {
        let conn = db.sea();  // Direct access - REMOVED
        Migrator::up(&conn, None).await?;
        Ok(())
    }
}
```

After (new API):
```rust
impl DatabaseCapability for MyModule {
    fn migrations(&self) -> Vec<Box<dyn sea_orm_migration::MigrationTrait>> {
        use sea_orm_migration::MigratorTrait;
        crate::infra::storage::migrations::Migrator::migrations()
    }
}
```

## [0.1.4](https://github.com/hypernetix/hyperspot/compare/cf-system-sdks-v0.1.3...cf-system-sdks-v0.1.4) - 2026-01-28

### Other

- updated the following local packages: cf-system-sdk-directory

## [0.1.4](https://github.com/hypernetix/hyperspot/compare/cf-system-sdk-directory-v0.1.3...cf-system-sdk-directory-v0.1.4) - 2026-01-28

### Other

- updated the following local packages: cf-modkit-transport-grpc

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-types-registry-v0.1.1) - 2026-01-27

### Other

- Merge branch 'main' into snake-case-enforcement (by @nanoandrew4) - #247
- *(modules)* Add module descriptions, missed README.MD and some crate meta (by @MikeFalcon77)
- release (by @github-actions[bot]) - #317
- Normalize modules (types registry and simple user settings) paths to snake_case (by @alizid10) - #314

### Contributors

* @nanoandrew4
* @MikeFalcon77
* @github-actions[bot]
* @alizid10

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-tenant-resolver-gw-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta (by @MikeFalcon77)
- release (by @github-actions[bot]) - #317
- Normalize modules (types registry and simple user settings) paths to snake_case (by @alizid10) - #314

### Contributors

* @MikeFalcon77
* @github-actions[bot]
* @alizid10

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-static-tr-plugin-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta (by @MikeFalcon77)
- release (by @github-actions[bot]) - #317
- Normalize modules (types registry and simple user settings) paths to snake_case (by @alizid10) - #314

### Contributors

* @MikeFalcon77
* @github-actions[bot]
* @alizid10

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-single-tenant-tr-plugin-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta (by @MikeFalcon77)
- release (by @github-actions[bot]) - #317
- Normalize modules (types registry and simple user settings) paths to snake_case (by @alizid10) - #314

### Contributors

* @MikeFalcon77
* @github-actions[bot]
* @alizid10

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-tenant-resolver-sdk-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta (by @MikeFalcon77)
- release (by @github-actions[bot]) - #317

### Contributors

* @MikeFalcon77
* @github-actions[bot]

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-nodes-registry-v0.1.1) - 2026-01-27

### Other

- Merge branch 'main' into snake-case-enforcement (by @nanoandrew4) - #247
- Merge branch 'main' into snake-case-enforcement (by @nanoandrew4) - #247

### Contributors

* @nanoandrew4

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-nodes-registry-sdk-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta (by @MikeFalcon77)
- release (by @github-actions[bot]) - #317

### Contributors

* @MikeFalcon77
* @github-actions[bot]

## [0.1.4](https://github.com/hypernetix/hyperspot/compare/cf-modkit-node-info-v0.1.3...cf-modkit-node-info-v0.1.4) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta (by @MikeFalcon77)

### Contributors

* @MikeFalcon77

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-module-orchestrator-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta (by @MikeFalcon77)
- release (by @github-actions[bot]) - #317

### Contributors

* @MikeFalcon77
* @github-actions[bot]

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-file-parser-v0.1.1) - 2026-01-27

### Other

- Merge branch 'main' into snake-case-enforcement (by @nanoandrew4) - #247
- Merge branch 'main' into snake-case-enforcement (by @nanoandrew4) - #247

### Contributors

* @nanoandrew4

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-types-registry-sdk-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta (by @MikeFalcon77)
- release (by @github-actions[bot]) - #317

### Contributors

* @MikeFalcon77
* @github-actions[bot]

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-grpc-hub-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta (by @MikeFalcon77)
- release (by @github-actions[bot]) - #317

### Contributors

* @MikeFalcon77
* @github-actions[bot]

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-api-gateway-v0.1.1) - 2026-01-27

### Other

- chore(deps): (by @MikeFalcon77) - #325
- Merge branch 'main' into snake-case-enforcement (by @nanoandrew4) - #247
- Merge branch 'main' into snake-case-enforcement (by @nanoandrew4) - #247

### Contributors

* @MikeFalcon77
* @nanoandrew4

## [0.1.4](https://github.com/hypernetix/hyperspot/compare/cf-modkit-auth-v0.1.3...cf-modkit-auth-v0.1.4) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta (by @MikeFalcon77)

### Contributors

* @MikeFalcon77

## [0.1.4](https://github.com/hypernetix/hyperspot/compare/cf-modkit-v0.1.3...cf-modkit-v0.1.4) - 2026-01-27

### Other

- Merge branch 'main' into snake-case-enforcement (by @nanoandrew4) - #247
- Merge branch 'main' into snake-case-enforcement (by @nanoandrew4) - #247

### Contributors

* @nanoandrew4

## [0.1.4](https://github.com/hypernetix/hyperspot/compare/cf-modkit-sdk-v0.1.3...cf-modkit-sdk-v0.1.4) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta (by @MikeFalcon77)

### Contributors

* @MikeFalcon77

## [0.1.4](https://github.com/hypernetix/hyperspot/compare/cf-modkit-odata-macros-v0.1.3...cf-modkit-odata-macros-v0.1.4) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta (by @MikeFalcon77)

### Contributors

* @MikeFalcon77

## [0.1.4](https://github.com/hypernetix/hyperspot/compare/cf-modkit-macros-v0.1.3...cf-modkit-macros-v0.1.4) - 2026-01-27

### Other

- Merge branch 'main' into snake-case-enforcement (by @nanoandrew4) - #247
- Merge branch 'main' into snake-case-enforcement (by @nanoandrew4) - #247

### Contributors

* @nanoandrew4

## [0.1.4](https://github.com/hypernetix/hyperspot/compare/cf-modkit-db-v0.1.3...cf-modkit-db-v0.1.4) - 2026-01-27

### Other

- update Cargo.toml dependencies

## [0.1.3](https://github.com/hypernetix/hyperspot/releases/tag/cf-system-sdks-v0.1.3) - 2026-01-27

### Other

- release (by @github-actions[bot]) - #323

### Contributors

* @github-actions[bot]

## [0.1.3](https://github.com/hypernetix/hyperspot/releases/tag/cf-system-sdk-directory-v0.1.3) - 2026-01-27

### Other

- release (by @github-actions[bot]) - #323

### Contributors

* @github-actions[bot]

## [0.1.4](https://github.com/hypernetix/hyperspot/compare/cf-modkit-security-v0.1.3...cf-modkit-security-v0.1.4) - 2026-01-27

### Other

- update Cargo.toml dependencies

## [0.1.3](https://github.com/hypernetix/hyperspot/compare/cf-system-sdks-v0.1.2...cf-system-sdks-v0.1.3) - 2026-01-27

### Other

- updated the following local packages: cf-system-sdk-directory

## [0.1.3](https://github.com/hypernetix/hyperspot/compare/cf-system-sdk-directory-v0.1.2...cf-system-sdk-directory-v0.1.3) - 2026-01-27

### Other

- updated the following local packages: cf-modkit-transport-grpc

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-types-registry-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta
- release
- Normalize modules (types registry and simple user settings) paths to snake_case

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-tenant-resolver-gw-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta
- release
- Normalize modules (types registry and simple user settings) paths to snake_case

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-static-tr-plugin-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta
- release
- Normalize modules (types registry and simple user settings) paths to snake_case

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-single-tenant-tr-plugin-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta
- release
- Normalize modules (types registry and simple user settings) paths to snake_case

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-tenant-resolver-sdk-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta
- release

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-nodes-registry-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta
- release

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-nodes-registry-sdk-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta
- release

## [0.1.2](https://github.com/hypernetix/hyperspot/compare/cf-modkit-node-info-v0.1.1...cf-modkit-node-info-v0.1.2) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-module-orchestrator-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta
- release

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-file-parser-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta
- release

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-types-registry-sdk-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta
- release

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-grpc-hub-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta
- release

## [0.1.1](https://github.com/hypernetix/hyperspot/releases/tag/cf-api-gateway-v0.1.1) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta
- release

## [0.1.2](https://github.com/hypernetix/hyperspot/compare/cf-modkit-security-v0.1.1...cf-modkit-security-v0.1.2) - 2026-01-27

### Other

- *(modules)* Add module descriptions, missed README.MD and some crate meta

## [0.1.1](https://github.com/hypernetix/hyperspot/compare/cf-system-sdk-directory-v0.1.0...cf-system-sdk-directory-v0.1.1) - 2026-01-26

### Other

- updated the following local packages: cf-modkit-transport-grpc

### Added

### Changed

### Fixed

## [0.1.0] - 2026-01-23

### Added

- Initial release.


