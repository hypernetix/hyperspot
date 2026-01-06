//! `UsersInfoClient` trait definition.
//!
//! This trait defines the public API for the `user_info` module.
//! All methods require a `SecurityCtx` for authorization and access control.

use async_trait::async_trait;
use modkit_odata::{ODataQuery, Page};
use modkit_security::SecurityContext;
use uuid::Uuid;

use crate::errors::UsersInfoError;
use crate::models::{
    Address, City, Language, NewAddress, NewCity, NewLanguage, NewUser, UpdateAddressRequest,
    UpdateCityRequest, UpdateLanguageRequest, UpdateUserRequest, User,
};

/// Public client trait for the `user_info` module.
///
/// This trait can be consumed by other modules via `ClientHub`:
/// ```ignore
/// let client = hub.get::<dyn UsersInfoClient>()?;
/// let user = client.get_user(&ctx, user_id).await?;
/// ```
///
/// All methods require a `SecurityContext` for proper authorization and access control.
#[async_trait]
pub trait UsersInfoClient: Send + Sync {
    // User operations
    async fn get_user(&self, ctx: &SecurityContext, id: Uuid) -> Result<User, UsersInfoError>;
    async fn list_users(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<User>, UsersInfoError>;
    async fn create_user(
        &self,
        ctx: &SecurityContext,
        new_user: NewUser,
    ) -> Result<User, UsersInfoError>;
    async fn update_user(
        &self,
        ctx: &SecurityContext,
        req: UpdateUserRequest,
    ) -> Result<User, UsersInfoError>;
    async fn delete_user(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), UsersInfoError>;

    // Address operations
    async fn get_address(&self, ctx: &SecurityContext, id: Uuid)
        -> Result<Address, UsersInfoError>;
    async fn get_address_by_user(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
    ) -> Result<Option<Address>, UsersInfoError>;
    async fn create_address(
        &self,
        ctx: &SecurityContext,
        new_address: NewAddress,
    ) -> Result<Address, UsersInfoError>;
    async fn update_address(
        &self,
        ctx: &SecurityContext,
        req: UpdateAddressRequest,
    ) -> Result<Address, UsersInfoError>;
    async fn delete_address(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), UsersInfoError>;

    // City operations
    async fn get_city(&self, ctx: &SecurityContext, id: Uuid) -> Result<City, UsersInfoError>;
    async fn list_cities(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<City>, UsersInfoError>;
    async fn create_city(
        &self,
        ctx: &SecurityContext,
        new_city: NewCity,
    ) -> Result<City, UsersInfoError>;
    async fn update_city(
        &self,
        ctx: &SecurityContext,
        req: UpdateCityRequest,
    ) -> Result<City, UsersInfoError>;
    async fn delete_city(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), UsersInfoError>;

    // Language operations
    async fn get_language(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<Language, UsersInfoError>;
    async fn list_languages(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<Language>, UsersInfoError>;
    async fn create_language(
        &self,
        ctx: &SecurityContext,
        new_language: NewLanguage,
    ) -> Result<Language, UsersInfoError>;
    async fn update_language(
        &self,
        ctx: &SecurityContext,
        req: UpdateLanguageRequest,
    ) -> Result<Language, UsersInfoError>;
    async fn delete_language(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), UsersInfoError>;

    // User-Language relationship operations
    async fn assign_language_to_user(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
        language_id: Uuid,
    ) -> Result<(), UsersInfoError>;
    async fn remove_language_from_user(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
        language_id: Uuid,
    ) -> Result<(), UsersInfoError>;
    async fn list_user_languages(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
    ) -> Result<Vec<Language>, UsersInfoError>;
}
