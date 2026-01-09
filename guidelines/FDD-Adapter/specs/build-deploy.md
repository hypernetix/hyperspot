# Build & Deployment Specification

**Build System**: Cargo + Make

---

## Build Commands

```bash
# Development build
cargo build

# Release build
cargo build --release

# Via Makefile
make build
```

---

## Clean Commands

```bash
# Clean build artifacts
cargo clean

# Via Makefile
make clean
```

---

## Lint Commands

```bash
# Run clippy with strict rules
cargo clippy -- -D warnings

# Run all lints (100+ deny rules)
cargo clippy --all-targets --all-features -- -D warnings

# Custom architectural lints
cargo dylint --all

# Via Makefile
make lint
```

---

## Quality Assurance Tools

**Formal Verification**:
```bash
# Kani Rust Verifier - Formal verification of safety properties
make kani
# Checks: memory safety, arithmetic overflow, assertion violations, undefined behavior
```

**Security Auditing**:
```bash
# Geiger - Scan for unsafe code in dependencies
make geiger

# Cargo Deny - License compliance and security vulnerabilities
make deny
cargo deny check
```

**Code Coverage**:
```bash
# Generate code coverage reports
make coverage          # Combined unit + e2e tests
make coverage-unit     # Unit tests only
make coverage-e2e-local  # E2E tests only

# Tool: cargo-llvm-cov
```

**Custom Architectural Lints**:
```bash
# List all custom lints
make dylint-list

# Test lint implementations
make dylint-test

# Run custom lints
make dylint
```

**Lint Categories**:
- **de01_contract_layer**: Contract layer purity (no serde, HTTP types)
- **de02_api_layer**: DTO placement and isolation
- **de08_rest_api_conventions**: API endpoint versioning
- **de09_gts_layer**: GTS identifier format validation

---

## Deployment

**Architecture**: Stateless design, horizontal scaling ready
**Runtime**: Tokio async runtime
**Configuration**: Hot-reloadable via config files
**Observability**: OpenTelemetry tracing, structured logging

---

## CI/CD

Build validation via GitHub Actions (see `.github/workflows/ci.yml`)
