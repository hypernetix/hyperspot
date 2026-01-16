# Project Structure Specification

**Source**: Repository file structure, README.md

## Root Structure

```
hyperspot/
├── FDD/                        # FDD core methodology
├── FDD-Adapter/                # Project-specific FDD adapter
├── apps/                       # Application binaries
│   └── hyperspot-server/       # Main server application
├── libs/                       # Shared libraries
│   ├── modkit/                 # Core module framework
│   ├── modkit-macros/          # Proc macros
│   ├── modkit-db/              # Database abstraction
│   ├── modkit-auth/            # Authentication
│   └── ...                     # Other libs
├── modules/                    # Application modules
│   ├── system/                 # System modules
│   │   ├── api_gateway/        # API Gateway
│   │   ├── nodes_registry/     # Nodes registry
│   │   └── types-registry/     # Types registry
│   └── file_parser/            # File parser module
├── examples/                   # Example modules
│   ├── modkit/                 # ModKit examples
│   └── oop-modules/            # OoP module examples
├── proto/                      # Protocol buffer definitions
├── testing/                    # Testing infrastructure
│   ├── docker/                 # Docker configs for testing
│   └── e2e/                    # E2E tests
├── scripts/                    # Build/CI scripts
├── config/                     # Configuration files
├── docs/                       # Documentation
├── guidelines/                 # Development guidelines
├── dylint_lints/               # Custom lints
├── Cargo.toml                  # Workspace manifest
├── Makefile                    # Build shortcuts
└── README.md                   # Project readme
```

## Module Structure

Each module follows this pattern:

```
modules/{module}/
├── src/
│   ├── lib.rs                  # Public exports
│   ├── module.rs               # Module implementation
│   ├── contract/               # Public contracts (DTOs)
│   │   └── mod.rs
│   ├── api/                    # API layer
│   │   └── rest/
│   │       └── routes.rs       # REST endpoints
│   ├── domain/                 # Business logic
│   │   ├── entities/           # Domain entities
│   │   ├── services/           # Domain services
│   │   └── repositories/       # Repository traits
│   ├── infra/                  # Infrastructure
│   │   ├── db/                 # Database implementation
│   │   └── external/           # External service clients
│   └── config.rs               # Module configuration
├── tests/                      # Integration tests
│   └── integration.rs
└── Cargo.toml                  # Crate manifest
```

## SDK Pattern (for OoP modules)

```
modules/{module}/
├── {module}/                   # Main module
├── {module}-sdk/               # Client SDK
└── {module}-grpc/              # gRPC definitions (optional)
```

## Configuration Location

**Config Files**: `config/*.yaml`

**Available Configs**:
- `quickstart.yaml` - Development with SQLite
- `no-db.yaml` - No database mode
- `e2e-local.yaml` - E2E testing
- `oop-example.yaml` - OoP module example

## Documentation Location

**Architecture Docs**: `docs/`
- `ARCHITECTURE_MANIFEST.md` - Architecture overview
- `MODKIT_UNIFIED_SYSTEM.md` - ModKit details
- `MODKIT_PLUGINS.md` - Plugin system
- `MODULES.md` - Module listing

**Guidelines**: `guidelines/`
- `NEW_MODULE.md` - Creating modules
- `DNA/` - Development standards
- `SECURITY.md` - Security guidelines

---

## Validation Checklist

### MUST Requirements (Mandatory)

- [ ] **Root structure documented** with all top-level directories
- [ ] **Module structure canonical** (lib.rs, module.rs, contract/, api/, domain/, infra/)
- [ ] **SDK pattern explained** for OoP modules
- [ ] **Configuration location specified** (config/*.yaml)
- [ ] **Documentation location specified** (docs/, guidelines/)
- [ ] **libs/ organized** (shared libraries separate from modules)
- [ ] **modules/ organized** by feature/domain
- [ ] **testing/ contains** E2E tests and docker configs
- [ ] **scripts/ contains** CI/build scripts

### SHOULD Requirements (Strongly Recommended)

- [ ] examples/ with working examples
- [ ] proto/ for gRPC definitions (if used)
- [ ] dylint_lints/ for custom lints
- [ ] FDD-Adapter/ at project root
- [ ] Cargo workspace properly configured

### MAY Requirements (Optional)

- [ ] benchmarks/ for performance tests
- [ ] tools/ for development utilities
- [ ] migrations/ for database migrations

## Compliance Criteria

**Pass**: All MUST requirements met (9/9) + structure matches spec  
**Fail**: Any MUST requirement missing or incorrect structure

### Agent Instructions

When creating files/directories:
1. ✅ **ALWAYS follow root structure** (apps/, libs/, modules/, config/, etc.)
2. ✅ **ALWAYS use canonical module structure** (contract/, api/, domain/, infra/)
3. ✅ **ALWAYS place shared code in libs/**
4. ✅ **ALWAYS place modules in modules/**
5. ✅ **ALWAYS place config in config/**
6. ✅ **ALWAYS place tests in testing/e2e/** (E2E) or tests/ (unit/integration)
7. ✅ **ALWAYS use SDK pattern** for OoP modules
8. ✅ **ALWAYS organize by domain** (not by technology)
9. ❌ **NEVER create flat structure** (no module organization)
10. ❌ **NEVER mix concerns** (business logic in wrong layer)
11. ❌ **NEVER put modules in libs/** (libs for shared utilities only)
12. ❌ **NEVER skip canonical structure** (contract/, api/, domain/, infra/)

### Structure Review Checklist

Before creating new components:
- [ ] Determined correct top-level directory
- [ ] Module follows canonical structure
- [ ] Separated contracts from implementation
- [ ] API layer isolated
- [ ] Domain layer pure
- [ ] Infrastructure layer separate
- [ ] Tests in correct location
- [ ] Configuration in config/
- [ ] Documentation updated

**API Docs**: `docs/api/api.json` (auto-generated OpenAPI)

## Testing Location

**Unit Tests**: Within each crate (`src/` or `tests/`)  
**Integration Tests**: `{crate}/tests/`  
**E2E Tests**: `testing/e2e/`  
**Docker Tests**: `testing/docker/`

## Build Artifacts

**Target Directory**: `target/` (gitignored)  
**Binary Output**: `target/release/hyperspot-server`  
**Test Binaries**: `target/debug/deps/`

## Git Structure

**Ignored**:
- `target/` - Build artifacts
- `Cargo.lock` (for libraries, kept for apps)
- `*.db` - Database files
- `logs/` - Log files

**Tracked**:
- All source code (`src/`, `tests/`)
- Configuration files (`config/`)
- Documentation (`docs/`, `guidelines/`)
- Build scripts (`scripts/`, `Makefile`)
