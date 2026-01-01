//! Local implementation of `UsersInfoClient`.
//!
//! This client is used for inter-module communication within the same process.
//! It delegates to the domain service and converts errors to SDK error types.

use async_trait::async_trait;
use modkit_odata::{ODataQuery, Page};
use modkit_security::SecurityContext;
use std::sync::Arc;
use uuid::Uuid;

use user_info_sdk::{
    Address, City, Language, NewAddress, NewCity, NewLanguage, NewUser, UpdateAddressRequest,
    UpdateCityRequest, UpdateLanguageRequest, UpdateUserRequest, User, UsersInfoClient,
    UsersInfoError,
};

use crate::domain::service::Service;

/// Local implementation of the `UsersInfoClient` trait that delegates to the domain service.
///
/// This client is used for inter-module communication within the same process.
/// It accepts a `SecurityCtx` from the caller and forwards it directly to the domain service,
/// ensuring proper authorization and access control throughout the call chain.
pub struct UsersInfoLocalClient {
    service: Arc<Service>,
}

impl UsersInfoLocalClient {
    /// Create a new local client wrapping the domain service.
    #[must_use]
    pub fn new(service: Arc<Service>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl UsersInfoClient for UsersInfoLocalClient {
    // User operations
    async fn get_user(&self, ctx: &SecurityContext, id: Uuid) -> Result<User, UsersInfoError> {
        self.service.get_user(ctx, id).await.map_err(Into::into)
    }

    async fn list_users(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<User>, UsersInfoError> {
        self.service
            .list_users_page(ctx, &query)
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "Unexpected OData error in gateway");
                UsersInfoError::internal()
            })
    }

    async fn create_user(
        &self,
        ctx: &SecurityContext,
        new_user: NewUser,
    ) -> Result<User, UsersInfoError> {
        self.service
            .create_user(ctx, new_user)
            .await
            .map_err(Into::into)
    }

    async fn update_user(
        &self,
        ctx: &SecurityContext,
        req: UpdateUserRequest,
    ) -> Result<User, UsersInfoError> {
        self.service
            .update_user(ctx, req.id, req.patch)
            .await
            .map_err(Into::into)
    }

    async fn delete_user(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), UsersInfoError> {
        self.service.delete_user(ctx, id).await.map_err(Into::into)
    }

    // Address operations
    async fn get_address(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<Address, UsersInfoError> {
        self.service.get_address(ctx, id).await.map_err(Into::into)
    }

    async fn get_address_by_user(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
    ) -> Result<Option<Address>, UsersInfoError> {
        self.service
            .get_address_by_user(ctx, user_id)
            .await
            .map_err(Into::into)
    }

    async fn create_address(
        &self,
        ctx: &SecurityContext,
        new_address: NewAddress,
    ) -> Result<Address, UsersInfoError> {
        self.service
            .create_address(ctx, new_address)
            .await
            .map_err(Into::into)
    }

    async fn update_address(
        &self,
        ctx: &SecurityContext,
        req: UpdateAddressRequest,
    ) -> Result<Address, UsersInfoError> {
        self.service
            .update_address(ctx, req.id, req.patch)
            .await
            .map_err(Into::into)
    }

    async fn delete_address(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<(), UsersInfoError> {
        self.service.delete_address(ctx, id).await.map_err(Into::into)
    }

    // City operations
    async fn get_city(&self, ctx: &SecurityContext, id: Uuid) -> Result<City, UsersInfoError> {
        self.service.get_city(ctx, id).await.map_err(Into::into)
    }

    async fn list_cities(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<City>, UsersInfoError> {
        self.service
            .list_cities_page(ctx, &query)
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "Unexpected error in list_cities");
                UsersInfoError::internal()
            })
    }

    async fn create_city(
        &self,
        ctx: &SecurityContext,
        new_city: NewCity,
    ) -> Result<City, UsersInfoError> {
        self.service
            .create_city(ctx, new_city)
            .await
            .map_err(Into::into)
    }

    async fn update_city(
        &self,
        ctx: &SecurityContext,
        req: UpdateCityRequest,
    ) -> Result<City, UsersInfoError> {
        self.service
            .update_city(ctx, req.id, req.patch)
            .await
            .map_err(Into::into)
    }

    async fn delete_city(&self, ctx: &SecurityContext, id: Uuid) -> Result<(), UsersInfoError> {
        self.service.delete_city(ctx, id).await.map_err(Into::into)
    }

    // Language operations
    async fn get_language(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<Language, UsersInfoError> {
        self.service.get_language(ctx, id).await.map_err(Into::into)
    }

    async fn list_languages(
        &self,
        ctx: &SecurityContext,
        query: ODataQuery,
    ) -> Result<Page<Language>, UsersInfoError> {
        self.service
            .list_languages_page(ctx, &query)
            .await
            .map_err(|e| {
                tracing::error!(error = ?e, "Unexpected error in list_languages");
                UsersInfoError::internal()
            })
    }

    async fn create_language(
        &self,
        ctx: &SecurityContext,
        new_language: NewLanguage,
    ) -> Result<Language, UsersInfoError> {
        self.service
            .create_language(ctx, new_language)
            .await
            .map_err(Into::into)
    }

    async fn update_language(
        &self,
        ctx: &SecurityContext,
        req: UpdateLanguageRequest,
    ) -> Result<Language, UsersInfoError> {
        self.service
            .update_language(ctx, req.id, req.patch)
            .await
            .map_err(Into::into)
    }

    async fn delete_language(
        &self,
        ctx: &SecurityContext,
        id: Uuid,
    ) -> Result<(), UsersInfoError> {
        self.service.delete_language(ctx, id).await.map_err(Into::into)
    }

    // User-Language relationship operations
    async fn assign_language_to_user(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
        language_id: Uuid,
    ) -> Result<(), UsersInfoError> {
        self.service
            .assign_language_to_user(ctx, user_id, language_id)
            .await
            .map_err(Into::into)
    }

    async fn remove_language_from_user(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
        language_id: Uuid,
    ) -> Result<(), UsersInfoError> {
        self.service
            .remove_language_from_user(ctx, user_id, language_id)
            .await
            .map_err(Into::into)
    }

    async fn list_user_languages(
        &self,
        ctx: &SecurityContext,
        user_id: Uuid,
    ) -> Result<Vec<Language>, UsersInfoError> {
        self.service
            .list_user_languages(ctx, user_id)
            .await
            .map_err(Into::into)
    }
}
