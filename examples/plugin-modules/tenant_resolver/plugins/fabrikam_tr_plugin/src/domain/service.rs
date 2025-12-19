//! Fabrikam plugin service implementing `ThrPluginApi`.

use std::collections::HashMap;

use async_trait::async_trait;
use modkit_odata::{
    validate_cursor_against, CursorV1, ODataOrderBy, ODataQuery, OrderKey, Page, PageInfo, SortDir,
};
use modkit_security::SecurityCtx;
use tenant_resolver_sdk::{
    AccessOptions, GetParentsResponse, Tenant, TenantFilter, TenantResolverError, TenantSpec,
    ThrPluginApi,
};

use crate::config::TenantConfig;

/// DFS visit state for cycle detection.
#[derive(Clone, Copy)]
enum VisitState {
    Enter,
    Exit,
}

/// Fabrikam plugin service implementing the tenant resolver plugin API.
///
/// Stores an in-memory tenant tree built from configuration.
pub struct Service {
    /// Map from tenant ID to tenant data.
    tenants: HashMap<String, Tenant>,
    /// Map from parent ID to list of child IDs.
    children: HashMap<String, Vec<String>>,
    /// Root tenant ID.
    root_id: Option<String>,
}

impl Service {
    /// Creates a new service from tenant configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid (duplicate IDs, missing root, cycles, etc.).
    pub fn new(tenant_configs: &[TenantConfig]) -> Result<Self, TenantResolverError> {
        fn normalize_parent_id(v: Option<&str>) -> String {
            v.unwrap_or_default().trim().to_owned()
        }

        let mut roots = Vec::new();
        let mut tenant_parent: HashMap<String, String> = HashMap::new();
        let mut tenants: HashMap<String, Tenant> = HashMap::new();

        // First pass: build tenants and record parent links, validate uniqueness.
        for cfg in tenant_configs {
            let id = cfg.id.trim().to_owned();
            if id.is_empty() {
                return Err(TenantResolverError::Internal(
                    "invalid tenant tree: empty tenant id".to_owned(),
                ));
            }
            if tenants.contains_key(&id) {
                return Err(TenantResolverError::Internal(format!(
                    "invalid tenant tree: duplicate tenant id '{id}'"
                )));
            }

            let parent_id = normalize_parent_id(cfg.parent_id.as_deref());
            if parent_id.is_empty() {
                roots.push(id.clone());
            }
            tenant_parent.insert(id.clone(), parent_id.clone());

            let tenant = Tenant {
                id: id.clone(),
                parent_id,
                status: cfg.status,
                r#type: TenantSpec::GTS_SCHEMA_ID.to_owned(),
                is_accessible_by_parent: cfg.is_accessible_by_parent,
            };
            tenants.insert(id, tenant);
        }

        // Validate exactly one root.
        if roots.len() != 1 {
            return Err(TenantResolverError::Internal(format!(
                "invalid tenant tree: expected exactly 1 root, got {}",
                roots.len()
            )));
        }
        let root_id = roots.pop();

        // Validate parent existence.
        for (id, parent_id) in &tenant_parent {
            if parent_id.is_empty() {
                continue;
            }
            if !tenants.contains_key(parent_id) {
                return Err(TenantResolverError::Internal(format!(
                    "invalid tenant tree: tenant '{id}' references missing parent '{parent_id}'"
                )));
            }
        }

        // Build children adjacency.
        let mut children: HashMap<String, Vec<String>> = HashMap::new();
        for (id, parent_id) in &tenant_parent {
            if parent_id.is_empty() {
                continue;
            }
            children
                .entry(parent_id.clone())
                .or_default()
                .push(id.clone());
        }

        // Stabilize traversal order: sort children IDs for deterministic behavior.
        for ids in children.values_mut() {
            ids.sort();
        }

        // Cycle detection via iterative DFS (white/gray/black sets).
        let root = root_id.as_deref().ok_or_else(|| {
            TenantResolverError::Internal("invalid tenant tree: root missing".to_owned())
        })?;

        let mut visited: HashMap<&str, u8> = HashMap::new(); // 1=gray, 2=black
        let mut stack: Vec<(&str, VisitState)> = vec![(root, VisitState::Enter)];
        while let Some((node, st)) = stack.pop() {
            match st {
                VisitState::Enter => {
                    match visited.get(node).copied() {
                        Some(1) => {
                            return Err(TenantResolverError::Internal(format!(
                                "invalid tenant tree: cycle detected at '{node}'"
                            )));
                        }
                        Some(2) => continue,
                        _ => {}
                    }
                    visited.insert(node, 1);
                    stack.push((node, VisitState::Exit));
                    if let Some(kids) = children.get(node) {
                        for kid in kids {
                            stack.push((kid.as_str(), VisitState::Enter));
                        }
                    }
                }
                VisitState::Exit => {
                    visited.insert(node, 2);
                }
            }
        }

        // Ensure all nodes are reachable from root (tree/forest check).
        if visited.len() != tenants.len() {
            return Err(TenantResolverError::Internal(format!(
                "invalid tenant tree: {} tenant(s) are unreachable from root",
                tenants.len().saturating_sub(visited.len())
            )));
        }

        Ok(Self {
            tenants,
            children,
            root_id,
        })
    }

