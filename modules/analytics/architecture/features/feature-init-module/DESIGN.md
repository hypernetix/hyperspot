# Init Module - Feature Design

**Status**: ✅ IMPLEMENTED  
**Module**: Analytics

**Verification**: 
- ✅ Compilation: `cargo check --package analytics-sdk --package analytics` - SUCCESS
- ✅ Workspace integration: Both crates added to workspace
- ✅ ModKit registration: Module macro expands correctly
- ✅ All requirements completed (1/1 = 100%)
- ✅ All changes completed (1/1 = 100%)

---

## A. Feature Context

### Overview

Create **minimal module structure** following SDK pattern with ModKit compliance. This establishes the compilable skeleton for business features to build upon.

**Critical Scope Constraint**: Init creates **structure only**, NO business logic.

### Purpose

Initialize the analytics module with:
- Empty compilable module structure
- ModKit integration
- SDK pattern (transport-agnostic API)
- Layer folders ready for business features

### Actors

- **Developer/Architect**: Creates module structure using init workflow

### References

**MANDATORY Reading**:
- `@/guidelines/hyperspot-fdd-adapter/INIT_MODULE_PATTERNS.md` - ModKit integration patterns
- `@/guidelines/NEW_MODULE.md` - Module structure patterns
- `@/docs/MODKIT_UNIFIED_SYSTEM.md` - ModKit integration
- `@/modules/analytics/architecture/DESIGN.md` - Overall Design

### What Init IS vs IS NOT

**Init module creates** (✅):
- SDK crate with empty API trait
- Module crate with ModKit registration
- Stub local client (no method implementations)
- Empty layer folders (domain/, infra/, api/)
- Basic configuration with defaults
- Workspace integration (Cargo.toml)

**Init module does NOT create** (❌):
- Business models or GTS types
- API method definitions
- Database entities or repositories
- REST handlers or DTOs
- Domain services
- Any business logic

**Rule**: If it requires understanding business requirements or GTS types, it's a **feature**, not init.

---

## B. Actor Flows

*Intentionally minimal for init-module - structural task, not business logic.*

Developer runs init workflow → creates module skeleton → verifies compilation.

---

## C. Algorithms

*Intentionally minimal for init-module - structural task, not business logic.*

See OpenSpec change 001 for implementation details.

---

## D. States

*Not applicable for init-module*

---

## E. Technical Details

### File Structure Created

```
modules/analytics/
├── analytics-sdk/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── api.rs          # Empty trait
│       ├── errors.rs       # Base error type
│       └── models.rs       # Placeholder
├── analytics/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── module.rs       # ModKit registration
│       ├── config.rs       # Configuration
│       ├── local_client.rs # Stub implementation
│       ├── domain/
│       │   └── mod.rs      # Empty
│       ├── infra/
│       │   └── mod.rs      # Empty
│       └── api/
│           └── mod.rs      # Empty
```

### ModKit Integration

**Module macro**:
```rust
use async_trait::async_trait;
use modkit::{DbModule, Module, ModuleCtx, RestfulModule};
use modkit::api::OpenApiRegistry;

#[modkit::module(
    name = "analytics",
    capabilities = [db, rest]
)]
#[derive(Clone)]
pub struct AnalyticsModule {
    config: AnalyticsConfig,
}
```

**Trait implementations**:
```rust
#[async_trait]
impl Module for AnalyticsModule {
    async fn init(&self, _ctx: &ModuleCtx) -> anyhow::Result<()> {
        tracing::info!(module = "analytics", "Analytics module initialized");
        Ok(())
    }
}

#[async_trait]
impl DbModule for AnalyticsModule {
    async fn migrate(&self, _db: &modkit_db::DbHandle) -> anyhow::Result<()> {
        // Migrations will be added by business features
        Ok(())
    }
}

impl RestfulModule for AnalyticsModule {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        _openapi: &dyn OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        // REST routes will be added by business features
        Ok(router)
    }
}
```

