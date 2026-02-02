use std::sync::Arc;
use tracing::instrument;

use crate::domain::error::DomainError;
use crate::domain::events::UserDomainEvent;
use crate::domain::ports::{AuditPort, EventPublisher};
use crate::domain::repos::{AddressesRepository, CitiesRepository, UsersRepository};
use crate::domain::service::DbProvider;
use crate::domain::service::{AddressesService, CitiesService, ServiceConfig};
use modkit_odata::{ODataQuery, Page};
use modkit_security::{PolicyEngineRef, SecurityContext};
use tenant_resolver_sdk::TenantResolverGatewayClient;
use time::OffsetDateTime;
use user_info_sdk::{NewUser, User, UserFull, UserPatch};
use uuid::Uuid;

/// Users service.
///
/// # Design
///
/// Services acquire database connections internally via `DBProvider`. Handlers
/// call service methods with business parameters only - no DB objects.
///
/// This design:
/// - Keeps handlers clean and focused on HTTP concerns
/// - Centralizes DB error mapping in the domain layer
/// - Maintains transaction safety via the task-local guard
pub struct UsersService<R: UsersRepository + 'static, CR: CitiesRepository, AR: AddressesRepository>
{
    db: Arc<DbProvider>,
    policy_engine: PolicyEngineRef,
    repo: Arc<R>,
    events: Arc<dyn EventPublisher<UserDomainEvent>>,
    audit: Arc<dyn AuditPort>,
    resolver: Arc<dyn TenantResolverGatewayClient>,
    config: ServiceConfig,
    cities: Arc<CitiesService<CR>>,
    addresses: Arc<AddressesService<AR, R>>,
}

impl<R: UsersRepository + 'static, CR: CitiesRepository, AR: AddressesRepository>
    UsersService<R, CR, AR>
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db: Arc<DbProvider>,
        repo: Arc<R>,
        events: Arc<dyn EventPublisher<UserDomainEvent>>,
        audit: Arc<dyn AuditPort>,
        policy_engine: PolicyEngineRef,
        resolver: Arc<dyn TenantResolverGatewayClient>,
        config: ServiceConfig,
        cities: Arc<CitiesService<CR>>,
        addresses: Arc<AddressesService<AR, R>>,
    ) -> Self {
        Self {
            db,
            policy_engine,
            repo,
            events,
            audit,
            resolver,
            config,
            cities,
            addresses,
        }
    }
}

async fn audit_get_user_access_best_effort<
    R: UsersRepository,
    CR: CitiesRepository,
    AR: AddressesRepository,
>(
    svc: &UsersService<R, CR, AR>,
    id: Uuid,
) {
    let audit_result = svc.audit.get_user_access(id).await;
    if let Err(e) = audit_result {
        tracing::debug!("Audit service call failed (continuing): {}", e);
    }
}

