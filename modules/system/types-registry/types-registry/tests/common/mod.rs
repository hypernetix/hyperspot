#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Common test utilities for types-registry integration tests

use std::sync::Arc;

use gts::GtsConfig;
use types_registry::{
    config::TypesRegistryConfig, domain::service::TypesRegistryService,
    infra::InMemoryGtsRepository,
};

pub fn default_config() -> GtsConfig {
    TypesRegistryConfig::default().to_gts_config()
}

pub fn create_service() -> Arc<TypesRegistryService> {
    let repo = Arc::new(InMemoryGtsRepository::new(default_config()));
    Arc::new(TypesRegistryService::new(
        repo,
        TypesRegistryConfig::default(),
    ))
}
