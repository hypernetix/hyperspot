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

## Deployment

**Architecture**: Stateless design, horizontal scaling ready
**Runtime**: Tokio async runtime
**Configuration**: Hot-reloadable via config files
**Observability**: OpenTelemetry tracing, structured logging

---

## CI/CD

Build validation via GitHub Actions (see `.github/workflows/ci.yml`)
