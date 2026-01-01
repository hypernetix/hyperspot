use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::events::UserDomainEvent;
use crate::domain::ports::{AuditPort, EventPublisher};
use crate::infra::storage::entity::{
    user::{ActiveModel as UserAM, Column, Entity as UserEntity},
    city::{ActiveModel as CityAM, Entity as CityEntity},
    language::{ActiveModel as LanguageAM, Column as LanguageColumn, Entity as LanguageEntity},
    address::{ActiveModel as AddressAM, Column as AddressColumn, Entity as AddressEntity},
    user_language::{ActiveModel as UserLanguageAM, Column as UserLanguageColumn, Entity as UserLanguageEntity},
};
use crate::infra::storage::odata_mapper::{CityODataMapper, LanguageODataMapper, UserODataMapper};
use crate::api::rest::dto::{CityDtoFilterField, LanguageDtoFilterField, UserDtoFilterField};
use modkit_db::odata::{paginate_odata, LimitCfg};
use modkit_db::secure::SecureConn;
use modkit_odata::{ODataQuery, Page, SortDir};
use modkit_security::{PolicyEngineRef, SecurityContext};
use sea_orm::sea_query::Expr;
use sea_orm::{PaginatorTrait, QueryFilter, Set};
use time::OffsetDateTime;
use tracing::{debug, info, instrument};
use user_info_sdk::{
    Address, AddressPatch, City, CityPatch, Language, LanguagePatch, NewAddress, NewCity,
    NewLanguage, NewUser, User, UserPatch,
};
use uuid::Uuid;