// Business logic methods
impl<R: UsersRepository + 'static, CR: CitiesRepository, AR: AddressesRepository>
    UsersService<R, CR, AR>
{
    #[instrument(skip(self, ctx), fields(user_id = %id))]
    pub async fn get_user(&self, ctx: &SecurityContext, id: Uuid) -> Result<User, DomainError> {
        tracing::debug!("Getting user by id");

        let conn = self.db.conn().map_err(DomainError::from)?;

        audit_get_user_access_best_effort(self, id).await;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let found = self.repo.get(&conn, &scope, id).await?;

        let user = found.ok_or_else(|| DomainError::user_not_found(id))?;

        tracing::debug!("Successfully retrieved user");
        Ok(user)
    }

    /// List users with cursor-based pagination
    #[instrument(skip(self, ctx, query))]
    pub async fn list_users_page(
        &self,
        ctx: &SecurityContext,
        query: &ODataQuery,
    ) -> Result<Page<User>, DomainError> {
        tracing::debug!("Listing users with cursor pagination");

        let conn = self.db.conn().map_err(DomainError::from)?;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let page = self.repo.list_page(&conn, &scope, query).await?;

        tracing::debug!("Successfully listed {} users in page", page.items.len());
        Ok(page)
    }

    /// Create a new user.
    #[allow(clippy::cognitive_complexity)]
    #[instrument(
        skip(self, ctx),
        fields(email = %new_user.email, display_name = %new_user.display_name)
    )]
    pub async fn create_user(
        &self,
        ctx: &SecurityContext,
        new_user: NewUser,
    ) -> Result<User, DomainError> {
        tracing::info!("Creating new user");

        self.validate_new_user(&new_user)?;

        let conn = self.db.conn().map_err(DomainError::from)?;

        let NewUser {
            id: provided_id,
            tenant_id,
            email,
            display_name,
        } = new_user;

        let id = provided_id.unwrap_or_else(Uuid::now_v7);

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let now = OffsetDateTime::now_utc();

        let user = User {
            id,
            tenant_id,
            email,
            display_name,
            created_at: now,
            updated_at: now,
        };

        // Uniqueness checks and insert
        if provided_id.is_some() && self.repo.exists(&conn, &scope, id).await? {
            return Err(DomainError::validation(
                "id",
                "User with this ID already exists",
            ));
        }

        if self.repo.count_by_email(&conn, &scope, &user.email).await? > 0 {
            return Err(DomainError::email_already_exists(user.email.clone()));
        }

        let created_user = self.repo.create(&conn, &scope, user).await?;

        let notification_result = self.audit.notify_user_created().await;
        if let Err(e) = notification_result {
            tracing::debug!("Notification service call failed (continuing): {}", e);
        }

        self.events.publish(&UserDomainEvent::Created {
            id: created_user.id,
            at: created_user.created_at,
        });

        tracing::info!("Successfully created user with id={}", created_user.id);
        Ok(created_user)
    }

    /// Update an existing user.
    #[instrument(skip(self, ctx), fields(user_id = %id))]
    pub async fn update_user(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        patch: UserPatch,
    ) -> Result<User, DomainError> {
        tracing::info!("Updating user");

        self.validate_user_patch(&patch)?;

        let conn = self.db.conn().map_err(DomainError::from)?;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let found = self.repo.get(&conn, &scope, id).await?;
        let mut current: User = match found {
            Some(u) => u,
            None => return Err(DomainError::user_not_found(id)),
        };

        if let Some(ref new_email) = patch.email
            && new_email != &current.email
        {
            let count = self.repo.count_by_email(&conn, &scope, new_email).await?;
            if count > 0 {
                return Err(DomainError::email_already_exists(new_email.clone()));
            }
        }

        if let Some(email) = patch.email {
            current.email = email;
        }
        if let Some(display_name) = patch.display_name {
            current.display_name = display_name;
        }
        current.updated_at = OffsetDateTime::now_utc();

        let updated_user = self.repo.update(&conn, &scope, current).await?;

        self.events.publish(&UserDomainEvent::Updated {
            id: updated_user.id,
            at: updated_user.updated_at,
        });

        tracing::info!("Successfully updated user");
        Ok(updated_user)
    }

    #[instrument(skip(self, ctx), fields(user_id = %id))]
    pub async fn delete_user(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), DomainError> {
        tracing::info!("Deleting user");

        let conn = self.db.conn().map_err(DomainError::from)?;

        let tenant_ids = super::resolve_accessible_tenants(self.resolver.as_ref(), ctx).await?;
        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_accessible_tenants(tenant_ids)
            .prepare()
            .await?;

        let deleted = self.repo.delete(&conn, &scope, id).await?;

        if !deleted {
            return Err(DomainError::user_not_found(id));
        }

        self.events.publish(&UserDomainEvent::Deleted {
            id,
            at: OffsetDateTime::now_utc(),
        });

        tracing::info!("Successfully deleted user");
        Ok(())
    }

    fn validate_new_user(&self, new_user: &NewUser) -> Result<(), DomainError> {
        Self::validate_email(&new_user.email)?;
        self.validate_display_name(&new_user.display_name)?;
        Ok(())
    }

    fn validate_user_patch(&self, patch: &UserPatch) -> Result<(), DomainError> {
        if let Some(ref email) = patch.email {
            Self::validate_email(email)?;
        }
        if let Some(ref display_name) = patch.display_name {
            self.validate_display_name(display_name)?;
        }
        Ok(())
    }

    fn validate_email(email: &str) -> Result<(), DomainError> {
        if email.is_empty() || !email.contains('@') || !email.contains('.') {
            return Err(DomainError::invalid_email(email.to_owned()));
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

    #[instrument(skip(self, ctx), fields(user_id = %id))]
    pub async fn get_user_full(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<UserFull, DomainError> {
        tracing::debug!("Getting aggregated user with related entities");

        let user = self.get_user(ctx, id).await?;

        let address = self.addresses.get_address_by_user(ctx, id).await?;

        let city = if let Some(ref addr) = address {
            Some(self.cities.get_city(ctx, addr.city_id).await?)
        } else {
            None
        };

        Ok(UserFull {
            user,
            address,
            city,
        })
    }
}
