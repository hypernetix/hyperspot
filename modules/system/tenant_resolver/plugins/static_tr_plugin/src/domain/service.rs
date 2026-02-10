//! Domain service for the static tenant resolver plugin.

use std::collections::{HashMap, HashSet};

use modkit_macros::domain_model;
use tenant_resolver_sdk::{
    BarrierMode, TenantId, TenantInfo, TenantRef, TenantResolverError, TenantStatus, matches_status,
};

use crate::config::StaticTrPluginConfig;

/// Static tenant resolver service.
///
/// Stores tenant data in memory, loaded from configuration.
/// Supports hierarchical tenant model with parent-child relationships.
#[domain_model]
pub struct Service {
    /// Tenant info by ID.
    pub(super) tenants: HashMap<TenantId, TenantInfo>,

    /// Children index: `parent_id` -> list of child tenant IDs.
    pub(super) children: HashMap<TenantId, Vec<TenantId>>,
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
                        parent_id: t.parent_id,
                        self_managed: t.self_managed,
                    },
                )
            })
            .collect();

        // Build children index
        let mut children: HashMap<TenantId, Vec<TenantId>> = HashMap::new();
        for tenant in tenants.values() {
            if let Some(parent_id) = tenant.parent_id {
                children.entry(parent_id).or_default().push(tenant.id);
            }
        }

        Self { tenants, children }
    }

    /// Check if a tenant matches the status filter.
    pub(super) fn matches_status_filter(tenant: &TenantInfo, statuses: &[TenantStatus]) -> bool {
        matches_status(tenant, statuses)
    }

    /// Collect ancestors from a tenant to root.
    ///
    /// Returns ancestors ordered from direct parent to root.
    /// Stops at barriers unless `barrier_mode` is `Ignore`.
    /// If the starting tenant itself is a barrier, returns empty (consistent
    /// with `is_ancestor` returning `false` for barrier descendants).
    ///
    /// Note: The starting tenant is NOT included in the result.
    pub(super) fn collect_ancestors(
        &self,
        id: TenantId,
        barrier_mode: BarrierMode,
    ) -> Vec<TenantRef> {
        let mut ancestors = Vec::new();
        let mut visited = HashSet::new();
        visited.insert(id);

        // Start from the tenant's parent
        let Some(tenant) = self.tenants.get(&id) else {
            return ancestors;
        };

        // If the starting tenant is a barrier, it cannot see its parent chain
        if barrier_mode == BarrierMode::Respect && tenant.self_managed {
            return ancestors;
        }

        let mut current_parent_id = tenant.parent_id;

        while let Some(parent_id) = current_parent_id {
            if !visited.insert(parent_id) {
                break; // Cycle detected
            }

            let Some(parent) = self.tenants.get(&parent_id) else {
                break;
            };

            // Barrier semantics: include the barrier tenant, but stop traversal
            // at it (don't continue to its parent).
            ancestors.push(parent.into());

            if barrier_mode == BarrierMode::Respect && parent.self_managed {
                break;
            }

            current_parent_id = parent.parent_id;
        }

        ancestors
    }

    /// Collect descendants subtree using pre-order traversal.
    ///
    /// Returns descendants (not including the starting tenant) in pre-order:
    /// parent is visited before children.
    ///
    /// Traversal stops when:
    /// - `self_managed` barrier is encountered (unless `barrier_mode` is `Ignore`)
    /// - Node doesn't pass the filter (filtered nodes and their subtrees are excluded)
    /// - `max_depth` is reached
    pub(super) fn collect_descendants(
        &self,
        id: TenantId,
        statuses: &[TenantStatus],
        barrier_mode: BarrierMode,
        max_depth: Option<u32>,
    ) -> Vec<TenantRef> {
        let mut collector = DescendantCollector {
            tenants: &self.tenants,
            children: &self.children,
            statuses,
            barrier_mode,
            max_depth,
            result: Vec::new(),
            visited: HashSet::new(),
        };
        collector.visited.insert(id);
        collector.collect(id, 1);
        collector.result
    }

    /// Check if `ancestor_id` is an ancestor of `descendant_id`.
    ///
    /// Returns `true` if `ancestor_id` is in the parent chain of `descendant_id`.
    /// Returns `false` if `ancestor_id == descendant_id` (self is not an ancestor of self).
    ///
    /// Respects barriers: if there's a barrier between them, returns `false`.
    ///
    /// # Errors
    ///
    /// Returns `TenantNotFound` if either tenant does not exist.
    pub(super) fn is_ancestor_of(
        &self,
        ancestor_id: TenantId,
        descendant_id: TenantId,
        barrier_mode: BarrierMode,
    ) -> Result<bool, TenantResolverError> {
        // Self is NOT an ancestor of self
        if ancestor_id == descendant_id {
            if self.tenants.contains_key(&ancestor_id) {
                return Ok(false);
            }
            return Err(TenantResolverError::TenantNotFound {
                tenant_id: ancestor_id,
            });
        }

        // Check both tenants exist
        if !self.tenants.contains_key(&ancestor_id) {
            return Err(TenantResolverError::TenantNotFound {
                tenant_id: ancestor_id,
            });
        }

        let descendant =
            self.tenants
                .get(&descendant_id)
                .ok_or(TenantResolverError::TenantNotFound {
                    tenant_id: descendant_id,
                })?;

        // If the descendant itself is a barrier, the ancestor cannot claim
        // parentage — consistent with get_descendants excluding barriers.
        if barrier_mode == BarrierMode::Respect && descendant.self_managed {
            return Ok(false);
        }

        // Walk up the chain from descendant
        let mut visited = HashSet::new();
        visited.insert(descendant_id);
        let mut current_parent_id = descendant.parent_id;

        while let Some(parent_id) = current_parent_id {
            if !visited.insert(parent_id) {
                break; // Cycle detected
            }

            let Some(parent) = self.tenants.get(&parent_id) else {
                break;
            };

            // Found the ancestor
            if parent_id == ancestor_id {
                return Ok(true);
            }

            // Barrier semantics: if the parent is self_managed and not the target
            // ancestor, traversal is blocked beyond this point.
            if barrier_mode == BarrierMode::Respect && parent.self_managed {
                return Ok(false);
            }

            current_parent_id = parent.parent_id;
        }

        // Reached root without finding ancestor
        Ok(false)
    }
}

