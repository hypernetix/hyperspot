//! Local client implementing the `TypesRegistryApi` trait.

use std::sync::Arc;

use async_trait::async_trait;
use modkit_security::SecurityCtx;
use types_registry_sdk::{
    GtsEntity, ListQuery, RegisterResult, TypesRegistryApi, TypesRegistryError,
};

use crate::domain::service::TypesRegistryService;

/// Local client for the Types Registry module.
///
/// This client implements the `TypesRegistryApi` trait and delegates
/// to the domain service. It is registered in the `ClientHub` for
/// inter-module communication.
pub struct TypesRegistryLocalClient {
    service: Arc<TypesRegistryService>,
}

impl TypesRegistryLocalClient {
    /// Creates a new local client with the given service.
    #[must_use]
    pub fn new(service: Arc<TypesRegistryService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl TypesRegistryApi for TypesRegistryLocalClient {
    async fn register(
        &self,
        _ctx: &SecurityCtx,
        entities: Vec<serde_json::Value>,
    ) -> Result<Vec<RegisterResult>, TypesRegistryError> {
        Ok(self.service.register(entities))
    }

    async fn list(
        &self,
        _ctx: &SecurityCtx,
        query: ListQuery,
    ) -> Result<Vec<GtsEntity>, TypesRegistryError> {
        self.service.list(&query).map_err(TypesRegistryError::from)
    }

    async fn get(&self, _ctx: &SecurityCtx, gts_id: &str) -> Result<GtsEntity, TypesRegistryError> {
        self.service.get(gts_id).map_err(TypesRegistryError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infra::InMemoryGtsRepository;
    use gts::GtsConfig;
    use serde_json::json;

    fn default_config() -> GtsConfig {
        crate::config::TypesRegistryConfig::default().to_gts_config()
    }

    fn create_client() -> TypesRegistryLocalClient {
        let repo = Arc::new(InMemoryGtsRepository::new(default_config()));
        let service = Arc::new(TypesRegistryService::new(
            repo,
            crate::config::TypesRegistryConfig::default(),
        ));
        TypesRegistryLocalClient::new(service)
    }

    fn test_ctx() -> SecurityCtx {
        SecurityCtx::root_ctx()
    }

    #[tokio::test]
    async fn test_register_and_get() {
        let client = create_client();
        let ctx = test_ctx();

        let entity = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object",
            "properties": {
                "userId": { "type": "string" }
            }
        });

        let results = client.register(&ctx, vec![entity]).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());

        client.service.switch_to_ready().unwrap();

        let retrieved = client
            .get(&ctx, "gts.acme.core.events.user_created.v1~")
            .await
            .unwrap();
        assert_eq!(retrieved.gts_id, "gts.acme.core.events.user_created.v1~");
    }

    #[tokio::test]
    async fn test_list_entities() {
        let client = create_client();
        let ctx = test_ctx();

        let type1 = json!({
            "$id": "gts://gts.acme.core.events.user_created.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });
        let type2 = json!({
            "$id": "gts://gts.globex.core.events.order_placed.v1~",
            "$schema": "http://json-schema.org/draft-07/schema#",
            "type": "object"
        });

        client.register(&ctx, vec![type1, type2]).await.unwrap();
        client.service.switch_to_ready().unwrap();

        let all = client.list(&ctx, ListQuery::default()).await.unwrap();
        assert_eq!(all.len(), 2);

        let acme_only = client
            .list(&ctx, ListQuery::default().with_vendor("acme"))
            .await
            .unwrap();
        assert_eq!(acme_only.len(), 1);
        assert_eq!(acme_only[0].vendor(), Some("acme"));
    }

    #[tokio::test]
    async fn test_get_not_found() {
        let client = create_client();
        let ctx = test_ctx();

        client.service.switch_to_ready().unwrap();

        let result = client.get(&ctx, "gts.unknown.pkg.ns.type.v1~").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().is_not_found());
    }
}
