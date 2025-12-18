# Change: Add Types Registry Module - Phase 1.1

## Why

HyperSpot needs a centralized registry for GTS (Global Type System) entities to enable type-safe, extensible data management across the platform. GTS provides human-readable, globally unique identifiers for data types (JSON Schemas) and instances (JSON objects), enabling cross-vendor interoperability, schema validation, and fine-grained access control.

Phase 1.1 establishes the foundational contracts and in-memory storage with core CRUD operations and standard GTS operations from [gts-rust](https://github.com/GlobalTypeSystem/gts-rust).

## What Changes

### New Capability: types-registry

**SDK API (`types-registry-sdk`):**
- `TypesRegistryApi` trait with 3 core methods:
  - `register` — Register a GTS entity (well-known type or instance)
  - `list` — List GTS entities with filtering/pagination
  - `get` — Get a GTS entity by ID

**Standard GTS Operations (from gts-rust):**
- OP#1 - ID Validation: Verify identifier syntax using regex patterns
- OP#2 - ID Extraction: Fetch identifiers from JSON objects or JSON Schema documents
- OP#3 - ID Parsing: Decompose identifiers into constituent parts (vendor, package, namespace, type, version)
- OP#4 - ID Pattern Matching: Match identifiers against patterns containing wildcards
- OP#5 - ID to UUID Mapping: Generate deterministic UUIDs from GTS identifiers
- OP#6 - Schema Validation: Validate object instances against their corresponding schemas
- OP#7 - Relationship Resolution: Load schemas and instances, resolve inter-dependencies, detect broken references
- OP#8 - Compatibility Checking: Verify schema compatibility across MINOR versions (backward, forward, full)
- OP#9 - Version Casting: Transform instances between compatible MINOR versions
- OP#10 - Query Execution: Filter identifier collections using GTS query language
- OP#11 - Attribute Access: Retrieve property values using attribute selector (@)

**Module Implementation (`types-registry`):**
- In-memory storage for GTS entities (Phase 1.1 scope)
- Optional validation on registration (validate GTS references)
- Local client implementing SDK trait
- Domain service with business logic

## Impact

- **Affected specs**: New `types-registry` capability
- **Affected code**: New module under `modules/types-registry/`
- **Dependencies**: `gts-rust` library for GTS operations

## References

- [GTS Specification](https://github.com/GlobalTypeSystem/gts-spec)
- [gts-rust Implementation](https://github.com/GlobalTypeSystem/gts-rust)
- [Issue #63](https://github.com/hypernetix/hyperspot/issues/63)
