# Change: Response Processing

**Status**: ✅ COMPLETED  
**Created**: 2026-01-06  
**Started**: 2026-01-06  
**Completed**: 2026-01-06

**Feature**: [GTS Core](../../architecture/features/feature-gts-core/DESIGN.md)  
**Change**: [fdd-analytics-feature-gts-core-change-response-processing](../../architecture/features/feature-gts-core/DESIGN.md#g-implementation-plan)  
**Implements**: [`fdd-analytics-feature-gts-core-req-tolerant-reader`](../../architecture/features/feature-gts-core/DESIGN.md#fdd-analytics-feature-gts-core-req-tolerant-reader), [`fdd-analytics-feature-gts-core-req-routing`](../../architecture/features/feature-gts-core/DESIGN.md#fdd-analytics-feature-gts-core-req-routing)

---

## Why

This change implements the Tolerant Reader pattern for GTS Core response processing. Building on the routing infrastructure (Change 1) and request middleware (Change 2), this change adds intelligent field handling that distinguishes between client-provided fields, server-managed fields, computed fields, and secrets. This is essential for secure API operations and proper field semantics across create, read, and update operations.

Without this response processing, the system cannot:
- Prevent clients from overriding system-managed fields (id, type, tenant, timestamps)
- Hide sensitive information (API keys, credentials) in responses
- Add computed fields (asset_path, derived values) on read operations
- Enforce proper JSON Patch semantics (restrict to /entity/* paths)

## What Changes

This change implements:

- **Tolerant Reader pattern**: Field categorization (client-provided, server-managed, computed, secrets)
- **Field filtering on read**: Remove secrets and credentials from GET/LIST responses
- **System field protection**: Ignore client attempts to set id, type, registered_at, tenant fields
- **Computed field injection**: Add server-computed fields (asset_path, etc.) to responses
- **JSON Patch path restrictions**: Only allow /entity/* paths in PATCH operations
- **OData field projection**: Apply $select parameter to filter response fields

## Impact

**Affected specs**:
- `openspec/specs/fdd-analytics-feature-gts-core/spec.md` (ADDED: tolerant-reader requirement)

**Affected code**:
- `modules/analytics/analytics/src/domain/gts_core/field_handler.rs` (new)
- `modules/analytics/analytics/src/api/rest/gts_core/response_processor.rs` (new)
- `modules/analytics/analytics/src/api/rest/gts_core/handlers.rs` (modified: add response processing)

**Dependencies**: 
- `fdd-analytics-feature-gts-core-change-routing-infrastructure` (Change 1 - COMPLETED)
- `fdd-analytics-feature-gts-core-change-request-middleware` (Change 2 - COMPLETED)

**Breaking changes**: None (enhancement to existing endpoints, maintains backward compatibility)
