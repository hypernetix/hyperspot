# Build & Deploy Specification

**Source**: README.md, Makefile, scripts/ci.py

## Build Commands

**Development Build**:
```bash
cargo build
```

**Release Build**:
```bash
cargo build --release
```

**Clean Build**:
```bash
cargo clean
```

**Check (no binary)**:
```bash
cargo check --workspace
```

## Run Commands

**Quick Start** (SQLite):
```bash
make quickstart
# or
cargo run --bin hyperspot-server -- --config config/quickstart.yaml run
```

**With Specific Config**:
```bash
cargo run --bin hyperspot-server -- --config config/{config}.yaml run
```

**No-DB Mode** (testing):
```bash
cargo run --bin hyperspot-server -- --config config/no-db.yaml run
```

**Mock Mode** (in-memory):
```bash
cargo run --bin hyperspot-server -- --config config/quickstart.yaml --mock run
```

## CI Commands

**Full CI Suite**:
```bash
python scripts/ci.py all
# or on Unix/Linux/macOS
make all
```

**Individual Checks**:
```bash
python scripts/ci.py fmt              # Check formatting
python scripts/ci.py fmt --fix        # Auto-format
python scripts/ci.py clippy           # Lint
python scripts/ci.py clippy --fix     # Auto-fix warnings
python scripts/ci.py test             # Run tests
python scripts/ci.py dylint           # Custom lints
python scripts/ci.py audit            # Security audit
python scripts/ci.py deny             # License checks
```

**Makefile Shortcuts** (Unix/Linux/macOS):
```bash
make check          # Full check suite
make fmt            # Check formatting
make dev-fmt        # Auto-format
make clippy         # Lint
make test           # Run tests
make dylint         # Custom lints
make deny           # License/dependency checks
```

## Testing Commands

**Unit Tests**:
```bash
cargo test
```

**Specific Module Tests**:
```bash
cargo test -p {module_name}
```

**Integration Tests**:
```bash
cargo test --test integration
```

**E2E Tests** (requires Python):
```bash
pip install -r testing/requirements.txt
make e2e-local      # Against running server
make e2e-docker     # In Docker container
```

**Coverage**:
```bash
make coverage-unit  # Unit test coverage
make coverage-e2e   # E2E test coverage
make coverage       # Both
```

## Docker

**Dockerfile**: `testing/docker/hyperspot.Dockerfile`

**Build Image**:
```bash
docker build -f testing/docker/hyperspot.Dockerfile -t hyperspot:latest .
```

**Run Container**:
```bash
docker run -p 8087:8087 hyperspot:latest
```

**Docker Compose**: `testing/docker/docker-compose.yml`

## Configuration

**Config Files**: `config/*.yaml`

**Available Configs**:
- `quickstart.yaml` - SQLite, development default
- `no-db.yaml` - No database mode
- `e2e-local.yaml` - E2E testing config

**Environment Variables**:
```bash
export HYPERSPOT_DATABASE_URL="postgres://user:pass@localhost/db"
export HYPERSPOT_MODULES_api_gateway_BIND_ADDR="0.0.0.0:8080"
export HYPERSPOT_LOGGING_DEFAULT_CONSOLE_LEVEL="debug"
```

## Health Checks

**Detailed Health**:
```bash
curl http://127.0.0.1:8087/health
```

**Liveness Probe** (Kubernetes):
```bash
curl http://127.0.0.1:8087/healthz
```

## API Documentation

**Access** (when server running):
- Swagger UI: `http://127.0.0.1:8087/docs`
- OpenAPI JSON: `http://127.0.0.1:8087/api-docs/openapi.json`

## Deployment Checklist

Before deploying:
- ✅ All tests pass (`cargo test`)
- ✅ No clippy warnings (`cargo clippy`)
- ✅ No custom lint violations (`make dylint`)
- ✅ Code formatted (`cargo fmt`)
- ✅ E2E tests pass (`make e2e-local`)
- ✅ Security audit clean (`cargo audit`)
- ✅ License compliance (`cargo deny check`)

---

## Validation Checklist

### MUST Requirements (Mandatory)

- [ ] **Build commands documented** (dev, release, clean, check)
- [ ] **Run commands specified** for all modes (quickstart, config, no-db, mock)
- [ ] **CI commands defined** (full suite + individual checks)
- [ ] **Testing commands listed** (unit, integration, E2E, coverage)
- [ ] **Docker setup documented** (Dockerfile, build, run, compose)
- [ ] **Configuration files location specified** (config/*.yaml)
- [ ] **Environment variables documented** with HYPERSPOT_ prefix
- [ ] **Health check endpoints defined** (/health, /healthz)
- [ ] **API docs access documented** (/docs, /api-docs/openapi.json)

### SHOULD Requirements (Strongly Recommended)

- [ ] Cross-platform CI commands (Python scripts)
- [ ] Platform-specific shortcuts (Makefile for Unix/Linux/macOS)
- [ ] Multiple config variants (dev, test, prod)
- [ ] Coverage commands for both unit and E2E
- [ ] Deployment checklist comprehensive

### MAY Requirements (Optional)

- [ ] Performance profiling commands
- [ ] Monitoring setup
- [ ] Deployment automation scripts

## Compliance Criteria

**Pass**: All MUST requirements met (9/9) + commands executable  
**Fail**: Any MUST requirement missing or commands fail

### Agent Instructions

When building/deploying:
1. ✅ **ALWAYS use cargo commands** (cross-platform)
2. ✅ **ALWAYS run full CI suite** before PR (`python scripts/ci.py all`)
3. ✅ **ALWAYS check formatting** (`cargo fmt --check`)
4. ✅ **ALWAYS resolve clippy warnings** (zero warnings policy)
5. ✅ **ALWAYS run custom lints** (`make dylint` or `python scripts/ci.py dylint`)
6. ✅ **ALWAYS run all tests** (unit + integration + E2E)
7. ✅ **ALWAYS verify security** (`cargo audit`, `cargo deny`)
8. ✅ **ALWAYS use environment variables** (never hardcode config)
9. ❌ **NEVER skip CI checks**
10. ❌ **NEVER deploy without passing tests**
11. ❌ **NEVER commit secrets** (use env vars)
12. ❌ **NEVER use platform-specific commands** in implementation

### Pre-Deployment Checklist

Before every deployment:
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` passes
- [ ] `python scripts/ci.py dylint` passes
- [ ] `cargo test --workspace` passes
- [ ] `make e2e-local` passes (if applicable)
- [ ] `cargo audit` clean (no vulnerabilities)
- [ ] `cargo deny check` passes (licenses + advisories)
- [ ] Config files validated
- [ ] Environment variables documented
- [ ] Health checks responding
- [ ] API docs accessible
