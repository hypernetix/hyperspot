# Testing Specification

**Source**: README.md, testing/requirements.txt, test files

## Testing Strategy

**Coverage Target**: 90%+

**Test Pyramid**:
1. Unit Tests (majority)
2. Integration Tests (module interactions)
3. E2E Tests (full system)

## Unit Tests

**Framework**: Rust built-in test framework + tokio-test for async

**Location**: 
- Same file: `#[cfg(test)] mod tests { ... }`
- Separate: `tests/` directory in each crate

**Run Command**:
```bash
cargo test
```

**Specific Module**:
```bash
cargo test -p {module_name}
```

**With Output**:
```bash
cargo test -- --nocapture
```

**Example**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_when_valid_pdf_then_success() {
        let result = parse_file("test.pdf").await;
        assert!(result.is_ok());
    }
}
```

## Integration Tests

**Framework**: Rust integration tests (`tests/` directory)

**Location**: `{crate}/tests/*.rs`

**Run Command**:
```bash
cargo test --test integration
```

**Example**:
```rust
// tests/integration_test.rs
use hyperspot_server::app::App;

#[tokio::test]
async fn test_module_integration() {
    let app = App::new().await.unwrap();
    // Test module interactions
}
```

## E2E Tests

**Framework**: pytest + httpx (Python)

**Location**: `testing/e2e/`

**Dependencies**:
```
pytest
pytest-asyncio
httpx
```

**Install**:
```bash
pip install -r testing/requirements.txt
```

**Run Commands**:
```bash
make e2e-local      # Against running server
make e2e-docker     # In Docker container
```

**Example Test** (`testing/e2e/test_file_parser.py`):
```python
import httpx
import pytest

@pytest.mark.asyncio
async def test_parser_info():
    async with httpx.AsyncClient() as client:
        response = await client.get(
            "http://127.0.0.1:8087/file-parser/v1/info"
        )
        assert response.status_code == 200
        data = response.json()
        assert "supported_formats" in data
```

## Test Configuration

**Config File**: `testing/e2e/conftest.py` (pytest fixtures)

**Test Data**: `testing/e2e/fixtures/` (if needed)

## Coverage

**Unit Test Coverage**:
```bash
make coverage-unit
```

**E2E Test Coverage**:
```bash
make coverage-e2e
```

**Combined Coverage**:
```bash
make coverage
```

**Coverage Tools**: 
- `cargo-tarpaulin` (Rust)
- `pytest-cov` (Python E2E)

## Mock/Stub Patterns

**Database Mocking**: 
```bash
cargo run --bin hyperspot-server -- --config config/quickstart.yaml --mock run
```

**HTTP Mocking**: Use `testing/docker/http-mock.Dockerfile`

## Test Naming Convention

**Pattern**: `test_{what}_when_{condition}_then_{expected}`

**Examples**:
- `test_parse_when_valid_pdf_then_success`
- `test_parse_when_invalid_format_then_error`
- `test_api_when_unauthorized_then_401`

## Test Organization

```
{crate}/
├── src/
│   └── lib.rs              # Unit tests here
├── tests/
│   ├── integration.rs      # Integration tests
│   └── helpers/            # Test utilities
└── Cargo.toml
```

## Continuous Integration

**Required Checks**:
- ✅ Unit tests pass
- ✅ Integration tests pass
- ✅ E2E tests pass (on relevant changes)
- ✅ Coverage meets threshold

**CI Command**:
```bash
python scripts/ci.py test
```

## Test Best Practices

1. **Isolate tests**: Each test independent
2. **Use fixtures**: Reusable test setup
3. **Clear assertions**: One concept per test
4. **Fast tests**: Optimize for speed
5. **Deterministic**: No flaky tests
6. **Descriptive names**: Clear test purpose

---

## Validation Checklist

### MUST Requirements (Mandatory)

- [ ] **Testing strategy defined** (unit, integration, E2E)
- [ ] **Coverage target specified** (90%+)
- [ ] **Unit test framework documented** (Rust built-in + tokio-test)
- [ ] **Integration test setup explained** (tests/ directory)
- [ ] **E2E test framework specified** (pytest + httpx)
- [ ] **Test naming convention defined** (test_{what}_when_{condition}_then_{expected})
- [ ] **Test organization structure documented**
- [ ] **Coverage commands provided** (unit, E2E, combined)
- [ ] **CI requirements listed**

### SHOULD Requirements (Strongly Recommended)

- [ ] Mock/stub patterns documented
- [ ] Test configuration files specified (conftest.py)
- [ ] Test data fixtures location
- [ ] Best practices enumerated
- [ ] Examples for each test type

### MAY Requirements (Optional)

- [ ] Performance test guidelines
- [ ] Load test patterns
- [ ] Chaos testing approach

## Compliance Criteria

**Pass**: All MUST requirements met (9/9) + coverage ≥90%  
**Fail**: Any MUST requirement missing or coverage <90%

### Agent Instructions

When writing tests:
1. ✅ **ALWAYS follow test pyramid** (many unit, fewer integration, few E2E)
2. ✅ **ALWAYS use descriptive names** (test_{what}_when_{condition}_then_{expected})
3. ✅ **ALWAYS isolate tests** (no shared state)
4. ✅ **ALWAYS use async tests** for async code (#[tokio::test])
5. ✅ **ALWAYS aim for 90%+ coverage**
6. ✅ **ALWAYS write tests before implementation** (TDD encouraged)
7. ✅ **ALWAYS test error cases** (not just happy path)
8. ✅ **ALWAYS use fixtures** for reusable setup
9. ❌ **NEVER write flaky tests** (deterministic only)
10. ❌ **NEVER skip tests** in CI
11. ❌ **NEVER test implementation details** (test behavior)
12. ❌ **NEVER share state between tests**

### Test Writing Checklist

Before committing tests:
- [ ] Test name follows convention
- [ ] Test is isolated (no shared state)
- [ ] Test is deterministic (repeatable)
- [ ] Assertions are clear
- [ ] Error cases tested
- [ ] Async code uses #[tokio::test]
- [ ] Test runs fast (<1s for unit tests)
- [ ] Coverage increased
- [ ] All tests pass locally
- [ ] No test warnings

### Coverage Checklist

To meet 90%+ coverage:
- [ ] All public functions tested
- [ ] All error paths tested
- [ ] All branches tested
- [ ] Edge cases tested
- [ ] Integration between modules tested
- [ ] E2E critical paths tested
- [ ] Coverage report generated
- [ ] Coverage meets threshold
