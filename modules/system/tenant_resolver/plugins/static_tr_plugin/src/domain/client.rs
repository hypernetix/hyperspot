//! Client implementation for the static tenant resolver plugin.
//!
//! Implements `TenantResolverPluginClient` using the domain service.

use async_trait::async_trait;
use modkit_security::SecurityContext;
use tenant_resolver_sdk::{
    AccessOptions, TenantFilter, TenantId, TenantInfo, TenantResolverError,
    TenantResolverPluginClient,
};

use super::service::Service;

#[async_trait]
impl TenantResolverPluginClient for Service {
    async fn get_tenant(
        &self,
        _ctx: &SecurityContext,
        id: TenantId,
    ) -> Result<TenantInfo, TenantResolverError> {
        self.tenants
            .get(&id)
            .cloned()
            .ok_or(TenantResolverError::TenantNotFound { tenant_id: id })
    }

    async fn can_access(
        &self,
        ctx: &SecurityContext,
        target: TenantId,
        _options: Option<&AccessOptions>,
    ) -> Result<bool, TenantResolverError> {
        let source = ctx.tenant_id();

        // First, check if target tenant exists
        if !self.tenants.contains_key(&target) {
            return Err(TenantResolverError::TenantNotFound { tenant_id: target });
        }

        // Self-access is always allowed
        if source == target {
            return Ok(true);
        }

        // Check if access rule exists
        Ok(self.access_rules.contains(&(source, target)))
    }

