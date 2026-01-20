use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect, sea_query::Expr,
};
use std::marker::PhantomData;

use crate::secure::cond::build_scope_condition;
use crate::secure::error::ScopeError;
use crate::secure::{AccessScope, ScopableEntity};

/// Typestate marker: query has not yet been scoped.
/// Cannot execute queries in this state.
#[derive(Debug, Clone, Copy)]
pub struct Unscoped;

/// Typestate marker: query has been scoped with access control.
/// Can now execute queries safely.
#[derive(Debug, Clone, Copy)]
pub struct Scoped;

/// A type-safe wrapper around `SeaORM`'s `Select` that enforces scoping.
///
/// This wrapper uses the typestate pattern to ensure that queries cannot
/// be executed without first applying access control via `.scope_with()`.
///
/// # Type Parameters
/// - `E`: The `SeaORM` entity type
/// - `S`: The typestate (`Unscoped` or `Scoped`)
///
/// # Example
/// ```rust,ignore
/// use modkit_db::secure::{AccessScope, SecureEntityExt};
///
/// let scope = AccessScope::tenants_only(vec![tenant_id]);
/// let users = user::Entity::find()
///     .secure()           // Returns SecureSelect<E, Unscoped>
///     .scope_with(&scope)? // Returns SecureSelect<E, Scoped>
///     .all(conn)          // Now can execute
///     .await?;
/// ```
#[must_use]
#[derive(Clone, Debug)]
pub struct SecureSelect<E: EntityTrait, S> {
    pub(crate) inner: sea_orm::Select<E>,
    pub(crate) _state: PhantomData<S>,
}

/// Extension trait to convert a regular `SeaORM` `Select` into a `SecureSelect`.
pub trait SecureEntityExt<E: EntityTrait>: Sized {
    /// Convert this select query into a secure (unscoped) select.
    /// You must call `.scope_with()` before executing the query.
    fn secure(self) -> SecureSelect<E, Unscoped>;
}

impl<E> SecureEntityExt<E> for sea_orm::Select<E>
where
    E: EntityTrait,
{
    fn secure(self) -> SecureSelect<E, Unscoped> {
        SecureSelect {
            inner: self,
            _state: PhantomData,
        }
    }
}

// Methods available only on Unscoped queries
impl<E> SecureSelect<E, Unscoped>
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    /// Apply access control scope to this query, transitioning to the `Scoped` state.
    ///
    /// This applies the implicit policy:
    /// - Empty scope → deny all
    /// - Tenants only → filter by tenant
    /// - Resources only → filter by resource IDs
    /// - Both → AND them together
    ///
    pub fn scope_with(self, scope: &AccessScope) -> SecureSelect<E, Scoped> {
        let cond = build_scope_condition::<E>(scope);
        SecureSelect {
            inner: self.inner.filter(cond),
            _state: PhantomData,
        }
    }
}

// Methods available only on Scoped queries
impl<E> SecureSelect<E, Scoped>
where
    E: EntityTrait,
{
    /// Execute the query and return all matching results.
    ///
    /// # Errors
    /// Returns `ScopeError::Db` if the database query fails.
    #[allow(clippy::disallowed_methods)]
    pub async fn all<C>(self, conn: &C) -> Result<Vec<E::Model>, ScopeError>
    where
        C: ConnectionTrait + Send + Sync,
    {
        Ok(self.inner.all(conn).await?)
    }

    /// Execute the query and return at most one result.
    ///
    /// # Errors
    /// Returns `ScopeError::Db` if the database query fails.
    #[allow(clippy::disallowed_methods)]
    pub async fn one<C>(self, conn: &C) -> Result<Option<E::Model>, ScopeError>
    where
        C: ConnectionTrait + Send + Sync,
    {
        Ok(self.inner.one(conn).await?)
    }

    /// Execute the query and return the number of matching results.
    ///
    /// # Errors
    /// Returns `ScopeError::Db` if the database query fails.
    #[allow(clippy::disallowed_methods)]
    pub async fn count<C>(self, conn: &C) -> Result<u64, ScopeError>
    where
        C: ConnectionTrait + Send + Sync,
        E::Model: sea_orm::FromQueryResult + Send + Sync,
    {
        Ok(self.inner.count(conn).await?)
    }

    // Note: count() uses SeaORM's `PaginatorTrait::count` internally.

    // Note: For pagination, use `into_inner().paginate()` due to complex lifetime bounds

    /// Add an additional filter for a specific resource ID.
    ///
    /// This is useful when you want to further narrow a scoped query
    /// to a single resource.
    ///
    /// # Example
    /// ```ignore
    /// let user = User::find()
    ///     .secure()
    ///     .scope_with(&scope)?
    ///     .and_id(user_id)
    ///     .one(conn)
    ///     .await?;
    /// ```
    ///
    /// # Errors
    /// Returns `ScopeError::Invalid` if the entity doesn't have a resource column.
    pub fn and_id(self, id: uuid::Uuid) -> Result<Self, ScopeError>
    where
        E: ScopableEntity,
        E::Column: ColumnTrait + Copy,
    {
        let resource_col = E::resource_col().ok_or(ScopeError::Invalid(
            "Entity must have a resource_col to use and_id()",
        ))?;
        let cond = sea_orm::Condition::all().add(Expr::col(resource_col).eq(id));
        Ok(self.filter(cond))
    }

    /// Unwrap the inner `SeaORM` `Select` for advanced use cases.
    ///
    /// This is an escape hatch if you need to add additional filters,
    /// joins, or ordering after scoping has been applied.
    ///
    /// # Safety
    /// The caller must ensure they don't remove or bypass the security
    /// conditions that were applied during `.scope_with()`.
    #[must_use]
    pub fn into_inner(self) -> sea_orm::Select<E> {
        self.inner
    }
}

