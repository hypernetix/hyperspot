use crate::{claims::Claims, traits::ScopeBuilder};
use modkit_security::AccessScope;

/// Simple scope builder that converts tenant claims to AccessScope
#[derive(Debug, Clone, Default)]
pub struct SimpleScopeBuilder;

impl ScopeBuilder for SimpleScopeBuilder {
    fn tenants_to_scope(&self, claims: &Claims) -> AccessScope {
        if claims.tenants.is_empty() {
            // No explicit tenants - deny all by default
            AccessScope::default()
        } else {
            AccessScope::tenants_only(claims.tenants.clone())
        }
    }
}

