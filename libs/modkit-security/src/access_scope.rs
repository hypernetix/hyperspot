use uuid::Uuid;

/// Access scope defining which tenants and resources a request can access.
///
/// An empty scope (no tenants, no resources) is considered a "deny all" scope.
/// To access data, the scope must contain at least one tenant ID or resource ID.
#[derive(Clone, Debug, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct AccessScope {
    pub(crate) tenant_ids: Vec<Uuid>,
    pub(crate) types: Vec<Uuid>,
    pub(crate) resource_ids: Vec<Uuid>,
}

impl AccessScope {
    #[inline]
    #[must_use]
    pub fn tenant_ids(&self) -> &[Uuid] {
        &self.tenant_ids
    }

    #[inline]
    #[must_use]
    pub fn resource_ids(&self) -> &[Uuid] {
        &self.resource_ids
    }

    /// Returns true if this scope is empty (no tenants, no resources).
    /// An empty scope results in a "deny all" condition in queries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tenant_ids.is_empty() && self.resource_ids.is_empty()
    }

    #[must_use]
    pub fn has_tenants(&self) -> bool {
        !self.tenant_ids.is_empty()
    }

    #[must_use]
    pub fn has_resources(&self) -> bool {
        !self.resource_ids.is_empty()
    }

    #[must_use]
    pub fn tenants_only(tenant_ids: Vec<Uuid>) -> Self {
        Self {
            tenant_ids,
            types: vec![],
            resource_ids: vec![],
        }
    }

    #[must_use]
    pub fn resources_only(resource_ids: Vec<Uuid>) -> Self {
        Self {
            tenant_ids: vec![],
            types: vec![],
            resource_ids,
        }
    }

    #[must_use]
    pub fn tenant(tenant_id: Uuid) -> Self {
        Self::tenants_only(vec![tenant_id])
    }

    #[must_use]
    pub fn resource(resource_id: Uuid) -> Self {
        Self::resources_only(vec![resource_id])
    }

    /// Create a scope with both tenant and resource constraints (AND).
    /// This is less common but useful for very specific access scenarios.
    #[must_use]
    pub fn both(tenant_ids: Vec<Uuid>, resource_ids: Vec<Uuid>) -> Self {
        Self {
            tenant_ids,
            types: vec![],
            resource_ids,
        }
    }
}