    async fn get_accessible_tenants(
        &self,
        ctx: &SecurityContext,
        filter: Option<&TenantFilter>,
        _options: Option<&AccessOptions>,
    ) -> Result<Vec<TenantInfo>, TenantResolverError> {
        let source = ctx.tenant_id();

        let mut items: Vec<TenantInfo> = Vec::new();

        // Add self-tenant first if it exists and matches filter
        if let Some(self_info) = self.tenants.get(&source)
            && Self::matches_filter(self_info, filter)
        {
            items.push(self_info.clone());
        }

        // Get all targets accessible by this source
        let accessible_ids = self.accessible_by.get(&source);

        // Add accessible tenants (if any) that match the filter
        if let Some(ids) = accessible_ids {
            for id in ids {
                // Skip self (already added)
                if *id == source {
                    continue;
                }
                if let Some(info) = self.tenants.get(id)
                    && Self::matches_filter(info, filter)
                {
                    items.push(info.clone());
                }
            }
        }

        Ok(items)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::config::{AccessRuleConfig, StaticTrPluginConfig, TenantConfig};
    use tenant_resolver_sdk::TenantStatus;
    use uuid::Uuid;

    // Helper to create a test tenant config
    fn tenant(id: &str, name: &str, status: TenantStatus) -> TenantConfig {
        TenantConfig {
            id: Uuid::parse_str(id).unwrap(),
            name: name.to_owned(),
            status,
            tenant_type: None,
        }
    }

    // Helper to create an access rule config
    fn access_rule(source: &str, target: &str) -> AccessRuleConfig {
        AccessRuleConfig {
            source: Uuid::parse_str(source).unwrap(),
            target: Uuid::parse_str(target).unwrap(),
        }
    }

    // Helper to create a security context for a tenant
    fn ctx_for_tenant(tenant_id: &str) -> SecurityContext {
        SecurityContext::builder()
            .tenant_id(Uuid::parse_str(tenant_id).unwrap())
            .build()
    }

    // Filter for active tenants only
    fn active_filter() -> TenantFilter {
        TenantFilter {
            status: vec![TenantStatus::Active],
            ..Default::default()
        }
    }

    // Test UUIDs
    const TENANT_A: &str = "11111111-1111-1111-1111-111111111111";
    const TENANT_B: &str = "22222222-2222-2222-2222-222222222222";
    const TENANT_C: &str = "33333333-3333-3333-3333-333333333333";
    const NONEXISTENT: &str = "99999999-9999-9999-9999-999999999999";

    // ==================== get_tenant tests ====================

    #[tokio::test]
    async fn get_tenant_existing() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![tenant(TENANT_A, "Tenant A", TenantStatus::Active)],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let ctx = ctx_for_tenant(TENANT_A);

        let result = service
            .get_tenant(&ctx, Uuid::parse_str(TENANT_A).unwrap())
            .await;

        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.name, "Tenant A");
        assert_eq!(info.status, TenantStatus::Active);
    }

    #[tokio::test]
    async fn get_tenant_nonexistent() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![tenant(TENANT_A, "Tenant A", TenantStatus::Active)],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let ctx = ctx_for_tenant(TENANT_A);
        let nonexistent_id = Uuid::parse_str(NONEXISTENT).unwrap();

        let result = service.get_tenant(&ctx, nonexistent_id).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            TenantResolverError::TenantNotFound { tenant_id } => {
                assert_eq!(tenant_id, nonexistent_id);
            }
            other => panic!("Expected TenantNotFound, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_tenant_empty_service() {
        let cfg = StaticTrPluginConfig::default();
        let service = Service::from_config(&cfg);
        let ctx = ctx_for_tenant(TENANT_A);
        let tenant_a_id = Uuid::parse_str(TENANT_A).unwrap();

        let result = service.get_tenant(&ctx, tenant_a_id).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            TenantResolverError::TenantNotFound { tenant_id } => {
                assert_eq!(tenant_id, tenant_a_id);
            }
            other => panic!("Expected TenantNotFound, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_tenant_returns_any_status() {
        // get_tenant now returns tenant regardless of status
        let cfg = StaticTrPluginConfig {
            tenants: vec![tenant(TENANT_A, "Tenant A", TenantStatus::Suspended)],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let ctx = ctx_for_tenant(TENANT_A);
        let tenant_a_id = Uuid::parse_str(TENANT_A).unwrap();

        // Returns tenant even if suspended
        let result = service.get_tenant(&ctx, tenant_a_id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, TenantStatus::Suspended);
    }

    // ==================== can_access tests ====================

    #[tokio::test]
    async fn can_access_allowed() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "A", TenantStatus::Active),
                tenant(TENANT_B, "B", TenantStatus::Active),
            ],
            access_rules: vec![access_rule(TENANT_A, TENANT_B)],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let ctx = ctx_for_tenant(TENANT_A);

        let result = service
            .can_access(&ctx, Uuid::parse_str(TENANT_B).unwrap(), None)
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn can_access_denied_no_rule() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "A", TenantStatus::Active),
                tenant(TENANT_B, "B", TenantStatus::Active),
            ],
            access_rules: vec![], // No rules
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let ctx = ctx_for_tenant(TENANT_A);

        let result = service
            .can_access(&ctx, Uuid::parse_str(TENANT_B).unwrap(), None)
            .await;

        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn can_access_error_for_nonexistent_target() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![tenant(TENANT_A, "A", TenantStatus::Active)],
            access_rules: vec![],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let ctx = ctx_for_tenant(TENANT_A);
        let nonexistent_id = Uuid::parse_str(NONEXISTENT).unwrap();

        let result = service.can_access(&ctx, nonexistent_id, None).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            TenantResolverError::TenantNotFound { tenant_id } => {
                assert_eq!(tenant_id, nonexistent_id);
            }
            other => panic!("Expected TenantNotFound, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn can_access_not_symmetric() {
        // A can access B does NOT mean B can access A
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "A", TenantStatus::Active),
                tenant(TENANT_B, "B", TenantStatus::Active),
            ],
            access_rules: vec![access_rule(TENANT_A, TENANT_B)], // Only A -> B
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        // A can access B
        let ctx_a = ctx_for_tenant(TENANT_A);
        let result = service
            .can_access(&ctx_a, Uuid::parse_str(TENANT_B).unwrap(), None)
            .await;
        assert!(result.unwrap());

        // B cannot access A (no reverse rule)
        let ctx_b = ctx_for_tenant(TENANT_B);
        let result = service
            .can_access(&ctx_b, Uuid::parse_str(TENANT_A).unwrap(), None)
            .await;
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn can_access_not_transitive() {
        // A -> B and B -> C does NOT mean A -> C
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "A", TenantStatus::Active),
                tenant(TENANT_B, "B", TenantStatus::Active),
                tenant(TENANT_C, "C", TenantStatus::Active),
            ],
            access_rules: vec![
                access_rule(TENANT_A, TENANT_B), // A -> B
                access_rule(TENANT_B, TENANT_C), // B -> C
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let ctx = ctx_for_tenant(TENANT_A);

        // A can access B
        let result = service
            .can_access(&ctx, Uuid::parse_str(TENANT_B).unwrap(), None)
            .await;
        assert!(result.unwrap());

        // A cannot access C (no direct rule)
        let result = service
            .can_access(&ctx, Uuid::parse_str(TENANT_C).unwrap(), None)
            .await;
        assert!(!result.unwrap());
    }

    #[tokio::test]
    async fn can_access_self_allowed() {
        // Plugin handles self-access: always allowed
        let cfg = StaticTrPluginConfig {
            tenants: vec![tenant(TENANT_A, "A", TenantStatus::Active)],
            access_rules: vec![], // No explicit self-access rule needed
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let ctx = ctx_for_tenant(TENANT_A);

        // Plugin returns true for self-access
        let result = service
            .can_access(&ctx, Uuid::parse_str(TENANT_A).unwrap(), None)
            .await;
        assert!(result.unwrap());
    }

    #[tokio::test]
    async fn can_access_allows_any_status() {
        // can_access no longer filters by status - that's plugin policy
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "A", TenantStatus::Active),
                tenant(TENANT_B, "B", TenantStatus::Suspended),
            ],
            access_rules: vec![access_rule(TENANT_A, TENANT_B)],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let ctx = ctx_for_tenant(TENANT_A);
        let tenant_b_id = Uuid::parse_str(TENANT_B).unwrap();

        // Returns true even if target is suspended (access rule exists)
        let result = service.can_access(&ctx, tenant_b_id, None).await;
        assert!(result.unwrap());
    }

    // ==================== get_accessible_tenants tests ====================

    #[tokio::test]
    async fn get_accessible_tenants_with_rules() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "A", TenantStatus::Active),
                tenant(TENANT_B, "B", TenantStatus::Active),
                tenant(TENANT_C, "C", TenantStatus::Active),
            ],
            access_rules: vec![
                access_rule(TENANT_A, TENANT_B),
                access_rule(TENANT_A, TENANT_C),
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let ctx = ctx_for_tenant(TENANT_A);

        let result = service.get_accessible_tenants(&ctx, None, None).await;

        assert!(result.is_ok());
        let items = result.unwrap();
        // Self-tenant A + accessible B and C
        assert_eq!(items.len(), 3);

        // Self-tenant should be first
        assert_eq!(items[0].id, Uuid::parse_str(TENANT_A).unwrap());

        let ids: Vec<_> = items.iter().map(|t| t.id).collect();
        assert!(ids.contains(&Uuid::parse_str(TENANT_B).unwrap()));
        assert!(ids.contains(&Uuid::parse_str(TENANT_C).unwrap()));
    }

    #[tokio::test]
    async fn get_accessible_tenants_no_rules() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "A", TenantStatus::Active),
                tenant(TENANT_B, "B", TenantStatus::Active),
            ],
            access_rules: vec![],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let ctx = ctx_for_tenant(TENANT_A);

        let result = service.get_accessible_tenants(&ctx, None, None).await;

        assert!(result.is_ok());
        let items = result.unwrap();
        // Only self-tenant (no cross-tenant rules)
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, Uuid::parse_str(TENANT_A).unwrap());
    }

    #[tokio::test]
    async fn get_accessible_tenants_missing_tenant_info() {
        // Access rule references a tenant that doesn't exist in tenants list
        let cfg = StaticTrPluginConfig {
            tenants: vec![tenant(TENANT_A, "A", TenantStatus::Active)],
            access_rules: vec![access_rule(TENANT_A, TENANT_B)], // B not in tenants
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let ctx = ctx_for_tenant(TENANT_A);

        let result = service.get_accessible_tenants(&ctx, None, None).await;

        assert!(result.is_ok());
        let items = result.unwrap();
        // B is in access_rules but not in tenants, so it's skipped
        // Only self-tenant A is returned
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, Uuid::parse_str(TENANT_A).unwrap());
    }

    #[tokio::test]
    async fn get_accessible_tenants_filtered_by_status() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "A", TenantStatus::Active),
                tenant(TENANT_B, "B", TenantStatus::Active),
                tenant(TENANT_C, "C", TenantStatus::Suspended),
            ],
            access_rules: vec![
                access_rule(TENANT_A, TENANT_B),
                access_rule(TENANT_A, TENANT_C),
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let ctx = ctx_for_tenant(TENANT_A);

        // Without filter - returns self (A) plus B and C
        let result = service.get_accessible_tenants(&ctx, None, None).await;
        assert_eq!(result.unwrap().len(), 3);

        // With active filter - returns self (A) and B (C is suspended)
        let filter = active_filter();
        let result = service
            .get_accessible_tenants(&ctx, Some(&filter), None)
            .await;
        let items = result.unwrap();
        assert_eq!(items.len(), 2);
        // Self-tenant should be first
        assert_eq!(items[0].id, Uuid::parse_str(TENANT_A).unwrap());
        assert_eq!(items[1].id, Uuid::parse_str(TENANT_B).unwrap());
    }

    #[tokio::test]
    async fn get_accessible_tenants_filtered_by_id() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "A", TenantStatus::Active),
                tenant(TENANT_B, "B", TenantStatus::Active),
                tenant(TENANT_C, "C", TenantStatus::Active),
            ],
            access_rules: vec![
                access_rule(TENANT_A, TENANT_B),
                access_rule(TENANT_A, TENANT_C),
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let ctx = ctx_for_tenant(TENANT_A);

        // Filter by specific ID
        let filter = TenantFilter {
            id: vec![Uuid::parse_str(TENANT_B).unwrap()],
            ..Default::default()
        };
        let result = service
            .get_accessible_tenants(&ctx, Some(&filter), None)
            .await;
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, Uuid::parse_str(TENANT_B).unwrap());
    }
}
