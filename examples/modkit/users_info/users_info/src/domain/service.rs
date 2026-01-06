use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::events::UserDomainEvent;
use crate::domain::ports::{AuditPort, EventPublisher};
use crate::infra::storage::entity::{
    address::{ActiveModel as AddressAM, Column as AddressColumn, Entity as AddressEntity},
    city::{ActiveModel as CityAM, Entity as CityEntity},
    language::{ActiveModel as LanguageAM, Column as LanguageColumn, Entity as LanguageEntity},
    user::{ActiveModel as UserAM, Column, Entity as UserEntity},
    user_language::{
        ActiveModel as UserLanguageAM, Column as UserLanguageColumn, Entity as UserLanguageEntity,
    },
};
use crate::infra::storage::odata_mapper::{CityODataMapper, LanguageODataMapper, UserODataMapper};
use crate::query::{CityFilterField, LanguageFilterField, UserFilterField};
use modkit_db::odata::{paginate_odata, LimitCfg};
use modkit_db::secure::SecureConn;
use modkit_odata::{ODataQuery, Page, SortDir};
use modkit_security::{PolicyEngineRef, SecurityContext};
use sea_orm::sea_query::Expr;
use sea_orm::Set;
use time::OffsetDateTime;
use tracing::{debug, info, instrument};
use user_info_sdk::{
    Address, AddressPatch, City, CityPatch, Language, LanguagePatch, NewAddress, NewCity,
    NewLanguage, NewUser, User, UserPatch,
};
use uuid::Uuid;

#[path = "service/addresses.rs"]
mod addresses;
#[path = "service/cities.rs"]
mod cities;
#[path = "service/languages.rs"]
mod languages;
#[path = "service/user_languages.rs"]
mod user_languages;
#[path = "service/users.rs"]
mod users;

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
        users::get_user(self, ctx, id).await
    }

    /// List users with cursor-based pagination
    #[instrument(skip(self, ctx, query))]
    pub async fn list_users_page(
        &self,
        ctx: &SecurityContext,
        query: &ODataQuery,
    ) -> Result<Page<User>, DomainError> {
        users::list_users_page(self, ctx, query).await
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
        users::create_user(self, ctx, new_user).await
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
        users::update_user(self, ctx, id, patch).await
    }

    #[instrument(
        skip(self, ctx),
        fields(user_id = %id)
    )]
    pub async fn delete_user(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), DomainError> {
        users::delete_user(self, ctx, id).await
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
        cities::get_city(self, ctx, id).await
    }

    #[instrument(skip(self, ctx, query))]
    pub async fn list_cities_page(
        &self,
        ctx: &SecurityContext,
        query: &ODataQuery,
    ) -> Result<Page<City>, DomainError> {
        cities::list_cities_page(self, ctx, query).await
    }

    #[instrument(skip(self, ctx), fields(name = %new_city.name, country = %new_city.country))]
    pub async fn create_city(
        &self,
        ctx: &SecurityContext,
        new_city: NewCity,
    ) -> Result<City, DomainError> {
        cities::create_city(self, ctx, new_city).await
    }

    #[instrument(skip(self, ctx), fields(city_id = %id))]
    pub async fn update_city(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        patch: CityPatch,
    ) -> Result<City, DomainError> {
        cities::update_city(self, ctx, id, patch).await
    }

    #[instrument(skip(self, ctx), fields(city_id = %id))]
    pub async fn delete_city(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), DomainError> {
        cities::delete_city(self, ctx, id).await
    }

    // ==================== Language Operations ====================

    #[instrument(skip(self, ctx), fields(language_id = %id))]
    pub async fn get_language(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<Language, DomainError> {
        languages::get_language(self, ctx, id).await
    }

    #[instrument(skip(self, ctx, query))]
    pub async fn list_languages_page(
        &self,
        ctx: &SecurityContext,
        query: &ODataQuery,
    ) -> Result<Page<Language>, DomainError> {
        languages::list_languages_page(self, ctx, query).await
    }

    #[instrument(skip(self, ctx), fields(code = %new_language.code, name = %new_language.name))]
    pub async fn create_language(
        &self,
        ctx: &SecurityContext,
        new_language: NewLanguage,
    ) -> Result<Language, DomainError> {
        languages::create_language(self, ctx, new_language).await
    }

    #[instrument(skip(self, ctx), fields(language_id = %id))]
    pub async fn update_language(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        patch: LanguagePatch,
    ) -> Result<Language, DomainError> {
        languages::update_language(self, ctx, id, patch).await
    }

    #[instrument(skip(self, ctx), fields(language_id = %id))]
    pub async fn delete_language(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<(), DomainError> {
        languages::delete_language(self, ctx, id).await
    }

    // ==================== Address Operations ====================

    #[instrument(skip(self, ctx), fields(address_id = %id))]
    pub async fn get_address(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<Address, DomainError> {
        addresses::get_address(self, ctx, id).await
    }

    #[instrument(skip(self, ctx), fields(user_id = %user_id))]
    pub async fn get_user_address(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
    ) -> Result<Option<Address>, DomainError> {
        addresses::get_user_address(self, ctx, user_id).await
    }

    #[instrument(skip(self, ctx), fields(user_id = %user_id))]
    pub async fn get_address_by_user(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
    ) -> Result<Option<Address>, DomainError> {
        addresses::get_address_by_user(self, ctx, user_id).await
    }

    #[instrument(skip(self, ctx, address), fields(user_id = %user_id))]
    pub async fn put_user_address(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
        address: NewAddress,
    ) -> Result<Address, DomainError> {
        addresses::put_user_address(self, ctx, user_id, address).await
    }

    #[instrument(skip(self, ctx), fields(user_id = %user_id))]
    pub async fn delete_user_address(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
    ) -> Result<(), DomainError> {
        addresses::delete_user_address(self, ctx, user_id).await
    }

    #[instrument(skip(self, ctx), fields(user_id = %new_address.user_id))]
    pub async fn create_address(
        &self,
        ctx: &SecurityContext,
        new_address: NewAddress,
    ) -> Result<Address, DomainError> {
        addresses::create_address(self, ctx, new_address).await
    }

    #[instrument(skip(self, ctx), fields(address_id = %id))]
    pub async fn update_address(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
        patch: AddressPatch,
    ) -> Result<Address, DomainError> {
        addresses::update_address(self, ctx, id, patch).await
    }

    #[instrument(skip(self, ctx), fields(address_id = %id))]
    pub async fn delete_address(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), DomainError> {
        addresses::delete_address(self, ctx, id).await
    }

    // ==================== User-Language Relationship Operations ====================

    #[instrument(skip(self, ctx), fields(user_id = %user_id, language_id = %language_id))]
    pub async fn assign_language_to_user(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
        language_id: Uuid,
    ) -> Result<(), DomainError> {
        user_languages::assign_language_to_user(self, ctx, user_id, language_id).await
    }

    #[instrument(skip(self, ctx), fields(user_id = %user_id, language_id = %language_id))]
    pub async fn remove_language_from_user(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
        language_id: Uuid,
    ) -> Result<(), DomainError> {
        user_languages::remove_language_from_user(self, ctx, user_id, language_id).await
    }

    #[instrument(skip(self, ctx), fields(user_id = %user_id))]
    pub async fn list_user_languages(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
    ) -> Result<Vec<Language>, DomainError> {
        user_languages::list_user_languages(self, ctx, user_id).await
    }
}
