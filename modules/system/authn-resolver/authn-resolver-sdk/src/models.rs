//! Domain models for the `AuthN` resolver module.

use modkit_security::SecurityContext;

/// Result of a successful authentication.
///
/// Contains the validated `SecurityContext` with identity information
/// populated from the token (`subject_id`, `subject_tenant_id`, `token_scopes`, etc.).
#[derive(Debug, Clone)]
pub struct AuthenticationResult {
    /// The validated security context with identity fields populated.
    ///
    /// Contains:
    /// - `subject_id` — The authenticated user/service ID
    /// - `subject_tenant_id` — The subject's home tenant
    /// - `token_scopes` — Token capability restrictions
    /// - `bearer_token` — Original token for PDP forwarding
    /// - `tenant_id` — Context tenant (may be set by `AuthN` or later by middleware)
    pub security_context: SecurityContext,
}
