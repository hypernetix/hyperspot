# Feature Status Validation - MANDATORY Specification

**Version**: 1.0  
**Status**: REQUIRED for all feature lifecycle workflows  
**Last Updated**: 2026-01-09

---

## Overview

**Critical**: Features marked as `IMPLEMENTED` MUST NOT contain incomplete business logic, but MAY contain architectural delegation points.

**Why**: 
- Status must reflect feature completion scope
- Architectural stubs (routing, delegation) are valid design patterns
- Business logic TODOs indicate incomplete implementation
- Clear distinction prevents misleading documentation

---

## MANDATORY Rule

### Feature Status ↔ Implementation Consistency

A feature can be marked `✅ IMPLEMENTED` if it delivers its **defined scope**, even with delegation points.

```
✅ IMPLEMENTED status REQUIRES:
  ├─ All planned business logic complete
  ├─ All tests passing (no #[ignore] without justification)
  ├─ No TODO/FIXME in domain/service layers
  ├─ No unimplemented!() in business logic
  └─ Documentation reflects actual state
  
✅ IMPLEMENTED status ALLOWS:
  ├─ Architectural delegation points (routing layers)
  ├─ NOT_IMPLEMENTED in gateway/proxy handlers
  ├─ Trait default impl with unimplemented!()
  └─ Public API contracts awaiting downstream features
```

---

## Feature Phases (Optional) ↔ Status Consistency

**Purpose**: Allow partial delivery visibility inside a single feature while keeping feature-level status truthful.

**Phase ID Format**: `ph-{N}`

**Rule**:
- A feature marked `✅ IMPLEMENTED` MUST NOT have any phase in 🔄 IN_PROGRESS or ⏳ NOT_STARTED.
- If phases are used, each phase marked ✅ IMPLEMENTED MUST be traceable to code via phase postfixes on feature-scoped tags.

**Code Tagging**:
- Standalone phase tags MUST NOT be used.
- Phase MUST be encoded as a postfix on feature-scoped tags:
  - `@fdd-change:{id}:ph-{N}`, `@fdd-flow:{id}:ph-{N}`, `@fdd-algo:{id}:ph-{N}`, `@fdd-state:{id}:ph-{N}`, `@fdd-req:{id}:ph-{N}`, `@fdd-test:{id}:ph-{N}`

---

## Distinguishing Architectural Stubs vs Incomplete Work

### ✅ ALLOWED: Architectural Stubs

These patterns are **VALID** in `IMPLEMENTED` features:

#### 1. Routing/Gateway Layer Delegation

```rust
// ✅ VALID: GTS Core routing layer delegates to domain features
pub async fn get_entity(
    Path(id): Path<String>,
    Extension(router): Extension<Arc<GtsCoreRouter>>,
) -> Result<Json<GtsEntityDto>, Problem> {
    match router.route(&id) {
        Ok(Some(_handler_id)) => Err(Problem::new(
            StatusCode::NOT_IMPLEMENTED,
            "Not Implemented",
            "GTS Core routing is ready, but domain feature delegation is not implemented yet",
        )),
        // ... routing errors
    }
}
```

**Why valid**: Feature scope is "routing layer" - delegation to other features is expected.

---

#### 2. Trait Default Implementations

```rust
// ✅ VALID: Public trait with default stubs for optional methods
pub trait DomainFeature {
    fn handle_create(&self, data: Value) -> Result<Entity>;
    
    // Optional - implementors can override
    fn handle_batch_create(&self, items: Vec<Value>) -> Result<Vec<Entity>> {
        unimplemented!("Batch operations not required for minimal implementation")
    }
}
```

**Why valid**: Trait defines contract, default impl is intentional design.

---

#### 3. Public API Contracts (SDK Crate)

```rust
// ✅ VALID: SDK trait awaiting implementation by consumers
#[async_trait]
pub trait AnalyticsClient {
    async fn execute_query(&self, query: Query) -> Result<QueryResult>;
    async fn get_datasource(&self, id: &str) -> Result<Datasource>;
}
```

**Why valid**: SDK provides contract, implementation is consumer responsibility.

---

### ❌ FORBIDDEN: Incomplete Work

These patterns are **INVALID** in `IMPLEMENTED` features:

#### 1. Business Logic TODOs

```rust
// ❌ INVALID: Domain service with incomplete logic
pub async fn create_entity(&self, data: Value) -> Result<Entity> {
    // TODO: Add validation
    // TODO: Check permissions
    // TODO: Persist to database
    Ok(Entity::default())
}
```

**Why invalid**: Core business logic is incomplete.

---

#### 2. Unimplemented Domain Methods

```rust
// ❌ INVALID: Service method not implemented
impl QueryService {
    pub async fn execute(&self, query: Query) -> Result<QueryResult> {
        unimplemented!("Query execution logic pending")
    }
}
```

