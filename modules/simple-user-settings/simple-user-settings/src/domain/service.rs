use std::sync::Arc;

use authz_resolver_sdk::PolicyEnforcer;
use authz_resolver_sdk::pep::ResourceType;
use modkit_db::DBProvider;
use modkit_macros::domain_model;
use modkit_security::{SecurityContext, pep_properties};
use simple_user_settings_sdk::models::{
    SimpleUserSettings, SimpleUserSettingsPatch, SimpleUserSettingsUpdate,
};

use super::error::DomainError;
use super::fields::SettingsFields;
use super::repo::SettingsRepository;

pub(crate) type DbProvider = DBProvider<modkit_db::DbError>;

/// Authorization resource type for user settings.
///
/// Settings are scoped by tenant + user (resource). The PDP uses
/// `supported_properties` to decide which predicates it can return.
pub(crate) const SETTINGS_RESOURCE: ResourceType = ResourceType {
    name: "simple_user_settings.settings",
    supported_properties: &[pep_properties::OWNER_TENANT_ID, pep_properties::RESOURCE_ID],
};

pub(crate) mod actions {
    pub const GET: &str = "get";
    pub const UPDATE: &str = "update";
}

// ============================================================================
// Service Configuration
// ============================================================================

#[domain_model]
pub struct ServiceConfig {
    pub max_field_length: usize,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            max_field_length: 100,
        }
    }
}

// ============================================================================
// Service Implementation
// ============================================================================

#[domain_model]
pub struct Service<R: SettingsRepository> {
    db: Arc<DbProvider>,
    repo: Arc<R>,
    policy_enforcer: PolicyEnforcer,
    config: ServiceConfig,
}

impl<R: SettingsRepository> Service<R> {
    pub fn new(
        db: Arc<DbProvider>,
        repo: Arc<R>,
        policy_enforcer: PolicyEnforcer,
        config: ServiceConfig,
    ) -> Self {
        Self {
            db,
            repo,
            policy_enforcer,
            config,
        }
    }

    pub async fn get_settings(
        &self,
        ctx: &SecurityContext,
    ) -> Result<SimpleUserSettings, DomainError> {
        let user_id = ctx.subject_id();

        let scope = self
            .policy_enforcer
            .access_scope(ctx, &SETTINGS_RESOURCE, actions::GET, Some(user_id))
            .await?;
        let tenant_id = scope
            .extract_single_value(pep_properties::OWNER_TENANT_ID)
            .map_err(|_| {
                DomainError::Internal("unable to extract tenant id from scope".to_owned())
            })?;

        let conn = self.db.conn().map_err(DomainError::from)?;

        if let Some(settings) = self.repo.find_by_user(&conn, &scope).await? {
            Ok(settings)
        } else {
            Ok(SimpleUserSettings {
                user_id,
                tenant_id,
                theme: None,
                language: None,
            })
        }
    }

    pub async fn update_settings(
        &self,
        ctx: &SecurityContext,
        update: SimpleUserSettingsUpdate,
    ) -> Result<SimpleUserSettings, DomainError> {
        self.validate_field(SettingsFields::THEME, &update.theme)?;
        self.validate_field(SettingsFields::LANGUAGE, &update.language)?;

        let user_id = ctx.subject_id();

        let scope = self
            .policy_enforcer
            .access_scope(ctx, &SETTINGS_RESOURCE, actions::UPDATE, Some(user_id))
            .await?;
        let tenant_id = scope
            .extract_single_value(pep_properties::OWNER_TENANT_ID)
            .map_err(|_| {
                DomainError::Internal("unable to extract tenant id from scope".to_owned())
            })?;

        let conn = self.db.conn().map_err(DomainError::from)?;

        let settings = self
            .repo
            .upsert_full(
                &conn,
                &scope,
                user_id,
                tenant_id,
                Some(update.theme),
                Some(update.language),
            )
            .await?;
        Ok(settings)
    }

    pub async fn patch_settings(
        &self,
        ctx: &SecurityContext,
        patch: SimpleUserSettingsPatch,
    ) -> Result<SimpleUserSettings, DomainError> {
        if let Some(ref theme) = patch.theme {
            self.validate_field(SettingsFields::THEME, theme)?;
        }
        if let Some(ref language) = patch.language {
            self.validate_field(SettingsFields::LANGUAGE, language)?;
        }

        let user_id = ctx.subject_id();

        let scope = self
            .policy_enforcer
            .access_scope(ctx, &SETTINGS_RESOURCE, actions::UPDATE, Some(user_id))
            .await?;
        let tenant_id = scope
            .extract_single_value(pep_properties::OWNER_TENANT_ID)
            .map_err(|_| {
                DomainError::Internal("unable to extract tenant id from scope".to_owned())
            })?;

        let conn = self.db.conn().map_err(DomainError::from)?;

        let settings = self
            .repo
            .upsert_patch(&conn, &scope, user_id, tenant_id, patch)
            .await?;
        Ok(settings)
    }

    fn validate_field(&self, field: &str, value: &str) -> Result<(), DomainError> {
        if value.len() > self.config.max_field_length {
            return Err(DomainError::validation(
                field,
                format!("exceeds maximum length of {}", self.config.max_field_length),
            ));
        }
        Ok(())
    }
}
