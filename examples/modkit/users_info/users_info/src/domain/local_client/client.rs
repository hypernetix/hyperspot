use std::sync::Arc;

use async_trait::async_trait;
use modkit_security::SecurityContext;
use uuid::Uuid;

use user_info_sdk::{
    client::{AddressesStreamingClient, CitiesStreamingClient, UsersStreamingClient},
    Address, City, NewAddress, NewCity, NewUser, UpdateAddressRequest, UpdateCityRequest,
    UpdateUserRequest, User, UserFull, UsersInfoClient, UsersInfoError,
};

use crate::domain::local_client::{
    addresses::LocalAddressesStreamingClient, cities::LocalCitiesStreamingClient,
    users::LocalUsersStreamingClient,
};
use crate::module::ConcreteAppServices;

/// Local implementation of the object-safe `UsersInfoClient`.
///
/// Acts as the SDK boundary adapter: converts `DomainError` into `UsersInfoError`,
/// and exposes streaming-first APIs via boxed streaming client facades.
#[derive(Clone)]
pub struct UsersInfoLocalClient {
    services: Arc<ConcreteAppServices>,
}

impl UsersInfoLocalClient {
    #[must_use]
    pub(crate) fn new(services: Arc<ConcreteAppServices>) -> Self {
        Self { services }
    }
}

#[async_trait]
impl UsersInfoClient for UsersInfoLocalClient {
    fn users(&self) -> Box<dyn UsersStreamingClient> {
        Box::new(LocalUsersStreamingClient::new(Arc::clone(&self.services)))
    }

    fn cities(&self) -> Box<dyn CitiesStreamingClient> {
        Box::new(LocalCitiesStreamingClient::new(Arc::clone(&self.services)))
    }

    fn addresses(&self) -> Box<dyn AddressesStreamingClient> {
        Box::new(LocalAddressesStreamingClient::new(Arc::clone(
            &self.services,
        )))
    }

    // ==================== Single-Item Operations ====================

    async fn get_user(&self, ctx: SecurityContext, id: Uuid) -> Result<User, UsersInfoError> {
        self.services
            .users
            .get_user(&ctx, id)
            .await
            .map_err(UsersInfoError::from)
    }

    async fn get_user_full(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> Result<UserFull, UsersInfoError> {
        self.services
            .users
            .get_user_full(&ctx, id)
            .await
            .map_err(UsersInfoError::from)
    }

    async fn get_city(&self, ctx: SecurityContext, id: Uuid) -> Result<City, UsersInfoError> {
        self.services
            .cities
            .get_city(&ctx, id)
            .await
            .map_err(UsersInfoError::from)
    }

    async fn get_address(&self, ctx: SecurityContext, id: Uuid) -> Result<Address, UsersInfoError> {
        self.services
            .addresses
            .get_address(&ctx, id)
            .await
            .map_err(UsersInfoError::from)
    }

    async fn get_address_by_user(
        &self,
        ctx: SecurityContext,
        user_id: Uuid,
    ) -> Result<Option<Address>, UsersInfoError> {
        self.services
            .addresses
            .get_address_by_user(&ctx, user_id)
            .await
            .map_err(UsersInfoError::from)
    }

    // ==================== Mutation Operations ====================

    async fn create_user(
        &self,
        ctx: SecurityContext,
        new_user: NewUser,
    ) -> Result<User, UsersInfoError> {
        self.services
            .users
            .create_user(&ctx, new_user)
            .await
            .map_err(UsersInfoError::from)
    }

    async fn update_user(
        &self,
        ctx: SecurityContext,
        req: UpdateUserRequest,
    ) -> Result<User, UsersInfoError> {
        self.services
            .users
            .update_user(&ctx, req.id, req.patch)
            .await
            .map_err(UsersInfoError::from)
    }

    async fn delete_user(&self, ctx: SecurityContext, id: Uuid) -> Result<(), UsersInfoError> {
        self.services
            .users
            .delete_user(&ctx, id)
            .await
            .map_err(UsersInfoError::from)
    }

    async fn create_city(
        &self,
        ctx: SecurityContext,
        new_city: NewCity,
    ) -> Result<City, UsersInfoError> {
        self.services
            .cities
            .create_city(&ctx, new_city)
            .await
            .map_err(UsersInfoError::from)
    }

    async fn update_city(
        &self,
        ctx: SecurityContext,
        req: UpdateCityRequest,
    ) -> Result<City, UsersInfoError> {
        self.services
            .cities
            .update_city(&ctx, req.id, req.patch)
            .await
            .map_err(UsersInfoError::from)
    }

    async fn delete_city(&self, ctx: SecurityContext, id: Uuid) -> Result<(), UsersInfoError> {
        self.services
            .cities
            .delete_city(&ctx, id)
            .await
            .map_err(UsersInfoError::from)
    }

    async fn create_address(
        &self,
        ctx: SecurityContext,
        new_address: NewAddress,
    ) -> Result<Address, UsersInfoError> {
        self.services
            .addresses
            .create_address(&ctx, new_address)
            .await
            .map_err(UsersInfoError::from)
    }

    async fn update_address(
        &self,
        ctx: SecurityContext,
        req: UpdateAddressRequest,
    ) -> Result<Address, UsersInfoError> {
        self.services
            .addresses
            .update_address(&ctx, req.id, req.patch)
            .await
            .map_err(UsersInfoError::from)
    }

    async fn delete_address(&self, ctx: SecurityContext, id: Uuid) -> Result<(), UsersInfoError> {
        self.services
            .addresses
            .delete_address(&ctx, id)
            .await
            .map_err(UsersInfoError::from)
    }
}
