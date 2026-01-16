# Domain Model Specification

**Source**: libs/modkit/src/contracts.rs, module source code

## Technology

**Rust Type System** + **GTS (Global Type System)**

## Format

Domain models are defined as Rust `struct` and `enum` types with derives:
- `serde::Serialize` / `serde::Deserialize` for JSON serialization
- `utoipa::ToSchema` for OpenAPI schema generation
- Custom derives from `modkit-macros` as needed

## Locations

**Core Types**: `libs/modkit/src/contracts.rs`  
**Module Contracts**: `modules/{module}/src/contract/`  
**SDK Types**: `modules/{module}/{module}-sdk/src/`

## Type Reference Syntax

Use fully-qualified Rust paths when referencing domain types:

```rust
// Example references
modkit::contracts::ModuleInfo
file_parser::contract::FileParserInfoDto
```

## GTS Integration

**GTS (Global Type System)** enables extensible type definitions:
- Custom schema extensions
- Plugin-provided types
- Tenant-specific type customizations

**GTS Location**: Integrated in modkit framework

## Validation

**Command**: `cargo check --workspace`  
**Expected**: No type errors, all contracts compile

## Example

```rust
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModuleInfo {
    pub name: String,
    pub version: String,
    pub status: ModuleStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ModuleStatus {
    Active,
    Inactive,
    Error,
}
```

## GTS Validation (MANDATORY)

**Tool**: `gts` CLI from https://github.com/GlobalTypeSystem/gts-rust

**Purpose**: Validate GTS entities, identifiers, and schemas to ensure compliance with Global Type System standards.

**Installation**:
```bash
cargo install gts
```

**Key Commands**:

### Validate GTS ID Format
```bash
gts validate-id <GTS_ID>
```

### Parse GTS ID Components
```bash
gts parse-id <GTS_ID>
```

### Validate Instance Against Schema
```bash
gts validate-instance --schema <schema_file> --instance <instance_file>
```

### Generate Schemas from Rust Code
```bash
gts generate-from-rust --input <source_dir> --output <schema_dir>
```

**Annotations for Schema Generation**:
```rust
use gts::struct_to_gts_schema;

#[struct_to_gts_schema]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ModuleInfo {
    pub name: String,
    pub version: String,
    pub status: ModuleStatus,
}
```

### Validation Workflow

**Before committing domain types**:
1. Annotate types with `#[struct_to_gts_schema]`
2. Generate GTS schemas: `gts generate-from-rust --input libs/modkit/src --output gts_schemas/`
3. Validate IDs: `gts validate-id <entity_id>`
4. Validate instances: `gts validate-instance --schema gts_schemas/<type>.json --instance instances/<entity>.json`

**CI Integration**:
- GTS validation MUST pass in CI pipeline
- Schema generation MUST be automated
- ID format validation MUST be enforced

**Common Commands**:
```bash
# List all entities
gts list --path gts_schemas/

# Query entities
gts query --path gts_schemas/ --expr "type == 'Module'"

# Check schema compatibility
gts compatibility --schema1 v1/module.json --schema2 v2/module.json

# Get attribute value
gts attr --entity <entity_file> --path "status"
```

## Traceability

All feature DESIGN.md files must reference domain types using Rust paths:
- Format: `modkit::contracts::TypeName`
- Example: `@DomainModel.modkit::contracts::ModuleInfo`

---

## Validation Checklist

### MUST Requirements (Mandatory)

- [ ] **Technology specified** (Rust + GTS)
- [ ] **Format defined** (struct/enum with derives)
- [ ] **Locations documented** (libs, modules, SDK paths)
- [ ] **Required derives present**: `Serialize`, `Deserialize`, `ToSchema`
- [ ] **GTS annotations added** (`#[struct_to_gts_schema]`)
- [ ] **GTS tool installed** (`cargo install gts`)
- [ ] **GTS schemas generated** from Rust types
- [ ] **GTS IDs validated** (format compliance)
- [ ] **GTS instances validated** against schemas
- [ ] **Type reference syntax defined** (fully-qualified Rust paths)
- [ ] **Validation command provided** (`cargo check` + `gts validate-*`)
- [ ] **Traceability format specified** for DESIGN.md

### SHOULD Requirements (Strongly Recommended)

- [ ] GTS integration explained
- [ ] Examples include all common types (struct, enum)
- [ ] Debug derive included for development
- [ ] Clone derive for convenience

### MAY Requirements (Optional)

- [ ] Additional derive macros documented
- [ ] Custom validation logic
- [ ] Type aliases for complex types

## Compliance Criteria

**Pass**: All MUST requirements met (12/12) + GTS validation passes  
**Fail**: Any MUST requirement missing, types don't compile, or GTS validation fails

### Agent Instructions

When defining domain types:
1. ✅ **ALWAYS use Rust structs/enums** (no other type systems)
2. ✅ **ALWAYS derive Serialize + Deserialize** for JSON
3. ✅ **ALWAYS derive ToSchema** for OpenAPI
4. ✅ **ALWAYS annotate with #[struct_to_gts_schema]** for GTS
5. ✅ **ALWAYS use snake_case for JSON** (#[serde(rename_all = "snake_case")])
6. ✅ **ALWAYS place in correct location** (libs/modkit or modules/{module}/contract)
7. ✅ **ALWAYS use fully-qualified paths** in references
8. ✅ **ALWAYS validate with cargo check**
9. ✅ **ALWAYS generate GTS schemas** (gts generate-from-rust)
10. ✅ **ALWAYS validate GTS IDs** (gts validate-id)
11. ✅ **ALWAYS validate instances** (gts validate-instance)
12. ✅ **ALWAYS run GTS validation in CI**
13. ❌ **NEVER use serde in SDK contracts** (transport-agnostic)
14. ❌ **NEVER define types outside designated locations**
15. ❌ **NEVER skip required derives**
16. ❌ **NEVER skip GTS annotations**
17. ❌ **NEVER commit without GTS validation**

### Type Definition Checklist

Before creating domain types:
- [ ] Determined correct location (core vs module)
- [ ] Added all required derives (Serialize, Deserialize, ToSchema)
- [ ] Added GTS annotation (#[struct_to_gts_schema])
- [ ] Used snake_case for JSON serialization
- [ ] Ran `cargo check --workspace`
- [ ] Generated GTS schemas (gts generate-from-rust)
- [ ] Validated GTS IDs (gts validate-id)
- [ ] Validated instances (gts validate-instance)
- [ ] Added doc comments
- [ ] Referenced in DESIGN.md with full path
- [ ] GTS validation passes in CI
