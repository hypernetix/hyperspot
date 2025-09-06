use chrono::Utc;
use sea_orm::DatabaseConnection;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::contract::model::{NewUser, User, UserPatch};
use crate::domain::error::DomainError;
use crate::infra::storage::entity;

/// Domain service containing business logic for user management
#[derive(Clone)]
pub struct Service {
    db: DatabaseConnection,
    config: ServiceConfig,
}

/// Configuration for the domain service
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub max_display_name_length: usize,
    pub default_page_size: u32,
    pub max_page_size: u32,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            max_display_name_length: 100,
            default_page_size: 50,
            max_page_size: 1000,
        }
    }
}

impl Service {
    pub fn new(db: DatabaseConnection, config: ServiceConfig) -> Self {
        Self { db, config }
    }

    #[instrument(skip(self), fields(user_id = %id))]
    pub async fn get_user(&self, id: Uuid) -> Result<User, DomainError> {
        debug!("Getting user by id");

        let entity = entity::find_by_id(&self.db, id)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?
            .ok_or_else(|| DomainError::user_not_found(id))?;

        let user: User = entity.into();
        debug!("Successfully retrieved user");
        Ok(user)
    }

    #[instrument(skip(self))]
    pub async fn list_users(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<User>, DomainError> {
        let limit = limit
            .unwrap_or(self.config.default_page_size)
            .min(self.config.max_page_size);
        let offset = offset.unwrap_or(0);

        debug!("Listing users with limit={}, offset={}", limit, offset);

        let entities = entity::find_paginated(&self.db, limit, offset)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let users: Vec<User> = entities.into_iter().map(Into::into).collect();

        debug!("Successfully listed {} users", users.len());
        Ok(users)
    }

    #[instrument(skip(self), fields(email = %new_user.email, display_name = %new_user.display_name))]
    pub async fn create_user(&self, new_user: NewUser) -> Result<User, DomainError> {
        info!("Creating new user");

        // Validate input
        self.validate_new_user(&new_user)?;

        // Check for email uniqueness
        if entity::email_exists(&self.db, &new_user.email)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?
        {
            return Err(DomainError::email_already_exists(new_user.email));
        }

        let now = Utc::now();
        let id = Uuid::new_v4();

        let entity_model = entity::NewUserEntity {
            id,
            email: new_user.email,
            display_name: new_user.display_name,
            created_at: now,
            updated_at: now,
        };

        let created_entity = entity::create(&self.db, entity_model)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let user: User = created_entity.into();
        info!("Successfully created user with id={}", user.id);
        Ok(user)
    }

    #[instrument(skip(self), fields(user_id = %id))]
    pub async fn update_user(&self, id: Uuid, patch: UserPatch) -> Result<User, DomainError> {
        info!("Updating user");

        // Validate patch
        self.validate_user_patch(&patch)?;

        // Check if user exists
        let existing = entity::find_by_id(&self.db, id)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?
            .ok_or_else(|| DomainError::user_not_found(id))?;

        // Check email uniqueness if email is being changed
        if let Some(ref new_email) = patch.email {
            if new_email != &existing.email
                && entity::email_exists(&self.db, new_email)
                    .await
                    .map_err(|e| DomainError::database(e.to_string()))?
            {
                return Err(DomainError::email_already_exists(new_email.clone()));
            }
        }

        let update_data = entity::UpdateUserEntity {
            email: patch.email,
            display_name: patch.display_name,
            updated_at: Some(Utc::now()),
        };

        let updated_entity = entity::update(&self.db, id, update_data)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let user: User = updated_entity.into();
        info!("Successfully updated user");
        Ok(user)
    }

    #[instrument(skip(self), fields(user_id = %id))]
    pub async fn delete_user(&self, id: Uuid) -> Result<(), DomainError> {
        info!("Deleting user");

        let deleted = entity::delete(&self.db, id)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        if !deleted {
            return Err(DomainError::user_not_found(id));
        }

        info!("Successfully deleted user");
        Ok(())
    }

    /// Validate new user data
    fn validate_new_user(&self, new_user: &NewUser) -> Result<(), DomainError> {
        self.validate_email(&new_user.email)?;
        self.validate_display_name(&new_user.display_name)?;
        Ok(())
    }

    /// Validate user patch data
    fn validate_user_patch(&self, patch: &UserPatch) -> Result<(), DomainError> {
        if let Some(ref email) = patch.email {
            self.validate_email(email)?;
        }
        if let Some(ref display_name) = patch.display_name {
            self.validate_display_name(display_name)?;
        }
        Ok(())
    }

    /// Validate email format
    fn validate_email(&self, email: &str) -> Result<(), DomainError> {
        if email.is_empty() || !email.contains('@') || !email.contains('.') {
            return Err(DomainError::invalid_email(email.to_string()));
        }
        Ok(())
    }

    /// Validate display name
    fn validate_display_name(&self, display_name: &str) -> Result<(), DomainError> {
        if display_name.trim().is_empty() {
            return Err(DomainError::empty_display_name());
        }
        if display_name.len() > self.config.max_display_name_length {
            return Err(DomainError::display_name_too_long(
                display_name.len(),
                self.config.max_display_name_length,
            ));
        }
        Ok(())
    }
}