**Why invalid**: Feature claims to provide query execution but doesn't.

---

#### 3. Ignored/Placeholder Tests

```rust
// ❌ INVALID: Tests that don't actually test
#[test]
#[ignore]  // No justification
fn test_query_execution() {
    // TODO: Implement when query engine is ready
}

#[test]
fn test_validation() {
    assert!(true, "placeholder");
}
```

**Why invalid**: Tests don't verify actual behavior.

---

#### 4. Missing Error Handling

```rust
// ❌ INVALID: Swallowing errors or panicking
pub async fn process(&self, data: Value) -> Result<()> {
    let result = self.do_work(data).unwrap();  // TODO: Handle errors
    Ok(())
}
```

**Why invalid**: Production code with panics/unwraps.

---

## Validation Checklist

Before marking feature as `✅ IMPLEMENTED`:

### Business Logic Review
- [ ] All domain services implement their contracts
- [ ] No `TODO`/`FIXME` in `domain/` or `service/` code
- [ ] No `unimplemented!()` in business logic
- [ ] Error handling complete (no bare `unwrap()`)

### Tests Review
- [ ] All tests pass
- [ ] No `#[ignore]` without documented reason
- [ ] Tests verify actual behavior (not placeholders)
- [ ] Integration tests cover main flows

### Architectural Stubs (Optional)
- [ ] If routing/gateway: delegation points documented
- [ ] If SDK: trait contracts defined and documented
- [ ] NOT_IMPLEMENTED responses include clear messages

### Documentation
- [ ] DESIGN.md reflects actual implementation state
- [ ] Known limitations documented
- [ ] Delegation points clearly marked

---

## Validation Commands

### 1. Find Business Logic TODOs

Search the feature domain/service code for incomplete work markers:
- `TODO`
- `FIXME`
- `XXX`
- `HACK`

**Expected**: No matches in business logic for `✅ IMPLEMENTED` features.

---

### 2. Find Unimplemented Business Logic

Search the feature code (domain/service/infra) for incomplete implementation markers:
- `unimplemented!`
- `todo!`

**Expected**: No matches, except permitted trait defaults (if adapter allows).

---

### 3. Find Ignored Tests

Search the feature test code for ignored tests:
- `#[ignore]`

**Expected**: Every ignored test has documented justification.

---

### 4. Find Placeholder Tests

Search the feature test code for placeholder assertions:
- `assert!(true)`
- `assert_eq!(1, 1)`

**Expected**: No placeholder tests.

---

## Example: GTS Core Feature

### Scope: Thin routing layer

**Status**: `✅ IMPLEMENTED` is **VALID** because:

✅ Routing table works  
✅ GTS ID parsing works  
✅ Handler delegation logic works  
✅ Tests cover routing behavior  
✅ NOT_IMPLEMENTED responses are **architectural** (awaiting domain features)  

**Code**: Handlers return `Problem::new(StatusCode::NOT_IMPLEMENTED, "domain feature delegation is not implemented yet")`

**Interpretation**: Feature delivers routing layer. Domain features are **out of scope** for this feature.

---

## Example: Query Execution Feature

### Scope: Execute analytical queries

**Status**: `✅ IMPLEMENTED` is **INVALID** if:

❌ Query parser has `TODO: Add JOIN support`  
❌ Execution engine has `unimplemented!("aggregations")`  
❌ Tests are `#[ignore]` with "will test later"  

**Fix**: 
- Remove TODOs, implement or document as future enhancement
- Implement basic aggregations or remove from scope
- Enable and complete tests

---

## Migration Guide

If feature currently has misleading IMPLEMENTED status:

1. **Audit**: Run validation commands above
2. **Categorize**: Separate architectural stubs from incomplete work
3. **Decision**:
   - If only architectural stubs → Document in DESIGN.md, keep IMPLEMENTED
   - If incomplete work → Change status to IN_PROGRESS
4. **Cleanup**: Remove TODOs or complete work
5. **Document**: Update DESIGN.md with actual scope

---

## References

- Feature lifecycle: `@/guidelines/FDD/requirements/workflow-selection.md`
- Feature validation: `@/guidelines/FDD/workflows/feature-validate.md`
- Code conventions: `@/guidelines/FDD-Adapter/specs/conventions.md`
- ModKit patterns: `@/docs/MODKIT_UNIFIED_SYSTEM.md`

---

## Questions for Feature Author

When unsure about IMPLEMENTED status, ask:

1. **Scope**: Does this feature define an interface or implement logic?
2. **Consumers**: Can downstream features use this as-is?
3. **Tests**: Do tests verify the feature's actual purpose?
4. **TODOs**: Are TODOs architectural notes or incomplete work?

If unsure → Mark as `🔄 IN_PROGRESS` until clarified.
