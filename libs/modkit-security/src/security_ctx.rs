use crate::{constants, AccessScope, Subject};
use uuid::Uuid;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug)]
pub struct SecurityCtx {
    scope: AccessScope,
    subject: Subject,
}

impl SecurityCtx {
    pub fn new(scope: AccessScope, subject: Subject) -> Self {
        Self { scope, subject }
    }

    pub fn for_tenants(tenant_ids: Vec<Uuid>, subject_id: Uuid) -> Self {
        Self {
            scope: AccessScope::tenants_only(tenant_ids),
            subject: Subject::new(subject_id),
        }
    }
    pub fn for_tenant(tenant_id: Uuid, subject_id: Uuid) -> Self {
        Self::for_tenants(vec![tenant_id], subject_id)
    }

    pub fn for_resources(resource_ids: Vec<Uuid>, subject_id: Uuid) -> Self {
        Self {
            scope: AccessScope::resources_only(resource_ids),
            subject: Subject::new(subject_id),
        }
    }
    pub fn for_resource(resource_id: Uuid, subject_id: Uuid) -> Self {
        Self::for_resources(vec![resource_id], subject_id)
    }

    pub fn deny_all(subject_id: Uuid) -> Self {
        Self {
            scope: AccessScope::default(),
            subject: Subject::new(subject_id),
        }
    }

    /// Anonymous/unauthenticated context with no access to any resources.
    /// Use this for public routes where no authentication is required.
    pub fn anonymous() -> Self {
        Self::deny_all(constants::ANONYMOUS_SUBJECT_ID)
    }

    /// Root subject operating within the root tenant (system context).
    pub fn root_ctx() -> Self {
        Self::new(AccessScope::root_tenant(), Subject::root())
    }

    #[inline]
    pub fn scope(&self) -> &AccessScope {
        &self.scope
    }
    #[inline]
    pub fn subject(&self) -> &Subject {
        &self.subject
    }
    #[inline]
    pub fn subject_id(&self) -> Uuid {
        self.subject.id
    }

    pub fn is_denied(&self) -> bool {
        self.scope.is_empty()
    }
    pub fn has_tenant_access(&self) -> bool {
        self.scope.has_tenants()
    }
    pub fn has_resource_access(&self) -> bool {
        self.scope.has_resources()
    }

    // audit helpers
    #[inline]
    pub fn created_by(&self) -> Uuid {
        self.subject_id()
    }
    #[inline]
    pub fn updated_by(&self) -> Uuid {
        self.subject_id()
    }
}
