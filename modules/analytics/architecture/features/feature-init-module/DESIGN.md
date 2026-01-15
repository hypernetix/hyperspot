# Feature: Init Module
 
 - **Feature ID**: `fdd-analytics-feature-init-module`
 - **Feature Directory**: `modules/analytics/architecture/features/feature-init-module/`
 - **Status**: ✅ IMPLEMENTED
 - **Module**: Analytics

**Verification**: 
- ✅ Compilation: `cargo check --package analytics-sdk --package analytics` - SUCCESS
- ✅ Workspace integration: Both crates added to workspace
- ✅ ModKit registration: Module macro expands correctly
- ✅ All requirements completed (1/1 = 100%)
- ✅ All changes completed (1/1 = 100%)

---

## A. Feature Context

### 1. Overview

Create **minimal module structure** following SDK pattern with ModKit compliance. This establishes the compilable skeleton for business features to build upon.

**Critical Scope Constraint**: Init creates **structure only**, NO business logic.


### 2. Purpose

Initialize the analytics module with:
- Empty compilable module structure
- ModKit integration
- SDK pattern (transport-agnostic API)
- Layer folders ready for business features


### 3. Actors
 
 
### 4. References
 
 1. Features manifest entry: [FEATURES.md](../FEATURES.md)
 2. Overall Design: [DESIGN.md](../../DESIGN.md)
 3. Module creation conventions: [NEW_MODULE.md](../../../../../guidelines/NEW_MODULE.md)
 4. ModKit integration reference: [MODKIT_UNIFIED_SYSTEM.md](../../../../../docs/MODKIT_UNIFIED_SYSTEM.md)
 
### What Init IS vs IS NOT
 
 **Init module creates** (✅):
1. SDK crate with empty API trait
2. Module crate with ModKit registration
3. Stub local client (no method implementations)
4. Empty layer folders (domain/, infra/, api/)
5. Basic configuration with defaults
6. Workspace integration (Cargo.toml)

**Init module does NOT create** (❌):
1. Business models or GTS types
2. API method definitions
3. Database entities or repositories
4. REST handlers or DTOs
5. Domain services
6. Any business logic

**Rule**: If it requires understanding business requirements or GTS types, it's a **feature**, not init.

---

## B. Actor Flows
 
 No actor flows are defined for this feature.
 
 ---
 
 ## C. Algorithms
 
 No algorithms are defined for this feature.
 
 ---
 
 ## D. States

No state machines are defined for this feature.

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
 ### Module structure exists and is compilable
 
 - [x] **ID**: fdd-analytics-feature-init-module-req-module-structure
 **Status**: ✅ IMPLEMENTED
 **Description**: The system MUST provide a minimal, compilable Analytics module structure that follows the SDK pattern and integrates with ModKit, while containing no business logic.
 **References**:
  - [What Init IS vs IS NOT](#what-init-is-vs-is-not)
  - [File Structure Created](#file-structure-created)
  - [ModKit Integration](#modkit-integration)
  - [Workspace Changes](#workspace-changes)
 **Implements**:
  - Module structure implemented in `modules/analytics/analytics-sdk` and `modules/analytics/analytics`
 **Phases**:
  - [x] `ph-1`: Create both crates, integrate into workspace, and ensure `cargo check` passes
 **Testing Scenarios (FDL)**:
 - [x] **ID**: fdd-analytics-feature-init-module-test-module-compiles
   1. [x] - `ph-1` - Execute compilation check for `analytics-sdk` crate - `inst-check-sdk-compiles`
   2. [x] - `ph-1` - Execute compilation check for `analytics` crate - `inst-check-module-compiles`
   3. [x] - `ph-1` - Verify compilation has no errors - `inst-verify-no-compilation-errors`
   4. [x] - `ph-1` - Verify public SDK types are importable by an external crate - `inst-verify-sdk-types-importable`
 **Acceptance Criteria**:
 - The workspace contains `modules/analytics/analytics-sdk` and `modules/analytics/analytics`.
 - Both crates compile successfully.
 - The module registers via ModKit without defining business logic.

---

## G. Additional Context (Optional)
 
This feature intentionally delivers only the **module foundation** (SDK + module crate + ModKit registration) and is expected to be extended by subsequent Analytics features.

---
