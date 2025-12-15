//! Minimal fluent builder for combining Secure ORM scoping with `OData` pagination.
//!
//! This module provides `OPager`, a small ergonomic builder that:
//! - Applies security scope via `SecureConn::find::<E>(&SecurityCtx)`
//! - Applies `OData` filter + cursor + order + limit via `paginate_with_odata`
//! - Keeps all existing types without introducing facades or macros
//!
//! # Quick Start
//!
//! ```ignore
//! use modkit_db::odata::{FieldMap, FieldKind, pager::OPager};
//! use modkit_db::secure::{SecureConn, SecurityCtx};
//! use modkit_odata::{ODataQuery, SortDir, Page, Error as ODataError};
//!
//! // Define field mappings once (typically as a static or const)
//! fn user_field_map() -> FieldMap<user::Entity> {
//!     FieldMap::new()
//!         .insert("id", user::Column::Id, FieldKind::Uuid)
//!         .insert("name", user::Column::Name, FieldKind::String)
//!         .insert("email", user::Column::Email, FieldKind::String)
//!         .insert("created_at", user::Column::CreatedAt, FieldKind::DateTimeUtc)
//! }
//!
//! // In your repository or service layer
//! pub async fn list_users(
//!     db: &SecureConn,
//!     ctx: &SecurityCtx,
//!     q: &ODataQuery,
//! ) -> Result<Page<UserDto>, ODataError> {
//!     OPager::<user::Entity, _>::new(db, ctx, db.conn(), &user_field_map())
//!         .tiebreaker("created_at", SortDir::Desc)
//!         .limits(50, 500)
//!         .fetch(q, |model| UserDto {
//!             id: model.id,
//!             name: model.name,
//!             email: model.email,
//!         })
//!         .await
//! }
//! ```
//!
//! # Complete Example
//!
//! ```ignore
//! use modkit_db::odata::{FieldMap, FieldKind, pager::OPager};
//! use modkit_db::secure::{SecureConn, SecurityCtx, ScopableEntity};
//! use modkit_odata::{ODataQuery, SortDir};
//! use sea_orm::entity::prelude::*;
//! use uuid::Uuid;
//!
//! // 1. Define your entity with Scopable
//! #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Scopable)]
//! #[sea_orm(table_name = "users")]
//! #[secure(tenant_col = "tenant_id")]
//! pub struct Model {
//!     #[sea_orm(primary_key)]
//!     pub id: Uuid,
//!     pub tenant_id: Uuid,
//!     pub name: String,
//!     pub email: String,
//!     pub created_at: DateTime<Utc>,
//! }
//!
//! // 2. Define field mappings
//! static USER_FIELD_MAP: Lazy<FieldMap<Entity>> = Lazy::new(|| {
//!     FieldMap::new()
//!         .insert("id", Column::Id, FieldKind::Uuid)
//!         .insert("name", Column::Name, FieldKind::String)
//!         .insert("email", Column::Email, FieldKind::String)
//!         .insert("created_at", Column::CreatedAt, FieldKind::DateTimeUtc)
//! });
//!
//! // 3. Use in your service
//! pub struct UserService<'a> {
//!     db: &'a SecureConn,
//! }
//!
//! impl<'a> UserService<'a> {
//!     pub async fn list_users(
//!         &self,
//!         ctx: &SecurityCtx,
//!         odata_query: &ODataQuery,
//!     ) -> Result<Page<UserDto>, ODataError> {
//!         OPager::<Entity, _>::new(self.db, ctx, self.db.conn(), &USER_FIELD_MAP)
//!             .tiebreaker("id", SortDir::Desc)
//!             .limits(25, 1000)
//!             .fetch(odata_query, |m| UserDto {
//!                 id: m.id,
//!                 name: m.name,
//!                 email: m.email,
//!             })
//!             .await
//!     }
//! }
//! ```
//!
//! # Security
//!
//! `OPager` automatically enforces tenant isolation and access control:
//! - Security scope is applied before any filters
//! - Empty scopes result in deny-all (no data returned)
//! - All queries are scoped by the `SecurityCtx` provided
//!
//! # Performance
//!
//! - Uses cursor-based pagination for efficient large dataset traversal
//! - Fetches limit+1 rows to detect "has more" without separate COUNT query
//! - Applies filters at the database level (not in application memory)
//! - Supports indexed columns via field mappings for optimal query performance

use modkit_odata::{Error as ODataError, ODataQuery, Page, SortDir};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait};

use crate::odata::{paginate_with_odata, FieldMap, LimitCfg};
use crate::secure::{ScopableEntity, SecureConn, SecurityCtx};

