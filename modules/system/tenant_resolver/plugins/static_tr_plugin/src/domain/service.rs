//! Domain service for the static tenant resolver plugin.

use std::collections::{HashMap, HashSet};

use tenant_resolver_sdk::{TenantFilter, TenantId, TenantInfo};

use crate::config::StaticTrPluginConfig;

/// Static tenant resolver service.
///
/// Stores tenant data and access rules in memory, loaded from configuration.
pub struct Service {
    /// Tenant info by ID.
    pub(super) tenants: HashMap<TenantId, TenantInfo>,

    /// Access rules: set of (source, target) pairs.
    pub(super) access_rules: HashSet<(TenantId, TenantId)>,

    /// Reverse index: target -> list of sources that can access it.
    /// Used for efficient `get_accessible_tenants`.
    pub(super) accessible_by: HashMap<TenantId, Vec<TenantId>>,
}

impl Service {
    /// Creates a new service from configuration.
    #[must_use]
    pub fn from_config(cfg: &StaticTrPluginConfig) -> Self {
        let tenants: HashMap<TenantId, TenantInfo> = cfg
            .tenants
            .iter()
            .map(|t| {
                (
                    t.id,
                    TenantInfo {
                        id: t.id,
                        name: t.name.clone(),
                        status: t.status,
                        tenant_type: t.tenant_type.clone(),
                    },
                )
            })
            .collect();

        let access_rules: HashSet<(TenantId, TenantId)> = cfg
            .access_rules
            .iter()
            .map(|r| (r.source, r.target))
            .collect();

        // Build reverse index: for each source, which targets can it access?
        let mut accessible_by: HashMap<TenantId, Vec<TenantId>> = HashMap::new();
        for (source, target) in &access_rules {
            accessible_by.entry(*source).or_default().push(*target);
        }

        Self {
            tenants,
            access_rules,
            accessible_by,
        }
    }

    /// Check if a tenant matches the filter criteria.
    pub(super) fn matches_filter(tenant: &TenantInfo, filter: Option<&TenantFilter>) -> bool {
        let Some(filter) = filter else {
            return true;
        };

        // Check ID filter
        if !filter.id.is_empty() && !filter.id.contains(&tenant.id) {
            return false;
        }

        // Check status filter
        if !filter.status.is_empty() && !filter.status.contains(&tenant.status) {
            return false;
        }

        true
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::config::{AccessRuleConfig, TenantConfig};
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

    // Test UUIDs
    const TENANT_A: &str = "11111111-1111-1111-1111-111111111111";
    const TENANT_B: &str = "22222222-2222-2222-2222-222222222222";
    const TENANT_C: &str = "33333333-3333-3333-3333-333333333333";

    // ==================== from_config tests ====================

    #[test]
    fn from_config_empty() {
        let cfg = StaticTrPluginConfig::default();
        let service = Service::from_config(&cfg);

        assert!(service.tenants.is_empty());
        assert!(service.access_rules.is_empty());
        assert!(service.accessible_by.is_empty());
    }

    #[test]
    fn from_config_with_tenants_only() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "Tenant A", TenantStatus::Active),
                tenant(TENANT_B, "Tenant B", TenantStatus::Suspended),
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        assert_eq!(service.tenants.len(), 2);
        assert!(service.access_rules.is_empty());
        assert!(service.accessible_by.is_empty());

        let a = service
            .tenants
            .get(&Uuid::parse_str(TENANT_A).unwrap())
            .unwrap();
        assert_eq!(a.name, "Tenant A");
        assert_eq!(a.status, TenantStatus::Active);

        let b = service
            .tenants
            .get(&Uuid::parse_str(TENANT_B).unwrap())
            .unwrap();
        assert_eq!(b.name, "Tenant B");
        assert_eq!(b.status, TenantStatus::Suspended);
    }

    #[test]
    fn from_config_with_access_rules() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "A", TenantStatus::Active),
                tenant(TENANT_B, "B", TenantStatus::Active),
                tenant(TENANT_C, "C", TenantStatus::Active),
            ],
            access_rules: vec![
                access_rule(TENANT_A, TENANT_B), // A can access B
                access_rule(TENANT_A, TENANT_C), // A can access C
                access_rule(TENANT_B, TENANT_C), // B can access C
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        assert_eq!(service.access_rules.len(), 3);

        // Check reverse index
        let a_id = Uuid::parse_str(TENANT_A).unwrap();
        let b_id = Uuid::parse_str(TENANT_B).unwrap();
        let c_id = Uuid::parse_str(TENANT_C).unwrap();

        let a_accessible = service.accessible_by.get(&a_id).unwrap();
        assert_eq!(a_accessible.len(), 2);
        assert!(a_accessible.contains(&b_id));
        assert!(a_accessible.contains(&c_id));

        let b_accessible = service.accessible_by.get(&b_id).unwrap();
        assert_eq!(b_accessible.len(), 1);
        assert!(b_accessible.contains(&c_id));

        // C has no access rules
        assert!(!service.accessible_by.contains_key(&c_id));
    }

    #[test]
    fn from_config_with_tenant_type() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![TenantConfig {
                id: Uuid::parse_str(TENANT_A).unwrap(),
                name: "Enterprise".to_owned(),
                status: TenantStatus::Active,
                tenant_type: Some("enterprise".to_owned()),
            }],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        let a = service
            .tenants
            .get(&Uuid::parse_str(TENANT_A).unwrap())
            .unwrap();
        assert_eq!(a.tenant_type, Some("enterprise".to_owned()));
    }
}