/// Encapsulates traversal state for collecting descendants.
///
/// Eliminates the need for passing many arguments through recursive calls.
#[domain_model]
struct DescendantCollector<'a> {
    tenants: &'a HashMap<TenantId, TenantInfo>,
    children: &'a HashMap<TenantId, Vec<TenantId>>,
    statuses: &'a [TenantStatus],
    barrier_mode: BarrierMode,
    max_depth: Option<u32>,
    result: Vec<TenantRef>,
    visited: HashSet<TenantId>,
}

impl DescendantCollector<'_> {
    fn collect(&mut self, parent_id: TenantId, current_depth: u32) {
        // Check depth limit (None = unlimited)
        if self.max_depth.is_some_and(|d| current_depth > d) {
            return;
        }

        let Some(child_ids) = self.children.get(&parent_id) else {
            return;
        };

        for child_id in child_ids {
            if !self.visited.insert(*child_id) {
                continue;
            }

            let Some(child) = self.tenants.get(child_id) else {
                continue;
            };

            // If respecting barriers and this child is self_managed, skip it and its subtree
            if self.barrier_mode == BarrierMode::Respect && child.self_managed {
                continue;
            }

            // If child doesn't pass status filter, skip it AND its subtree
            if !Service::matches_status_filter(child, self.statuses) {
                continue;
            }

            self.result.push(child.into());

            // Recurse into children
            self.collect(*child_id, current_depth + 1);
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::config::TenantConfig;
    use tenant_resolver_sdk::TenantStatus;
    use uuid::Uuid;

    // Helper to create a test tenant config
    fn tenant(id: &str, name: &str, status: TenantStatus) -> TenantConfig {
        TenantConfig {
            id: Uuid::parse_str(id).unwrap(),
            name: name.to_owned(),
            status,
            tenant_type: None,
            parent_id: None,
            self_managed: false,
        }
    }

    fn tenant_with_parent(id: &str, name: &str, parent: &str) -> TenantConfig {
        TenantConfig {
            id: Uuid::parse_str(id).unwrap(),
            name: name.to_owned(),
            status: TenantStatus::Active,
            tenant_type: None,
            parent_id: Some(Uuid::parse_str(parent).unwrap()),
            self_managed: false,
        }
    }

    fn tenant_barrier(id: &str, name: &str, parent: &str) -> TenantConfig {
        TenantConfig {
            id: Uuid::parse_str(id).unwrap(),
            name: name.to_owned(),
            status: TenantStatus::Active,
            tenant_type: None,
            parent_id: Some(Uuid::parse_str(parent).unwrap()),
            self_managed: true,
        }
    }

    // Test UUIDs
    const TENANT_A: &str = "11111111-1111-1111-1111-111111111111";
    const TENANT_B: &str = "22222222-2222-2222-2222-222222222222";
    const TENANT_C: &str = "33333333-3333-3333-3333-333333333333";
    const TENANT_D: &str = "44444444-4444-4444-4444-444444444444";

    // ==================== from_config tests ====================

    #[test]
    fn from_config_empty() {
        let cfg = StaticTrPluginConfig::default();
        let service = Service::from_config(&cfg);

        assert!(service.tenants.is_empty());
        assert!(service.children.is_empty());
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
        assert!(service.children.is_empty()); // No parent-child relationships

        let a = service
            .tenants
            .get(&Uuid::parse_str(TENANT_A).unwrap())
            .unwrap();
        assert_eq!(a.name, "Tenant A");
        assert_eq!(a.status, TenantStatus::Active);
        assert!(a.parent_id.is_none());
        assert!(!a.self_managed);
    }

    #[test]
    fn from_config_with_hierarchy() {
        // A -> B -> C (linear hierarchy)
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "Root", TenantStatus::Active),
                tenant_with_parent(TENANT_B, "Child", TENANT_A),
                tenant_with_parent(TENANT_C, "Grandchild", TENANT_B),
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        assert_eq!(service.tenants.len(), 3);

        // Check children index
        let a_id = Uuid::parse_str(TENANT_A).unwrap();
        let b_id = Uuid::parse_str(TENANT_B).unwrap();
        let c_id = Uuid::parse_str(TENANT_C).unwrap();

        let a_children = service.children.get(&a_id).unwrap();
        assert_eq!(a_children.len(), 1);
        assert!(a_children.contains(&b_id));

        let b_children = service.children.get(&b_id).unwrap();
        assert_eq!(b_children.len(), 1);
        assert!(b_children.contains(&c_id));

        // C has no children
        assert!(!service.children.contains_key(&c_id));
    }

    // ==================== collect_ancestors tests ====================

    #[test]
    fn collect_ancestors_root_tenant() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![tenant(TENANT_A, "Root", TenantStatus::Active)],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let a_id = Uuid::parse_str(TENANT_A).unwrap();

        let ancestors = service.collect_ancestors(a_id, BarrierMode::Respect);
        assert!(ancestors.is_empty());
    }

    #[test]
    fn collect_ancestors_linear_hierarchy() {
        // A -> B -> C
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "Root", TenantStatus::Active),
                tenant_with_parent(TENANT_B, "Child", TENANT_A),
                tenant_with_parent(TENANT_C, "Grandchild", TENANT_B),
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        let a_id = Uuid::parse_str(TENANT_A).unwrap();
        let b_id = Uuid::parse_str(TENANT_B).unwrap();
        let c_id = Uuid::parse_str(TENANT_C).unwrap();

        // Ancestors of C should be [B, A]
        let ancestors = service.collect_ancestors(c_id, BarrierMode::Respect);
        assert_eq!(ancestors.len(), 2);
        assert_eq!(ancestors[0].id, b_id);
        assert_eq!(ancestors[1].id, a_id);

        // Ancestors of B should be [A]
        let ancestors = service.collect_ancestors(b_id, BarrierMode::Respect);
        assert_eq!(ancestors.len(), 1);
        assert_eq!(ancestors[0].id, a_id);
    }

    #[test]
    fn collect_ancestors_with_barrier() {
        // A -> B (barrier) -> C
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "Root", TenantStatus::Active),
                tenant_barrier(TENANT_B, "Barrier", TENANT_A),
                tenant_with_parent(TENANT_C, "Grandchild", TENANT_B),
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        let a_id = Uuid::parse_str(TENANT_A).unwrap();
        let b_id = Uuid::parse_str(TENANT_B).unwrap();
        let c_id = Uuid::parse_str(TENANT_C).unwrap();

        // With BarrierMode::Respect, ancestors of C stop at B
        let ancestors = service.collect_ancestors(c_id, BarrierMode::Respect);
        assert_eq!(ancestors.len(), 1);
        assert_eq!(ancestors[0].id, b_id);

        // With BarrierMode::Ignore, ancestors of C include both B and A
        let ancestors = service.collect_ancestors(c_id, BarrierMode::Ignore);
        assert_eq!(ancestors.len(), 2);
        assert_eq!(ancestors[0].id, b_id);
        assert_eq!(ancestors[1].id, a_id);
    }

    #[test]
    fn collect_ancestors_starting_tenant_is_barrier() {
        // A -> B (barrier)
        // get_ancestors(B) should return empty because B is a barrier
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "Root", TenantStatus::Active),
                tenant_barrier(TENANT_B, "Barrier", TENANT_A),
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        let a_id = Uuid::parse_str(TENANT_A).unwrap();
        let b_id = Uuid::parse_str(TENANT_B).unwrap();

        // With BarrierMode::Respect, B cannot see its parent chain
        let ancestors = service.collect_ancestors(b_id, BarrierMode::Respect);
        assert!(ancestors.is_empty());

        // With BarrierMode::Ignore, B can see A
        let ancestors = service.collect_ancestors(b_id, BarrierMode::Ignore);
        assert_eq!(ancestors.len(), 1);
        assert_eq!(ancestors[0].id, a_id);
    }

    // ==================== collect_descendants tests ====================

    #[test]
    fn collect_descendants_no_children() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![tenant(TENANT_A, "Root", TenantStatus::Active)],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let a_id = Uuid::parse_str(TENANT_A).unwrap();

        let descendants = service.collect_descendants(a_id, &[], BarrierMode::Respect, None);
        assert!(descendants.is_empty());
    }

    #[test]
    fn collect_descendants_linear_hierarchy() {
        // A -> B -> C
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "Root", TenantStatus::Active),
                tenant_with_parent(TENANT_B, "Child", TENANT_A),
                tenant_with_parent(TENANT_C, "Grandchild", TENANT_B),
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        let a_id = Uuid::parse_str(TENANT_A).unwrap();
        let b_id = Uuid::parse_str(TENANT_B).unwrap();
        let c_id = Uuid::parse_str(TENANT_C).unwrap();

        // Descendants of A (unlimited depth)
        let descendants = service.collect_descendants(a_id, &[], BarrierMode::Respect, None);
        assert_eq!(descendants.len(), 2);
        // Pre-order: B first, then C
        assert_eq!(descendants[0].id, b_id);
        assert_eq!(descendants[1].id, c_id);

        // Descendants of A (depth 1 = direct children only)
        let descendants = service.collect_descendants(a_id, &[], BarrierMode::Respect, Some(1));
        assert_eq!(descendants.len(), 1);
        assert_eq!(descendants[0].id, b_id);
    }

    #[test]
    fn collect_descendants_with_barrier() {
        // A -> B (barrier) -> C
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "Root", TenantStatus::Active),
                tenant_barrier(TENANT_B, "Barrier", TENANT_A),
                tenant_with_parent(TENANT_C, "Grandchild", TENANT_B),
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        let a_id = Uuid::parse_str(TENANT_A).unwrap();
        let b_id = Uuid::parse_str(TENANT_B).unwrap();
        let c_id = Uuid::parse_str(TENANT_C).unwrap();

        // With BarrierMode::Respect, descendants of A exclude B (barrier) and its subtree
        let descendants = service.collect_descendants(a_id, &[], BarrierMode::Respect, None);
        assert!(descendants.is_empty());

        // With BarrierMode::Ignore, descendants include B and C
        let descendants = service.collect_descendants(a_id, &[], BarrierMode::Ignore, None);
        assert_eq!(descendants.len(), 2);
        assert_eq!(descendants[0].id, b_id);
        assert_eq!(descendants[1].id, c_id);
    }

    #[test]
    fn collect_descendants_mixed_barrier() {
        // A -> B (barrier) -> C
        //   -> D (no barrier)
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "Root", TenantStatus::Active),
                tenant_barrier(TENANT_B, "Barrier", TENANT_A),
                tenant_with_parent(TENANT_C, "Under Barrier", TENANT_B),
                tenant_with_parent(TENANT_D, "Normal Child", TENANT_A),
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        let a_id = Uuid::parse_str(TENANT_A).unwrap();
        let d_id = Uuid::parse_str(TENANT_D).unwrap();

        // With BarrierMode::Respect, only D is visible
        let descendants = service.collect_descendants(a_id, &[], BarrierMode::Respect, None);
        assert_eq!(descendants.len(), 1);
        assert_eq!(descendants[0].id, d_id);
    }

    // ==================== is_ancestor_of tests ====================

    #[test]
    fn is_ancestor_of_self() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![tenant(TENANT_A, "Root", TenantStatus::Active)],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);
        let a_id = Uuid::parse_str(TENANT_A).unwrap();

        // Self is NOT an ancestor of self
        assert!(
            !service
                .is_ancestor_of(a_id, a_id, BarrierMode::Respect)
                .unwrap()
        );
    }

    #[test]
    fn is_ancestor_of_direct_parent() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "Root", TenantStatus::Active),
                tenant_with_parent(TENANT_B, "Child", TENANT_A),
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        let a_id = Uuid::parse_str(TENANT_A).unwrap();
        let b_id = Uuid::parse_str(TENANT_B).unwrap();

        assert!(
            service
                .is_ancestor_of(a_id, b_id, BarrierMode::Respect)
                .unwrap()
        );
        assert!(
            !service
                .is_ancestor_of(b_id, a_id, BarrierMode::Respect)
                .unwrap()
        );
    }

    #[test]
    fn is_ancestor_of_grandparent() {
        // A -> B -> C
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "Root", TenantStatus::Active),
                tenant_with_parent(TENANT_B, "Child", TENANT_A),
                tenant_with_parent(TENANT_C, "Grandchild", TENANT_B),
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        let a_id = Uuid::parse_str(TENANT_A).unwrap();
        let c_id = Uuid::parse_str(TENANT_C).unwrap();

        assert!(
            service
                .is_ancestor_of(a_id, c_id, BarrierMode::Respect)
                .unwrap()
        );
    }

    #[test]
    fn is_ancestor_of_with_barrier() {
        // A -> B (barrier) -> C
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "Root", TenantStatus::Active),
                tenant_barrier(TENANT_B, "Barrier", TENANT_A),
                tenant_with_parent(TENANT_C, "Grandchild", TENANT_B),
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        let a_id = Uuid::parse_str(TENANT_A).unwrap();
        let b_id = Uuid::parse_str(TENANT_B).unwrap();
        let c_id = Uuid::parse_str(TENANT_C).unwrap();

        // B is direct parent of C - no barrier crossed
        assert!(
            service
                .is_ancestor_of(b_id, c_id, BarrierMode::Respect)
                .unwrap()
        );

        // A is blocked by barrier B
        assert!(
            !service
                .is_ancestor_of(a_id, c_id, BarrierMode::Respect)
                .unwrap()
        );

        // With BarrierMode::Ignore, A is ancestor of C
        assert!(
            service
                .is_ancestor_of(a_id, c_id, BarrierMode::Ignore)
                .unwrap()
        );
    }

    #[test]
    fn is_ancestor_of_direct_barrier_child() {
        // A -> B (barrier)
        // is_ancestor(A, B) should return false because B is a barrier
        // (consistent with get_descendants(A) excluding B)
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "Root", TenantStatus::Active),
                tenant_barrier(TENANT_B, "Barrier", TENANT_A),
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        let a_id = Uuid::parse_str(TENANT_A).unwrap();
        let b_id = Uuid::parse_str(TENANT_B).unwrap();

        // A is NOT ancestor of B when B is a barrier (BarrierMode::Respect)
        assert!(
            !service
                .is_ancestor_of(a_id, b_id, BarrierMode::Respect)
                .unwrap()
        );

        // With BarrierMode::Ignore, A IS ancestor of B
        assert!(
            service
                .is_ancestor_of(a_id, b_id, BarrierMode::Ignore)
                .unwrap()
        );

        // B is NOT ancestor of itself (self is not an ancestor of self)
        assert!(
            !service
                .is_ancestor_of(b_id, b_id, BarrierMode::Respect)
                .unwrap()
        );
    }

    #[test]
    fn is_ancestor_of_nonexistent() {
        let cfg = StaticTrPluginConfig {
            tenants: vec![tenant(TENANT_A, "Root", TenantStatus::Active)],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        let a_id = Uuid::parse_str(TENANT_A).unwrap();
        let nonexistent = Uuid::parse_str(TENANT_B).unwrap();

        // Nonexistent ancestor
        assert!(matches!(
            service.is_ancestor_of(nonexistent, a_id, BarrierMode::Respect),
            Err(TenantResolverError::TenantNotFound { tenant_id }) if tenant_id == nonexistent
        ));

        // Nonexistent descendant
        assert!(matches!(
            service.is_ancestor_of(a_id, nonexistent, BarrierMode::Respect),
            Err(TenantResolverError::TenantNotFound { tenant_id }) if tenant_id == nonexistent
        ));
    }

    #[test]
    fn collect_ancestors_cycle_terminates() {
        // Create a cycle: A -> B -> A (via parent_id)
        let a_id = Uuid::parse_str(TENANT_A).unwrap();
        let b_id = Uuid::parse_str(TENANT_B).unwrap();

        let cfg = StaticTrPluginConfig {
            tenants: vec![
                TenantConfig {
                    id: a_id,
                    name: "A".to_owned(),
                    status: TenantStatus::Active,
                    tenant_type: None,
                    parent_id: Some(b_id),
                    self_managed: false,
                },
                TenantConfig {
                    id: b_id,
                    name: "B".to_owned(),
                    status: TenantStatus::Active,
                    tenant_type: None,
                    parent_id: Some(a_id),
                    self_managed: false,
                },
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        // Should terminate (not loop forever) and return at most 2 ancestors
        let ancestors = service.collect_ancestors(a_id, BarrierMode::Ignore);
        assert!(ancestors.len() <= 2);
    }

    #[test]
    fn is_ancestor_of_cycle_terminates() {
        // Create a cycle: A -> B -> A (via parent_id)
        let a_id = Uuid::parse_str(TENANT_A).unwrap();
        let b_id = Uuid::parse_str(TENANT_B).unwrap();
        let c_id = Uuid::parse_str(TENANT_C).unwrap();

        let cfg = StaticTrPluginConfig {
            tenants: vec![
                TenantConfig {
                    id: a_id,
                    name: "A".to_owned(),
                    status: TenantStatus::Active,
                    tenant_type: None,
                    parent_id: Some(b_id),
                    self_managed: false,
                },
                TenantConfig {
                    id: b_id,
                    name: "B".to_owned(),
                    status: TenantStatus::Active,
                    tenant_type: None,
                    parent_id: Some(a_id),
                    self_managed: false,
                },
                TenantConfig {
                    id: c_id,
                    name: "C".to_owned(),
                    status: TenantStatus::Active,
                    tenant_type: None,
                    parent_id: None,
                    self_managed: false,
                },
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        // Should terminate (not loop forever), C is not in the cycle
        assert!(
            !service
                .is_ancestor_of(c_id, a_id, BarrierMode::Ignore)
                .unwrap()
        );
    }

    #[test]
    fn is_ancestor_of_unrelated() {
        // A and B are both roots (unrelated)
        let cfg = StaticTrPluginConfig {
            tenants: vec![
                tenant(TENANT_A, "Root A", TenantStatus::Active),
                tenant(TENANT_B, "Root B", TenantStatus::Active),
            ],
            ..Default::default()
        };
        let service = Service::from_config(&cfg);

        let a_id = Uuid::parse_str(TENANT_A).unwrap();
        let b_id = Uuid::parse_str(TENANT_B).unwrap();

        assert!(
            !service
                .is_ancestor_of(a_id, b_id, BarrierMode::Respect)
                .unwrap()
        );
        assert!(
            !service
                .is_ancestor_of(b_id, a_id, BarrierMode::Respect)
                .unwrap()
        );
    }
}
