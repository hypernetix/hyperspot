use uuid::Uuid;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AccessScope {
    /// True if this is a root scope (system-level access with no tenant filtering)
    pub(crate) is_root: bool,
    pub(crate) tenant_ids: Vec<Uuid>,
    pub(crate) resource_ids: Vec<Uuid>,
    // future: include_descendants (unused in v1)
    // pub(crate) include_descendants: bool,
}

impl AccessScope {
    #[inline]
    pub fn tenant_ids(&self) -> &[Uuid] {
        &self.tenant_ids
    }
    #[inline]
    pub fn resource_ids(&self) -> &[Uuid] {
        &self.resource_ids
    }

    /// Returns true if this is a root scope (system-level access).
    #[inline]
    pub fn is_root(&self) -> bool {
        self.is_root
    }

    /// Returns true if this scope is empty (no tenants, no resources, not root).
    /// A root scope is never considered empty.
    pub fn is_empty(&self) -> bool {
        if self.is_root {
            return false;
        }
        self.tenant_ids.is_empty() && self.resource_ids.is_empty()
    }
    pub fn has_tenants(&self) -> bool {
        !self.tenant_ids.is_empty()
    }
    pub fn has_resources(&self) -> bool {
        !self.resource_ids.is_empty()
    }

    pub fn tenants_only(tenant_ids: Vec<Uuid>) -> Self {
        Self {
            is_root: false,
            tenant_ids,
            resource_ids: vec![],
        }
    }
    pub fn resources_only(resource_ids: Vec<Uuid>) -> Self {
        Self {
            is_root: false,
            tenant_ids: vec![],
            resource_ids,
        }
    }

    pub fn tenant(tenant_id: Uuid) -> Self {
        Self::tenants_only(vec![tenant_id])
    }
    pub fn resource(resource_id: Uuid) -> Self {
        Self::resources_only(vec![resource_id])
    }

    /// Create a scope with both tenant and resource constraints (AND).
    /// This is less common but useful for very specific access scenarios.
    pub fn both(tenant_ids: Vec<Uuid>, resource_ids: Vec<Uuid>) -> Self {
        Self {
            is_root: false,
            tenant_ids,
            resource_ids,
        }
    }

    /// Root scope for system-level access.
    /// This bypasses all tenant filtering and allows access to all tenants.
    /// Resource filters can still be applied if resource_ids are set.
    pub fn root_tenant() -> Self {
        Self {
            is_root: true,
            tenant_ids: Vec::new(),
            resource_ids: Vec::new(),
        }
    }

    /// True if this scope explicitly includes the root tenant.
    pub fn includes_root_tenant(&self) -> bool {
        self.tenant_ids.contains(&crate::constants::ROOT_TENANT_ID)
    }
}
