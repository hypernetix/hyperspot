# Tech Stack Specification

**Source**: Cargo.toml, README.md, rust-toolchain.toml

## Language

**Rust**
- **Edition**: 2021
- **Toolchain**: Stable
- **Version**: Latest stable (as per rust-toolchain.toml)

## Core Frameworks

**ModKit** (Custom Modular Framework)
- **Purpose**: Core framework for modular architecture
- **Location**: `libs/modkit/`
- **Features**: Module registry, lifecycle management, API generation

**Axum** (Web Framework)
- **Purpose**: HTTP server and REST API routing
- **Features**: Type-safe routing, middleware, async handlers

## Databases

**Multi-Database Support**:
- SQLite (development default)
- PostgreSQL (production)
- MariaDB (production)
- In-memory mock (testing)

**ORM**: Custom modkit-db abstraction layer

## Key Dependencies

- **serde**: Serialization/deserialization
- **tokio**: Async runtime
- **tracing**: Observability and logging
- **utoipa**: OpenAPI documentation generation
- **axum**: Web framework
- **sqlx**: Database access

## Build Tools

- **Cargo**: Package manager and build system
- **rustfmt**: Code formatting
- **clippy**: Linting
- **dylint**: Custom project-specific lints

## Testing Tools

- **cargo test**: Rust unit and integration tests
- **pytest**: Python E2E tests
- **httpx**: HTTP client for tests

## CI/CD

- **Scripts**: `scripts/ci.py` (cross-platform Python CI)
- **Makefile**: Unix/Linux shortcuts
- **GitHub Actions**: `.github/workflows/`

---

## Validation Checklist

### MUST Requirements (Mandatory)

- [ ] **Language specified** with edition/version
- [ ] **Core frameworks documented** with purpose
- [ ] **Database support listed** with variants (dev/prod/test)
- [ ] **Key dependencies identified** (at least 5 core deps)
- [ ] **Build tools specified** (cargo, rustfmt, clippy)
- [ ] **Testing tools listed** with test runners
- [ ] **CI/CD approach documented** with commands

### SHOULD Requirements (Strongly Recommended)

- [ ] Each framework has location in codebase (`libs/...`)
- [ ] Database ORM/abstraction layer specified
- [ ] All tools have executable commands
- [ ] Source references valid (`Cargo.toml`, `rust-toolchain.toml`)

### MAY Requirements (Optional)

- [ ] Version constraints for dependencies
- [ ] Alternative tooling options
- [ ] Platform-specific considerations

## Compliance Criteria

**Pass**: All MUST requirements met (7/7)  
**Fail**: Any MUST requirement missing

### Agent Instructions

When implementing features:
1. ✅ **ALWAYS use Rust Edition 2021**
2. ✅ **ALWAYS use ModKit framework** for modules
3. ✅ **ALWAYS use Axum** for HTTP/REST
4. ✅ **ALWAYS use specified databases** (SQLite dev, PostgreSQL/MariaDB prod)
5. ✅ **ALWAYS use cargo** for builds
6. ✅ **ALWAYS run rustfmt + clippy** before commit
7. ✅ **ALWAYS run dylint** for architectural compliance
8. ❌ **NEVER introduce new languages** without adapter update
9. ❌ **NEVER use alternative web frameworks** (e.g., Actix, Rocket)
10. ❌ **NEVER bypass CI checks**
