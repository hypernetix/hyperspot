# Design Reference

**Source**: [Feature DESIGN.md](../../architecture/features/feature-gts-core/DESIGN.md)

This change implements Section F requirement `fdd-analytics-feature-gts-core-req-tolerant-reader` as defined in the Feature DESIGN.md.

## Implementation Scope

**From Section C**: Algorithm 3 - Tolerant Reader Pattern

**From Section E**: 
- Tolerant Reader Pattern (field categorization)
- Field semantics across operations (create, read, update)

## Key Design Decisions

1. **Field Categories**:
   - **Client-provided**: Writable fields in /entity/* path
   - **Server-managed**: id, type, registered_at, updated_at, tenant (read-only)
   - **Computed**: asset_path, derived values (response-only, not persisted)
   - **Secrets**: credentials, api_keys (never returned, stored encrypted)

2. **Implementation Approach**:
   - Request processing: Filter out system fields before domain logic
   - Response processing: Add computed fields, remove secrets
   - JSON Patch: Validate paths start with /entity/
   - OData $select: Apply after secret filtering

3. **Platform Integration**:
   - Use existing `SecurityCtx` for tenant field
   - Integrate with OData middleware from Change 2
   - Follow RFC 7807 Problem Details for errors

## Testing Strategy

- Unit tests for each field category independently
- Integration tests for complete request/response cycle
- Edge cases for nested fields and path validation
