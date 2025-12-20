#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Common test utilities for types-registry integration tests

use std::sync::Arc;

use gts::GtsConfig;
use types_registry::{domain::service::TypesRegistryService, infra::InMemoryGtsRepository};

pub fn default_config() -> GtsConfig {
    GtsConfig {
        entity_id_fields: vec!["$id".to_owned(), "gtsId".to_owned(), "id".to_owned()],
        schema_id_fields: vec!["$schema".to_owned(), "gtsTid".to_owned(), "type".to_owned()],
    }
}

pub fn create_service() -> Arc<TypesRegistryService> {
    let repo = Arc::new(InMemoryGtsRepository::new(default_config()));
    Arc::new(TypesRegistryService::new(repo))
}