/// Minimal fluent builder for Secure + `OData` pagination.
///
/// This builder combines security-scoped queries with `OData` pagination
/// in a single, ergonomic interface. It enforces tenant isolation and
/// access control while providing cursor-based pagination with filtering
/// and ordering.
///
/// # Type Parameters
///
/// - `E`: The `SeaORM` entity type (must implement `ScopableEntity`)
/// - `C`: The database connection type (any `ConnectionTrait`)
///
/// # Usage
///
/// ```ignore
/// OPager::<UserEntity, _>::new(db, ctx, db.conn(), &FMAP)
///   .tiebreaker("id", SortDir::Desc)  // optional, defaults to ("id", Desc)
///   .limits(25, 1000)                  // optional, defaults to (25, 1000)
///   .fetch(&query, |m| dto_from(m))
///   .await
/// ```
///
/// # Default Behavior
///
/// - Tiebreaker: `("id", SortDir::Desc)` - ensures stable pagination
/// - Limits: `{ default: 25, max: 1000 }` - reasonable defaults for most APIs
#[must_use]
pub struct OPager<'a, E, C>
where
    E: EntityTrait,
    E::Column: ColumnTrait + Copy,
    C: ConnectionTrait + Send + Sync,
{
    db: &'a SecureConn,
    ctx: &'a SecurityCtx,
    conn: &'a C,
    fmap: &'a FieldMap<E>,
    tiebreaker: (&'a str, SortDir),
    limits: LimitCfg,
}

impl<'a, E, C> OPager<'a, E, C>
where
    E: EntityTrait,
    E::Column: ColumnTrait + Copy,
    C: ConnectionTrait + Send + Sync,
{
    /// Construct a new pager over a secured, scoped Select<E>.
    ///
    /// # Parameters
    ///
    /// - `db`: Secure database connection wrapper
    /// - `ctx`: Security context defining access scope (tenant/resource boundaries)
    /// - `conn`: Raw connection for executing queries
    /// - `fmap`: Field map defining `OData` field → entity column mappings
    ///
    /// # Example
    ///
    /// ```ignore
    /// let pager = OPager::<UserEntity, _>::new(
    ///     db,
    ///     &SecurityCtx::for_tenant(tenant_id, user_id),
    ///     db.conn(),
    ///     &USER_FIELD_MAP
    /// );
    /// ```
    pub fn new(
        db: &'a SecureConn,
        ctx: &'a SecurityCtx,
        conn: &'a C,
        fmap: &'a FieldMap<E>,
    ) -> Self {
        Self {
            db,
            ctx,
            conn,
            fmap,
            // Sane defaults that work for most use cases
            tiebreaker: ("id", SortDir::Desc),
            limits: LimitCfg {
                default: 25,
                max: 1000,
            },
        }
    }

    /// Override the default tiebreaker ("id", Desc).
    ///
    /// The tiebreaker ensures stable, deterministic pagination by providing
    /// a final sort key when the primary order has duplicate values.
    ///
    /// # Parameters
    ///
    /// - `field`: The field name (as defined in the `FieldMap`) to use as tiebreaker
    /// - `dir`: Sort direction for the tiebreaker field
    ///
    /// # Example
    ///
    /// ```ignore
    /// pager.tiebreaker("created_at", SortDir::Asc)
    /// ```
    pub fn tiebreaker(mut self, field: &'a str, dir: SortDir) -> Self {
        self.tiebreaker = (field, dir);
        self
    }

    /// Override default/max limits (defaults: 25/1000).
    ///
    /// Controls pagination limits:
    /// - `default`: Used when client doesn't specify a limit
    /// - `max`: Maximum limit value (client requests clamped to this)
    ///
    /// # Parameters
    ///
    /// - `default`: Default page size (if client doesn't specify)
    /// - `max`: Maximum allowed page size (requests clamped to this)
    ///
    /// # Example
    ///
    /// ```ignore
    /// pager.limits(10, 100)  // Smaller pages for this endpoint
    /// ```
    pub fn limits(mut self, default: u64, max: u64) -> Self {
        self.limits = LimitCfg { default, max };
        self
    }

    /// Execute paging and map models to domain DTOs.
    ///
    /// This is the terminal operation that:
    /// 1. Applies security scope (tenant/resource filtering)
    /// 2. Applies `OData` filter (if present in query)
    /// 3. Applies cursor-based pagination
    /// 4. Fetches limit+1 rows (to detect "has more")
    /// 5. Maps entity models to domain DTOs
    /// 6. Returns a `Page<D>` with items and pagination metadata
    ///
    /// # Type Parameters
    ///
    /// - `D`: The domain DTO type (result of mapping)
    /// - `F`: Mapper function from `E::Model` to `D`
    ///
    /// # Parameters
    ///
    /// - `q`: `OData` query containing filter, order, cursor, and limit
    /// - `map`: Function to convert entity models to domain DTOs
    ///
    /// # Errors
    ///
    /// Returns `ODataError` if:
    /// - Security scope cannot be applied
    /// - `OData` filter is invalid
    /// - Database query fails
    /// - Cursor is malformed or inconsistent
    ///
    /// # Example
    ///
    /// ```ignore
    /// let page: Page<UserDto> = pager
    ///     .fetch(&odata_query, |model| UserDto {
    ///         id: model.id,
    ///         name: model.name,
    ///         email: model.email,
    ///     })
    ///     .await?;
    /// ```
    pub async fn fetch<D, F>(self, q: &ODataQuery, map: F) -> Result<Page<D>, ODataError>
    where
        E: ScopableEntity,
        F: Fn(E::Model) -> D + Copy,
    {
        // Apply security scope first - this enforces tenant isolation
        let select = self.db.find::<E>(self.ctx).into_inner();

        // Now apply OData filters, cursor, order, and limits
        paginate_with_odata::<E, D, _, _>(
            select,
            self.conn,
            q,
            self.fmap,
            self.tiebreaker,
            self.limits,
            map,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Type safety tests (compile-time validation)
    #[allow(dead_code)]
    fn test_type_safety() {
        // These should compile, demonstrating the API shape
        fn _compile_check<'a, E, C>(
            db: &'a SecureConn,
            ctx: &'a SecurityCtx,
            conn: &'a C,
            fmap: &'a FieldMap<E>,
        ) where
            E: ScopableEntity + EntityTrait,
            E::Column: ColumnTrait + Copy,
            C: ConnectionTrait + Send + Sync,
        {
            let _pager = OPager::<E, C>::new(db, ctx, conn, fmap);
            let _pager = OPager::<E, C>::new(db, ctx, conn, fmap)
                .tiebreaker("id", SortDir::Asc)
                .limits(10, 100);
        }
    }
}
