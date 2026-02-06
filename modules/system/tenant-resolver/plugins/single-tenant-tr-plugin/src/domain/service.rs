//! Domain service for the single-tenant resolver plugin.

/// Single-tenant resolver service.
///
/// Zero-configuration service for single-tenant deployments.
/// No state is needed - all tenant info is derived from the security context.
pub struct Service;
