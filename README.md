# Cyber Fabric
![Badge](./.github/badgeHN.svg)
[![OpenSSF Scorecard](https://api.scorecard.dev/projects/github.com/hypernetix/hyperspot/badge)](https://scorecard.dev/viewer/?uri=github.com/hypernetix/hyperspot)

**Cyber Fabric** is a modular, high-performance platform for building modern enterprise-grade SaaS services in Rust. It provides a comprehensive framework for building scalable AI-powered applications with automatic REST API generation, comprehensive OpenAPI documentation, and a extremely flexible modular architecture.

**Key Philosophy:**
- **Modular by Design**: Everything is a Module - composable, independent units with gateway patterns for pluggable workers
- **Extensible at Every Level**: [GTS](https://github.com/globaltypesystem/gts-spec)-powered extension points for custom data types, business logic, and third-party integrations
- **SaaS Ready**: Multi-tenancy, granular access control, usage tracking, and tenant customization built-in
- **Cloud Operations Excellence**: Production-grade observability, database agnostic design, API best practices, and resilience patterns via ModKit
- **Quality First**: 90%+ test coverage target with unit, integration, E2E, performance, and security testing
- **Universal Deployment**: Single codebase runs on cloud, on-prem Windows/Linux workstation, or mobile
- **Developer Friendly**: AI-assisted code generation, automatic OpenAPI docs, DDD-light structure, and type-safe APIs
- **Written in Rust**: Optimize recurring engineering work with compile-time safety and deep static analysis (including project-specific lints) so more issues are prevented before review/runtime.
- **Keep Monorepo while possible**: Keep core modules and contracts in one place to enable atomic refactors, consistent tooling/CI, and realistic local build + end-to-end testing; split only when scale forces it.

See the full architecture [MANIFEST](docs/ARCHITECTURE_MANIFEST.md) for more details, including rationales behind Rust and Monorepo choice.

## Quick Start

### Prerequisites

- Rust stable with Cargo ([Install via rustup](https://rustup.rs/))
- Protocol Buffers compiler (`protoc`):
  - macOS: `brew install protobuf`
  - Linux: `apt-get install protobuf-compiler`
  - Windows: Download from https://github.com/protocolbuffers/protobuf/releases
- MariaDB/PostgreSQL/SQLite or in-memory database

### CI/Development Commands

```bash
# Clone the repository
git clone <repository-url>
cd cyberfabric-core

make ci         # Run full CI pipeline
make fmt        # Check formatting (no changes). Use 'make dev-fmt' to auto-format
make clippy     # Lint (deny warnings). Use 'make dev-clippy' to attempt auto-fix
make test       # Run tests
make example    # Run modkit example module
make check      # Full check suite
make safety     # Extended safety checks (includes dylint/kani)
make deny       # License and dependency checks
```

### Running the Server

```bash
# Quick helper
make quickstart

# Option 1: Run with SQLite database (recommended for development)
cargo run --bin hyperspot-server -- --config config/quickstart.yaml run

# Option 2: Run without database (no-db mode)
cargo run --bin hyperspot-server -- --config config/no-db.yaml run

# Option 3: Run with mock in-memory database for testing
cargo run --bin hyperspot-server -- --config config/quickstart.yaml --mock run

# Check if server is ready (detailed JSON response)
curl http://127.0.0.1:8087/health

# Kubernetes-style liveness probe (simple "ok" response)
curl http://127.0.0.1:8087/healthz

# See API documentation:
# $ make quickstart
# visit: http://127.0.0.1:8087/docs
```

### Example Configuration (config/quickstart.yaml)

```yaml
# Cyber Fabric Configuration

# Core server configuration (global section)
server:
  home_dir: "~/.hyperspot"

# Database configuration (global section)
database:
  url: "sqlite://database/database.db"
  max_conns: 10
  busy_timeout_ms: 5000

# Logging configuration (global section)
logging:
  default:
    console_level: info
    file: "logs/hyperspot.log"
    file_level: warn
    max_age_days: 28
    max_backups: 3
    max_size_mb: 1000

# Per-module configurations moved under modules section
modules:
  api_gateway:
    bind_addr: "127.0.0.1:8087"
    enable_docs: true
    cors_enabled: false
```

### Creating Your First Module

See [NEW_MODULE.md](guidelines/NEW_MODULE.md), but also [MODKIT UNIFIED SYSTEM](docs/modkit_unified_system/README.md) and [MODKIT_PLUGINS.md](docs/MODKIT_PLUGINS.md) for more details.

## Documentation

- **[Architecture manifest](docs/ARCHITECTURE_MANIFEST.md)** - High-level overview of the architecture
- **[Components](docs/COMPONENTS.md)** - List of all components and their roles
- **[NEW_MODULE.md](guidelines/NEW_MODULE.md), [MODKIT UNIFIED SYSTEM](docs/modkit_unified_system/README.md) and [MODKIT_PLUGINS.md](docs/MODKIT_PLUGINS.md)** - how to add new modules.
- **[Contributing](CONTRIBUTING.md)** - Development workflow and coding standards

## Configuration

### YAML Configuration Structure

```yaml
# config/server.yaml

# Global server configuration
server:
  home_dir: "~/.hyperspot"

# Database configuration
database:
  servers:
    sqlite_users:
      params:
        WAL: "true"
        synchronous: "NORMAL"
        busy_timeout: "5000"
      pool:
        max_conns: 5
        acquire_timeout: "30s"

# Logging configuration
logging:
  default:
    console_level: info
    file: "logs/hyperspot.log"
    file_level: warn
    max_age_days: 28
    max_backups: 3
    max_size_mb: 1000

# Per-module configuration
modules:
  api_gateway:
    config:
      bind_addr: "127.0.0.1:8087"
      enable_docs: true
      cors_enabled: true
  users_info:
    database:
      server: "sqlite_users"
      file: "users_info.db"
    config:
      default_page_size: 5
      max_page_size: 100
```

### Environment Variable Overrides

Configuration supports environment variable overrides with `HYPERSPOT_` prefix:

```bash
export HYPERSPOT_DATABASE_URL="postgres://user:pass@localhost/db"
export HYPERSPOT_MODULES_api_gateway_BIND_ADDR="0.0.0.0:8080"
export HYPERSPOT_LOGGING_DEFAULT_CONSOLE_LEVEL="debug"
```

## Testing

```bash
# Run all tests
make test
# or
cargo test

# Run specific module tests
cargo test -p api_gateway
cargo test -p modkit

# Integration tests with database
cargo test --test integration

# Unit tests code coverage
make coverage-unit
```

### Fuzzing

Cyber Fabric uses continuous fuzzing to find bugs and security issues:

```bash
# Run fuzzing smoke tests
make fuzz

# Fuzz specific component
make fuzz-run FUZZ_TARGET=fuzz_odata_filter FUZZ_SECONDS=300

# See all available targets
make fuzz-list
```

Fuzzing runs automatically in CI via ClusterFuzzLite. See `fuzz/README.md` for details.

### CI / Development Commands


Cyber Fabric uses a unified, cross-platform Python CI script. Ensure you have Python 3.9+ installed.

```bash
# Clone the repository
git clone <repository-url>
cd cyberfabric-core

# All code must pass these checks before merging
python scripts/ci.py all          # Build and run all the checks
# Run individual checks
python scripts/ci.py check        # Full CI suite: fmt, clippy, test, audit, deny
python scripts/ci.py fmt          # Check formatting
python scripts/ci.py fmt --fix    # Auto-format code
python scripts/ci.py clippy       # Run linter
python scripts/ci.py clippy --fix # Attempt to fix warnings
python scripts/ci.py dylint       # runs custom project compliance lints on the workspace
python scripts/ci.py audit        # Security audit
python scripts/ci.py deny         # License & dependency checks
````

On Unix/Linux/macOS, the Makefile provides shortcuts:

```bash
# All code must pass these checks before merging
make all    # Build and run all the checks
# Run individual checks
make check  # Full check suite as defined in Makefile
make fmt    # formatting (cargo fmt --all -- --check)
make dev-fmt # auto-format code (cargo fmt --all)
make clippy # linting (cargo clippy --workspace --all-targets --all-features -- -D warnings -D clippy::perf)
make lint   # compilation with warnings denied (RUSTFLAGS="-D warnings" cargo check ...)
make dylint # runs custom project compliance lints on the workspace
make deny   # dependency license and policy checks (cargo deny check)
make kani   # optional deep safety verification (Kani verifier)
```

### E2E Tests

E2E tests require Python dependencies and pytest:

```bash
pip install -r testing/requirements.txt
```

```bash
make e2e-local  # Run e2e tests locally against a running server
make e2e-docker # Run e2e tests in a Docker container
make coverage-e2e # Run e2e tests with code coverage
make coverage # Run both unit and e2e tests with code coverage
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details.
