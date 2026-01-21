# Architectural Compliance Lints

**Source**: dylint_lints/README.md, dylint_lints/AGENTS.md

## Overview

HyperSpot uses **custom dylint linters** to enforce architectural patterns, layer separation, and REST API conventions at **compile time**.

**Why Custom Lints**:
- Prevent architectural violations before code review
- Enforce team conventions automatically
- Catch common mistakes early (compile-time, not runtime)
- Document architecture rules as executable code

## Running Lints

```bash
# From workspace root
make dylint              # Run all lints (auto-rebuilds if changed)
make dylint-list         # Show all available lints
make dylint-test         # Test UI cases
```

Or with Python CI:
```bash
python scripts/ci.py dylint
```

## Lint Categories

### DE01xx: Contract Layer

**Purpose**: Ensure contract layer is transport-agnostic (no HTTP/serialization concerns)

**Lints**:
- ✅ **DE0101**: No Serde in Contract
  - Contract types MUST NOT derive `Serialize`/`Deserialize`
  - Reason: Contracts are transport-agnostic
  - Use DTOs in API layer for serialization

- ✅ **DE0102**: No ToSchema in Contract
  - Contract types MUST NOT derive `ToSchema` (utoipa)
  - Reason: OpenAPI is HTTP concern, not contract
  - Use DTOs with `ToSchema` in API layer

- ✅ **DE0103**: No HTTP Types in Contract
  - Contract types MUST NOT use HTTP types (`axum`, `hyper`, etc.)
  - Reason: Contracts are protocol-independent
  - HTTP types belong in API handlers

**Example Violation**:
```rust
// ❌ BAD: src/contract/user.rs
#[derive(Serialize, Deserialize)]  // Serde in contract
pub struct User { ... }

// ✅ GOOD: src/contract/user.rs
#[derive(Debug, Clone)]  // No serde
pub struct User { ... }

// ✅ GOOD: src/api/rest/dto.rs
#[derive(Serialize, Deserialize)]  // Serde in DTO
pub struct UserDto { ... }
```

### DE02xx: API Layer

**Purpose**: Ensure DTOs are properly isolated and have required derives

**Lints**:
- ✅ **DE0201**: DTOs Only in API Rest Folder
  - Types with `Dto` suffix MUST be in `api/rest/` folder
  - Prevents DTO leakage into domain/contract

- ✅ **DE0202**: DTOs Not Referenced Outside API
  - DTOs MUST NOT be used outside `api/` module
  - Ensures clean layer separation

- ✅ **DE0203**: DTOs Must Have Serde Derives
  - DTOs MUST derive `Serialize` and `Deserialize`
  - Required for JSON serialization

- ✅ **DE0204**: DTOs Must Have ToSchema Derive
  - DTOs MUST derive `ToSchema` (utoipa)
  - Required for OpenAPI generation

**Example Violation**:
```rust
// ❌ BAD: src/domain/model.rs
pub struct UserDto { ... }  // DTO outside api/rest/

// ❌ BAD: src/api/rest/dto.rs
#[derive(Debug)]  // Missing Serialize, Deserialize, ToSchema
pub struct UserDto { ... }

// ✅ GOOD: src/api/rest/dto.rs
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserDto { ... }
```

### DE05xx: Client/Gateway Layer

**Purpose**: Enforce naming conventions for plugin clients

**Lints**:
- ✅ **DE0503**: Plugin Client Suffix
  - Plugin client types MUST end with `Client` suffix
  - Example: `MyServiceClient`, not `MyService`

### DE08xx: REST API Conventions

**Purpose**: Enforce REST API best practices

**Lints**:
- ✅ **DE0801**: API Endpoint Must Have Version
  - All REST endpoints MUST include version in path
  - Pattern: `/{module}/v{version}/{resource}`
  - Example: `/users/v1/users`, not `/users`

- ✅ **DE0802**: Use OData Extension Methods
  - Pagination handlers MUST use OData extension methods
  - Use `modkit_odata::ODataQueryExt` for filtering/sorting

**Example Violation**:
```rust
// ❌ BAD: No version in path
#[utoipa::path(get, path = "/users")]
pub async fn list_users() { ... }

// ✅ GOOD: Version included
#[utoipa::path(get, path = "/users/v1/users")]
pub async fn list_users() { ... }
```

### DE09xx: GTS Layer

**Purpose**: Enforce GTS (Global Type System) patterns

**Lints**:
- ✅ **DE0901**: GTS String Pattern
  - GTS string fields MUST follow pattern conventions
  
- ✅ **DE0902**: No Schema For on GTS Structs
  - GTS types have their own schema system
  - Don't mix with utoipa `ToSchema`

## Planned Lints (TODO)

### DE03xx: Domain Layer
- Validate domain logic isolation
- Ensure no infrastructure dependencies

### DE04xx: Infrastructure Layer
- Validate repository implementations
- Ensure proper entity/model separation

### DE06xx: Module Structure
- Enforce module organization
- Validate module declaration

### DE07xx: Security
- Detect potential security issues
- Enforce Secure ORM usage

### DE10xx: Error Handling
- Validate error types
- Ensure proper error propagation

### DE11xx: Testing
- Enforce test naming conventions
- Validate test coverage patterns

### DE12xx: Documentation
- Ensure public items have doc comments
- Validate doc example compilation

## Adding New Lints

When architectural rules emerge, convert them to lints:

