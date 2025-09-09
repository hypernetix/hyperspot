use sea_orm::DatabaseConnection;
use tracing::debug;
use uuid::Uuid;

use crate::contract::model::{Settings, SettingsPatch};
use crate::domain::error::DomainError;
use crate::infra::storage::entity;

/// Domain service containing business logic for settings management
#[derive(Clone)]
pub struct Service {
    db: DatabaseConnection,
    config: ServiceConfig,
}

/// Configuration for the domain service
#[derive(Debug, Clone)]
pub struct ServiceConfig {}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {}
    }
}

impl Service {
    pub fn new(db: DatabaseConnection, config: ServiceConfig) -> Self {
        Self { db, config }
    }

    pub async fn get_settings(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
    ) -> Result<Settings, DomainError> {
        debug!(
            "Getting settings for user_id: {} and tenant_id: {}",
            user_id, tenant_id
        );

        let entity = entity::find(&self.db, user_id, tenant_id)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let settings: Settings = match entity {
            Some(entity) => entity.into(),
            None => Settings::default(),
        };

        debug!("Successfully retrieved settings");
        Ok(settings)
    }

    pub async fn update_settings(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        patch: SettingsPatch,
    ) -> Result<Settings, DomainError> {
        let update_entity = entity::UpdateSettingsEntity {
            theme: patch.theme,
            language: patch.language,
        };

        let entity = entity::update(&self.db, user_id, tenant_id, update_entity)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        debug!("Successfully retrieved updated settings");
        Ok(entity.into())
    }
}