    /// Checks if access to tenant is allowed based on access options.
    fn check_access(tenant: &Tenant, access_options: &AccessOptions) -> bool {
        if access_options.ignore_parent_access_constraints {
            true
        } else {
            tenant.is_accessible_by_parent
        }
    }

    /// Collects all children recursively in pre-order traversal.
    fn collect_children(
        &self,
        parent_id: &str,
        filter: &TenantFilter,
        access_options: &AccessOptions,
        max_depth: u32,
        current_depth: u32,
        result: &mut Vec<Tenant>,
    ) {
        // Check depth limit (0 = unlimited)
        if max_depth > 0 && current_depth >= max_depth {
            return;
        }

        if let Some(child_ids) = self.children.get(parent_id) {
            for child_id in child_ids {
                if let Some(child) = self.tenants.get(child_id) {
                    // Check access constraints
                    if !Self::check_access(child, access_options) {
                        continue;
                    }

                    // Check filter
                    if filter.matches(child.status) {
                        result.push(child.clone());
                    }

                    // Recurse into children (pre-order: parent before subtree)
                    self.collect_children(
                        child_id,
                        filter,
                        access_options,
                        max_depth,
                        current_depth + 1,
                        result,
                    );
                }
            }
        }
    }
}

#[async_trait]
impl ThrPluginApi for Service {
    async fn get_root_tenant(&self, _ctx: &SecurityCtx) -> Result<Tenant, TenantResolverError> {
        self.root_id
            .as_ref()
            .and_then(|id| self.tenants.get(id))
            .cloned()
            .ok_or_else(|| TenantResolverError::NotFound("root tenant not configured".to_owned()))
    }

