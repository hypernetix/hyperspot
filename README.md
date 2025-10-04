# HyperSpot Server

HyperSpot Server is a modular, high-performance platform for AI services built in Rust. It provides a comprehensive framework for building scalable AI applications with automatic REST API generation, comprehensive OpenAPI documentation, and a flexible modular architecture.

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
make audit      # Security audit
make deny       # License and dependency checks

# Windows (using PowerShell script)
./scripts/ci.ps1 check        # Run full CI pipeline
./scripts/ci.ps1 fmt          # Check formatting
./scripts/ci.ps1 fmt -Fix     # Auto-format code
./scripts/ci.ps1 clippy       # Run linter
./scripts/ci.ps1 clippy -Fix  # Auto-fix linter issues
./scripts/ci.ps1 test         # Run tests
./scripts/ci.ps1 audit        # Security audit
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

### Smoke Test Examples

```bash
# Start the server in background
cargo run --bin hyperspot-server -- --config config/quickstart.yaml run &
SERVER_PID=$!

# Wait for server to start
sleep 3

# Health check
curl -f http://127.0.0.1:8087/health
# Expected: {"status":"healthy","timestamp":"..."}

# OpenAPI documentation
curl -f http://127.0.0.1:8087/openapi.json | jq '.info.title'
# Expected: "HyperSpot API"

# Interactive docs (in browser)
echo "Open http://127.0.0.1:8087/docs for Stoplight Elements"

# CORS test (if enabled)
curl -f -H "Origin: http://localhost:8080" \
     -H "Access-Control-Request-Method: GET" \
     -H "Access-Control-Request-Headers: Content-Type" \
     -X OPTIONS http://127.0.0.1:8087/health
# Expected: CORS headers in response

# Cleanup
kill $SERVER_PID
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
- **[Module Creation Prompt](docs/MODULE_CREATION_PROMT.md)** - Prompt for LLM-editor to generate a module from OpenAPI specification
- **[Contributing](CONTRIBUTING.md)** - Development workflow and coding standards

## Key Features

### Modular Architecture
- **Auto-discovery**: Modules register automatically via the `#[modkit::module]` macro
- **Dependency Management**: Topological sorting ensures proper initialization order
- **Lifecycle Management**: Standardized phases (Init → DB → REST → Start → Stop)
- **Type-Safe Integration**: Compile-time guarantees for module contracts

### ModKit Runtime
- **Unified Startup**: Single `modkit::runtime::run()` function manages entire lifecycle
- **Database Integration**: Automatic connection pooling and migration management
- **Configuration Provider**: Hierarchical YAML configuration with environment overrides
- **Graceful Shutdown**: Proper cleanup with cancellation tokens

### Type-Safe API Development
- **Type-Safe Builder**: Compile-time guarantees with `OperationBuilder`
- **Automatic OpenAPI**: Generate documentation from Rust types with `utoipa`
- **RFC-9457 Error Handling**: Standardized HTTP problem details with `Problem` (implements `IntoResponse`)
- **Schema Components**: Reusable type definitions with automatic registration
- **Handler Integration**: Direct Axum handler attachment with `.problem_response()` helpers

### Production Ready
- **Graceful Shutdown**: Proper cleanup with cancellation tokens
- **Error Handling**: Comprehensive error propagation with `anyhow`
- **Observability**: Structured logging with `tracing`
- **Configuration**: Flexible YAML-based configuration with environment overrides

### Developer Experience
- **Fast Development**: Hot reloading and quick iteration
- **Interactive Docs**: Stoplight Elements at `/docs` (CDN by default; embedded with `--features embed_elements`)
- **Health Checks**: Built-in `/health` endpoints
- **Type Safety**: Compile-time guarantees for API contracts

## RFC-9457 Error Handling

HyperSpot implements standardized HTTP error responses using RFC-9457 Problem Details:

### Built-in Problem Types

```rust
use modkit::{Problem, bad_request, not_found, conflict, internal_error};

// Convenience constructors return Problem directly
let error = bad_request("Invalid email format");
let error = not_found("User not found");
let error = conflict("Email already exists");
let error = internal_error("Database connection failed");

// Custom problem with full control
let error = Problem::new(StatusCode::UNPROCESSABLE_ENTITY, "Validation Failed", "Input validation errors")
    .with_code("VALIDATION_ERROR")
    .with_instance("/users/create")
    .with_errors(validation_errors);
```

### OpenAPI Integration

The `OperationBuilder` provides `.problem_response()` helpers that automatically:
- Register the `Problem` schema in OpenAPI components
- Set correct `application/problem+json` content type
- Reference the schema in response definitions

```rust
OperationBuilder::post("/users")
    .json_request::<CreateUserRequest>(openapi, "User data")
    .json_response_with_schema::<User>(openapi, 201, "User created")
    .problem_response(openapi, 400, "Validation errors")
    .problem_response(openapi, 409, "Email already exists")
    .problem_response(openapi, 500, "Internal server error")
    .handler(create_user_handler)
    .register(router, openapi)
```

### Handler Implementation

TBD

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

# Check compilation without running
cargo check
```

## Project Structure

```
├── apps/
│   └── hyperspot-server/         # Main server application with ModKit runtime
├── modules/
│   └── api_ingress/              # HTTP routing and OpenAPI documentation
├── libs/
│   ├── modkit/                   # Core ModKit framework and traits
│   ├── db/                       # Database abstraction layer
│   └── runtime/                  # Runtime utilities and configuration
└── config/                       # Configuration files
    ├── quickstart.yaml           # Development configuration with SQLite
    ├── server.yaml               # Full production configuration
    └── no-db.yaml                # No-database mode configuration
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

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Roadmap

- [ ] gRPC support for module communication
- [ ] Distributed tracing integration
- [ ] Kubernetes deployment manifests
- [ ] WebSocket support in API gateway
- [ ] Module marketplace and registry
- [ ] Out-of-process module support

## Performance

HyperSpot is designed for high performance:

- **Lock-free routing**: Efficient request dispatching with `DashMap` and `ArcSwap`
- **Async-first**: Built on Tokio for scalable concurrency
- **Type-safe hot paths**: Zero runtime type checks with compile-time guarantees
- **Fast serialization**: Optimized JSON handling with `serde`
- **Connection pooling**: Efficient database connection reuse

Benchmarks show excellent performance for production workloads with minimal resource overhead.

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

3. **Explore the API**:
   - Visit http://127.0.0.1:8087/docs for interactive documentation
   - Check health at http://127.0.0.1:8087/health

4. **Create Your First Module**: Follow the module creation example above

5. **Add to Configuration**: Update `config/quickstart.yaml` to include your module

The ModKit framework provides everything you need to build scalable, maintainable AI services with excellent developer experience and production-ready features out of the box.