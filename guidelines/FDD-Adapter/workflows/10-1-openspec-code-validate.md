# Validate Code Against OpenSpec (Hyperspot Adapter)

**Extends**: `../../FDD/workflows/10-1-openspec-code-validate.md`
**Purpose**: Add Hyperspot-specific validation rules (ModKit conventions, GTS compliance)

---

## AI Agent Instructions

Run `../../FDD/workflows/10-1-openspec-code-validate.md` with these additional validation steps:

### Additional Validation: ModKit Conventions

**After Section 2 (Validate Code Against Specifications)**:

Run ModKit-specific validation:

```bash
# Check for ModKit conventions compliance
cargo clippy --package <module-name> -- -D warnings

# Verify module structure follows NEW_MODULE.md guidelines
# Check for:
# - Proper DDD-light layering (api/, domain/, infra/)
# - utoipa::ToSchema on all DTOs
# - RFC-9457 Problem usage (or custom ProblemDetails with justification)
# - Proper module.rs with #[modkit::module]
```

**Validation Criteria**:
- [ ] All DTOs have `#[derive(utoipa::ToSchema)]`
- [ ] Error responses follow RFC-9457 or have documented justification
- [ ] Module structure follows `guidelines/NEW_MODULE.md`
- [ ] No violations of `guidelines/DNA/REST/API.md` conventions

**Report Format**:
```markdown
### ModKit Conventions Compliance

**Score**: X/100

**Issues**:
- Missing ToSchema on DTOs: [list files]
- Custom error format (non-blocking): [justification required]

**Status**: PASS / FAIL
```

---

### Additional Validation: GTS Specification Compliance

**After ModKit validation**:

**Requirement**: If module uses GTS identifiers, validate against GTS spec (v0.7)

**Detection**:
```bash
# Check if code uses GTS identifiers
grep -r "gts\." <module-src-path> --include="*.rs" | grep -v "//.*gts\."
```

**If GTS identifiers found**:

Run GTS-specific validation:

**Validation Criteria**:
1. **Identifier Format** (Section 2.1 of GTS spec):
   - [ ] Type identifiers end with `~`
   - [ ] Instance identifiers lack trailing `~`
   - [ ] Segments use only lowercase letters, digits, underscores
   - [ ] No hyphens in segments
   - [ ] Version format: `v<MAJOR>[.<MINOR>]`

2. **Chaining Rules** (Section 2.2):
   - [ ] Well-known instances use chained format: `gts.type~vendor.instance`
   - [ ] Anonymous instances use UUID + separate type field
   - [ ] No single-segment instance identifiers

3. **Production vs Test/Mock Data**:
   - [ ] Production code uses compliant identifiers
   - [ ] Test/mock data follows same rules

**Commands**:
```bash
# Validate GTS identifier format in code
# Check for common violations:
grep -r "gts\.[^~]*~[0-9a-f-]\{36\}" <module-src-path> --include="*.rs"
# This pattern catches: gts.type~{uuid} (WRONG - should be just UUID)

# Check for hyphens in segments (WRONG)
grep -r "gts\.[^~]*-[^~]*" <module-src-path> --include="*.rs"
```

**Report Format**:
```markdown
### GTS Specification Compliance (v0.7)

**Score**: X/100

**Identifier Format**: PASS / FAIL
- Type/Instance distinction: [status]
- Segment validation: [status]
- Chaining rules: [status]

**Issues Found**:
1. [Location]: [Issue description]
2. [Location]: [Issue description]

**Recommendations**:
- Use anonymous instance pattern for runtime objects: `id: uuid, type: "gts.type~"`
- Replace hyphens with underscores in segments
- Use chained format for well-known instances

**Status**: PASS / FAIL
```

---

## Modified Output Format

**Add to Section 5 (Output Validation Results)**:

```markdown
## Hyperspot-Specific Validation

### ModKit Conventions
- Score: X/100
- Issues: [list]
- Status: PASS/FAIL

### GTS Specification Compliance
- Score: X/100  
- Format: PASS/FAIL
- Chaining: PASS/FAIL
- Status: PASS/FAIL

## Overall Status

**OpenSpec Compliance**: X/100
**ModKit Compliance**: X/100
**GTS Compliance**: X/100 (if applicable)

**OVERALL**: PASS / FAIL
```

---

## Pass Criteria

**Modified from base workflow**:

- [ ] All base workflow checks pass (100/100)
- [ ] ModKit conventions: ≥85/100 (non-blocking if justified)
- [ ] GTS compliance: ≥90/100 (blocking if GTS used)

**Blocking Issues**:
- Missing OpenSpec requirements
- GTS format violations in production code
- Critical ModKit violations (no ToSchema, wrong structure)

**Non-Blocking Issues** (require justification):
- Custom error format instead of modkit::Problem
- ModKit convention deviations (85-100 range)
- GTS violations in test/mock data only

---

## References

- **ModKit Guide**: `../../docs/MODKIT_UNIFIED_SYSTEM.md`
- **Module Guidelines**: `../../guidelines/NEW_MODULE.md`
- **GTS Specification**: `../../guidelines/GTS/README.md`
- **REST API Conventions**: `../../guidelines/DNA/REST/API.md`
