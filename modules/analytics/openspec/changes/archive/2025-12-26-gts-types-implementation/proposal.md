# Proposal: GTS Type Definitions

## Overview

Create JSON Schema definitions for all GTS types defined in the Analytics module's `DESIGN.md`. This is preparatory work to establish the type system before implementing the registry service.

## Motivation

### Why This Change?

The Analytics module needs formal type definitions:
1. **Establish contracts** - Define data structures using JSON Schema (draft 2020-12)
2. **Enable validation** - Provide schemas for validating instances
3. **Support documentation** - Self-documenting type system
4. **Prepare for service** - Foundation for future registry service implementation

### Current State

- Domain model types exist only as documentation in `DESIGN.md`
- No formal JSON Schema definitions
- No example instances
- Cannot validate data structures

### Desired State

- All types defined as JSON Schema files
- Example instances for each type
- Organized directory structure following GTS conventions
- Ready for registry service implementation (future work)

## What Will Change

### New Directory Structure

Create `gts-types/` directory with JSON Schema definitions:

Register all base types from `DESIGN.md`:

**Schema Types:**
- `gts.hypernetix.hyperspot.ax.schema.v1~`
- `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_returns.v1~`
- `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.query_params.v1~`
- `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.template_config.v1~`
- `gts.hypernetix.hyperspot.ax.schema.v1~hypernetix.hyperspot.ax.values.v1~`

**Query Types:**
- `gts.hypernetix.hyperspot.ax.query_params.v1~`
- `gts.hypernetix.hyperspot.ax.query.v1~`
- `gts.hypernetix.hyperspot.ax.query.v1~hypernetix.hyperspot.ax.values.v1~`

**Datasource Type:**
- `gts.hypernetix.hyperspot.ax.datasource.v1~`

**Template Types:**
- `gts.hypernetix.hyperspot.ax.template.v1~`
- `gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.widget.v1~`
- `gts.hypernetix.hyperspot.ax.template.v1~hypernetix.hyperspot.ax.values_selector.v1~`

**Item Types:**
- `gts.hypernetix.hyperspot.ax.item.v1~`
- `gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.widget.v1~`
- `gts.hypernetix.hyperspot.ax.item.v1~hypernetix.hyperspot.ax.group.v1~`

**Layout Types:**
- `gts.hypernetix.hyperspot.ax.layout.v1~`
- `gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.dashboard.v1~`
- `gts.hypernetix.hyperspot.ax.layout.v1~hypernetix.hyperspot.ax.report.v1~`

**Category Type:**
- `gts.hypernetix.hyperspot.ax.category.v1~`

## Success Criteria

- [ ] All GTS types from DESIGN.md defined as JSON Schema files
- [ ] Example instances created for all types
- [ ] Directory structure follows GTS conventions
- [ ] All schemas use JSON Schema draft 2020-12
- [ ] GTS identifier format correctly used in `$id` fields
- [ ] Schema references use `$ref` with `gts://` prefix
- [ ] String fields containing GTS IDs marked with `x-gts-ref`

## Out of Scope

- Registry service implementation (future change)
- REST API endpoints (future change)
- Database schema (future change)
- Query execution logic (future change)
- Widget rendering (future change)
- Dashboard management (future change)

## Risks & Mitigation

**Risk:** Incorrect GTS identifier format
**Mitigation:** Validate against GTS spec, use examples as reference

**Risk:** Schema inheritance complexity with `allOf`
**Mitigation:** Follow GTS spec examples, keep inheritance shallow

**Risk:** Inconsistency with DESIGN.md
**Mitigation:** Regular cross-reference during creation

## Dependencies

### Required Tools
- **GTS Rust Tools** - For validation and linting of GTS schemas
  - Repository: https://github.com/GlobalTypeSystem/gts-rust
  - Installation: `cargo install gts-cli` (or build from source)
  - Used for: schema validation, identifier format checking, reference validation

### Documentation
- GTS specification: https://github.com/GlobalTypeSystem/gts-spec
- GTS spec examples: https://github.com/GlobalTypeSystem/gts-spec/tree/main/examples
- `DESIGN.md` as source of truth for type definitions
