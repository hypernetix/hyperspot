//! Object-safe streaming boundary for the `user_info` module.
//!
//! This API is designed for `ClientHub` registration as `Arc<dyn UsersInfoClientV1>`.
//! All type erasure (boxed streams/futures) lives here; internal implementations
//! remain strongly typed and GAT-based.

use async_trait::async_trait;
use futures_core::Stream;
use modkit_sdk::odata::QueryBuilder;
use modkit_security::SecurityContext;
use std::pin::Pin;
use uuid::Uuid;

use crate::errors::UsersInfoError;
use crate::models::{
    Address, City, NewAddress, NewCity, NewUser, UpdateAddressRequest, UpdateCityRequest,
    UpdateUserRequest, User, UserFull,
};

#[cfg(feature = "odata")]
use crate::odata::{AddressSchema, CitySchema, UserSchema};

/// Boxed stream type returned by streaming client facades.
pub type UsersInfoStream<T> =
    Pin<Box<dyn Stream<Item = Result<T, UsersInfoError>> + Send + 'static>>;

/// Object-safe client for inter-module consumption (`ClientHub` registered) (Version 1).
///
/// This trait is registered in `ClientHub`:
/// ```ignore
/// let users_info = hub.get::<dyn UsersInfoClientV1>()?;
/// ```
#[async_trait]
pub trait UsersInfoClientV1: Send + Sync {
    fn users(&self) -> Box<dyn UsersStreamingClientV1>;
    fn cities(&self) -> Box<dyn CitiesStreamingClientV1>;
    fn addresses(&self) -> Box<dyn AddressesStreamingClientV1>;

    // ==================== Single-Item Operations ====================

    /// Get a single user by ID.
    async fn get_user(&self, ctx: SecurityContext, id: Uuid) -> Result<User, UsersInfoError>;

    /// Get aggregated user with address and city.
    async fn get_user_full(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> Result<UserFull, UsersInfoError>;

    /// Get a single city by ID.
    async fn get_city(&self, ctx: SecurityContext, id: Uuid) -> Result<City, UsersInfoError>;

    /// Get a single address by ID.
    async fn get_address(&self, ctx: SecurityContext, id: Uuid) -> Result<Address, UsersInfoError>;

    /// Get address by user ID (1-to-1 relationship).
    async fn get_address_by_user(
        &self,
        ctx: SecurityContext,
        user_id: Uuid,
    ) -> Result<Option<Address>, UsersInfoError>;

    // ==================== Mutation Operations ====================

    /// Create a new user.
    async fn create_user(
        &self,
        ctx: SecurityContext,
        new_user: NewUser,
    ) -> Result<User, UsersInfoError>;

    /// Update an existing user.
    async fn update_user(
        &self,
        ctx: SecurityContext,
        req: UpdateUserRequest,
    ) -> Result<User, UsersInfoError>;

    /// Delete a user by ID.
    async fn delete_user(&self, ctx: SecurityContext, id: Uuid) -> Result<(), UsersInfoError>;

    /// Create a new city.
    async fn create_city(
        &self,
        ctx: SecurityContext,
        new_city: NewCity,
    ) -> Result<City, UsersInfoError>;

    /// Update an existing city.
    async fn update_city(
        &self,
        ctx: SecurityContext,
        req: UpdateCityRequest,
    ) -> Result<City, UsersInfoError>;

    /// Delete a city by ID.
    async fn delete_city(&self, ctx: SecurityContext, id: Uuid) -> Result<(), UsersInfoError>;

    /// Create a new address.
    async fn create_address(
        &self,
        ctx: SecurityContext,
        new_address: NewAddress,
    ) -> Result<Address, UsersInfoError>;

    /// Update an existing address.
    async fn update_address(
        &self,
        ctx: SecurityContext,
        req: UpdateAddressRequest,
    ) -> Result<Address, UsersInfoError>;

    /// Delete an address by ID.
    async fn delete_address(&self, ctx: SecurityContext, id: Uuid) -> Result<(), UsersInfoError>;
}

/// Streaming interface for users (Version 1).
pub trait UsersStreamingClientV1: Send + Sync {
    fn stream(
        &self,
        ctx: SecurityContext,
        query: QueryBuilder<UserSchema>,
    ) -> UsersInfoStream<User>;
}

/// Streaming interface for cities (Version 1).
pub trait CitiesStreamingClientV1: Send + Sync {
    fn stream(
        &self,
        ctx: SecurityContext,
        query: QueryBuilder<CitySchema>,
    ) -> UsersInfoStream<City>;
}

/// Streaming interface for addresses (Version 1).
pub trait AddressesStreamingClientV1: Send + Sync {
    fn stream(
        &self,
        ctx: SecurityContext,
        query: QueryBuilder<AddressSchema>,
    ) -> UsersInfoStream<Address>;
}
