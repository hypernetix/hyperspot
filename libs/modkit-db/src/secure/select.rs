use sea_orm::{
    ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect,
    Related, sea_query::Expr,
};
use std::sync::Arc;

use crate::secure::cond::build_scope_condition;
use crate::secure::error::ScopeError;
use crate::secure::{AccessScope, DBRunner, DBRunnerInternal, ScopableEntity, SeaOrmRunner};

/// Typestate marker: query has not yet been scoped.
/// Cannot execute queries in this state.
#[derive(Debug, Clone, Copy)]
pub struct Unscoped;

/// Typestate marker: query has been scoped with access control.
/// Can now execute queries safely.
///
/// This marker carries the `AccessScope` internally so that related-entity
/// queries can automatically apply the same scope without requiring it
/// to be passed again.
#[derive(Debug, Clone)]
pub struct Scoped {
    scope: Arc<AccessScope>,
}

/// A type-safe wrapper around `SeaORM`'s `Select` that enforces scoping.
///
/// This wrapper uses the typestate pattern to ensure that queries cannot
/// be executed without first applying access control via `.scope_with()`.
///
/// When scoped (`SecureSelect<E, Scoped>`), the query carries the `AccessScope`
/// internally. This allows related-entity queries (`find_also_related`,
/// `find_with_related`) to automatically apply the same scope to related
/// entities without requiring the scope to be passed again.
///
/// # Type Parameters
/// - `E`: The `SeaORM` entity type
/// - `S`: The typestate (`Unscoped` or `Scoped`)
///
/// # Example
/// ```rust,ignore
/// use modkit_db::secure::{AccessScope, SecureEntityExt};
///
/// let scope = AccessScope::for_tenants(vec![tenant_id]);
/// let users = user::Entity::find()
///     .secure()           // Returns SecureSelect<E, Unscoped>
///     .scope_with(&scope) // Returns SecureSelect<E, Scoped>
///     .all(conn)          // Now can execute
///     .await?;
///
/// // Related queries auto-apply scope:
/// let orders_with_items = Order::find()
///     .secure()
///     .scope_with(&scope)
///     .find_with_related(line_item::Entity)  // scope auto-applied to LineItem
///     .all(conn)
///     .await?;
/// ```
#[must_use]
#[derive(Clone, Debug)]
pub struct SecureSelect<E: EntityTrait, S> {
    pub(crate) inner: sea_orm::Select<E>,
    pub(crate) state: S,
}

/// A type-safe wrapper around `SeaORM`'s `SelectTwo` that enforces scoping.
///
/// This wrapper is used for `find_also_related` queries where you want to fetch
/// an entity along with an optional related entity (1-to-0..1 relationship).
///
/// The wrapper carries the `AccessScope` internally so further chained operations
/// can apply scoping consistently.
///
/// # Type Parameters
/// - `E`: The primary `SeaORM` entity type
/// - `F`: The related `SeaORM` entity type
/// - `S`: The typestate (`Scoped` - note: only Scoped state is supported)
///
/// # Example
/// ```rust,ignore
/// use modkit_db::secure::{AccessScope, SecureEntityExt};
///
/// let scope = AccessScope::for_tenants(vec![tenant_id]);
/// let rows: Vec<(fruit::Model, Option<cake::Model>)> = Fruit::find()
///     .secure()
///     .scope_with(&scope)
///     .find_also_related(cake::Entity)  // scope auto-applied to cake
///     .all(db)
///     .await?;
/// ```
#[must_use]
#[derive(Clone, Debug)]
pub struct SecureSelectTwo<E: EntityTrait, F: EntityTrait, S> {
    pub(crate) inner: sea_orm::SelectTwo<E, F>,
    pub(crate) state: S,
}

