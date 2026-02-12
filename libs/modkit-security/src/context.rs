use crate::permission::Permission;
use crate::{AccessScope, PolicyEngineRef};
use uuid::Uuid;

/// `SecurityContext` encapsulates the security-related information for a request or operation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityContext {
    tenant_id: Uuid,
    subject_id: Uuid,
    subject_type: Option<String>,
    permissions: Vec<Permission>,
    environment: Vec<(String, String)>,
}

impl SecurityContext {
    /// Create a new `SecurityContext` builder
    #[must_use]
    pub fn builder() -> SecurityContextBuilder {
        SecurityContextBuilder::default()
    }

    /// Create an anonymous `SecurityContext` with no tenant, subject, or permissions
    #[must_use]
    pub fn anonymous() -> Self {
        SecurityContextBuilder::default().build()
    }

    /// Get the tenant ID associated with the security context
    #[must_use]
    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    /// Get the subject ID (user, service, or system) associated with the security context
    #[must_use]
    pub fn subject_id(&self) -> Uuid {
        self.subject_id
    }

    /// Get the permissions assigned to the security context
    #[must_use]
    pub fn permissions(&self) -> Vec<Permission> {
        self.permissions.clone()
    }

    /// Get the environmental attributes associated with the security context
    /// (e.g., IP address, device type, location, time, etc.)
    #[must_use]
    pub fn environment(&self) -> Vec<(String, String)> {
        self.environment.clone()
    }

    pub fn scope(&self, policy_engine: PolicyEngineRef) -> AccessScopeResolver {
        AccessScopeResolver {
            _policy_engine: policy_engine,
            context: self.clone(),
            accessible_tenants: None,
        }
    }
}

pub struct AccessScopeResolver {
    _policy_engine: PolicyEngineRef,
    context: SecurityContext,
    /// Accessible tenant IDs (set via `include_accessible_tenants`).
    accessible_tenants: Option<Vec<Uuid>>,
}

impl AccessScopeResolver {
    /// Include a list of accessible tenant IDs in the scope.
    ///
    /// Use this method when the caller has already resolved which tenants
    /// the current security context can access (typically via `TenantResolverClient`).
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Discover accessible tenants via hierarchy
    /// let response = resolver.get_descendants(&ctx, tenant_id, None, None, None).await?;
    /// let accessible: Vec<Uuid> = std::iter::once(response.tenant.id)
    ///     .chain(response.descendants.iter().map(|t| t.id))
    ///     .collect();
    ///
    /// // Build scope with accessible tenants
    /// let scope = ctx
    ///     .scope(policy_engine)
    ///     .include_accessible_tenants(accessible)
    ///     .prepare()
    ///     .await?;
    /// ```
    #[must_use]
    pub fn include_accessible_tenants(mut self, tenants: Vec<Uuid>) -> Self {
        self.accessible_tenants = Some(tenants);
        self
    }

    #[must_use]
    pub fn include_resource_ids(&self) -> &Self {
        self
    }

    /// Prepare and build the final `AccessScope` based on the resolver configuration
    ///
    /// # Errors
    /// This function may return an error if the scope preparation fails
    pub async fn prepare(&self) -> Result<AccessScope, Box<dyn std::error::Error>> {
        // Keep this async to allow future policy-engine / IO-backed resolution without
        // changing the public API. This no-op await also satisfies clippy::unused_async.
        std::future::ready(()).await;

        // If accessible tenants were provided, use them
        if let Some(ref tenants) = self.accessible_tenants {
            return Ok(AccessScope::tenants_only(tenants.clone()));
        }

        // Fallback: single tenant from context
        if self.context.tenant_id != Uuid::default() {
            return Ok(AccessScope::tenants_only(vec![self.context.tenant_id]));
        }

        // Empty scope = deny all
        Ok(AccessScope::default())
    }
}

#[derive(Default)]
pub struct SecurityContextBuilder {
    tenant_id: Option<Uuid>,
    subject_id: Option<Uuid>,
    subject_type: Option<String>,
    permissions: Vec<Permission>,
    environment: Vec<(String, String)>,
}