    async fn list_tenants(
        &self,
        _ctx: &SecurityCtx,
        filter: TenantFilter,
        query: ODataQuery,
    ) -> Result<Page<Tenant>, TenantResolverError> {
        tracing::debug!(
            limit = query.limit,
            has_cursor = query.cursor.is_some(),
            "Listing tenants (Fabrikam)"
        );
        // This example supports ONLY cursor-based pagination by `id` (ascending) and ignores $filter/$orderby.
        if query.filter.is_some() {
            return Err(TenantResolverError::Internal(
                "OData $filter is not supported by this plugin".to_owned(),
            ));
        }
        if !query.order.0.is_empty() {
            return Err(TenantResolverError::Internal(
                "OData $orderby is not supported by this plugin".to_owned(),
            ));
        }

        let limit = query.limit.unwrap_or(100);
        let effective_order = ODataOrderBy(vec![OrderKey {
            field: "id".to_owned(),
            dir: SortDir::Asc,
        }]);

        if let Some(cursor) = query.cursor.as_ref() {
            validate_cursor_against(cursor, &effective_order, query.filter_hash.as_deref())
                .map_err(|e| TenantResolverError::Internal(format!("invalid cursor: {e}")))?;
        }

        let mut items: Vec<Tenant> = self
            .tenants
            .values()
            .filter(|t| filter.matches(t.status))
            .cloned()
            .collect();
        items.sort_by(|a, b| a.id.cmp(&b.id));

        // Apply cursor (forward-only) by comparing last seen `id`
        if let Some(cursor) = query.cursor.as_ref() {
            let after_id = cursor.k.first().cloned().unwrap_or_default();
            if !after_id.is_empty() {
                items.retain(|t| t.id > after_id);
            }
        }

        let take = usize::try_from(limit).unwrap_or(100);
        let mut page_items = items.into_iter().take(take + 1).collect::<Vec<_>>();

        let next_cursor = if page_items.len() > take {
            let last = page_items
                .get(take.saturating_sub(1))
                .map(|t| t.id.clone())
                .unwrap_or_default();
            page_items.truncate(take);

            if last.is_empty() {
                None
            } else {
                let c = CursorV1 {
                    k: vec![last],
                    o: SortDir::Asc,
                    s: "+id".to_owned(),
                    f: query.filter_hash,
                    d: "fwd".to_owned(),
                };
                Some(c.encode().map_err(|e| {
                    TenantResolverError::Internal(format!("cursor encode failed: {e}"))
                })?)
            }
        } else {
            None
        };

        Ok(Page::new(
            page_items,
            PageInfo {
                next_cursor,
                prev_cursor: None,
                limit,
            },
        ))
    }

    async fn get_parents(
        &self,
        _ctx: &SecurityCtx,
        id: &str,
        filter: TenantFilter,
        access_options: AccessOptions,
    ) -> Result<GetParentsResponse, TenantResolverError> {
        tracing::debug!(tenant.id = %id, "Get parents (Fabrikam)");
        // Find the target tenant
        let tenant = self
            .tenants
            .get(id)
            .ok_or_else(|| TenantResolverError::NotFound(id.to_owned()))?;

        // Check filter on target
        if !filter.matches(tenant.status) {
            return Err(TenantResolverError::NotFound(format!(
                "tenant {id} does not match filter"
            )));
        }

        // Traverse up to collect parents
        let mut parents = Vec::new();
        let mut current_parent_id = tenant.parent_id.clone();

        while !current_parent_id.is_empty() {
            let parent = self.tenants.get(&current_parent_id).ok_or_else(|| {
                TenantResolverError::Internal(format!(
                    "parent {current_parent_id} not found (broken hierarchy)"
                ))
            })?;

            // Check access constraints
            if !Self::check_access(parent, &access_options) {
                return Err(TenantResolverError::PermissionDenied(format!(
                    "access to parent {} denied",
                    parent.id
                )));
            }

            // Apply filter to parents
            if filter.matches(parent.status) {
                parents.push(parent.clone());
            }

            current_parent_id = parent.parent_id.clone();
        }

        Ok(GetParentsResponse {
            tenant: tenant.clone(),
            parents,
        })
    }

    async fn get_children(
        &self,
        _ctx: &SecurityCtx,
        id: &str,
        filter: TenantFilter,
        access_options: AccessOptions,
        max_depth: u32,
    ) -> Result<Vec<Tenant>, TenantResolverError> {
        tracing::debug!(tenant.id = %id, max_depth, "Get children (Fabrikam)");
        // Verify parent tenant exists
        if !self.tenants.contains_key(id) {
            return Err(TenantResolverError::NotFound(id.to_owned()));
        }

        let mut result = Vec::new();
        self.collect_children(id, &filter, &access_options, max_depth, 0, &mut result);

        Ok(result)
    }
}
