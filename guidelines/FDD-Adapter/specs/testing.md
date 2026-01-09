# Testing Specification

**Technology Stack**: Rust testing + Python E2E tests

---

## Testing Framework

**Unit Tests**: Rust built-in testing with `cargo test`
**Integration Tests**: Cargo integration tests with testcontainers
**E2E Tests**: pytest with playwright/httpx

---

## Test Location

- Unit tests: `modules/*/src/` (inline), `modules/*/tests/`
- Integration tests: `modules/*/tests/`
- E2E tests: `testing/e2e/`

---

## Test Commands

```bash
# Run all tests
make test

# Run unit tests only
cargo test --lib

# Run integration tests
cargo test --test '*'

# Run E2E tests
pytest testing/e2e/

# Run with coverage
make coverage
cargo tarpaulin --out Html
```

---

## Coverage Requirements

Comprehensive test coverage with testcontainers for integration tests.

---

## Testing Tools

- Rust: cargo test, testcontainers
- Python: pytest, httpx
- Mocking: mockito (Rust), pytest-mock (Python)
- Database: testcontainers with PostgreSQL/SQLite
