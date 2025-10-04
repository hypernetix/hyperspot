use std::sync::Arc;

use crate::contract::model::{NewUser, User, UserPatch};
use crate::domain::error::DomainError;
use crate::domain::events::UserDomainEvent;
use crate::domain::ports::{AuditPort, EventPublisher};
use crate::domain::repo::UsersRepository;
use chrono::Utc;
use modkit_odata::{ODataQuery, Page};
use tracing::{debug, info, instrument};
use uuid::Uuid;

/// Domain service with business rules for user management.
/// Depends only on the repository port, not on infra types.
#[derive(Clone)]
pub struct Service {
    repo: Arc<dyn UsersRepository>,
    events: Arc<dyn EventPublisher<UserDomainEvent>>,
    audit: Arc<dyn AuditPort>,
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
    /// Create a service with dependencies.
    pub fn new(
        repo: Arc<dyn UsersRepository>,
        events: Arc<dyn EventPublisher<UserDomainEvent>>,
        audit: Arc<dyn AuditPort>,
        config: ServiceConfig,
    ) -> Self {
        Self {
            repo,
            events,
            audit,
            config,
        }
    }

    #[instrument(name = "users_info.service.get_user", skip(self), fields(user_id = %id))]
    pub async fn get_user(&self, id: Uuid) -> Result<User, DomainError> {
        debug!("Getting user by id");

        // Call audit service to log user access
        let audit_result = self.audit.get_user_access(id).await;
        if let Err(e) = audit_result {
            debug!("Audit service call failed (continuing): {}", e);
        }

        let user = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?
            .ok_or_else(|| DomainError::user_not_found(id))?;
        debug!("Successfully retrieved user");
        Ok(user)
    }

    /// List users with cursor-based pagination
    #[instrument(name = "users_info.service.list_users_page", skip(self, query))]
    pub async fn list_users_page(
        &self,
        query: ODataQuery,
    ) -> Result<Page<User>, modkit_odata::Error> {
        debug!("Listing users with cursor pagination");

        // All validation is now handled centrally in paginate_with_odata
        let page = self.repo.list_users_page(&query).await?;

        debug!("Successfully listed {} users in page", page.items.len());
        Ok(page)
    }

    #[instrument(
        name = "users_info.service.create_user",
        skip(self),
        fields(email = %new_user.email, display_name = %new_user.display_name)
    )]
    pub async fn create_user(&self, new_user: NewUser) -> Result<User, DomainError> {
        info!("Creating new user");

        // Validate input
        self.validate_new_user(&new_user)?;

        // Check uniqueness
        if self
            .repo
            .email_exists(&new_user.email)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?
        {
            return Err(DomainError::email_already_exists(new_user.email));
        }

        let now = Utc::now();
        let id = Uuid::new_v4();
        let user = User {
            id,
            email: new_user.email,
            display_name: new_user.display_name,
            created_at: now,
            updated_at: now,
        };

        self.repo
            .insert(user.clone())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        // Notify external systems about user creation
        let notification_result = self.audit.notify_user_created().await;
        if let Err(e) = notification_result {
            debug!("Notification service call failed (continuing): {}", e);
        }

        // Publish domain event
        self.events.publish(&UserDomainEvent::Created {
            id: user.id,
            at: user.created_at,
        });

        info!("Successfully created user with id={}", user.id);
        Ok(user)
    }

    #[instrument(
        name = "users_info.service.update_user",
        skip(self),
        fields(user_id = %id)
    )]
    pub async fn update_user(&self, id: Uuid, patch: UserPatch) -> Result<User, DomainError> {
        info!("Updating user");

        // Validate patch
        self.validate_user_patch(&patch)?;

        // Load current
        let mut current = self
            .repo
            .find_by_id(id)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?
            .ok_or_else(|| DomainError::user_not_found(id))?;

        // Uniqueness for email change
        if let Some(ref new_email) = patch.email {
            if new_email != &current.email
                && self
                    .repo
                    .email_exists(new_email)
                    .await
                    .map_err(|e| DomainError::database(e.to_string()))?
            {
                return Err(DomainError::email_already_exists(new_email.clone()));
            }
        }

        // Apply patch
        if let Some(email) = patch.email {
            current.email = email;
        }
        if let Some(display_name) = patch.display_name {
            current.display_name = display_name;
        }
        current.updated_at = Utc::now();

        // Persist
        self.repo
            .update(current.clone())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        // Publish domain event
        self.events.publish(&UserDomainEvent::Updated {
            id: current.id,
            at: current.updated_at,
        });

        info!("Successfully updated user");
        Ok(current)
    }

    #[instrument(
        name = "users_info.service.delete_user",
        skip(self),
        fields(user_id = %id)
    )]
    pub async fn delete_user(&self, id: Uuid) -> Result<(), DomainError> {
        info!("Deleting user");

        let deleted = self
            .repo
            .delete(id)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        if !deleted {
            return Err(DomainError::user_not_found(id));
        }

        // Publish domain event
        self.events
            .publish(&UserDomainEvent::Deleted { id, at: Utc::now() });

        info!("Successfully deleted user");
        Ok(())
    }

    // --- validation helpers ---

    fn validate_new_user(&self, new_user: &NewUser) -> Result<(), DomainError> {
        self.validate_email(&new_user.email)?;
        self.validate_display_name(&new_user.display_name)?;
        Ok(())
    }

    fn validate_user_patch(&self, patch: &UserPatch) -> Result<(), DomainError> {
        if let Some(ref email) = patch.email {
            self.validate_email(email)?;
        }
        if let Some(ref display_name) = patch.display_name {
            self.validate_display_name(display_name)?;
        }
        Ok(())
    }

    fn validate_email(&self, email: &str) -> Result<(), DomainError> {
        if email.is_empty() || !email.contains('@') || !email.contains('.') {
            return Err(DomainError::invalid_email(email.to_string()));
        }
        Ok(())
    }

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
