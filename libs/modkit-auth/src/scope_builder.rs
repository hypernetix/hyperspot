use crate::{claims::Claims, traits::SecurityContextBuilder};
use modkit_security::SecurityContext;

/// Simple scope builder that converts tenant claims to `AccessScope`
#[derive(Debug, Clone, Default)]
pub struct SimpleSecurityContextBuilder;

impl SecurityContextBuilder for SimpleSecurityContextBuilder {
    fn build(&self, claims: &Claims) -> SecurityContext {
        if claims.tenant_id.is_nil() {
            // No explicit tenants - deny all by default
            SecurityContext::anonymous()
        } else {
            let mut builder = SecurityContext::builder()
                .tenant_id(claims.tenant_id)
                .subject_id(claims.subject);

            for perm in &claims.permissions {
                builder = builder.add_permission(perm.resource(), perm.action());
            }

            for (extra_key, extra_value) in &claims.extras {
                if let Some(value_str) = extra_value.as_str() {
                    builder = builder.add_environment_attribute(extra_key, value_str);
                }
            }

            builder.build()
        }
    }
}
