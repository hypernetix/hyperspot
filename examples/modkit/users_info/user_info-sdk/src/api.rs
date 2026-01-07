//! Object-safe streaming boundary for the `user_info` module.
//!
//! This API is designed for `ClientHub` registration as `Arc<dyn UsersInfoClient>`.
//! All type erasure (boxed streams/futures) lives here; internal implementations
//! remain strongly typed and GAT-based.

use futures::future::BoxFuture;
use futures_core::Stream;
use modkit_sdk::odata::QueryBuilder;
use modkit_security::SecurityContext;
use std::pin::Pin;
use uuid::Uuid;

use crate::errors::UsersInfoError;
use crate::models::{
    Address, City, Language, NewAddress, NewCity, NewLanguage, NewUser, UpdateAddressRequest,
    UpdateCityRequest, UpdateLanguageRequest, UpdateUserRequest, User,
};

#[cfg(feature = "odata")]
use crate::odata::{AddressSchema, CitySchema, LanguageSchema, UserSchema};

/// Boxed stream type returned by streaming client facades.
pub type UsersInfoStream<T> =
    Pin<Box<dyn Stream<Item = Result<T, UsersInfoError>> + Send + 'static>>;

/// Object-safe client for inter-module consumption (`ClientHub` registered).
pub trait UsersInfoClient: Send + Sync {
    fn users(&self) -> Box<dyn UsersStreamingClient>;
    fn cities(&self) -> Box<dyn CitiesStreamingClient>;
    fn languages(&self) -> Box<dyn LanguagesStreamingClient>;
    fn addresses(&self) -> Box<dyn AddressesStreamingClient>;

    // ==================== Single-Item Operations ====================

    /// Get a single user by ID.
    fn get_user(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> BoxFuture<'static, Result<User, UsersInfoError>>;

    /// Get a single city by ID.
    fn get_city(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> BoxFuture<'static, Result<City, UsersInfoError>>;

    /// Get a single language by ID.
    fn get_language(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> BoxFuture<'static, Result<Language, UsersInfoError>>;

    /// Get a single address by ID.
    fn get_address(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> BoxFuture<'static, Result<Address, UsersInfoError>>;

    /// Get address by user ID (1-to-1 relationship).
    fn get_address_by_user(
        &self,
        ctx: SecurityContext,
        user_id: Uuid,
    ) -> BoxFuture<'static, Result<Option<Address>, UsersInfoError>>;

    // ==================== Mutation Operations ====================

    /// Create a new user.
    fn create_user(
        &self,
        ctx: SecurityContext,
        new_user: NewUser,
    ) -> BoxFuture<'static, Result<User, UsersInfoError>>;

    /// Update an existing user.
    fn update_user(
        &self,
        ctx: SecurityContext,
        req: UpdateUserRequest,
    ) -> BoxFuture<'static, Result<User, UsersInfoError>>;

    /// Delete a user by ID.
    fn delete_user(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> BoxFuture<'static, Result<(), UsersInfoError>>;

    /// Create a new city.
    fn create_city(
        &self,
        ctx: SecurityContext,
        new_city: NewCity,
    ) -> BoxFuture<'static, Result<City, UsersInfoError>>;

    /// Update an existing city.
    fn update_city(
        &self,
        ctx: SecurityContext,
        req: UpdateCityRequest,
    ) -> BoxFuture<'static, Result<City, UsersInfoError>>;

    /// Delete a city by ID.
    fn delete_city(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> BoxFuture<'static, Result<(), UsersInfoError>>;

    /// Create a new language.
    fn create_language(
        &self,
        ctx: SecurityContext,
        new_language: NewLanguage,
    ) -> BoxFuture<'static, Result<Language, UsersInfoError>>;

    /// Update an existing language.
    fn update_language(
        &self,
        ctx: SecurityContext,
        req: UpdateLanguageRequest,
    ) -> BoxFuture<'static, Result<Language, UsersInfoError>>;

    /// Delete a language by ID.
    fn delete_language(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> BoxFuture<'static, Result<(), UsersInfoError>>;

    /// Create a new address.
    fn create_address(
        &self,
        ctx: SecurityContext,
        new_address: NewAddress,
    ) -> BoxFuture<'static, Result<Address, UsersInfoError>>;

    /// Update an existing address.
    fn update_address(
        &self,
        ctx: SecurityContext,
        req: UpdateAddressRequest,
    ) -> BoxFuture<'static, Result<Address, UsersInfoError>>;

    /// Delete an address by ID.
    fn delete_address(
        &self,
        ctx: SecurityContext,
        id: Uuid,
    ) -> BoxFuture<'static, Result<(), UsersInfoError>>;

    // ==================== Relationship Operations ====================

    /// Assign a language to a user (many-to-many).
    fn assign_language_to_user(
        &self,
        ctx: SecurityContext,
        user_id: Uuid,
        language_id: Uuid,
    ) -> BoxFuture<'static, Result<(), UsersInfoError>>;

    /// Remove a language from a user.
    fn remove_language_from_user(
        &self,
        ctx: SecurityContext,
        user_id: Uuid,
        language_id: Uuid,
    ) -> BoxFuture<'static, Result<(), UsersInfoError>>;

    /// List all languages assigned to a user.
    fn list_user_languages(
        &self,
        ctx: SecurityContext,
        user_id: Uuid,
    ) -> BoxFuture<'static, Result<Vec<Language>, UsersInfoError>>;
}

/// Streaming interface for users.
pub trait UsersStreamingClient: Send + Sync {
    fn stream(
        &self,
        ctx: SecurityContext,
        query: QueryBuilder<UserSchema>,
    ) -> UsersInfoStream<User>;
}

/// Streaming interface for cities.
pub trait CitiesStreamingClient: Send + Sync {
    fn stream(
        &self,
        ctx: SecurityContext,
        query: QueryBuilder<CitySchema>,
    ) -> UsersInfoStream<City>;
}

/// Streaming interface for languages.
pub trait LanguagesStreamingClient: Send + Sync {
    fn stream(
        &self,
        ctx: SecurityContext,
        query: QueryBuilder<LanguageSchema>,
    ) -> UsersInfoStream<Language>;
}

/// Streaming interface for addresses.
pub trait AddressesStreamingClient: Send + Sync {
    fn stream(
        &self,
        ctx: SecurityContext,
        query: QueryBuilder<AddressSchema>,
    ) -> UsersInfoStream<Address>;
}
