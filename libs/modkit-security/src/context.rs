use crate::{ROOT_SUBJECT_ID, ROOT_TENANT_ID};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Permission {
    resource: String,
    action: String,
}

impl Permission {
    #[must_use]
    pub fn resource(&self) -> &str {
        &self.resource
    }

    #[must_use]
    pub fn action(&self) -> &str {
        &self.action
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SecurityContext {
    tenant_id: Uuid,
    subject_id: Uuid,
    permissions: Vec<Permission>,
    environment: Vec<(String, String)>,
}

impl SecurityContext {
    /// Create a new SecurityContext builder
    pub fn builder() -> SecurityContextBuilder {
        SecurityContextBuilder::default()
    }

    pub fn root() -> Self {
        SecurityContextBuilder::default()
            .tenant_id(ROOT_TENANT_ID)
            .subject_id(ROOT_SUBJECT_ID)
            .build()
    }

    /// Create an anonymous SecurityContext with no tenant, subject, or permissions
    pub fn anonymous() -> Self {
        SecurityContextBuilder::default().build()
    }
    /// Get the tenant ID associated with the security context
    pub fn tenant_id(&self) -> Uuid {
        self.tenant_id
    }

    /// Get the subject ID (user, service, or system) associated with the security context
    pub fn subject_id(&self) -> Uuid {
        self.subject_id
    }

    // resource:uuid
    // gts.htx.events.tenant.v1~{uuid}:edit
    // gts.htx.events.topic.v1~vendor.tenants.v1:publish
    // gts.htx.events.topic.v1~*:publish
    // gts.htx.events.subject_types.v1~vendor.xxx.*:subscribe

    /// Get the permissions assigned to the security context
    pub fn permissions(&self) -> Vec<Permission> {
        self.permissions.clone()
    }

    /// Get the environmental attributes associated with the security context
    /// (e.g., IP address, device type, location, time, etc.)
    pub fn environment(&self) -> Vec<(String, String)> {
        self.environment.clone()
    }
}

#[derive(Default)]
pub struct SecurityContextBuilder {
    tenant_id: Option<Uuid>,
    subject_id: Option<Uuid>,
    permissions: Vec<Permission>,
    environment: Vec<(String, String)>,
}

impl SecurityContextBuilder {
    pub fn tenant_id(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = Some(tenant_id);
        self
    }

    pub fn subject_id(mut self, subject_id: Uuid) -> Self {
        self.subject_id = Some(subject_id);
        self
    }

    pub fn add_permission(mut self, resource: &str, action: &str) -> Self {
        self.permissions.push(Permission {
            resource: resource.to_string(),
            action: action.to_string(),
        });
        self
    }

    pub fn add_environment_attribute(mut self, key: &str, value: &str) -> Self {
        self.environment.push((key.to_string(), value.to_string()));
        self
    }

    pub fn build(self) -> SecurityContext {
        SecurityContext {
            tenant_id: self.tenant_id.unwrap_or_else(Uuid::nil),
            subject_id: self.subject_id.unwrap_or_else(Uuid::nil),
            permissions: self.permissions,
            environment: self.environment,
        }
    }
}

/// Policy Engine - Zero Trust Policy Engine, responsible for evaluating and enforcing policies or rules
pub trait PolicyEngine: Send + Sync {
    fn context(&self) -> &SecurityContext;

    fn allows(&self, resource: &str, action: &str) -> bool;

    fn tenant_parents(&self) -> Vec<Uuid>;

    fn tenant_children(&self) -> Vec<Uuid>;

    fn can_access_tenant(&self, tenant_id: Uuid) -> bool;
}

// resource + action
// environment variables

// tenant + eb:subscriber - RBAC
// "Resource": "arn:aws:sqs:us-east-2:account-ID-without-hyphens:queue1"

/*
"Resource": [
    "arn:aws:dynamodb:us-east-2:account-ID-without-hyphens:table/books_table",
    "arn:aws:dynamodb:us-east-2:account-ID-without-hyphens:table/magazines_table"
]
 */
/*
   1. EM: tenant + [event types] + [subject types] + [publish|subscribe|fetch] -ABAC
   2. Tenant Resolver: tenant + ignore_barriers + [tr_fetcher]
   3.
*/

pub struct SimplePolicyEngine {
    context: SecurityContext,
}

impl SimplePolicyEngine {
    pub fn new(context: SecurityContext) -> Self {
        Self { context }
    }
}

impl PolicyEngine for SimplePolicyEngine {
    fn context(&self) -> &SecurityContext {
        &self.context
    }

    fn allows(&self, resource: &str, action: &str) -> bool {
        self.context
            .permissions()
            .iter()
            .any(|perm| perm.resource() == resource && perm.action() == action)
    }

    fn tenant_parents(&self) -> Vec<Uuid> {
        vec![]
    }

    fn tenant_children(&self) -> Vec<Uuid> {
        vec![]
    }

    fn can_access_tenant(&self, tenant_id: Uuid) -> bool {
        self.context.tenant_id() == tenant_id
    }
}