1. **Create lint**: `cd dylint_lints && cargo dylint new de0xxx_lint_name`
2. **Implement check**: Define what the lint detects
3. **Add examples**: Create good/bad examples in `ui/`
4. **Test**: Run `make dylint-test`
5. **Document**: Add to this spec and README

## Best Practices

- ✅ **Fix lint violations immediately** (don't accumulate technical debt)
- ✅ **Run lints in CI** (fail build on violations)
- ✅ **Add lints for recurring code review feedback**
- ✅ **Keep lints simple and focused** (one rule per lint)
- ✅ **Provide clear error messages** with examples
- ❌ **Don't disable lints** without architecture team approval
- ❌ **Don't use `#[allow(...)]`** unless absolutely necessary

## CI Integration

```bash
# In CI pipeline
python scripts/ci.py dylint

# Or with Make
make dylint

# Fails with exit code 1 on violations
```

## Troubleshooting

**"dylint library not found"**:
```bash
cd dylint_lints && cargo build --release
```

**"feature may not be used on stable"**:
- Dylint requires nightly
- `rust-toolchain.toml` in `dylint_lints/` sets this automatically

**Lint not triggering**:
- Check file path matches pattern
- Verify lint is registered in `lib.rs`
- Rebuild: `cd dylint_lints && cargo build --release`

**Changes not reflected**:
- Use `make dylint` (auto-rebuilds if sources changed)

## Benefits

**Compile-Time Safety**:
- Catch violations before code review
- No runtime overhead
- Impossible to bypass (fails build)

**Living Documentation**:
- Lints document architecture rules
- Examples show correct patterns
- Self-validating (UI tests)

**Team Alignment**:
- Consistent code style
- Reduced code review churn
- Onboarding aid (violations explain rules)

## Reference

- Complete lint catalog: `dylint_lints/README.md`
- Lint development guide: `dylint_lints/AGENTS.md`
- Example lints: `dylint_lints/de01_contract_layer/`, `dylint_lints/de02_api_layer/`

---

## Validation Checklist

### MUST Requirements (Mandatory)

- [ ] **Custom lints documented** (purpose, categories)
- [ ] **Running commands specified** (`make dylint`, `python scripts/ci.py dylint`)
- [ ] **Lint categories defined** (DE01xx, DE02xx, DE05xx, DE08xx, DE09xx)
- [ ] **Contract layer lints enforced** (no serde/ToSchema/HTTP types)
- [ ] **API layer lints enforced** (DTOs in api/rest/, required derives)
- [ ] **REST convention lints enforced** (versioning, OData)
- [ ] **CI integration configured**
- [ ] **Development instructions provided** (adding new lints)
- [ ] **Troubleshooting section included**
- [ ] **Benefits documented** (compile-time safety, living documentation)

### SHOULD Requirements (Strongly Recommended)

- [ ] UI tests for each lint
- [ ] Examples of violations and fixes
- [ ] Lint development guide detailed
- [ ] Planned lints roadmap
- [ ] Performance considerations documented

### MAY Requirements (Optional)

- [ ] Auto-fix suggestions
- [ ] IDE integration instructions
- [ ] Custom lint templates
- [ ] Lint metrics/statistics

## Compliance Criteria

**Pass**: All MUST requirements met (10/10) + all lints pass  
**Fail**: Any MUST requirement missing or lint violations present

### Agent Instructions

When writing code:
1. ✅ **ALWAYS run dylint** before committing (`make dylint` or `python scripts/ci.py dylint`)
2. ✅ **ALWAYS fix lint violations** (compile-time enforcement)
3. ✅ **ALWAYS follow contract layer rules** (no serde in contracts)
4. ✅ **ALWAYS keep DTOs in api/rest/** (DE0201, DE0202)
5. ✅ **ALWAYS add required derives** to DTOs (Serialize, Deserialize, ToSchema)
6. ✅ **ALWAYS version endpoints** (DE0801)
7. ✅ **ALWAYS use OData patterns** when applicable (DE0802)
8. ✅ **ALWAYS check lint categories** for your changes
9. ❌ **NEVER disable lints** without architecture team approval
10. ❌ **NEVER use #[allow(...)]** for architectural lints
11. ❌ **NEVER bypass lint checks** in CI
12. ❌ **NEVER commit with lint violations**

### Lint Compliance Checklist

Before committing code:
- [ ] Ran `make dylint` or `python scripts/ci.py dylint`
- [ ] All lint violations resolved
- [ ] No serde/ToSchema in contracts (DE01xx)
- [ ] DTOs only in api/rest/ (DE0201)
- [ ] DTOs have Serialize/Deserialize/ToSchema (DE0203, DE0204)
- [ ] Endpoints versioned (DE0801)
- [ ] OData extension methods used (DE0802)
- [ ] No #[allow(...)] for architectural lints
- [ ] CI will pass dylint check
- [ ] Architecture rules followed

### Adding New Lints Checklist

When creating new architectural rules:
- [ ] Rule is architectural (not stylistic)
- [ ] Rule enforces team decision
- [ ] Rule prevents common mistakes
- [ ] Lint category assigned (DE0xxx)
- [ ] Lint pass implemented (pre-expansion, early, or late)
- [ ] UI tests created (good/bad examples)
- [ ] Documentation updated (README.md)
- [ ] Added to this spec
- [ ] Tested with `make dylint-test`
- [ ] CI integration verified
