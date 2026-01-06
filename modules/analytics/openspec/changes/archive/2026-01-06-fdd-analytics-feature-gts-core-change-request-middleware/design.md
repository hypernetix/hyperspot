# Design Reference

**Source**: [Feature DESIGN.md](../../architecture/features/feature-gts-core/DESIGN.md)

This change implements Section F requirement `fdd-analytics-feature-gts-core-req-middleware` as defined in the Feature DESIGN.md.

## Implementation Scope

**From Section B**: Flow 4 - GTS Core Routes CRUD Operations (middleware chain steps)

**From Section C**: Algorithm 2 - Query Optimization Validator

**From Section E**: 
- Access Control (JWT validation, SecurityCtx)
- OData Query Parameters (parsing and validation)

## Key Design Decisions

1. **Middleware Order**: JWT → SecurityCtx → OData → Routing (fixed order for security)
2. **Platform Helpers**:
   - `modkit-auth::AuthDispatcher` for JWT validation
   - `modkit-security::SecurityCtx` for tenant isolation
   - `modkit-odata` for OData parameter parsing
3. **Query Optimization**: Validate $filter against indexed fields list to prevent full table scans
4. **Error Handling**: RFC 7807 Problem Details for all errors (401, 400)

## Testing Strategy

- Unit tests for each middleware component independently
- Integration tests for complete middleware chain
- Edge cases for authentication and query validation failures