/// Domain service with business rules for user management.
/// Uses Secure ORM directly for database operations.
#[derive(Clone)]
pub struct Service {
    policy_engine: PolicyEngineRef,
    sec: SecureConn,
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
        sec: SecureConn,
        events: Arc<dyn EventPublisher<UserDomainEvent>>,
        audit: Arc<dyn AuditPort>,
        config: ServiceConfig,
    ) -> Self {
        Self {
            policy_engine: Arc::new(modkit_security::DummyPolicyEngine),
            sec,
            events,
            audit,
            config,
        }
    }

    #[instrument(skip(self, ctx), fields(user_id = %id))]
    pub async fn get_user(&self, ctx: &SecurityContext, id: Uuid) -> Result<User, DomainError> {
        debug!("Getting user by id");

        let audit_result = self.audit.get_user_access(id).await;
        if let Err(e) = audit_result {
            debug!("Audit service call failed (continuing): {}", e);
        }

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let found = self
            .sec
            .find_by_id::<UserEntity>(&scope, id)
            .map_err(|e| DomainError::database(e.to_string()))?
            .one(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let user = found
            .map(Into::into)
            .ok_or_else(|| DomainError::user_not_found(id))?;
        debug!("Successfully retrieved user");
        Ok(user)
    }

    /// List users with cursor-based pagination
    #[instrument(skip(self, ctx, query))]
    pub async fn list_users_page(
        &self,
        ctx: &SecurityContext,
        query: &ODataQuery,
    ) -> Result<Page<User>, DomainError> {
        debug!("Listing users with cursor pagination");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let secure_query = self.sec.find::<UserEntity>(&scope);
        let base_query = secure_query.into_inner();

        let page = paginate_odata::<UserDtoFilterField, UserODataMapper, _, _, _, _>(
            base_query,
            self.sec.conn(),
            query,
            ("id", SortDir::Desc),
            LimitCfg {
                default: self.config.default_page_size as u64,
                max: self.config.max_page_size as u64,
            },
            Into::into,
        )
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

        debug!("Successfully listed {} users in page", page.items.len());
        Ok(page)
    }

    #[instrument(
        skip(self, ctx),
        fields(email = %new_user.email, display_name = %new_user.display_name)
    )]
    pub async fn create_user(
        &self,
        ctx: &SecurityContext,
        new_user: NewUser,
    ) -> Result<User, DomainError> {
        info!("Creating new user");

        self.validate_new_user(&new_user)?;

        let id = new_user.id.unwrap_or_else(Uuid::now_v7);

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        if new_user.id.is_some() {
            let found = self
                .sec
                .find_by_id::<UserEntity>(&scope, id)
                .map_err(|e| DomainError::database(e.to_string()))?
                .one(self.sec.conn())
                .await
                .map_err(|e| DomainError::database(e.to_string()))?;

            if found.is_some() {
                return Err(DomainError::validation(
                    "id",
                    "User with this ID already exists",
                ));
            }
        }

        let secure_query = self
            .sec
            .find::<UserEntity>(&scope)
            .filter(sea_orm::Condition::all().add(Expr::col(Column::Email).eq(&new_user.email)));

        let count = secure_query
            .into_inner()
            .count(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        if count > 0 {
            return Err(DomainError::email_already_exists(new_user.email));
        }

        let now = OffsetDateTime::now_utc();
        let id = new_user.id.unwrap_or_else(uuid::Uuid::now_v7);

        let user = User {
            id,
            tenant_id: new_user.tenant_id,
            email: new_user.email,
            display_name: new_user.display_name,
            created_at: now,
            updated_at: now,
        };

        let m = UserAM {
            id: Set(user.id),
            tenant_id: Set(user.tenant_id),
            email: Set(user.email.clone()),
            display_name: Set(user.display_name.clone()),
            created_at: Set(user.created_at),
            updated_at: Set(user.updated_at),
        };

        let _ = self
            .sec
            .insert::<UserEntity>(&scope, m)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let notification_result = self.audit.notify_user_created().await;
        if let Err(e) = notification_result {
            debug!("Notification service call failed (continuing): {}", e);
        }

        self.events.publish(&UserDomainEvent::Created {
            id: user.id,
            at: user.created_at,
        });

        info!("Successfully created user with id={}", user.id);
        Ok(user)
    }

    #[instrument(
        skip(self, ctx),
        fields(user_id = %id)
    )]
    pub async fn update_user(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        patch: UserPatch,
    ) -> Result<User, DomainError> {
        info!("Updating user");

        self.validate_user_patch(&patch)?;

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let found = self
            .sec
            .find_by_id::<UserEntity>(&scope, id)
            .map_err(|e| DomainError::database(e.to_string()))?
            .one(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let mut current: User = found
            .ok_or_else(|| DomainError::user_not_found(id))?
            .into();

        if let Some(ref new_email) = patch.email {
            if new_email != &current.email {
                let secure_query = self
                    .sec
                    .find::<UserEntity>(&scope)
                    .filter(sea_orm::Condition::all().add(Expr::col(Column::Email).eq(new_email)));

                let count = secure_query
                    .into_inner()
                    .count(self.sec.conn())
                    .await
                    .map_err(|e| DomainError::database(e.to_string()))?;

                if count > 0 {
                    return Err(DomainError::email_already_exists(new_email.clone()));
                }
            }
        }

        if let Some(email) = patch.email {
            current.email = email;
        }
        if let Some(display_name) = patch.display_name {
            current.display_name = display_name;
        }
        current.updated_at = OffsetDateTime::now_utc();

        let m = UserAM {
            id: Set(current.id),
            tenant_id: Set(current.tenant_id),
            email: Set(current.email.clone()),
            display_name: Set(current.display_name.clone()),
            created_at: Set(current.created_at),
            updated_at: Set(current.updated_at),
        };

        let _ = self
            .sec
            .update_with_ctx::<UserEntity>(&scope, current.id, m)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        self.events.publish(&UserDomainEvent::Updated {
            id: current.id,
            at: current.updated_at,
        });

        info!("Successfully updated user");
        Ok(current)
    }

    #[instrument(
        skip(self, ctx),
        fields(user_id = %id)
    )]
    pub async fn delete_user(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), DomainError> {
        info!("Deleting user");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let deleted = self
            .sec
            .delete_by_id::<UserEntity>(&scope, id)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        if !deleted {
            return Err(DomainError::user_not_found(id));
        }

        self.events.publish(&UserDomainEvent::Deleted {
            id,
            at: OffsetDateTime::now_utc(),
        });

        info!("Successfully deleted user");
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

    // ==================== City Operations ====================

    #[instrument(skip(self, ctx), fields(city_id = %id))]
    pub async fn get_city(&self, ctx: &SecurityContext, id: Uuid) -> Result<City, DomainError> {
        debug!("Getting city by id");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let found = self
            .sec
            .find_by_id::<CityEntity>(&scope, id)
            .map_err(|e| DomainError::database(e.to_string()))?
            .one(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        found
            .map(Into::into)
            .ok_or_else(|| DomainError::not_found("City", id))
    }

    #[instrument(skip(self, ctx, query))]
    pub async fn list_cities_page(
        &self,
        ctx: &SecurityContext,
        query: &ODataQuery,
    ) -> Result<Page<City>, DomainError> {
        debug!("Listing cities with cursor pagination");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let secure_query = self.sec.find::<CityEntity>(&scope);
        let base_query = secure_query.into_inner();

        let page = paginate_odata::<CityDtoFilterField, CityODataMapper, _, _, _, _>(
            base_query,
            self.sec.conn(),
            query,
            ("id", SortDir::Desc),
            LimitCfg {
                default: self.config.default_page_size as u64,
                max: self.config.max_page_size as u64,
            },
            Into::into,
        )
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

        debug!("Successfully listed {} cities in page", page.items.len());
        Ok(page)
    }

    #[instrument(skip(self, ctx), fields(name = %new_city.name, country = %new_city.country))]
    pub async fn create_city(
        &self,
        ctx: &SecurityContext,
        new_city: NewCity,
    ) -> Result<City, DomainError> {
        info!("Creating new city");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let now = OffsetDateTime::now_utc();
        let id = new_city.id.unwrap_or_else(Uuid::now_v7);

        let city = City {
            id,
            tenant_id: new_city.tenant_id,
            name: new_city.name,
            country: new_city.country,
            created_at: now,
            updated_at: now,
        };

        let m = CityAM {
            id: Set(city.id),
            tenant_id: Set(city.tenant_id),
            name: Set(city.name.clone()),
            country: Set(city.country.clone()),
            created_at: Set(city.created_at),
            updated_at: Set(city.updated_at),
        };

        let _ = self
            .sec
            .insert::<CityEntity>(&scope, m)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        info!("Successfully created city with id={}", city.id);
        Ok(city)
    }

    #[instrument(skip(self, ctx), fields(city_id = %id))]
    pub async fn update_city(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        patch: CityPatch,
    ) -> Result<City, DomainError> {
        info!("Updating city");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let found = self
            .sec
            .find_by_id::<CityEntity>(&scope, id)
            .map_err(|e| DomainError::database(e.to_string()))?
            .one(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let mut current: City = found
            .ok_or_else(|| DomainError::not_found("City", id))?
            .into();

        if let Some(name) = patch.name {
            current.name = name;
        }
        if let Some(country) = patch.country {
            current.country = country;
        }
        current.updated_at = OffsetDateTime::now_utc();

        let m = CityAM {
            id: Set(current.id),
            tenant_id: Set(current.tenant_id),
            name: Set(current.name.clone()),
            country: Set(current.country.clone()),
            created_at: Set(current.created_at),
            updated_at: Set(current.updated_at),
        };

        let _ = self
            .sec
            .update_with_ctx::<CityEntity>(&scope, current.id, m)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        info!("Successfully updated city");
        Ok(current)
    }

    #[instrument(skip(self, ctx), fields(city_id = %id))]
    pub async fn delete_city(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), DomainError> {
        info!("Deleting city");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let deleted = self
            .sec
            .delete_by_id::<CityEntity>(&scope, id)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        if !deleted {
            return Err(DomainError::not_found("City", id));
        }

        info!("Successfully deleted city");
        Ok(())
    }

    // ==================== Language Operations ====================

    #[instrument(skip(self, ctx), fields(language_id = %id))]
    pub async fn get_language(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<Language, DomainError> {
        debug!("Getting language by id");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let found = self
            .sec
            .find_by_id::<LanguageEntity>(&scope, id)
            .map_err(|e| DomainError::database(e.to_string()))?
            .one(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        found
            .map(Into::into)
            .ok_or_else(|| DomainError::not_found("Language", id))
    }

    #[instrument(skip(self, ctx, query))]
    pub async fn list_languages_page(
        &self,
        ctx: &SecurityContext,
        query: &ODataQuery,
    ) -> Result<Page<Language>, DomainError> {
        debug!("Listing languages with cursor pagination");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let secure_query = self.sec.find::<LanguageEntity>(&scope);
        let base_query = secure_query.into_inner();

        let page = paginate_odata::<LanguageDtoFilterField, LanguageODataMapper, _, _, _, _>(
            base_query,
            self.sec.conn(),
            query,
            ("id", SortDir::Desc),
            LimitCfg {
                default: self.config.default_page_size as u64,
                max: self.config.max_page_size as u64,
            },
            Into::into,
        )
        .await
        .map_err(|e| DomainError::database(e.to_string()))?;

        debug!("Successfully listed {} languages in page", page.items.len());
        Ok(page)
    }

    #[instrument(skip(self, ctx), fields(code = %new_language.code, name = %new_language.name))]
    pub async fn create_language(
        &self,
        ctx: &SecurityContext,
        new_language: NewLanguage,
    ) -> Result<Language, DomainError> {
        info!("Creating new language");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let now = OffsetDateTime::now_utc();
        let id = new_language.id.unwrap_or_else(Uuid::now_v7);

        let language = Language {
            id,
            tenant_id: new_language.tenant_id,
            code: new_language.code,
            name: new_language.name,
            created_at: now,
            updated_at: now,
        };

        let m = LanguageAM {
            id: Set(language.id),
            tenant_id: Set(language.tenant_id),
            code: Set(language.code.clone()),
            name: Set(language.name.clone()),
            created_at: Set(language.created_at),
            updated_at: Set(language.updated_at),
        };

        let _ = self
            .sec
            .insert::<LanguageEntity>(&scope, m)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        info!("Successfully created language with id={}", language.id);
        Ok(language)
    }

    #[instrument(skip(self, ctx), fields(language_id = %id))]
    pub async fn update_language(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        patch: LanguagePatch,
    ) -> Result<Language, DomainError> {
        info!("Updating language");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let found = self
            .sec
            .find_by_id::<LanguageEntity>(&scope, id)
            .map_err(|e| DomainError::database(e.to_string()))?
            .one(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let mut current: Language = found
            .ok_or_else(|| DomainError::not_found("Language", id))?
            .into();

        if let Some(code) = patch.code {
            current.code = code;
        }
        if let Some(name) = patch.name {
            current.name = name;
        }
        current.updated_at = OffsetDateTime::now_utc();

        let m = LanguageAM {
            id: Set(current.id),
            tenant_id: Set(current.tenant_id),
            code: Set(current.code.clone()),
            name: Set(current.name.clone()),
            created_at: Set(current.created_at),
            updated_at: Set(current.updated_at),
        };

        let _ = self
            .sec
            .update_with_ctx::<LanguageEntity>(&scope, current.id, m)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        info!("Successfully updated language");
        Ok(current)
    }

    #[instrument(skip(self, ctx), fields(language_id = %id))]
    pub async fn delete_language(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<(), DomainError> {
        info!("Deleting language");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let deleted = self
            .sec
            .delete_by_id::<LanguageEntity>(&scope, id)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        if !deleted {
            return Err(DomainError::not_found("Language", id));
        }

        info!("Successfully deleted language");
        Ok(())
    }

    // ==================== Address Operations ====================

    #[instrument(skip(self, ctx), fields(address_id = %id))]
    pub async fn get_address(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<Address, DomainError> {
        debug!("Getting address by id");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let found = self
            .sec
            .find_by_id::<AddressEntity>(&scope, id)
            .map_err(|e| DomainError::database(e.to_string()))?
            .one(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        found
            .map(Into::into)
            .ok_or_else(|| DomainError::not_found("Address", id))
    }

    #[instrument(skip(self, ctx), fields(user_id = %user_id))]
    pub async fn get_user_address(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
    ) -> Result<Option<Address>, DomainError> {
        debug!("Getting address by user_id");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let found = self
            .sec
            .find::<AddressEntity>(&scope)
            .filter(sea_orm::Condition::all().add(Expr::col(AddressColumn::UserId).eq(user_id)))
            .into_inner()
            .one(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        Ok(found.map(Into::into))
    }

    #[instrument(skip(self, ctx), fields(user_id = %user_id))]
    pub async fn get_address_by_user(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
    ) -> Result<Option<Address>, DomainError> {
        self.get_user_address(ctx, user_id).await
    }

    #[instrument(skip(self, ctx, address), fields(user_id = %user_id))]
    pub async fn put_user_address(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
        address: NewAddress,
    ) -> Result<Address, DomainError> {
        info!("Upserting address for user");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let user = self
            .sec
            .find_by_id::<UserEntity>(&scope, user_id)
            .map_err(|e| DomainError::database(e.to_string()))?
            .one(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?
            .ok_or_else(|| DomainError::user_not_found(user_id))?;

        let existing = self
            .sec
            .find::<AddressEntity>(&scope)
            .filter(sea_orm::Condition::all().add(Expr::col(AddressColumn::UserId).eq(user_id)))
            .into_inner()
            .one(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let now = OffsetDateTime::now_utc();

        if let Some(existing_model) = existing {
            let mut updated: Address = existing_model.into();
            updated.city_id = address.city_id;
            updated.street = address.street;
            updated.postal_code = address.postal_code;
            updated.updated_at = now;

            let m = AddressAM {
                id: Set(updated.id),
                tenant_id: Set(updated.tenant_id),
                user_id: Set(updated.user_id),
                city_id: Set(updated.city_id),
                street: Set(updated.street.clone()),
                postal_code: Set(updated.postal_code.clone()),
                created_at: Set(updated.created_at),
                updated_at: Set(updated.updated_at),
            };

            let _ = self
                .sec
                .update_with_ctx::<AddressEntity>(&scope, updated.id, m)
                .await
                .map_err(|e| DomainError::database(e.to_string()))?;

            info!("Successfully updated address for user");
            Ok(updated)
        } else {
            let id = address.id.unwrap_or_else(Uuid::now_v7);

            let new_address = Address {
                id,
                tenant_id: user.tenant_id,
                user_id,
                city_id: address.city_id,
                street: address.street,
                postal_code: address.postal_code,
                created_at: now,
                updated_at: now,
            };

            let m = AddressAM {
                id: Set(new_address.id),
                tenant_id: Set(new_address.tenant_id),
                user_id: Set(new_address.user_id),
                city_id: Set(new_address.city_id),
                street: Set(new_address.street.clone()),
                postal_code: Set(new_address.postal_code.clone()),
                created_at: Set(new_address.created_at),
                updated_at: Set(new_address.updated_at),
            };

            let _ = self
                .sec
                .insert::<AddressEntity>(&scope, m)
                .await
                .map_err(|e| DomainError::database(e.to_string()))?;

            info!("Successfully created address for user");
            Ok(new_address)
        }
    }

    #[instrument(skip(self, ctx), fields(user_id = %user_id))]
    pub async fn delete_user_address(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
    ) -> Result<(), DomainError> {
        info!("Deleting address for user");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let result = self
            .sec
            .delete_many::<AddressEntity>(&scope)
            .into_inner()
            .filter(sea_orm::Condition::all().add(Expr::col(AddressColumn::UserId).eq(user_id)))
            .exec(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        if result.rows_affected == 0 {
            return Err(DomainError::not_found("Address", user_id));
        }

        info!("Successfully deleted address for user");
        Ok(())
    }

    #[instrument(skip(self, ctx), fields(user_id = %new_address.user_id))]
    pub async fn create_address(
        &self,
        ctx: &SecurityContext,
        new_address: NewAddress,
    ) -> Result<Address, DomainError> {
        info!("Creating new address");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let now = OffsetDateTime::now_utc();
        let id = new_address.id.unwrap_or_else(Uuid::now_v7);

        let address = Address {
            id,
            tenant_id: new_address.tenant_id,
            user_id: new_address.user_id,
            city_id: new_address.city_id,
            street: new_address.street,
            postal_code: new_address.postal_code,
            created_at: now,
            updated_at: now,
        };

        let m = AddressAM {
            id: Set(address.id),
            tenant_id: Set(address.tenant_id),
            user_id: Set(address.user_id),
            city_id: Set(address.city_id),
            street: Set(address.street.clone()),
            postal_code: Set(address.postal_code.clone()),
            created_at: Set(address.created_at),
            updated_at: Set(address.updated_at),
        };

        let _ = self
            .sec
            .insert::<AddressEntity>(&scope, m)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        info!("Successfully created address with id={}", address.id);
        Ok(address)
    }

    #[instrument(skip(self, ctx), fields(address_id = %id))]
    pub async fn update_address(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        patch: AddressPatch,
    ) -> Result<Address, DomainError> {
        info!("Updating address");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let found = self
            .sec
            .find_by_id::<AddressEntity>(&scope, id)
            .map_err(|e| DomainError::database(e.to_string()))?
            .one(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let mut current: Address = found
            .ok_or_else(|| DomainError::not_found("Address", id))?
            .into();

        if let Some(city_id) = patch.city_id {
            current.city_id = city_id;
        }
        if let Some(street) = patch.street {
            current.street = street;
        }
        if let Some(postal_code) = patch.postal_code {
            current.postal_code = postal_code;
        }
        current.updated_at = OffsetDateTime::now_utc();

        let m = AddressAM {
            id: Set(current.id),
            tenant_id: Set(current.tenant_id),
            user_id: Set(current.user_id),
            city_id: Set(current.city_id),
            street: Set(current.street.clone()),
            postal_code: Set(current.postal_code.clone()),
            created_at: Set(current.created_at),
            updated_at: Set(current.updated_at),
        };

        let _ = self
            .sec
            .update_with_ctx::<AddressEntity>(&scope, current.id, m)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        info!("Successfully updated address");
        Ok(current)
    }

    #[instrument(skip(self, ctx), fields(address_id = %id))]
    pub async fn delete_address(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<(), DomainError> {
        info!("Deleting address");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let deleted = self
            .sec
            .delete_by_id::<AddressEntity>(&scope, id)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        if !deleted {
            return Err(DomainError::not_found("Address", id));
        }

        info!("Successfully deleted address");
        Ok(())
    }

    // ==================== User-Language Relationship Operations ====================

    #[instrument(skip(self, ctx), fields(user_id = %user_id, language_id = %language_id))]
    pub async fn assign_language_to_user(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
        language_id: Uuid,
    ) -> Result<(), DomainError> {
        info!("Assigning language to user (idempotent)");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let user = self
            .sec
            .find_by_id::<UserEntity>(&scope, user_id)
            .map_err(|e| DomainError::database(e.to_string()))?
            .one(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?
            .ok_or_else(|| DomainError::user_not_found(user_id))?;

        let _language = self
            .sec
            .find_by_id::<LanguageEntity>(&scope, language_id)
            .map_err(|e| DomainError::database(e.to_string()))?
            .one(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?
            .ok_or_else(|| DomainError::not_found("Language", language_id))?;

        let existing = self
            .sec
            .find::<UserLanguageEntity>(&scope)
            .filter(
                sea_orm::Condition::all()
                    .add(Expr::col(UserLanguageColumn::UserId).eq(user_id))
                    .add(Expr::col(UserLanguageColumn::LanguageId).eq(language_id)),
            )
            .into_inner()
            .one(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        if existing.is_some() {
            debug!("Language already assigned to user, operation is idempotent");
            return Ok(());
        }

        let now = OffsetDateTime::now_utc();
        let id = Uuid::now_v7();

        let m = UserLanguageAM {
            id: Set(id),
            tenant_id: Set(user.tenant_id),
            user_id: Set(user_id),
            language_id: Set(language_id),
            created_at: Set(now),
        };

        let _ = self
            .sec
            .insert::<UserLanguageEntity>(&scope, m)
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        info!("Successfully assigned language to user");
        Ok(())
    }

    #[instrument(skip(self, ctx), fields(user_id = %user_id, language_id = %language_id))]
    pub async fn remove_language_from_user(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
        language_id: Uuid,
    ) -> Result<(), DomainError> {
        info!("Removing language from user (idempotent)");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let result = self
            .sec
            .delete_many::<UserLanguageEntity>(&scope)
            .into_inner()
            .filter(
                sea_orm::Condition::all()
                    .add(Expr::col(UserLanguageColumn::UserId).eq(user_id))
                    .add(Expr::col(UserLanguageColumn::LanguageId).eq(language_id)),
            )
            .exec(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        if result.rows_affected == 0 {
            debug!("Language not assigned to user, operation is idempotent");
        } else {
            info!("Successfully removed language from user");
        }

        Ok(())
    }

    #[instrument(skip(self, ctx), fields(user_id = %user_id))]
    pub async fn list_user_languages(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
    ) -> Result<Vec<Language>, DomainError> {
        debug!("Listing languages for user");

        let scope = ctx
            .scope(self.policy_engine.clone())
            .include_tenant_children()
            .prepare()
            .await?;

        let user_languages = self
            .sec
            .find::<UserLanguageEntity>(&scope)
            .filter(sea_orm::Condition::all().add(Expr::col(UserLanguageColumn::UserId).eq(user_id)))
            .into_inner()
            .all(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        let language_ids: Vec<Uuid> = user_languages.iter().map(|ul| ul.language_id).collect();

        if language_ids.is_empty() {
            return Ok(Vec::new());
        }

        let languages = self
            .sec
            .find::<LanguageEntity>(&scope)
            .filter(sea_orm::Condition::all().add(Expr::col(LanguageColumn::Id).is_in(language_ids)))
            .into_inner()
            .all(self.sec.conn())
            .await
            .map_err(|e| DomainError::database(e.to_string()))?;

        Ok(languages.into_iter().map(Into::into).collect())
    }
}
