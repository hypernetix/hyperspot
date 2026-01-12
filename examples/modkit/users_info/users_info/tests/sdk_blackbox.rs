#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashMap;
use std::sync::Arc;

use modkit::config::ConfigProvider;
use modkit::{ClientHub, DbModule, Module, ModuleCtx};
use modkit_db::{ConnectOpts, DbHandle};
use modkit_security::SecurityContext;
use serde_json::json;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use user_info_sdk::{NewUser, UsersInfoClient};
use users_info::UsersInfo;

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

    let sec = SecurityContext::root();
    let tenant_id = Uuid::new_v4();

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
