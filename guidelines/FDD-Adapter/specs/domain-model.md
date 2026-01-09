# Domain Model Specification

**Technology**: GTS (Global Type System) + JSON Schema

**Specification**: `guidelines/GTS`

**⚠️ CRITICAL**: Before using GTS, read the specification at `guidelines/GTS/README.md`

---

## GTS Identifier Rules

**⚠️ STRICTLY ENFORCED**:
- ALL segments MUST be lowercase (a-z, 0-9, underscore only)
- NO uppercase letters allowed in identifiers
- Format: `gts.vendor.package.namespace.type.v<MAJOR>[.<MINOR>]`
- Type names: use snake_case (e.g., `user_profile`, NOT `UserProfile`)

**Example**:
- ✅ Valid: `gts.ainetx.hyperspot.users.user_profile.v1`
- ❌ Invalid: `gts.ainetx.hyperspot.users.UserProfile.v1`

---

## Location

**Distributed across modules** following modkit architecture:
- Type definitions: `modules/*/src/domain/`
- Type registry: `modules/types-registry/`
- Schema validation: Via GTS library and JSON Schema

---

## DML Syntax

Use GTS reference format in JSON Schema `$ref` fields:
- Schema $id: `gts.vendor.package.namespace.type.v<version>`
- Schema $ref: `gts://gts.vendor.package.namespace.type.v<version>`
- All identifiers MUST be lowercase with underscores for word separation

---

## Validation

```bash
# Validate JSON schemas per JSON Schema draft-07 specification
# Verify GTS identifiers follow lowercase-only format
cargo test --package types-registry --lib
cargo run --package types-registry -- validate
```

---

## Architecture Pattern

Modkit system with strict layer separation:
- **Contract layer** (`-contracts`, `-sdk`): Pure domain types, GTS structs, no HTTP/serde
- **Domain layer**: Business logic, services
- **API layer** (`api/rest/`, `api/grpc/`): DTOs with serde, HTTP types allowed
- **Infra layer**: Database, external integrations
