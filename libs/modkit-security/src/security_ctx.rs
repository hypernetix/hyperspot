use crate::{AccessScope, Subject};
use uuid::Uuid;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
// TO BE DELETED AND REPLACED WITH SecurityContext
pub struct SecurityCtx {
    scope: AccessScope,
    subject: Subject,
}

impl SecurityCtx {
    #[must_use]
    #[deprecated]
    pub fn new(scope: AccessScope, subject: Subject) -> Self {
        Self { scope, subject }
    }

    #[must_use]
    #[deprecated]
    pub fn for_tenants(tenant_ids: Vec<Uuid>, subject_id: Uuid) -> Self {
        Self {
            scope: AccessScope::tenants_only(tenant_ids),
            subject: Subject::new(subject_id),
        }
    }
    #[must_use]
    #[deprecated]
    pub fn for_tenant(tenant_id: Uuid, subject_id: Uuid) -> Self {
        #[allow(deprecated)]
        Self::for_tenants(vec![tenant_id], subject_id)
    }

    #[must_use]
    #[deprecated]
    pub fn for_resources(resource_ids: Vec<Uuid>, subject_id: Uuid) -> Self {
        Self {
            scope: AccessScope::resources_only(resource_ids),
            subject: Subject::new(subject_id),
        }
    }
    #[must_use]
    #[deprecated]
    pub fn for_resource(resource_id: Uuid, subject_id: Uuid) -> Self {
        #[allow(deprecated)]
        Self::for_resources(vec![resource_id], subject_id)
    }

    #[must_use]
    #[deprecated]
    pub fn deny_all(subject_id: Uuid) -> Self {
        Self {
            scope: AccessScope::default(),
            subject: Subject::new(subject_id),
        }
    }

    #[inline]
    #[must_use]
    pub fn scope(&self) -> &AccessScope {
        &self.scope
    }

    #[inline]
    #[must_use]
    pub fn subject(&self) -> &Subject {
        &self.subject
    }

    #[inline]
    #[must_use]
    pub fn subject_id(&self) -> Uuid {
        self.subject.id
    }

    #[must_use]
    pub fn is_denied(&self) -> bool {
        self.scope.is_empty()
    }
    #[must_use]
    pub fn has_tenant_access(&self) -> bool {
        self.scope.has_tenants()
    }
    #[must_use]
    pub fn has_resource_access(&self) -> bool {
        self.scope.has_resources()
    }

    // audit helpers
    #[inline]
    #[must_use]
    pub fn created_by(&self) -> Uuid {
        self.subject_id()
    }
    #[inline]
    #[must_use]
    pub fn updated_by(&self) -> Uuid {
        self.subject_id()
    }
}