impl SecurityContextBuilder {
    #[must_use]
    pub fn tenant_id(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    #[must_use]
    pub fn subject_id(mut self, subject_id: Uuid) -> Self {
        self.subject_id = Some(subject_id);
        self
    }

    #[must_use]
    pub fn subject_type(mut self, subject_type: &str) -> Self {
        self.subject_type = Some(subject_type.to_owned());
        self
    }

    #[must_use]
    pub fn add_permission(mut self, permission: Permission) -> Self {
        self.permissions.push(permission);
        self
    }

    #[must_use]
    pub fn add_environment_attribute(mut self, key: &str, value: &str) -> Self {
        self.environment.push((key.to_owned(), value.to_owned()));
        self
    }

    #[must_use]
    pub fn build(self) -> SecurityContext {
        SecurityContext {
            tenant_id: self.tenant_id.unwrap_or_default(),
            subject_id: self.subject_id.unwrap_or_default(),
            subject_type: self.subject_type,
            permissions: self.permissions,
            environment: self.environment,
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_security_context_builder_full() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let subject_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();

        let permission1 = Permission::builder()
            .tenant_id(tenant_id)
            .resource_pattern("gts.x.core.events.topic.v1~*")
            .action("publish")
            .build()
            .unwrap();

        let permission2 = Permission::builder()
            .resource_pattern("file-parser")
            .action("edit")
            .build()
            .unwrap();

        let ctx = SecurityContext::builder()
            .tenant_id(tenant_id)
            .subject_id(subject_id)
            .subject_type("user")
            .add_permission(permission1)
            .add_permission(permission2)
            .add_environment_attribute("ip", "192.168.1.1")
            .add_environment_attribute("device", "mobile")
            .build();

        assert_eq!(ctx.tenant_id(), tenant_id);
        assert_eq!(ctx.subject_id(), subject_id);
        assert_eq!(ctx.permissions().len(), 2);
        assert_eq!(ctx.environment().len(), 2);
        assert_eq!(
            ctx.environment()[0],
            ("ip".to_owned(), "192.168.1.1".to_owned())
        );
        assert_eq!(
            ctx.environment()[1],
            ("device".to_owned(), "mobile".to_owned())
        );
    }

    #[test]
    fn test_security_context_builder_minimal() {
        let ctx = SecurityContext::builder().build();

        assert_eq!(ctx.tenant_id(), Uuid::default());
        assert_eq!(ctx.subject_id(), Uuid::default());
        assert_eq!(ctx.permissions().len(), 0);
        assert_eq!(ctx.environment().len(), 0);
    }

    #[test]
    fn test_security_context_builder_partial() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let ctx = SecurityContext::builder()
            .tenant_id(tenant_id)
            .subject_type("service")
            .build();

        assert_eq!(ctx.tenant_id(), tenant_id);
        assert_eq!(ctx.subject_id(), Uuid::default());
        assert_eq!(ctx.permissions().len(), 0);
        assert_eq!(ctx.environment().len(), 0);
    }

    #[test]
    fn test_security_context_anonymous() {
        let ctx = SecurityContext::anonymous();

        assert_eq!(ctx.tenant_id(), Uuid::default());
        assert_eq!(ctx.subject_id(), Uuid::default());
        assert_eq!(ctx.permissions().len(), 0);
        assert_eq!(ctx.environment().len(), 0);
    }

    #[test]
    fn test_security_context_with_multiple_permissions() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let permission1 = Permission::builder()
            .tenant_id(tenant_id)
            .resource_pattern("gts.x.core.events.topic.v1~vendor.*")
            .action("publish")
            .build()
            .unwrap();

        let permission2 = Permission::builder()
            .tenant_id(tenant_id)
            .resource_pattern("gts.x.core.events.topic.v1~vendor.*")
            .action("subscribe")
            .build()
            .unwrap();

        let permission3 = Permission::builder()
            .resource_pattern("file-parser")
            .action("edit")
            .build()
            .unwrap();

        let ctx = SecurityContext::builder()
            .tenant_id(tenant_id)
            .add_permission(permission1)
            .add_permission(permission2)
            .add_permission(permission3)
            .build();

        let perms = ctx.permissions();
        assert_eq!(perms.len(), 3);
        assert_eq!(perms[0].tenant_id(), Some(tenant_id));
        assert_eq!(
            perms[0].resource_pattern(),
            "gts.x.core.events.topic.v1~vendor.*"
        );
        assert_eq!(perms[0].action(), "publish");
        assert_eq!(perms[1].action(), "subscribe");
        assert_eq!(perms[2].resource_pattern(), "file-parser");
    }

