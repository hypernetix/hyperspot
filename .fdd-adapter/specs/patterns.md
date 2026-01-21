# Architecture Patterns Specification

**Source**: docs/ARCHITECTURE_MANIFEST.md, README.md, code analysis

## Core Pattern: Modular Architecture

**Description**: Everything is a Module - composable, independent units

**Implementation**:
- Every logical component is a Rust package (crate)
- Each module has a library crate (`lib.rs`) with module declaration
- Optional binary crate (`main.rs`) for out-of-process modules
- Modules discovered automatically via `inventory` crate

**Module Structure**:
```
modules/{module}/
├── src/
│   ├── lib.rs              # Module declaration
│   ├── module.rs           # Module implementation
│   ├── contract/           # Public contracts
│   ├── api/rest/           # REST API routes
│   ├── domain/             # Business logic
│   └── infra/              # Infrastructure
├── tests/                  # Integration tests
└── Cargo.toml
```

## Gateway Pattern

**Description**: Gateway modules with pluggable worker modules

**When to Use**: When you need:
- Multiple implementations of same capability
- Runtime plugin selection
- Tenant-specific customization
- Dynamic module loading

**Example**: `tenant_resolver` (gateway) + tenant resolver plugins

## In-Process vs Out-of-Process Modules

**In-Process**: Default, linked into main binary  
**Out-of-Process**: Separate process, gRPC communication

**OoP Benefits**:
- Language independence
- Fault isolation
- Independent scaling
- Security boundaries

**Reference**: `docs/MODKIT_UNIFIED_SYSTEM.md`

## Layered Architecture (within modules)

```
api/rest/           → REST API endpoints (DTOs)
domain/             → Business logic
infra/              → Infrastructure (DB, external services)
```

**Rules**:
- API layer uses DTOs (not domain entities)
- Domain layer is DB-agnostic
- Infrastructure layer handles external concerns

## Configuration Pattern

**Hierarchical YAML**:
```yaml
server:              # Global config
database:            # Global DB config
modules:             # Per-module config
  module_name:
    config: {...}
    database: {...}
```

**Environment Overrides**: `HYPERSPOT_` prefix

## Error Handling Pattern

**RFC 7807 Problem Details** for HTTP APIs  
**Structured Errors** in domain layer  
**Error Propagation**: Result<T, E> throughout

## Testing Strategy

**Unit Tests**: Per-module, `tests/` directory  
**Integration Tests**: Cross-module, `--test` flag  
**E2E Tests**: Python pytest, `testing/e2e/`  
**Coverage Target**: 90%+

## Custom Lints (Architectural Compliance)

**Location**: `dylint_lints/`

**Categories**:
- Contract Layer: No serde in contracts
- API Layer: DTOs must have derives
- REST Conventions: Versioned endpoints
- GTS Layer: Schema validation

**Purpose**: Enforce architecture rules at compile time

---

## Validation Checklist

### MUST Requirements (Mandatory)

- [ ] **Modular architecture followed** (everything is a module)
- [ ] **Module structure canonical** (lib.rs, module.rs, contract/, api/, domain/, infra/)
- [ ] **inventory used for discovery**
- [ ] **Layered architecture within modules** (API → Domain → Infra)
- [ ] **DTOs isolated to API layer** (not in domain/infra)
- [ ] **Domain layer DB-agnostic**
- [ ] **Error handling uses Result<T, E>**
- [ ] **Configuration hierarchical YAML** with module sections
- [ ] **Custom lints pass** (dylint)

### SHOULD Requirements (Strongly Recommended)

- [ ] Gateway pattern used for pluggable modules
- [ ] OoP modules for language independence/isolation
- [ ] RFC 7807 Problem Details for HTTP errors
- [ ] Structured errors in domain layer
- [ ] Environment overrides with HYPERSPOT_ prefix

### MAY Requirements (Optional)

- [ ] Additional architectural patterns documented
- [ ] Pattern decision rationale in ADRs
- [ ] Pattern examples from real features

## Compliance Criteria

**Pass**: All MUST requirements met (9/9) + dylint passes  
**Fail**: Any MUST requirement violated or architectural lints fail

### Agent Instructions

When designing features:
1. ✅ **ALWAYS create modules** (not monolithic code)
2. ✅ **ALWAYS follow canonical structure** (contract/, api/, domain/, infra/)
3. ✅ **ALWAYS use inventory** for module discovery
4. ✅ **ALWAYS separate layers** (API/Domain/Infra)
5. ✅ **ALWAYS keep DTOs in API layer**
6. ✅ **ALWAYS make domain DB-agnostic**
7. ✅ **ALWAYS use Result<T, E>** (no panics)
8. ✅ **ALWAYS run dylint** for compliance
9. ❌ **NEVER mix layers** (domain referencing API, etc.)
10. ❌ **NEVER bypass modular architecture**
11. ❌ **NEVER put business logic in API layer**
12. ❌ **NEVER access DB from domain directly**

### Architecture Review Checklist

Before implementing:
- [ ] Module structure matches canonical layout
- [ ] Layers properly separated
- [ ] DTOs only in api/rest/
- [ ] Domain types in domain/
- [ ] DB access only in infra/
- [ ] Contracts exported via lib.rs
- [ ] Module registered with inventory
- [ ] Configuration section in YAML
- [ ] Custom lints pass (`make dylint`)