/// A type-safe wrapper around `SeaORM`'s `SelectTwoMany` that enforces scoping.
///
/// This wrapper is used for `find_with_related` queries where you want to fetch
/// an entity along with all its related entities (1-to-many relationship).
///
/// The wrapper carries the `AccessScope` internally so further chained operations
/// can apply scoping consistently.
///
/// # Type Parameters
/// - `E`: The primary `SeaORM` entity type
/// - `F`: The related `SeaORM` entity type
/// - `S`: The typestate (`Scoped` - note: only Scoped state is supported)
///
/// # Example
/// ```rust,ignore
/// use modkit_db::secure::{AccessScope, SecureEntityExt};
///
/// let scope = AccessScope::for_tenants(vec![tenant_id]);
/// let rows: Vec<(cake::Model, Vec<fruit::Model>)> = Cake::find()
///     .secure()
///     .scope_with(&scope)
///     .find_with_related(fruit::Entity)  // scope auto-applied to fruit
///     .all(db)
///     .await?;
/// ```
#[must_use]
#[derive(Clone, Debug)]
pub struct SecureSelectTwoMany<E: EntityTrait, F: EntityTrait, S> {
    pub(crate) inner: sea_orm::SelectTwoMany<E, F>,
    pub(crate) state: S,
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
            state: Unscoped,
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
    /// The scope is stored internally and will be automatically applied to any
    /// related-entity queries (e.g., `find_also_related`, `find_with_related`).
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
            state: Scoped {
                scope: Arc::new(scope.clone()),
            },
        }
    }

    /// Apply access control scope using an `Arc<AccessScope>`.
    ///
    /// This is useful when you already have the scope in an `Arc` and want to
    /// avoid an extra clone.
    pub fn scope_with_arc(self, scope: Arc<AccessScope>) -> SecureSelect<E, Scoped> {
        let cond = build_scope_condition::<E>(&scope);
        SecureSelect {
            inner: self.inner.filter(cond),
            state: Scoped { scope },
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
    pub async fn all(self, runner: &impl DBRunner) -> Result<Vec<E::Model>, ScopeError> {
        match DBRunnerInternal::as_seaorm(runner) {
            SeaOrmRunner::Conn(db) => Ok(self.inner.all(db).await?),
            SeaOrmRunner::Tx(tx) => Ok(self.inner.all(tx).await?),
        }
    }

    /// Execute the query and return at most one result.
    ///
    /// # Errors
    /// Returns `ScopeError::Db` if the database query fails.
    #[allow(clippy::disallowed_methods)]
    pub async fn one(self, runner: &impl DBRunner) -> Result<Option<E::Model>, ScopeError> {
        match DBRunnerInternal::as_seaorm(runner) {
            SeaOrmRunner::Conn(db) => Ok(self.inner.one(db).await?),
            SeaOrmRunner::Tx(tx) => Ok(self.inner.one(tx).await?),
        }
    }

    /// Execute the query and return the number of matching results.
    ///
    /// # Errors
    /// Returns `ScopeError::Db` if the database query fails.
    #[allow(clippy::disallowed_methods)]
    pub async fn count(self, runner: &impl DBRunner) -> Result<u64, ScopeError>
    where
        E::Model: sea_orm::FromQueryResult + Send + Sync,
    {
        match DBRunnerInternal::as_seaorm(runner) {
            SeaOrmRunner::Conn(db) => Ok(self.inner.count(db).await?),
            SeaOrmRunner::Tx(tx) => Ok(self.inner.count(tx).await?),
        }
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
        let tenant_ids = scope.all_values_for(modkit_security::pep_properties::OWNER_TENANT_ID);
        if !tenant_ids.is_empty()
            && let Some(tcol) = J::tenant_col()
        {
            let condition =
                sea_orm::Condition::all().add(Expr::col((J::default(), tcol)).is_in(tenant_ids));
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
    ///     .scope_with(&AccessScope::for_resources(vec![]))?
    ///     .scope_via_exists::<TenantSetting>(&scope)
    ///     .all(conn)
    ///     .await?
    /// ```
    pub fn scope_via_exists<J>(mut self, scope: &AccessScope) -> Self
    where
        J: ScopableEntity + EntityTrait,
        J::Column: ColumnTrait + Copy,
    {
        let tenant_ids = scope.all_values_for(modkit_security::pep_properties::OWNER_TENANT_ID);
        if !tenant_ids.is_empty()
            && let Some(tcol) = J::tenant_col()
        {
            // Build EXISTS clause with tenant filter on joined entity
            use sea_orm::sea_query::Query;

            let mut sub = Query::select();
            sub.expr(Expr::value(1))
                .from(J::default())
                .cond_where(Expr::col((J::default(), tcol)).is_in(tenant_ids));

            self.inner =
                QueryFilter::filter(self.inner, sea_orm::Condition::all().add(Expr::exists(sub)));
        }
        self
    }

    /// Unwrap the inner `SeaORM` `Select` for advanced use cases.
    ///
    /// # Safety
    /// The caller must ensure they don't remove or bypass the security
    /// conditions that were applied during `.scope_with()`.
    #[must_use]
    pub fn into_inner(self) -> sea_orm::Select<E> {
        self.inner
    }
}

// =============================================================================
// Relationship Query Methods on SecureSelect<E, Scoped>
// =============================================================================

/// Helper to apply scope filtering to a related entity if it has a tenant column.
fn apply_related_scope<R>(scope: &AccessScope) -> Option<sea_orm::Condition>
where
    R: ScopableEntity + EntityTrait,
    R::Column: ColumnTrait + Copy,
{
    let tenant_ids = scope.all_values_for(modkit_security::pep_properties::OWNER_TENANT_ID);
    if !tenant_ids.is_empty()
        && let Some(tcol) = R::tenant_col()
    {
        Some(sea_orm::Condition::all().add(Expr::col((R::default(), tcol)).is_in(tenant_ids)))
    } else {
        None
    }
}

impl<E> SecureSelect<E, Scoped>
where
    E: EntityTrait,
{
    /// Get a reference to the stored scope.
    ///
    /// This is useful when you need to pass the scope to other secure operations.
    #[must_use]
    pub fn scope(&self) -> &AccessScope {
        &self.state.scope
    }

    /// Get the stored scope as an `Arc`.
    ///
    /// This is useful when you need to share the scope without cloning.
    #[must_use]
    pub fn scope_arc(&self) -> Arc<AccessScope> {
        Arc::clone(&self.state.scope)
    }

    /// Find related entities using `find_also_related` with automatic scoping.
    ///
    /// This executes a LEFT JOIN to fetch the primary entity along with an
    /// optional related entity. The related entity will be `None` if no
    /// matching row exists.
    ///
    /// # Automatic Scoping
    /// - The primary entity `E` is already scoped by the parent `SecureSelect`.
    /// - The related entity `R` will automatically have tenant filtering applied
    ///   **if it has a tenant column** (i.e., `R::tenant_col()` returns `Some`).
    /// - For **global entities** (those with `#[secure(no_tenant)]`), no additional
    ///   filtering is applied — the scoping becomes a no-op automatically.
    ///
    /// This unified API handles both tenant-scoped and global entities transparently.
    /// The caller does not need to know or care whether the related entity is
    /// tenant-scoped or global.
    ///
    /// # Entity Requirements
    /// All entities used with this method must derive `Scopable`. For global entities,
    /// use `#[secure(no_tenant, no_resource, no_owner, no_type)]`.
    ///
    /// # Example
    /// ```rust,ignore
    /// let scope = AccessScope::for_tenants(vec![tenant_id]);
    ///
    /// // Tenant-scoped related entity - scope is auto-applied to Customer
    /// let rows: Vec<(order::Model, Option<customer::Model>)> = Order::find()
    ///     .secure()
    ///     .scope_with(&scope)
    ///     .find_also_related(customer::Entity)
    ///     .all(db)
    ///     .await?;
    ///
    /// // Global related entity (no tenant column) - no filtering applied to GlobalConfig
    /// let rows: Vec<(order::Model, Option<global_config::Model>)> = Order::find()
    ///     .secure()
    ///     .scope_with(&scope)
    ///     .find_also_related(global_config::Entity)  // same API, auto no-op!
    ///     .all(db)
    ///     .await?;
    /// ```
    pub fn find_also_related<R>(self, r: R) -> SecureSelectTwo<E, R, Scoped>
    where
        R: ScopableEntity + EntityTrait,
        R::Column: ColumnTrait + Copy,
        E: Related<R>,
    {
        let select_two = self.inner.find_also_related(r);

        // Auto-apply scope to the related entity R (no-op if R has no tenant_col)
        let select_two = if let Some(cond) = apply_related_scope::<R>(&self.state.scope) {
            QueryFilter::filter(select_two, cond)
        } else {
            select_two
        };

        SecureSelectTwo {
            inner: select_two,
            state: self.state,
        }
    }

    /// Find all related entities using `find_with_related` with automatic scoping.
    ///
    /// This executes a query to fetch the primary entity along with all its
    /// related entities (one-to-many relationship).
    ///
    /// # Automatic Scoping
    /// - The primary entity `E` is already scoped by the parent `SecureSelect`.
    /// - The related entity `R` will automatically have tenant filtering applied
    ///   **if it has a tenant column** (i.e., `R::tenant_col()` returns `Some`).
    /// - For **global entities** (those with `#[secure(no_tenant)]`), no additional
    ///   filtering is applied — the scoping becomes a no-op automatically.
    ///
    /// This unified API handles both tenant-scoped and global entities transparently.
    /// The caller does not need to know or care whether the related entity is
    /// tenant-scoped or global.
    ///
    /// # Entity Requirements
    /// All entities used with this method must derive `Scopable`. For global entities,
    /// use `#[secure(no_tenant, no_resource, no_owner, no_type)]`.
    ///
    /// # Example
    /// ```rust,ignore
    /// let scope = AccessScope::for_tenants(vec![tenant_id]);
    ///
    /// // Tenant-scoped related entity - scope is auto-applied to LineItem
    /// let rows: Vec<(order::Model, Vec<line_item::Model>)> = Order::find()
    ///     .secure()
    ///     .scope_with(&scope)
    ///     .find_with_related(line_item::Entity)
    ///     .all(db)
    ///     .await?;
    ///
    /// // Global related entity (no tenant column) - no filtering applied to SystemTag
    /// let rows: Vec<(order::Model, Vec<system_tag::Model>)> = Order::find()
    ///     .secure()
    ///     .scope_with(&scope)
    ///     .find_with_related(system_tag::Entity)  // same API, auto no-op!
    ///     .all(db)
    ///     .await?;
    /// ```
    pub fn find_with_related<R>(self, r: R) -> SecureSelectTwoMany<E, R, Scoped>
    where
        R: ScopableEntity + EntityTrait,
        R::Column: ColumnTrait + Copy,
        E: Related<R>,
    {
        let select_two_many = self.inner.find_with_related(r);

        // Auto-apply scope to the related entity R (no-op if R has no tenant_col)
        let select_two_many = if let Some(cond) = apply_related_scope::<R>(&self.state.scope) {
            QueryFilter::filter(select_two_many, cond)
        } else {
            select_two_many
        };

        SecureSelectTwoMany {
            inner: select_two_many,
            state: self.state,
        }
    }
}

// =============================================================================
// SecureSelectTwo<E, F, Scoped> - Execution methods
// =============================================================================

impl<E, F> SecureSelectTwo<E, F, Scoped>
where
    E: EntityTrait,
    F: EntityTrait,
{
    /// Get a reference to the stored scope.
    #[must_use]
    pub fn scope(&self) -> &AccessScope {
        &self.state.scope
    }

    /// Get the stored scope as an `Arc`.
    #[must_use]
    pub fn scope_arc(&self) -> Arc<AccessScope> {
        Arc::clone(&self.state.scope)
    }

    /// Execute the query and return all matching results.
    ///
    /// Returns pairs of `(E::Model, Option<F::Model>)`.
    ///
    /// # Errors
    /// Returns `ScopeError::Db` if the database query fails.
    #[allow(clippy::disallowed_methods)]
    pub async fn all(
        self,
        runner: &impl DBRunner,
    ) -> Result<Vec<(E::Model, Option<F::Model>)>, ScopeError> {
        match DBRunnerInternal::as_seaorm(runner) {
            SeaOrmRunner::Conn(db) => Ok(self.inner.all(db).await?),
            SeaOrmRunner::Tx(tx) => Ok(self.inner.all(tx).await?),
        }
    }

    /// Execute the query and return at most one result.
    ///
    /// # Errors
    /// Returns `ScopeError::Db` if the database query fails.
    #[allow(clippy::disallowed_methods)]
    pub async fn one(
        self,
        runner: &impl DBRunner,
    ) -> Result<Option<(E::Model, Option<F::Model>)>, ScopeError> {
        match DBRunnerInternal::as_seaorm(runner) {
            SeaOrmRunner::Conn(db) => Ok(self.inner.one(db).await?),
            SeaOrmRunner::Tx(tx) => Ok(self.inner.one(tx).await?),
        }
    }

    /// Add additional filters to the query.
    pub fn filter(mut self, filter: sea_orm::Condition) -> Self {
        self.inner = QueryFilter::filter(self.inner, filter);
        self
    }

    /// Add ordering to the query.
    pub fn order_by<C>(mut self, col: C, order: sea_orm::Order) -> Self
    where
        C: sea_orm::IntoSimpleExpr,
    {
        self.inner = QueryOrder::order_by(self.inner, col, order);
        self
    }

    /// Add a limit to the query.
    pub fn limit(mut self, limit: u64) -> Self {
        self.inner = QuerySelect::limit(self.inner, limit);
        self
    }

    /// Unwrap the inner `SeaORM` `SelectTwo` for advanced use cases.
    #[must_use]
    pub fn into_inner(self) -> sea_orm::SelectTwo<E, F> {
        self.inner
    }
}

// =============================================================================
// SecureSelectTwoMany<E, F, Scoped> - Execution methods
// =============================================================================

impl<E, F> SecureSelectTwoMany<E, F, Scoped>
where
    E: EntityTrait,
    F: EntityTrait,
{
    /// Get a reference to the stored scope.
    #[must_use]
    pub fn scope(&self) -> &AccessScope {
        &self.state.scope
    }

    /// Get the stored scope as an `Arc`.
    #[must_use]
    pub fn scope_arc(&self) -> Arc<AccessScope> {
        Arc::clone(&self.state.scope)
    }

    /// Execute the query and return all matching results.
    ///
    /// Returns pairs of `(E::Model, Vec<F::Model>)`.
    ///
    /// # Errors
    /// Returns `ScopeError::Db` if the database query fails.
    #[allow(clippy::disallowed_methods)]
    pub async fn all(
        self,
        runner: &impl DBRunner,
    ) -> Result<Vec<(E::Model, Vec<F::Model>)>, ScopeError> {
        match DBRunnerInternal::as_seaorm(runner) {
            SeaOrmRunner::Conn(db) => Ok(self.inner.all(db).await?),
            SeaOrmRunner::Tx(tx) => Ok(self.inner.all(tx).await?),
        }
    }

    /// Add additional filters to the query.
    pub fn filter(mut self, filter: sea_orm::Condition) -> Self {
        self.inner = QueryFilter::filter(self.inner, filter);
        self
    }

    /// Add ordering to the query.
    pub fn order_by<C>(mut self, col: C, order: sea_orm::Order) -> Self
    where
        C: sea_orm::IntoSimpleExpr,
    {
        self.inner = QueryOrder::order_by(self.inner, col, order);
        self
    }

    /// Unwrap the inner `SeaORM` `SelectTwoMany` for advanced use cases.
    #[must_use]
    pub fn into_inner(self) -> sea_orm::SelectTwoMany<E, F> {
        self.inner
    }
}

// =============================================================================
// Model-level find_related Extension Trait
// =============================================================================

/// Extension trait to perform secure `find_related` queries from a model instance.
///
/// This trait provides a way to find entities related to an already-loaded model
/// while maintaining security scope constraints.
///
/// # Example
/// ```rust,ignore
/// use modkit_db::secure::{AccessScope, SecureFindRelatedExt};
///
/// // Load a cake
/// let cake: cake::Model = db.find_by_id::<cake::Entity>(&scope, cake_id)?
///     .one(db)
///     .await?
///     .unwrap();
///
/// // Find all related fruits with scoping
/// let fruits: Vec<fruit::Model> = cake
///     .secure_find_related(fruit::Entity, &scope)
///     .all(db)
///     .await?;
/// ```
pub trait SecureFindRelatedExt: ModelTrait {
    /// Find related entities with access scope applied.
    ///
    /// This creates a scoped query for entities related to this model.
    /// The scope is applied to the related entity to ensure tenant isolation.
    ///
    /// # Type Parameters
    /// - `R`: The related entity type that must implement `ScopableEntity`
    ///
    /// # Arguments
    /// - `r`: The related entity marker (e.g., `fruit::Entity`)
    /// - `scope`: The access scope to apply to the related entity query
    fn secure_find_related<R>(&self, r: R, scope: &AccessScope) -> SecureSelect<R, Scoped>
    where
        R: ScopableEntity + EntityTrait,
        R::Column: ColumnTrait + Copy,
        Self::Entity: Related<R>;
}

impl<M> SecureFindRelatedExt for M
where
    M: ModelTrait,
{
    fn secure_find_related<R>(&self, r: R, scope: &AccessScope) -> SecureSelect<R, Scoped>
    where
        R: ScopableEntity + EntityTrait,
        R::Column: ColumnTrait + Copy,
        Self::Entity: Related<R>,
    {
        // Use SeaORM's find_related to build the base query
        let select = self.find_related(r);

        // Apply scope to the related entity
        select.secure().scope_with(scope)
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
        let unscoped = Unscoped;
        assert!(std::mem::size_of_val(&unscoped) == 0); // Unscoped is zero-sized

        // Scoped now requires an AccessScope
        let scope = AccessScope::default();
        let scoped = Scoped {
            scope: Arc::new(scope),
        };
        assert!(
            !scoped
                .scope
                .has_property(modkit_security::pep_properties::OWNER_TENANT_ID)
        ); // default scope has no tenants
    }

    #[test]
    fn test_scoped_state_holds_scope() {
        let tenant_id = uuid::Uuid::new_v4();
        let scope = AccessScope::for_tenants(vec![tenant_id]);
        let scoped = Scoped {
            scope: Arc::new(scope),
        };

        // Verify the scope is accessible
        assert!(
            scoped
                .scope
                .has_property(modkit_security::pep_properties::OWNER_TENANT_ID)
        );
        assert_eq!(
            scoped
                .scope
                .all_values_for(modkit_security::pep_properties::OWNER_TENANT_ID)
                .len(),
            1
        );
        assert!(
            scoped
                .scope
                .all_values_for(modkit_security::pep_properties::OWNER_TENANT_ID)
                .contains(&tenant_id)
        );
    }

    #[test]
    fn test_scoped_state_is_cloneable() {
        let scope = AccessScope::for_tenants(vec![uuid::Uuid::new_v4()]);
        let scoped = Scoped {
            scope: Arc::new(scope),
        };

        // Cloning should share the Arc
        let cloned = scoped.clone();
        assert!(Arc::ptr_eq(&scoped.scope, &cloned.scope));
    }
}
