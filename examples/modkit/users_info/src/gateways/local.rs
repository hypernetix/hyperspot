use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::contract::{
    client::UsersInfoApi,
    error::UsersInfoError,
    model::{NewUser, User, UserPatch},
};
use crate::domain::service::Service;
use modkit_db::secure::SecurityCtx;
use modkit_odata::{ODataQuery, Page};

/// Local implementation of the UsersInfoApi trait that delegates to the domain service.
///
/// This client is used for inter-module communication within the same process.
/// It uses a system-level security context with resource-based access control.
pub struct UsersInfoLocalClient {
    service: Arc<Service>,
}

impl UsersInfoLocalClient {
    pub fn new(service: Arc<Service>) -> Self {
        Self { service }
    }

    /// Create a security context for inter-module calls.
    ///
    /// For resource-specific operations (get, update, delete), we pass the resource ID
    /// in the security scope to allow access to that specific resource.
    ///
    /// For list operations, we could either:
    /// 1. Pass no scope (deny all) - not useful
    /// 2. Pass all resource IDs - impractical
    /// 3. Use a special "system" context that bypasses filtering
    ///
    /// In this example, since users are a global entity (no tenant column),
    /// we'll pass an empty resource list which will result in deny-all filtering.
    /// In production, you'd want to add proper tenant isolation or use a
    /// system-level bypass mechanism.
    fn system_ctx_for_resource(&self, resource_id: Uuid) -> SecurityCtx {
        // System user ID for inter-module calls
        let system_user = Uuid::nil();
        SecurityCtx::for_resource(resource_id, system_user)
    }

    /// Create a deny-all security context for operations without a specific resource.
    ///
    /// Note: This will cause list operations to return no results for global entities
    /// without tenant columns. In production, you'd implement a proper solution like:
    /// - Adding tenant_id to the users table
    /// - Using a special "system" context that bypasses filtering
    /// - Implementing a different access model for inter-module calls
    fn system_ctx_global(&self) -> SecurityCtx {
        let system_user = Uuid::nil();
        SecurityCtx::deny_all(system_user)
    }
}

#[async_trait]
impl UsersInfoApi for UsersInfoLocalClient {
    async fn get_user(&self, id: Uuid) -> Result<User, UsersInfoError> {
        let ctx = self.system_ctx_for_resource(id);
        self.service.get_user(&ctx, id).await.map_err(Into::into)
    }

    async fn list_users(&self, query: ODataQuery) -> Result<Page<User>, UsersInfoError> {
        // Note: Using deny_all context for global entities without proper tenant isolation
        // This is a limitation of the example. In production, implement proper multi-tenancy.
        let ctx = self.system_ctx_global();
        self.service
            .list_users_page(&ctx, query)
            .await
            .map_err(|e| {
                // OData errors at this layer are unexpected (query construction errors)
                // Log and convert to internal error
                tracing::error!(error = ?e, "Unexpected OData error in gateway");
                UsersInfoError::internal()
            })
    }

    async fn create_user(&self, new_user: NewUser) -> Result<User, UsersInfoError> {
        // For create, we use deny_all since the resource doesn't exist yet
        let ctx = self.system_ctx_global();
        self.service
            .create_user(&ctx, new_user)
            .await
            .map_err(Into::into)
    }

    async fn update_user(&self, id: Uuid, patch: UserPatch) -> Result<User, UsersInfoError> {
        let ctx = self.system_ctx_for_resource(id);
        self.service
            .update_user(&ctx, id, patch)
            .await
            .map_err(Into::into)
    }

    async fn delete_user(&self, id: Uuid) -> Result<(), UsersInfoError> {
        let ctx = self.system_ctx_for_resource(id);
        self.service.delete_user(&ctx, id).await.map_err(Into::into)
    }
}
