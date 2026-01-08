use std::sync::Arc;

use futures_util::future::BoxFuture;
use modkit_security::SecurityContext;
use uuid::Uuid;

use user_info_sdk::{
    api::{
        AddressesStreamingClient, CitiesStreamingClient, LanguagesStreamingClient,
        UsersStreamingClient,
    },
    Address, City, Language, NewAddress, NewCity, NewLanguage, NewUser, UpdateAddressRequest,
    UpdateCityRequest, UpdateLanguageRequest, UpdateUserRequest, User, UsersInfoClient,
    UsersInfoError,
};

use crate::domain::local_client::{
    addresses::LocalAddressesStreamingClient, cities::LocalCitiesStreamingClient,
    languages::LocalLanguagesStreamingClient, users::LocalUsersStreamingClient,
};
use crate::domain::service::Service;

/// Local implementation of the object-safe `UsersInfoClient`.
///
/// Acts as the SDK boundary adapter: converts `DomainError` into `UsersInfoError`,
/// and exposes streaming-first APIs via boxed streaming client facades.
#[derive(Clone)]
pub struct UsersInfoLocalClient {
    service: Arc<Service>,
}

impl UsersInfoLocalClient {
    #[must_use]
    pub fn new(service: Arc<Service>) -> Self {
        Self { service }
    }
}

impl UsersInfoClient for UsersInfoLocalClient {
    fn users(&self) -> Box<dyn UsersStreamingClient> {
        Box::new(LocalUsersStreamingClient::new(self.service.clone()))
    }

    fn cities(&self) -> Box<dyn CitiesStreamingClient> {
        Box::new(LocalCitiesStreamingClient::new(self.service.clone()))
    }

    fn languages(&self) -> Box<dyn LanguagesStreamingClient> {
        Box::new(LocalLanguagesStreamingClient::new(self.service.clone()))
    }

    fn addresses(&self) -> Box<dyn AddressesStreamingClient> {
        Box::new(LocalAddressesStreamingClient::new(self.service.clone()))
    }

    // ==================== Single-Item Operations ====================

    fn get_user(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> BoxFuture<'static, Result<User, UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .get_user(&ctx, id)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn get_city(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> BoxFuture<'static, Result<City, UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .get_city(&ctx, id)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn get_language(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> BoxFuture<'static, Result<Language, UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .get_language(&ctx, id)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn get_address(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> BoxFuture<'static, Result<Address, UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .get_address(&ctx, id)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn get_address_by_user(
        &self,
        ctx: SecurityContext,
        user_id: Uuid,
    ) -> BoxFuture<'static, Result<Option<Address>, UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .get_address_by_user(&ctx, user_id)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    // ==================== Mutation Operations ====================

    fn create_user(
        &self,
        ctx: SecurityContext,
        new_user: NewUser,
    ) -> BoxFuture<'static, Result<User, UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .create_user(&ctx, new_user)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn update_user(
        &self,
        ctx: SecurityContext,
        req: UpdateUserRequest,
    ) -> BoxFuture<'static, Result<User, UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .update_user(&ctx, req.id, req.patch)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn delete_user(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> BoxFuture<'static, Result<(), UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .delete_user(&ctx, id)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn create_city(
        &self,
        ctx: SecurityContext,
        new_city: NewCity,
    ) -> BoxFuture<'static, Result<City, UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .create_city(&ctx, new_city)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn update_city(
        &self,
        ctx: SecurityContext,
        req: UpdateCityRequest,
    ) -> BoxFuture<'static, Result<City, UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .update_city(&ctx, req.id, req.patch)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn delete_city(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> BoxFuture<'static, Result<(), UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .delete_city(&ctx, id)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn create_language(
        &self,
        ctx: SecurityContext,
        new_language: NewLanguage,
    ) -> BoxFuture<'static, Result<Language, UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .create_language(&ctx, new_language)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn update_language(
        &self,
        ctx: SecurityContext,
        req: UpdateLanguageRequest,
    ) -> BoxFuture<'static, Result<Language, UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .update_language(&ctx, req.id, req.patch)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn delete_language(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> BoxFuture<'static, Result<(), UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .delete_language(&ctx, id)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn create_address(
        &self,
        ctx: SecurityContext,
        new_address: NewAddress,
    ) -> BoxFuture<'static, Result<Address, UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .create_address(&ctx, new_address)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn update_address(
        &self,
        ctx: SecurityContext,
        req: UpdateAddressRequest,
    ) -> BoxFuture<'static, Result<Address, UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .update_address(&ctx, req.id, req.patch)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn delete_address(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> BoxFuture<'static, Result<(), UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .delete_address(&ctx, id)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    // ==================== Relationship Operations ====================

    fn assign_language_to_user(
        &self,
        ctx: SecurityContext,
        user_id: Uuid,
        language_id: Uuid,
    ) -> BoxFuture<'static, Result<(), UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .assign_language_to_user(&ctx, user_id, language_id)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn remove_language_from_user(
        &self,
        ctx: SecurityContext,
        user_id: Uuid,
        language_id: Uuid,
    ) -> BoxFuture<'static, Result<(), UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .remove_language_from_user(&ctx, user_id, language_id)
                .await
                .map_err(UsersInfoError::from)
        })
    }

    fn list_user_languages(
        &self,
        ctx: SecurityContext,
        user_id: Uuid,
    ) -> BoxFuture<'static, Result<Vec<Language>, UsersInfoError>> {
        let service = Arc::clone(&self.service);
        Box::pin(async move {
            service
                .list_user_languages(&ctx, user_id)
                .await
                .map_err(UsersInfoError::from)
        })
    }
}
