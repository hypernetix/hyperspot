# HyperSpot Server

**HyperSpot Server** is a modular, high-performance platform for building AI services built in Rust. It provides a comprehensive framework for building scalable AI applications with automatic REST API generation, comprehensive OpenAPI documentation, and a flexible modular architecture.

**Key Philosophy:**
- **Modular by Design**: Everything is a Module - composable, independent units with gateway patterns for pluggable workers
- **Extensible at Every Level**: [GTS](https://github.com/globaltypesystem/gts-spec)-powered extension points for custom data types, business logic, and third-party integrations
- **SaaS Ready**: Multi-tenancy, granular access control, usage tracking, and tenant customization built-in
- **Cloud Operations Excellence**: Production-grade observability, database agnostic design, API best practices, and resilience patterns via ModKit
- **Quality First**: 90%+ test coverage target with unit, integration, E2E, performance, and security testing
- **Universal Deployment**: Single codebase runs on cloud, on-prem Windows/Linux workstation, or mobile
- **Developer Friendly**: AI-assisted code generation, automatic OpenAPI docs, DDD-light structure, and type-safe APIs

See the full architecture [MANIFEST](docs/ARCHITECTURE_MANIFEST.md) for more details.

## Quick Start

### Prerequisites

- Rust stable with Cargo
- Optional: PostgreSQL (can run with SQLite or in-memory database)

### CI/Development Commands

```bash
# Clone the repository
git clone <repository-url>
cd hyperspot

# Unix/Linux/macOS (using Makefile)
make ci         # Run full CI pipeline (fmt-check, clippy, tests, security)
make fmt        # Check formatting (no changes). Use 'make dev-fmt' to auto-format
make clippy     # Lint (deny warnings). Use 'make dev-clippy' to attempt auto-fix
make test       # Run tests
make example    # Run modkit example module
make check      # All checks (fmt-check + clippy + test + audit + deny)
make deny       # License and dependency checks

# Windows (using PowerShell script)
./scripts/ci.ps1 check        # Run full CI pipeline
./scripts/ci.ps1 fmt          # Check formatting
./scripts/ci.ps1 fmt -Fix     # Auto-format code
./scripts/ci.ps1 clippy       # Run linter
./scripts/ci.ps1 clippy -Fix  # Auto-fix linter issues
./scripts/ci.ps1 test         # Run tests
./scripts/ci.ps1 deny         # License and dependency checks
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

# Check if server is ready
curl http://127.0.0.1:8087/health
```

### Example Configuration (config/quickstart.yaml)

```yaml
# HyperSpot Server Configuration

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
  api_ingress:
    bind_addr: "127.0.0.1:8087"
    enable_docs: true
    cors_enabled: false
```

### Creating Your First Module

```rust
use modkit::*;
use serde::{Deserialize, Serialize};
use axum::{Json, routing::get, http::StatusCode};
use utoipa::ToSchema;
use async_trait::async_trait;
use std::sync::Arc;

#[derive(Serialize, Deserialize, ToSchema)]
#[schema(title = "MyResource")]
pub struct MyResource {
    pub id: u64,
    pub name: String,
    pub description: String,
}

#[modkit::module(
    name = "my_module",
    deps = [],
    capabilities = [rest]
)]
#[derive(Clone, Default)]
pub struct MyModule;

#[async_trait]
impl Module for MyModule {
    async fn init(&self, ctx: &ModuleCtx) -> anyhow::Result<()> {
        tracing::info!("My module initialized");
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl RestfulModule for MyModule {
    fn register_rest(
        &self,
        _ctx: &ModuleCtx,
        router: axum::Router,
        openapi: &dyn modkit::api::OpenApiRegistry,
    ) -> anyhow::Result<axum::Router> {
        use modkit::api::OperationBuilder;

        // GET /my-resources - List all resources with RFC-9457 error handling
        let router = OperationBuilder::get("/my-resources")
            .operation_id("my_module.list")
            .summary("List all resources")
            .description("Retrieve a list of all available resources")
            .tag("my_module")
            .json_response_with_schema::<Vec<MyResource>>(openapi, 200, "List of resources")
            .problem_response(openapi, 400, "Bad Request")
            .problem_response(openapi, 500, "Internal Server Error")
            .handler(get(list_resources_handler))
            .register(router, openapi);

        Ok(router)
    }
}

async fn list_resources_handler() -> Result<Json<Vec<MyResource>>, modkit::Problem> {
    // Simulate potential error conditions
    let resources = vec![
        MyResource {
            id: 1,
            name: "Resource 1".to_string(),
            description: "First resource".to_string()
        }
    ];

    if resources.is_empty() {
        return Err(modkit::not_found("No resources available"));
    }

    Ok(Json(resources))
}
```

## Documentation

- **[Module Development Guide](docs/MODKIT_UNIFIED_SYSTEM.md)** - How to create modules with the ModKit framework
- **[Module Creation Prompt](docs/MODULE_CREATION_PROMT.md)** - Prompt for LLM-editor to generate a module from OpenAPI
  specification
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
  url: "sqlite://database/database.db"
  max_conns: 10
  busy_timeout_ms: 5000

# Logging configuration
logging:
  default:
    console_level: info
    file: "logs/hyperspot.log"
    file_level: warn
    max_age_days: 28
    max_backups: 3
    max_size_mb: 1000

# Module-specific configuration
modules:
  api_ingress:
    bind_addr: "127.0.0.1:8087"
    enable_docs: true
    cors_enabled: true
```

### Environment Variable Overrides

Configuration supports environment variable overrides with `HYPERSPOT_` prefix:

```bash
export HYPERSPOT_DATABASE_URL="postgres://user:pass@localhost/db"
export HYPERSPOT_MODULES_API_INGRESS_BIND_ADDR="0.0.0.0:8080"
export HYPERSPOT_LOGGING_DEFAULT_CONSOLE_LEVEL="debug"
```

## Testing

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test -p api_ingress
cargo test -p modkit

# Integration tests with database
cargo test --test integration
```

### CI / Development Commands

HyperSpot uses a unified, cross-platform Python CI script. Ensure you have Python 3.9+ installed.

```bash
# Clone the repository
git clone <repository-url>
cd hyperspot

# Execute CI tasks
python scripts/ci.py check        # Full CI suite: fmt, clippy, test, audit, deny
python scripts/ci.py fmt          # Check formatting
python scripts/ci.py fmt --fix    # Auto-format code
python scripts/ci.py clippy       # Run linter
python scripts/ci.py clippy --fix # Attempt to fix warnings
python scripts/ci.py test         # Unit tests
python scripts/ci.py audit        # Security audit
python scripts/ci.py deny         # License & dependency checks
````

On Unix/Linux/macOS, the Makefile provides shortcuts:

```bash
make check
make fmt
make clippy
make test
make audit
make deny
```

### E2E Tests

E2E tests require Python dependencies and pytest:

```bash
pip install -r testing/e2e/requirements.txt
```

Run against a **locally running** server:

```bash
python scripts/ci.py e2e
```

Run in a **Docker-based environment**:

```bash
python scripts/ci.py e2e --docker
```

Pass additional pytest arguments after `--`:

```bash
python scripts/ci.py e2e --docker -- -k Smoke
```

### Windows Development Notes

On Windows, ensure:

1. **Python 3.9+** is installed and added to PATH

2. **pip** installed dependencies:

   ```bash
   pip install -r testing/e2e/requirements.txt
   ```

3. **Docker CLI** must be available in PATH (if using Docker mode)

    * Install via Chocolatey:

      ```powershell
      choco install docker-cli
      ```
    * Or install Docker Desktop, but **disable its own daemon** if you use Docker inside WSL2

4. If Docker Engine runs in **WSL2**, configure Docker CLI in Windows to use it:

   ```powershell
   setx DOCKER_HOST "tcp://127.0.0.1:2375"
   ```

   (Adjust if using different proxy or WSL IP routing)

Test connectivity:

```powershell
docker version
```

If E2E uses Docker mode, ensure `docker compose` is available:

```powershell
docker compose version
```

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes with tests
4. Run `cargo fmt` and `cargo clippy`
5. Commit changes (`git commit -am 'Add amazing feature'`)
6. Push to branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## Getting Started Tutorial

1. **Clone and Setup**:
   ```bash
   git clone <repository-url>
   cd hyperspot
   cargo build
   ```

2. **Run Development Server**:
   ```bash
   cargo run --bin hyperspot-server -- --config config/quickstart.yaml run
   ```

   ```cmd
   cargo run --bin hyperspot-server -- --config config/quickstart-windows.yaml run
   ```

3. **Explore the API**:
    - Visit http://127.0.0.1:8087/docs for interactive documentation
    - Check health at http://127.0.0.1:8087/health

4. **Create Your First Module**: Follow the module creation example above

5. **Add to Configuration**: Update `config/quickstart.yaml` to include your module

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
