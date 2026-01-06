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