### SDK Dependencies

**analytics-sdk/Cargo.toml**:
```toml
[dependencies]
thiserror = "2.0"
async-trait = "0.1"
modkit-security = { path = "../../../libs/modkit-security" }
```

**Key points**:
- Use `modkit-security` for SecurityCtx (NOT `modkit::security`)
- No serde in SDK (transport-agnostic)

### Module Dependencies

**analytics/Cargo.toml** (critical dependencies):
```toml
[dependencies]
analytics-sdk = { path = "../analytics-sdk" }
modkit = { path = "../../../libs/modkit" }
modkit-db = { path = "../../../libs/modkit-db" }
async-trait = "0.1"
axum = "0.8"
anyhow = "1.0"
tracing = "0.1"
inventory = "0.3"  # REQUIRED for modkit macro
serde = { version = "1.0", features = ["derive"] }
```

**Critical**: `inventory` dependency is REQUIRED for `#[modkit::module]` macro to work.

### Workspace Changes

**Root Cargo.toml**:
```toml
[workspace]
members = [
    # ... other members
    "modules/analytics/analytics-sdk",
    "modules/analytics/analytics",
]
```

### Error Handling

**analytics-sdk/src/errors.rs**:
```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AnalyticsError {
    #[error("Internal error: {0}")]
    Internal(String),
}

pub type AnalyticsResult<T> = Result<T, AnalyticsError>;
```

---

## F. Requirements

### fdd-analytics-feature-init-module-req-module-structure

**Status**: ✅ COMPLETED

**Description**: The system SHALL create a minimal compilable module structure following SDK pattern with ModKit compliance. The module structure MUST include transport-agnostic SDK crate, domain-layered module crate, ModKit registration, and workspace integration without any business logic.

**References**:
- [Section A: What Init IS vs IS NOT](#what-init-is-vs-is-not)
- [Section E: File Structure Created](#file-structure-created)
- [Section E: ModKit Integration](#modkit-integration)
- [Section E: Workspace Changes](#workspace-changes)

**Testing Scenarios**:

1. **Compilation Test**:
   - Run `cargo check` on workspace
   - Verify both analytics-sdk and analytics crates compile
   - Expected: No compilation errors, zero business logic warnings

2. **Module Registration Test**:
   - Verify ModKit macro expands correctly
   - Check module appears in module registry via inventory
   - Expected: AnalyticsModule registered with db and rest capabilities

3. **Workspace Integration Test**:
   - Import SDK from external crate
   - Verify re-exports work correctly
   - Expected: All SDK types accessible, no circular dependencies

**Acceptance Criteria**:
- Both analytics-sdk and analytics crates compile without errors
- ModKit `#[modkit::module]` macro expands correctly with db and rest capabilities
- Workspace Cargo.toml includes both crates as members
- All layer folders (domain/, infra/, api/) exist and contain only empty mod.rs files
- No business logic, GTS types, or API method definitions present
- SDK crate has no serde dependencies (transport-agnostic)
- inventory dependency included for macro support

---

## G. Implementation Plan

**Total Changes**: 1

1. **fdd-analytics-feature-init-module-change-structure** ✅ COMPLETED
   - **Description**: Create complete module structure with SDK pattern following ModKit compliance
   - **Implements Requirements**: `fdd-analytics-feature-init-module-req-module-structure`
   - **Dependencies**: None
   - **Completed**: 2026-01-06
   - **Scope**:
     - SDK crate (analytics-sdk) with empty API trait and base error types
     - Module crate (analytics) with ModKit registration and empty layer folders
     - Local client stub with no method implementations
     - Workspace integration in root Cargo.toml
     - Configuration structure with defaults
   - **Effort**: 1-2 hours
   - **Verification**:
     - `cargo check --package analytics-sdk --package analytics` passes
     - Both crates compile successfully
     - ModKit registration macro expands without errors
     - Layer folders (domain/, infra/, api/) exist and are empty
     - No business logic present in any files

---
