#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;
use std::sync::Arc;

use hs_tenant_resolver_sdk::{
    TenantFilter, TenantResolverError, TenantResolverGatewayClient, TenantStatus,
};
use modkit::config::ConfigProvider;
use modkit::{ClientHub, DatabaseCapability, Module, ModuleCtx};
use modkit_db::{ConnectOpts, DbHandle};
use modkit_security::SecurityContext;
use serde_json::json;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use user_info_sdk::{NewUser, UsersInfoClient};
use users_info::UsersInfo;

/// Mock tenant resolver for tests.
struct MockTenantResolver;

#[async_trait::async_trait]
impl TenantResolverGatewayClient for MockTenantResolver {
    async fn get_tenant(
        &self,
        _ctx: &SecurityContext,
        id: hs_tenant_resolver_sdk::TenantId,
    ) -> Result<hs_tenant_resolver_sdk::TenantInfo, TenantResolverError> {
        Ok(hs_tenant_resolver_sdk::TenantInfo {
            id,
            name: format!("Tenant {id}"),
            status: TenantStatus::Active,
            tenant_type: None,
        })
    }

    async fn can_access(
        &self,
        ctx: &SecurityContext,
        target: hs_tenant_resolver_sdk::TenantId,
        _options: Option<&hs_tenant_resolver_sdk::AccessOptions>,
    ) -> Result<bool, TenantResolverError> {
        Ok(ctx.tenant_id() == target)
    }

    async fn get_accessible_tenants(
        &self,
        ctx: &SecurityContext,
        _filter: Option<&TenantFilter>,
        _options: Option<&hs_tenant_resolver_sdk::AccessOptions>,
    ) -> Result<Vec<hs_tenant_resolver_sdk::TenantInfo>, TenantResolverError> {
        let tenant_id = ctx.tenant_id();
        if tenant_id == Uuid::default() {
            return Ok(vec![]);
        }
        Ok(vec![hs_tenant_resolver_sdk::TenantInfo {
            id: tenant_id,
            name: format!("Tenant {tenant_id}"),
            status: TenantStatus::Active,
            tenant_type: None,
        }])
    }
}

struct MockConfigProvider {
    modules: HashMap<String, serde_json::Value>,
}

impl MockConfigProvider {
    fn new_users_info_default() -> Self {
        let mut modules = HashMap::new();
        // ModuleCtx::raw_config expects: modules.<name> = { database: ..., config: ... }
        // For this test we supply config only; DB handle is injected directly.
        modules.insert(
            "users_info".to_owned(),
            json!({
                "config": {
                    "default_page_size": 50,
                    "max_page_size": 1000,
                    "audit_base_url": "http://audit.local",
                    "notifications_base_url": "http://notifications.local",
                }
            }),
        );
        Self { modules }
    }
}

impl ConfigProvider for MockConfigProvider {
    fn get_module_config(&self, module_name: &str) -> Option<&serde_json::Value> {
        self.modules.get(module_name)
    }
}

#[tokio::test]
async fn users_info_registers_sdk_client_and_handles_basic_crud() {
    // Arrange: build a real DbHandle for sqlite in-memory, run module migrations, then init module.
    let db = DbHandle::connect("sqlite::memory:", ConnectOpts::default())
        .await
        .expect("db connect");
    let db = Arc::new(db);

    let hub = Arc::new(ClientHub::new());

    // Register mock tenant resolver before initializing the module
    hub.register::<dyn TenantResolverGatewayClient>(Arc::new(MockTenantResolver));

    let ctx = ModuleCtx::new(
        "users_info",
        Uuid::new_v4(),
        Arc::new(MockConfigProvider::new_users_info_default()),
        hub.clone(),
        CancellationToken::new(),
        Some(db.clone()),
    );

    let module = UsersInfo::default();
    module.migrate(db.as_ref()).await.expect("migrate");
    module.init(&ctx).await.expect("init");

    // Act: resolve SDK client from hub and do basic CRUD.
    let client = ctx
        .client_hub()
        .get::<dyn UsersInfoClient>()
        .expect("UsersInfoClient must be registered");

    // Create a security context with tenant access
    let tenant_id = Uuid::new_v4();
    let sec = SecurityContext::builder()
        .tenant_id(tenant_id)
        .subject_id(Uuid::new_v4())
        .build();

    let created = client
        .create_user(
            sec.clone(),
            NewUser {
                id: None,
                tenant_id,
                email: "test@example.com".to_owned(),
                display_name: "Test".to_owned(),
            },
        )
        .await
        .unwrap();

    let fetched = client.get_user(sec.clone(), created.id).await.unwrap();
    assert_eq!(fetched.email, "test@example.com");

    client.delete_user(sec, created.id).await.unwrap();
}