    #[test]
    fn test_security_context_with_multiple_environment_attributes() {
        let ctx = SecurityContext::builder()
            .add_environment_attribute("ip", "192.168.1.1")
            .add_environment_attribute("device", "mobile")
            .add_environment_attribute("location", "US")
            .add_environment_attribute("time_zone", "PST")
            .build();

        let env = ctx.environment();
        assert_eq!(env.len(), 4);
        assert_eq!(env[0], ("ip".to_owned(), "192.168.1.1".to_owned()));
        assert_eq!(env[1], ("device".to_owned(), "mobile".to_owned()));
        assert_eq!(env[2], ("location".to_owned(), "US".to_owned()));
        assert_eq!(env[3], ("time_zone".to_owned(), "PST".to_owned()));
    }

    #[test]
    fn test_security_context_builder_chaining() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let subject_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();

        // Test that builder methods can be chained fluently
        let ctx = SecurityContext::builder()
            .tenant_id(tenant_id)
            .subject_id(subject_id)
            .subject_type("user")
            .build();

        assert_eq!(ctx.tenant_id(), tenant_id);
        assert_eq!(ctx.subject_id(), subject_id);
    }

    #[test]
    fn test_security_context_getters_dont_mutate() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let permission = Permission::builder()
            .resource_pattern("file-parser")
            .action("edit")
            .build()
            .unwrap();

        let ctx = SecurityContext::builder()
            .tenant_id(tenant_id)
            .add_permission(permission)
            .add_environment_attribute("ip", "192.168.1.1")
            .build();

        // Call getters multiple times
        let _perms1 = ctx.permissions();
        let perms2 = ctx.permissions();
        assert_eq!(perms2.len(), 1);

        let _env1 = ctx.environment();
        let env2 = ctx.environment();
        assert_eq!(env2.len(), 1);

        // Original context should be unchanged
        assert_eq!(ctx.tenant_id(), tenant_id);
    }

    #[test]
    fn test_security_context_clone() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let subject_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();

        let permission = Permission::builder()
            .resource_pattern("file-parser")
            .action("edit")
            .build()
            .unwrap();

        let ctx1 = SecurityContext::builder()
            .tenant_id(tenant_id)
            .subject_id(subject_id)
            .add_permission(permission)
            .add_environment_attribute("ip", "192.168.1.1")
            .build();

        let ctx2 = ctx1.clone();

        assert_eq!(ctx2.tenant_id(), ctx1.tenant_id());
        assert_eq!(ctx2.subject_id(), ctx1.subject_id());
        assert_eq!(ctx2.permissions().len(), ctx1.permissions().len());
        assert_eq!(ctx2.environment().len(), ctx1.environment().len());
    }

    #[test]
    fn test_security_context_serialize_deserialize() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let subject_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();

        let permission = Permission::builder()
            .tenant_id(tenant_id)
            .resource_pattern("gts.x.core.events.topic.v1~*")
            .action("publish")
            .build()
            .unwrap();

        let original = SecurityContext::builder()
            .tenant_id(tenant_id)
            .subject_id(subject_id)
            .subject_type("user")
            .add_permission(permission)
            .add_environment_attribute("ip", "192.168.1.1")
            .build();

        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: SecurityContext = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.tenant_id(), original.tenant_id());
        assert_eq!(deserialized.subject_id(), original.subject_id());
        assert_eq!(deserialized.permissions().len(), 1);
        assert_eq!(deserialized.environment().len(), 1);
    }

    #[test]
    fn test_security_context_with_no_subject_type() {
        let tenant_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();

        let ctx = SecurityContext::builder().tenant_id(tenant_id).build();

        // subject_type is optional and should be None when not set
        assert_eq!(ctx.tenant_id(), tenant_id);
    }

    #[test]
    fn test_security_context_empty_permissions() {
        let ctx = SecurityContext::builder().build();

        assert!(ctx.permissions().is_empty());
    }
}
