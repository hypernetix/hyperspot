//! Domain service for the single-tenant resolver plugin.

use modkit_macros::domain_model;

/// Single-tenant resolver service.
///
/// Zero-configuration service for single-tenant deployments.
/// No state is needed - all tenant info is derived from the security context.
#[domain_model]
pub struct Service;