// Allow further chaining on Scoped queries before execution
impl<E> SecureSelect<E, Scoped>
where
    E: EntityTrait,
{
    /// Add additional filters to the scoped query.
    /// The scope conditions remain in place.
    pub fn filter(mut self, filter: sea_orm::Condition) -> Self {
        self.inner = QueryFilter::filter(self.inner, filter);
        self
    }

    /// Add ordering to the scoped query.
    pub fn order_by<C>(mut self, col: C, order: sea_orm::Order) -> Self
    where
        C: sea_orm::IntoSimpleExpr,
    {
        self.inner = QueryOrder::order_by(self.inner, col, order);
        self
    }

    /// Add a limit to the scoped query.
    pub fn limit(mut self, limit: u64) -> Self {
        self.inner = QuerySelect::limit(self.inner, limit);
        self
    }

    /// Add an offset to the scoped query.
    pub fn offset(mut self, offset: u64) -> Self {
        self.inner = QuerySelect::offset(self.inner, offset);
        self
    }

    /// Apply scoping for a joined entity.
    ///
    /// This is useful when you need to filter by tenant on a joined table.
    ///
    /// # Example
    /// ```ignore
    /// // Select orders, ensuring both Order and Customer match tenant scope
    /// Order::find()
    ///     .secure()
    ///     .scope_with(&scope)?
    ///     .and_scope_for::<customer::Entity>(&scope)
    ///     .all(conn)
    ///     .await?
    /// ```
    pub fn and_scope_for<J>(mut self, scope: &AccessScope) -> Self
    where
        J: ScopableEntity + EntityTrait,
        J::Column: ColumnTrait + Copy,
    {
        if !scope.tenant_ids().is_empty()
            && let Some(tcol) = J::tenant_col()
        {
            let condition = sea_orm::Condition::all()
                .add(Expr::col((J::default(), tcol)).is_in(scope.tenant_ids().to_vec()));
            self.inner = QueryFilter::filter(self.inner, condition);
        }
        self
    }

    /// Apply scoping via EXISTS subquery on a related entity.
    ///
    /// This is particularly useful when the base entity doesn't have a tenant column
    /// but is related to one that does.
    ///
    /// # Note
    /// This is a simplified version that filters by tenant on the joined entity.
    /// For complex join predicates, use `into_inner()` and build custom EXISTS clauses.
    ///
    /// # Example
    /// ```ignore
    /// // Find settings that exist in a tenant-scoped relationship
    /// GlobalSetting::find()
    ///     .secure()
    ///     .scope_with(&AccessScope::resources_only(vec![]))?
    ///     .scope_via_exists::<TenantSetting>(&scope)
    ///     .all(conn)
    ///     .await?
    /// ```
    pub fn scope_via_exists<J>(mut self, scope: &AccessScope) -> Self
    where
        J: ScopableEntity + EntityTrait,
        J::Column: ColumnTrait + Copy,
    {
        if !scope.tenant_ids().is_empty()
            && let Some(tcol) = J::tenant_col()
        {
            // Build EXISTS clause with tenant filter on joined entity
            use sea_orm::sea_query::Query;

            let mut sub = Query::select();
            sub.expr(Expr::value(1))
                .from(J::default())
                .cond_where(Expr::col((J::default(), tcol)).is_in(scope.tenant_ids().to_vec()));

            self.inner =
                QueryFilter::filter(self.inner, sea_orm::Condition::all().add(Expr::exists(sub)));
        }
        self
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    // Note: Full integration tests with real SeaORM entities should be written
    // in application code where actual entities are available.
    // The typestate pattern is enforced at compile time.
    //
    // See USAGE_EXAMPLE.md for complete usage patterns.

    #[test]
    fn test_typestate_markers_exist() {
        // This test verifies the typestate markers compile
        // The actual enforcement happens at compile time
        let unscoped: std::marker::PhantomData<Unscoped> = std::marker::PhantomData;
        let scoped: std::marker::PhantomData<Scoped> = std::marker::PhantomData;
        // Use the variables to avoid unused warnings
        let _ = (unscoped, scoped);
    }
}
