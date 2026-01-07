//! Typed `OData` query builder - re-exported from `modkit-odata`
//!
//! This module re-exports the canonical `OData` query building functionality from `modkit-odata`,
//! along with SDK-specific streaming utilities for cursor-based pagination.
//!
//! The SDK re-exports the canonical `QueryBuilder` from `modkit-odata`.
//! Streaming adapters are provided as free functions `pages_stream` and `items_stream`.
//!
//! # Example
//!
//! ```rust,ignore
//! use modkit_sdk::odata::{items_stream, pages_stream, FieldRef, QueryBuilder, Schema};
//! use modkit_odata::SortDir;
//!
//! #[derive(Copy, Clone, Eq, PartialEq)]
//! enum UserField {
//!     Id,
//!     Name,
//!     Email,
//! }
//!
//! struct UserSchema;
//!
//! impl Schema for UserSchema {
//!     type Field = UserField;
//!
//!     fn field_name(field: Self::Field) -> &'static str {
//!         match field {
//!             UserField::Id => "id",
//!             UserField::Name => "name",
//!             UserField::Email => "email",
//!         }
//!     }
//! }
//!
//! // Define typed field references
//! const ID: FieldRef<UserSchema, uuid::Uuid> = FieldRef::new(UserField::Id);
//! const NAME: FieldRef<UserSchema, String> = FieldRef::new(UserField::Name);
//!
//! // Build a query
//! let user_id = uuid::Uuid::new_v4();
//! let query = QueryBuilder::<UserSchema>::new()
//!     .filter(ID.eq(user_id).and(NAME.contains("john")))
//!     .order_by(NAME, SortDir::Asc)
//!     .page_size(50)
//!     .build();
//!
//! // Stream pages
//! let pages = pages_stream(
//!     QueryBuilder::<UserSchema>::new()
//!         .filter(ID.eq(user_id).and(NAME.contains("john")))
//!         .page_size(50),
//!     |q| async move { client.list_users(q).await },
//! );
//!
//! // Stream items
//! let items = items_stream(
//!     QueryBuilder::<UserSchema>::new()
//!         .filter(ID.eq(user_id).and(NAME.contains("john")))
//!         .page_size(50),
//!     |q| async move { client.list_users(q).await },
//! );
//! ```

pub use modkit_odata::ODataQuery;

// Re-export core OData types from modkit-odata (the canonical source)
pub use modkit_odata::schema::{AsFieldKey, AsFieldName, FieldRef, IntoODataValue, Schema};
pub use modkit_odata::QueryBuilder;
use std::future::Future;
use std::pin::Pin;

/// Boxed future for `OData` page fetchers.
pub type BoxedODataFuture<'a, T, E> =
    Pin<Box<dyn Future<Output = Result<modkit_odata::Page<T>, E>> + Send + 'a>>;

/// Boxed fetcher for `OData` pagination (accepts an `ODataQuery` and returns a boxed future).
pub type BoxedODataFetcher<'a, T, E> =
    Box<dyn FnMut(ODataQuery) -> BoxedODataFuture<'a, T, E> + Send + 'a>;

/// Named stream type produced by `items_stream` when using boxed fetchers.
pub type ItemsStream<'a, T, E> =
    crate::pager::CursorPager<T, E, BoxedODataFetcher<'a, T, E>, BoxedODataFuture<'a, T, E>>;

/// Create a stream of pages using cursor pagination.
///
/// This consumes the builder, builds `ODataQuery`, then returns a `PagesPager`.
pub fn pages_stream<S, T, E, F, Fut>(
    builder: QueryBuilder<S>,
    fetcher: F,
) -> crate::pager::PagesPager<T, E, F, Fut>
where
    S: Schema,
    F: FnMut(ODataQuery) -> Fut,
    Fut: std::future::Future<Output = Result<modkit_odata::Page<T>, E>>,
{
    let query = builder.build();
    crate::pager::PagesPager::new(query, fetcher)
}

/// Create a stream of items using cursor pagination.
///
/// This consumes the builder, builds `ODataQuery`, then returns a `CursorPager`.
pub fn items_stream<S, T, E, F, Fut>(
    builder: QueryBuilder<S>,
    fetcher: F,
) -> crate::pager::CursorPager<T, E, F, Fut>
where
    S: Schema,
    F: FnMut(ODataQuery) -> Fut,
    Fut: std::future::Future<Output = Result<modkit_odata::Page<T>, E>>,
{
    let query = builder.build();
    crate::pager::CursorPager::new(query, fetcher)
}

/// Create a boxed stream of pages using cursor pagination.
///
/// This helper mirrors `pages_stream` but accepts a boxed fetcher and returns the boxed pager type.
///
/// # Example
/// ```rust,ignore
/// use modkit_sdk::odata::{pages_stream_boxed, QueryBuilder};
/// use std::pin::Pin;
///
/// let stream = pages_stream_boxed(
///     QueryBuilder::<UserSchema>::new().page_size(50),
///     Box::new(|q| Box::pin(async move { service.list_users_page(&ctx, &q).await })),
/// );
/// ```
#[must_use]
pub fn pages_stream_boxed<S, T, E>(
    builder: QueryBuilder<S>,
    fetcher: BoxedODataFetcher<'_, T, E>,
) -> crate::pager::PagesPager<T, E, BoxedODataFetcher<'_, T, E>, BoxedODataFuture<'_, T, E>>
where
    S: Schema,
{
    let query = builder.build();
    crate::pager::PagesPager::new(query, fetcher)
}

/// Create a boxed stream of items using cursor pagination.
///
/// This helper mirrors `items_stream` but accepts a boxed fetcher and returns the named `ItemsStream` alias.
///
/// # Example
/// ```rust,ignore
/// use modkit_sdk::odata::{items_stream_boxed, QueryBuilder};
///
/// let stream = items_stream_boxed(
///     QueryBuilder::<UserSchema>::new().page_size(50),
///     Box::new(|q| Box::pin(async move { service.list_users_page(&ctx, &q).await })),
/// );
/// ```
#[must_use]
pub fn items_stream_boxed<S, T, E>(
    builder: QueryBuilder<S>,
    fetcher: BoxedODataFetcher<'_, T, E>,
) -> ItemsStream<'_, T, E>
where
    S: Schema,
{
    let query = builder.build();
    crate::pager::CursorPager::new(query, fetcher)
}
