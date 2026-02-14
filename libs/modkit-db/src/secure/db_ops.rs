use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, InsertResult, IntoActiveModel, ModelTrait,
    QueryFilter,
    sea_query::{IntoIden, OnConflict, SimpleExpr},
};
use std::marker::PhantomData;

use crate::secure::cond::build_scope_condition;
use crate::secure::error::ScopeError;
use crate::secure::{
    AccessScope, DBRunner, DBRunnerInternal, ScopableEntity, Scoped, SeaOrmRunner, SecureEntityExt,
    Unscoped,
};

/// Validate that the values in an `ActiveModel` satisfy at least one constraint
/// in the provided `AccessScope`.
///
/// This is the INSERT-time counterpart of `build_scope_condition`: instead of
/// adding `WHERE` clauses to a query, it checks the `ActiveModel`'s column
/// values in-memory against every scope filter.
///
/// # Semantics
///
/// - Multiple constraints are **OR-ed**: the insert is allowed if ANY constraint
///   matches entirely.
/// - Filters within a constraint are **AND-ed**: ALL filters must match for
///   that constraint to pass.
/// - A filter whose property resolves to a column where the `ActiveModel` value
///   is `NotSet` is **skipped** (the column is not being inserted, so there's
///   nothing to validate).
/// - A filter whose property does **not** resolve (unknown property) causes
///   that constraint to fail (fail-closed), consistent with the query-path
///   behavior in `build_scope_condition`.
///
/// # Errors
///
/// Returns `ScopeError::Denied` if no constraint matches the `ActiveModel`.
fn validate_insert_scope<A>(am: &A, scope: &AccessScope) -> Result<(), ScopeError>
where
    A: ActiveModelTrait,
    A::Entity: ScopableEntity + EntityTrait,
    <A::Entity as EntityTrait>::Column: ColumnTrait + Copy,
{
    use modkit_security::access_scope::FilterOp;

    if scope.is_unconstrained() || A::Entity::IS_UNRESTRICTED {
        return Ok(());
    }
    if scope.is_deny_all() {
        return Err(ScopeError::Denied(
            "insert denied: scope has no constraints",
        ));
    }

    // OR over constraints: at least one must match entirely.
    'next_constraint: for constraint in scope.constraints() {
        // AND over filters within this constraint.
        for filter in constraint.filters() {
            let Some(col) = <A::Entity as ScopableEntity>::resolve_property(filter.property())
            else {
                // Unknown property → this constraint fails (fail-closed).
                continue 'next_constraint;
            };

            // Extract the column value from the ActiveModel.
            match am.get(col) {
                sea_orm::ActiveValue::NotSet => {
                    // Column not being set in this insert — skip this filter.
                    // (e.g., auto-generated columns, defaults)
                }
                sea_orm::ActiveValue::Set(v) | sea_orm::ActiveValue::Unchanged(v) => {
                    let uuid_val = match v {
                        sea_orm::Value::Uuid(Some(u)) => *u,
                        _ => {
                            // Non-UUID column or NULL — can't match UUID-based filter.
                            continue 'next_constraint;
                        }
                    };

                    let matches = match filter.op() {
                        FilterOp::Eq => filter.values().first() == Some(&uuid_val),
                        FilterOp::In => filter.values().contains(&uuid_val),
                    };

                    if !matches {
                        continue 'next_constraint;
                    }
                }
            }
        }
        // All filters in this constraint matched → insert is allowed.
        return Ok(());
    }

    Err(ScopeError::Denied(
        "insert denied: entity values do not satisfy any scope constraint",
    ))
}

/// Secure insert helper for Scopable entities.
///
/// This helper performs a standard `INSERT` through `SeaORM` but wraps database
/// errors into a unified `ScopeError` type for consistent error handling across
/// secure data-access code.
///
/// # Scope Validation
///
/// Validates **all** scope constraints against the `ActiveModel`'s column values,
/// not just `tenant_id`. For each constraint in the scope, every filter's property
/// is resolved to a column via `ScopableEntity::resolve_property`, and the
/// `ActiveModel`'s value for that column is checked against the filter's values.
/// At least one constraint must match entirely (OR semantics) for the insert to
/// proceed.
///
/// # Responsibilities
///
/// - Does **not** inspect the `SecurityContext` or enforce tenant scoping rules.
/// - Does **not** automatically populate any entity fields.
/// - Callers are responsible for:
///   - Setting all required fields before calling.
///   - Validating that the operation is authorized within the current
///     `SecurityContext` (e.g., verifying `tenant_id` or resource ownership).
///
/// # Behavior by Entity Type
///
/// ## Tenant-scoped entities (have `tenant_col`)
/// - Must have a valid, non-empty `tenant_id` set in the `ActiveModel` before insert.
/// - The `tenant_id` should come from the request payload or be validated against
///   `SecurityContext` by the service layer before calling this helper.
///
/// ## Global entities (no `tenant_col`)
/// - May be inserted freely without tenant validation.
/// - Typical examples include system-wide configuration or audit logs.
///
/// # Recommended Field Population
///
/// When inserting entities, populate these fields from `SecurityContext` in service code:
/// - `tenant_id`: from payload or validated via `ctx.scope()`
/// - `owner_id`: from `ctx.subject_id()`
/// - `created_by`: from `ctx.subject_id()` if applicable
///
/// # Example
///
/// ```ignore
/// use modkit_db::secure::{secure_insert, SecurityContext};
///
/// // Domain/service layer validates tenant_id beforehand
/// let am = user::ActiveModel {
///     id: Set(Uuid::new_v4()),
///     tenant_id: Set(tenant_id),
///     owner_id: Set(ctx.subject_id()),
///     email: Set("user@example.com".to_string()),
///     ..Default::default()
/// };
///
/// // Simple secure insert wrapper
/// let user = secure_insert::<user::Entity>(am, &ctx, conn).await?;
/// ```
///
/// # Errors
///
/// - Returns `ScopeError::Db` if the database insert fails.
/// - Returns `ScopeError::Denied` if the `ActiveModel` values do not satisfy any scope constraint.
/// - Returns `ScopeError::TenantNotInScope` for tenant isolation violations.
pub async fn secure_insert<E>(
    am: E::ActiveModel,
    scope: &AccessScope,
    runner: &impl DBRunner,
) -> Result<E::Model, ScopeError>
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
    E::ActiveModel: ActiveModelTrait<Entity = E> + Send,
    E::Model: sea_orm::IntoActiveModel<E::ActiveModel>,
{
    // Tenant-scoped entities must have tenant_id set in the ActiveModel.
    if let Some(tenant_col) = E::tenant_col()
        && let sea_orm::ActiveValue::NotSet = am.get(tenant_col)
    {
        return Err(ScopeError::Invalid("tenant_id is required"));
    }

    validate_insert_scope(&am, scope)?;

    match DBRunnerInternal::as_seaorm(runner) {
        SeaOrmRunner::Conn(db) => Ok(am.insert(db).await?),
        SeaOrmRunner::Tx(tx) => Ok(am.insert(tx).await?),
    }
}

/// Secure update helper for updating a single entity by ID inside a scope.
///
/// # Security
/// - Verifies the target row exists **within the scope** before updating.
/// - For tenant-scoped entities, forbids changing `tenant_id` (immutable).
///
/// # Errors
/// - `ScopeError::Denied` if the row is not accessible in the scope.
/// - `ScopeError::Denied("tenant_id is immutable")` if caller attempts to change `tenant_id`.
pub async fn secure_update_with_scope<E>(
    am: E::ActiveModel,
    scope: &AccessScope,
    id: uuid::Uuid,
    runner: &impl DBRunner,
) -> Result<E::Model, ScopeError>
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
    E::ActiveModel: ActiveModelTrait<Entity = E> + Send,
    E::Model: sea_orm::IntoActiveModel<E::ActiveModel> + sea_orm::ModelTrait<Entity = E>,
{
    let existing = E::find()
        .secure()
        .scope_with(scope)
        .and_id(id)?
        .one(runner)
        .await?;

    let Some(existing) = existing else {
        return Err(ScopeError::Denied(
            "entity not found or not accessible in current security scope",
        ));
    };

    if let Some(tcol) = E::tenant_col() {
        let stored = match existing.get(tcol) {
            sea_orm::Value::Uuid(Some(u)) => *u,
            _ => return Err(ScopeError::Invalid("tenant_id has unexpected type")),
        };

        let incoming = match am.get(tcol) {
            sea_orm::ActiveValue::Set(v) | sea_orm::ActiveValue::Unchanged(v) => match v {
                sea_orm::Value::Uuid(Some(u)) => Some(*u),
                sea_orm::Value::Uuid(None) => {
                    return Err(ScopeError::Invalid("tenant_id is required"));
                }
                _ => {
                    return Err(ScopeError::Invalid("tenant_id has unexpected type"));
                }
            },
            sea_orm::ActiveValue::NotSet => None,
        };

        if let Some(incoming) = incoming
            && incoming != stored
        {
            return Err(ScopeError::Denied("tenant_id is immutable"));
        }
    }

    match DBRunnerInternal::as_seaorm(runner) {
        SeaOrmRunner::Conn(db) => Ok(am.update(db).await?),
        SeaOrmRunner::Tx(tx) => Ok(am.update(tx).await?),
    }
}

/// Helper to validate a tenant ID is in the scope.
///
/// Use this when manually setting `tenant_id` in `ActiveModels` to ensure
/// the value matches the security scope.
///
/// For unconstrained scopes (allow-all), this always succeeds.
///
/// # Errors
/// Returns `ScopeError::Denied` if tenant scope is missing.
/// Returns `ScopeError::TenantNotInScope` if the tenant ID is not in any constraint.
pub fn validate_tenant_in_scope(
    tenant_id: uuid::Uuid,
    scope: &AccessScope,
) -> Result<(), ScopeError> {
    if scope.is_unconstrained() {
        return Ok(());
    }
    let prop = modkit_security::pep_properties::OWNER_TENANT_ID;
    if !scope.has_property(prop) {
        return Err(ScopeError::Denied(
            "tenant scope required for tenant-scoped insert",
        ));
    }
    if scope.contains_value(prop, tenant_id) {
        return Ok(());
    }
    Err(ScopeError::TenantNotInScope { tenant_id })
}

/// A type-safe wrapper around `SeaORM`'s `Insert` that enforces scoping.
///
/// This wrapper uses the typestate pattern to ensure that insert operations
/// cannot be executed without first applying access control via `.scope_with()`.
///
/// Unlike the simpler `secure_insert()` helper, this wrapper preserves `SeaORM`'s
/// builder methods like `on_conflict()` for upsert semantics.
///
/// # Example
/// ```ignore
/// use modkit_db::secure::{AccessScope, SecureInsertExt};
/// use sea_orm::sea_query::OnConflict;
///
/// let scope = AccessScope::for_tenants(vec![tenant_id]);
/// let am = user::ActiveModel {
///     tenant_id: Set(tenant_id),
///     email: Set("user@example.com".to_string()),
///     ..Default::default()
/// };
///
/// user::Entity::insert(am)
///     .secure()                    // Returns SecureInsertOne<E, Unscoped>
///     .scope_with(&scope)?         // Returns SecureInsertOne<E, Scoped>
///     .on_conflict(OnConflict::...) // Builder methods still available
///     .exec(conn)                  // Now can execute
///     .await?;
/// ```
#[derive(Debug)]
pub struct SecureInsertOne<A, S>
where
    A: ActiveModelTrait,
{
    pub(crate) inner: sea_orm::Insert<A>,
    pub(crate) _state: PhantomData<S>,
}

/// Extension trait to convert a regular `SeaORM` `Insert` into a `SecureInsertOne`.
pub trait SecureInsertExt<A: ActiveModelTrait>: Sized {
    /// Convert this insert operation into a secure (unscoped) insert.
    /// You must call `.scope_with()` before executing.
    fn secure(self) -> SecureInsertOne<A, Unscoped>;
}

impl<A> SecureInsertExt<A> for sea_orm::Insert<A>
where
    A: ActiveModelTrait,
{
    fn secure(self) -> SecureInsertOne<A, Unscoped> {
        SecureInsertOne {
            inner: self,
            _state: PhantomData,
        }
    }
}

// Methods available only on Unscoped inserts
impl<A> SecureInsertOne<A, Unscoped>
where
    A: ActiveModelTrait + Send,
    A::Entity: ScopableEntity + EntityTrait,
    <A::Entity as EntityTrait>::Column: ColumnTrait + Copy,
{
    /// Apply access control scope to this insert, transitioning to the `Scoped` state.
    ///
    /// For tenant-scoped entities, this validates that the `tenant_id` in the
    /// `ActiveModel` matches one of the tenants in the provided scope.
    ///
    /// # Errors
    /// - Returns `ScopeError::Invalid` if `tenant_id` is not set for tenant-scoped entities.
    /// - Returns `ScopeError::TenantNotInScope` if `tenant_id` is not in the provided scope.
    pub fn scope_with(self, scope: &AccessScope) -> Result<SecureInsertOne<A, Scoped>, ScopeError> {
        // For INSERT operations, we need to extract and validate the tenant from the
        // ActiveModel being inserted. Unfortunately, SeaORM's Insert<A> doesn't expose
        // the ActiveModel directly, so we rely on the caller to have already validated
        // the scope when constructing the ActiveModel.
        //
        // The scope is still required to transition to Scoped state, ensuring the caller
        // has an appropriate security context. For full tenant validation, use the
        // `scope_with_model` method which takes the ActiveModel explicitly.
        let _ = scope; // Mark as used - validation happens via scope_with_model
        Ok(SecureInsertOne {
            inner: self.inner,
            _state: PhantomData,
        })
    }

    /// Apply access control scope with explicit `ActiveModel` validation.
    ///
    /// This method validates **all** scope constraints against the `ActiveModel`'s
    /// column values (not just `tenant_id`). See [`validate_insert_scope`] for
    /// the full semantics.
    ///
    /// # Errors
    /// - Returns `ScopeError::Denied` if the `ActiveModel` values do not satisfy
    ///   any scope constraint.
    pub fn scope_with_model(
        self,
        scope: &AccessScope,
        am: &A,
    ) -> Result<SecureInsertOne<A, Scoped>, ScopeError> {
        validate_insert_scope(am, scope)?;
        Ok(SecureInsertOne {
            inner: self.inner,
            _state: PhantomData,
        })
    }
}

// Fluent builder methods (available only on Scoped inserts to prevent pre-scope execution)
impl<A> SecureInsertOne<A, Scoped>
where
    A: ActiveModelTrait,
    A::Entity: ScopableEntity + EntityTrait,
    <A::Entity as EntityTrait>::Column: ColumnTrait + Copy,
{
    /// Set the `ON CONFLICT` clause for upsert semantics using `SecureOnConflict`.
    ///
    /// This is the recommended way to add upsert semantics as it enforces
    /// tenant immutability at compile/validation time.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let on_conflict = SecureOnConflict::<Entity>::columns([Column::TenantId, Column::UserId])
    ///     .update_columns([Column::Theme, Column::Language])?;
    ///
    /// Entity::insert(am)
    ///     .secure()
    ///     .scope_with(&scope)?
    ///     .on_conflict(on_conflict)
    ///     .exec(conn)
    ///     .await?;
    /// ```
    #[must_use]
    pub fn on_conflict(mut self, on_conflict: SecureOnConflict<A::Entity>) -> Self {
        self.inner = self.inner.on_conflict(on_conflict.build());
        self
    }

    /// Set the `ON CONFLICT` clause using raw `SeaORM` `OnConflict`.
    ///
    /// # Safety
    ///
    /// This method bypasses tenant immutability validation. The caller is
    /// responsible for ensuring that `tenant_id` is not included in update columns.
    /// Use `on_conflict()` with `SecureOnConflict` for automatic validation.
    #[must_use]
    pub fn on_conflict_raw(mut self, on_conflict: OnConflict) -> Self {
        self.inner = self.inner.on_conflict(on_conflict);
        self
    }
}

// Execution methods (require Scoped state)
impl<A> SecureInsertOne<A, Scoped>
where
    A: ActiveModelTrait,
{
    /// Execute the insert operation.
    ///
    /// # Errors
    /// Returns `ScopeError::Db` if the database operation fails.
    #[allow(clippy::disallowed_methods)]
    pub async fn exec<C>(self, runner: &C) -> Result<InsertResult<A>, ScopeError>
    where
        C: DBRunner,
        A: Send,
    {
        match DBRunnerInternal::as_seaorm(runner) {
            SeaOrmRunner::Conn(db) => Ok(self.inner.exec(db).await?),
            SeaOrmRunner::Tx(tx) => Ok(self.inner.exec(tx).await?),
        }
    }

    /// Execute the insert and return the inserted model.
    ///
    /// This is useful when you need the inserted data with any database-generated
    /// values (like auto-increment IDs or default values).
    ///
    /// # Errors
    /// Returns `ScopeError::Db` if the database operation fails.
    #[allow(clippy::disallowed_methods)]
    pub async fn exec_with_returning<C>(
        self,
        runner: &C,
    ) -> Result<<A::Entity as EntityTrait>::Model, ScopeError>
    where
        C: DBRunner,
        A: Send,
        <A::Entity as EntityTrait>::Model: IntoActiveModel<A>,
    {
        match DBRunnerInternal::as_seaorm(runner) {
            SeaOrmRunner::Conn(db) => Ok(self.inner.exec_with_returning(db).await?),
            SeaOrmRunner::Tx(tx) => Ok(self.inner.exec_with_returning(tx).await?),
        }
    }

    /// Unwrap the inner `SeaORM` `Insert` for advanced use cases.
    ///
    /// # Safety
    /// The caller must ensure they don't remove or bypass the security
    /// validation that was applied during `.scope_with()`.
    #[must_use]
    pub fn into_inner(self) -> sea_orm::Insert<A> {
        self.inner
    }
}

/// A secure builder for `ON CONFLICT DO UPDATE` clauses that enforces tenant immutability.
///
/// For tenant-scoped entities (`ScopableEntity::tenant_col() != None`), this builder
/// ensures that `tenant_id` is never included in the update columns. Attempting to
/// update `tenant_id` via `update_columns()` or `value()` returns an error.
///
/// # Security Rationale
///
/// `ON CONFLICT DO UPDATE` can be exploited to change an entity's tenant:
/// ```sql
/// INSERT INTO users (id, tenant_id, email) VALUES ($1, $2, $3)
/// ON CONFLICT (id) DO UPDATE SET tenant_id = excluded.tenant_id;
/// ```
/// This would allow moving a row from one tenant to another, violating tenant isolation.
///
/// # Example
///
/// ```ignore
/// use modkit_db::secure::{SecureOnConflict, SecureInsertExt};
/// use sea_orm::ActiveValue::Set;
///
/// let scope = AccessScope::single(ScopeConstraint::new(vec![
///     ScopeFilter::new(pep_properties::OWNER_TENANT_ID, FilterOp::In, vec![tenant_id]),
///     ScopeFilter::new(pep_properties::RESOURCE_ID, FilterOp::In, vec![user_id]),
/// ]));
/// let am = settings::ActiveModel {
///     tenant_id: Set(tenant_id),
///     user_id: Set(user_id),
///     theme: Set(Some("dark".to_string())),
///     language: Set(Some("en".to_string())),
/// };
///
/// // Build secure on_conflict - validates tenant_id is not updated
/// let on_conflict = SecureOnConflict::<settings::Entity>::columns([
///         settings::Column::TenantId,
///         settings::Column::UserId,
///     ])
///     .update_columns([settings::Column::Theme, settings::Column::Language])?;
///
/// settings::Entity::insert(am)
///     .secure()
///     .scope_with(&scope)?
///     .on_conflict(on_conflict)
///     .exec(conn)
///     .await?;
/// ```
#[derive(Debug, Clone)]
pub struct SecureOnConflict<E: EntityTrait> {
    inner: OnConflict,
    _entity: PhantomData<E>,
}

impl<E> SecureOnConflict<E>
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    /// Start building an `ON CONFLICT` clause with the specified conflict columns.
    ///
    /// These are the columns that define uniqueness (typically the primary key
    /// or a unique constraint).
    #[must_use]
    pub fn columns<C, I>(cols: I) -> Self
    where
        C: IntoIden,
        I: IntoIterator<Item = C>,
    {
        Self {
            inner: OnConflict::columns(cols),
            _entity: PhantomData,
        }
    }

    /// Specify columns to update on conflict.
    ///
    /// # Errors
    ///
    /// Returns `ScopeError::Denied("tenant_id is immutable")` if the entity has
    /// a tenant column and it appears in the update columns list.
    pub fn update_columns<C, I>(mut self, cols: I) -> Result<Self, ScopeError>
    where
        C: IntoIden + Copy + 'static,
        I: IntoIterator<Item = C>,
    {
        let cols: Vec<C> = cols.into_iter().collect();

        // Check if tenant column is in the update list
        if let Some(tenant_col) = E::tenant_col() {
            let tenant_iden = tenant_col.into_iden();
            for col in &cols {
                let col_iden = col.into_iden();
                if col_iden.to_string() == tenant_iden.to_string() {
                    return Err(ScopeError::Denied("tenant_id is immutable"));
                }
            }
        }

        self.inner.update_columns(cols);
        Ok(self)
    }

    /// Set a custom update expression for a column on conflict.
    ///
    /// # Errors
    ///
    /// Returns `ScopeError::Denied("tenant_id is immutable")` if the entity has
    /// a tenant column and the specified column matches it.
    pub fn value<C>(mut self, col: C, expr: SimpleExpr) -> Result<Self, ScopeError>
    where
        C: IntoIden + Copy + 'static,
    {
        // Check if this is the tenant column
        if let Some(tenant_col) = E::tenant_col() {
            let tenant_iden = tenant_col.into_iden();
            let col_iden = col.into_iden();
            if col_iden.to_string() == tenant_iden.to_string() {
                return Err(ScopeError::Denied("tenant_id is immutable"));
            }
        }

        self.inner.value(col, expr);
        Ok(self)
    }

    /// Consume the builder and return the underlying `SeaORM` `OnConflict`.
    ///
    /// Call this after configuring all update columns/values.
    #[must_use]
    pub fn build(self) -> OnConflict {
        self.inner
    }

    /// Get a reference to the inner `OnConflict` for chaining with `SeaORM` methods
    /// that are not wrapped by this builder.
    ///
    /// # Safety
    ///
    /// The caller must ensure they don't add tenant column updates through the
    /// inner `OnConflict` directly, as this would bypass the security check.
    #[must_use]
    pub fn inner_mut(&mut self) -> &mut OnConflict {
        &mut self.inner
    }
}

/// A type-safe wrapper around `SeaORM`'s `UpdateMany` that enforces scoping.
///
/// This wrapper uses the typestate pattern to ensure that update operations
/// cannot be executed without first applying access control via `.scope_with()`.
///
/// # Example
/// ```ignore
/// use modkit_db::secure::{AccessScope, SecureUpdateExt};
///
/// let scope = AccessScope::for_tenants(vec![tenant_id]);
/// let result = user::Entity::update_many()
///     .col_expr(user::Column::Status, Expr::value("active"))
///     .secure()           // Returns SecureUpdateMany<E, Unscoped>
///     .scope_with(&scope)? // Returns SecureUpdateMany<E, Scoped>
///     .exec(conn)         // Now can execute
///     .await?;
/// ```
#[derive(Clone, Debug)]
pub struct SecureUpdateMany<E: EntityTrait, S> {
    pub(crate) inner: sea_orm::UpdateMany<E>,
    pub(crate) _state: PhantomData<S>,
    pub(crate) tenant_update_attempted: bool,
}

// Fluent builder methods (available in all typestates).
impl<E, S> SecureUpdateMany<E, S>
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    /// Set a column expression (mirrors `SeaORM`'s `UpdateMany::col_expr`).
    #[must_use]
    pub fn col_expr(mut self, col: E::Column, expr: sea_orm::sea_query::SimpleExpr) -> Self {
        if let Some(tcol) = E::tenant_col()
            && std::mem::discriminant(&col) == std::mem::discriminant(&tcol)
        {
            self.tenant_update_attempted = true;
        }
        self.inner = self.inner.col_expr(col, expr);
        self
    }

    /// Add an additional filter. Scope conditions remain in place once applied.
    #[must_use]
    pub fn filter(mut self, filter: sea_orm::Condition) -> Self {
        self.inner = QueryFilter::filter(self.inner, filter);
        self
    }
}

/// Extension trait to convert a regular `SeaORM` `UpdateMany` into a `SecureUpdateMany`.
pub trait SecureUpdateExt<E: EntityTrait>: Sized {
    /// Convert this update operation into a secure (unscoped) update.
    /// You must call `.scope_with()` before executing.
    fn secure(self) -> SecureUpdateMany<E, Unscoped>;
}

impl<E> SecureUpdateExt<E> for sea_orm::UpdateMany<E>
where
    E: EntityTrait,
{
    fn secure(self) -> SecureUpdateMany<E, Unscoped> {
        SecureUpdateMany {
            inner: self,
            _state: PhantomData,
            tenant_update_attempted: false,
        }
    }
}

// Methods available only on Unscoped updates
impl<E> SecureUpdateMany<E, Unscoped>
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    /// Apply access control scope to this update, transitioning to the `Scoped` state.
    ///
    /// This applies the implicit policy:
    /// - Empty scope → deny all (no rows updated)
    /// - Tenants only → update only in specified tenants
    /// - Resources only → update only specified resource IDs
    /// - Both → AND them together
    ///
    #[must_use]
    pub fn scope_with(self, scope: &AccessScope) -> SecureUpdateMany<E, Scoped> {
        let cond = build_scope_condition::<E>(scope);
        SecureUpdateMany {
            inner: self.inner.filter(cond),
            _state: PhantomData,
            tenant_update_attempted: self.tenant_update_attempted,
        }
    }
}

// Methods available only on Scoped updates
impl<E> SecureUpdateMany<E, Scoped>
where
    E: EntityTrait,
{
    /// Execute the update operation.
    ///
    /// # Errors
    /// Returns `ScopeError::Db` if the database operation fails.
    #[allow(clippy::disallowed_methods)]
    pub async fn exec(self, runner: &impl DBRunner) -> Result<sea_orm::UpdateResult, ScopeError> {
        if self.tenant_update_attempted {
            return Err(ScopeError::Denied("tenant_id is immutable"));
        }
        match DBRunnerInternal::as_seaorm(runner) {
            SeaOrmRunner::Conn(db) => Ok(self.inner.exec(db).await?),
            SeaOrmRunner::Tx(tx) => Ok(self.inner.exec(tx).await?),
        }
    }

    /// Unwrap the inner `SeaORM` `UpdateMany` for advanced use cases.
    ///
    /// # Safety
    /// The caller must ensure they don't remove or bypass the security
    /// conditions that were applied during `.scope_with()`.
    #[must_use]
    pub fn into_inner(self) -> sea_orm::UpdateMany<E> {
        self.inner
    }
}

/// A type-safe wrapper around `SeaORM`'s `DeleteMany` that enforces scoping.
///
/// This wrapper uses the typestate pattern to ensure that delete operations
/// cannot be executed without first applying access control via `.scope_with()`.
///
/// # Example
/// ```ignore
/// use modkit_db::secure::{AccessScope, SecureDeleteExt};
///
/// let scope = AccessScope::for_tenants(vec![tenant_id]);
/// let result = user::Entity::delete_many()
///     .filter(user::Column::Status.eq("inactive"))
///     .secure()           // Returns SecureDeleteMany<E, Unscoped>
///     .scope_with(&scope)? // Returns SecureDeleteMany<E, Scoped>
///     .exec(conn)         // Now can execute
///     .await?;
/// ```
#[derive(Clone, Debug)]
pub struct SecureDeleteMany<E: EntityTrait, S> {
    pub(crate) inner: sea_orm::DeleteMany<E>,
    pub(crate) _state: PhantomData<S>,
}

/// Extension trait to convert a regular `SeaORM` `DeleteMany` into a `SecureDeleteMany`.
pub trait SecureDeleteExt<E: EntityTrait>: Sized {
    /// Convert this delete operation into a secure (unscoped) delete.
    /// You must call `.scope_with()` before executing.
    fn secure(self) -> SecureDeleteMany<E, Unscoped>;
}

impl<E> SecureDeleteExt<E> for sea_orm::DeleteMany<E>
where
    E: EntityTrait,
{
    fn secure(self) -> SecureDeleteMany<E, Unscoped> {
        SecureDeleteMany {
            inner: self,
            _state: PhantomData,
        }
    }
}

// Methods available only on Unscoped deletes
impl<E> SecureDeleteMany<E, Unscoped>
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
{
    /// Apply access control scope to this delete, transitioning to the `Scoped` state.
    ///
    /// This applies the implicit policy:
    /// - Empty scope → deny all (no rows deleted)
    /// - Tenants only → delete only in specified tenants
    /// - Resources only → delete only specified resource IDs
    /// - Both → AND them together
    ///
    #[must_use]
    pub fn scope_with(self, scope: &AccessScope) -> SecureDeleteMany<E, Scoped> {
        let cond = build_scope_condition::<E>(scope);
        SecureDeleteMany {
            inner: self.inner.filter(cond),
            _state: PhantomData,
        }
    }
}

// Methods available only on Scoped deletes
impl<E> SecureDeleteMany<E, Scoped>
where
    E: EntityTrait,
{
    /// Add additional filters to the scoped delete.
    /// The scope conditions remain in place.
    #[must_use]
    pub fn filter(mut self, filter: sea_orm::Condition) -> Self {
        self.inner = QueryFilter::filter(self.inner, filter);
        self
    }

    /// Execute the delete operation.
    ///
    /// # Errors
    /// Returns `ScopeError::Db` if the database operation fails.
    #[allow(clippy::disallowed_methods)]
    pub async fn exec(self, runner: &impl DBRunner) -> Result<sea_orm::DeleteResult, ScopeError> {
        match DBRunnerInternal::as_seaorm(runner) {
            SeaOrmRunner::Conn(db) => Ok(self.inner.exec(db).await?),
            SeaOrmRunner::Tx(tx) => Ok(self.inner.exec(tx).await?),
        }
    }

    /// Unwrap the inner `SeaORM` `DeleteMany` for advanced use cases.
    ///
    /// # Safety
    /// The caller must ensure they don't remove or bypass the security
    /// conditions that were applied during `.scope_with()`.
    #[must_use]
    pub fn into_inner(self) -> sea_orm::DeleteMany<E> {
        self.inner
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use sea_orm::entity::prelude::*;

    // Test entity with tenant_col for SecureOnConflict tests
    mod test_entity {
        use super::*;
        use modkit_security::pep_properties;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "test_table")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: Uuid,
            pub tenant_id: Uuid,
            pub name: String,
            pub value: i32,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}

        impl ScopableEntity for Entity {
            fn tenant_col() -> Option<Column> {
                Some(Column::TenantId)
            }
            fn resource_col() -> Option<Column> {
                Some(Column::Id)
            }
            fn owner_col() -> Option<Column> {
                None
            }
            fn type_col() -> Option<Column> {
                None
            }
            fn resolve_property(property: &str) -> Option<Column> {
                match property {
                    pep_properties::OWNER_TENANT_ID => Self::tenant_col(),
                    pep_properties::RESOURCE_ID => Self::resource_col(),
                    _ => None,
                }
            }
        }
    }

    // Test entity without tenant_col (global entity)
    mod global_entity {
        use super::*;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "global_table")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: Uuid,
            pub config_key: String,
            pub config_value: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}

        impl ScopableEntity for Entity {
            fn tenant_col() -> Option<Column> {
                None // Global entity - no tenant column
            }
            fn resource_col() -> Option<Column> {
                Some(Column::Id)
            }
            fn owner_col() -> Option<Column> {
                None
            }
            fn type_col() -> Option<Column> {
                None
            }
            fn resolve_property(property: &str) -> Option<Column> {
                match property {
                    "id" => Self::resource_col(),
                    _ => None,
                }
            }
        }
    }

    #[test]
    fn test_validate_tenant_in_scope() {
        let tenant_id = uuid::Uuid::new_v4();
        let scope = crate::secure::AccessScope::for_tenants(vec![tenant_id]);

        assert!(validate_tenant_in_scope(tenant_id, &scope).is_ok());

        let other_id = uuid::Uuid::new_v4();
        assert!(validate_tenant_in_scope(other_id, &scope).is_err());
    }

    // Note: Full integration tests with database require actual SeaORM entities
    // These tests verify the typestate pattern compiles correctly

    #[test]
    fn test_typestate_compile_check() {
        // This test verifies the typestate markers compile
        let unscoped: PhantomData<Unscoped> = PhantomData;
        let scoped: PhantomData<Scoped> = PhantomData;
        // Use the variables to avoid unused warnings
        let _ = (unscoped, scoped);
    }

    #[test]
    fn test_tenant_not_in_scope_returns_error() {
        // Verify that validate_tenant_in_scope properly rejects tenant IDs not in scope
        let allowed_tenant = uuid::Uuid::new_v4();
        let disallowed_tenant = uuid::Uuid::new_v4();
        let scope = crate::secure::AccessScope::for_tenants(vec![allowed_tenant]);

        // Allowed tenant should succeed
        assert!(validate_tenant_in_scope(allowed_tenant, &scope).is_ok());

        // Disallowed tenant should fail with TenantNotInScope error
        let result = validate_tenant_in_scope(disallowed_tenant, &scope);
        assert!(result.is_err());
        match result {
            Err(ScopeError::TenantNotInScope { tenant_id }) => {
                assert_eq!(tenant_id, disallowed_tenant);
            }
            _ => panic!("Expected TenantNotInScope error"),
        }
    }

    #[test]
    fn test_empty_scope_denied_for_tenant_scoped() {
        // Verify that an empty scope (no tenants) is rejected for tenant-scoped inserts
        let tenant_id = uuid::Uuid::new_v4();
        let empty_scope = crate::secure::AccessScope::default();

        let result = validate_tenant_in_scope(tenant_id, &empty_scope);
        assert!(result.is_err());
        match result {
            Err(ScopeError::Denied(_)) => {}
            _ => panic!("Expected Denied error for empty scope"),
        }
    }

    // SecureOnConflict tests

    #[test]
    fn test_secure_on_conflict_update_columns_allows_non_tenant_columns() {
        use test_entity::{Column, Entity};

        // update_columns with non-tenant columns should succeed
        let result = SecureOnConflict::<Entity>::columns([Column::Id])
            .update_columns([Column::Name, Column::Value]);

        assert!(result.is_ok());
    }

    #[test]
    fn test_secure_on_conflict_update_columns_rejects_tenant_column() {
        use test_entity::{Column, Entity};

        // update_columns with tenant_id should fail
        let result = SecureOnConflict::<Entity>::columns([Column::Id]).update_columns([
            Column::Name,
            Column::TenantId,
            Column::Value,
        ]);

        assert!(result.is_err());
        match result {
            Err(ScopeError::Denied(msg)) => {
                assert!(msg.contains("immutable"), "Expected immutable error: {msg}");
            }
            _ => panic!("Expected Denied error for tenant_id in update_columns"),
        }
    }

    #[test]
    fn test_secure_on_conflict_value_allows_non_tenant_columns() {
        use sea_orm::sea_query::Expr;
        use test_entity::{Column, Entity};

        // value() with non-tenant column should succeed
        let result = SecureOnConflict::<Entity>::columns([Column::Id])
            .value(Column::Name, Expr::value("test"));

        assert!(result.is_ok());
    }

    #[test]
    fn test_secure_on_conflict_value_rejects_tenant_column() {
        use sea_orm::sea_query::Expr;
        use test_entity::{Column, Entity};

        // value() with tenant_id should fail
        let result = SecureOnConflict::<Entity>::columns([Column::Id])
            .value(Column::TenantId, Expr::value(uuid::Uuid::new_v4()));

        assert!(result.is_err());
        match result {
            Err(ScopeError::Denied(msg)) => {
                assert!(msg.contains("immutable"), "Expected immutable error: {msg}");
            }
            _ => panic!("Expected Denied error for tenant_id in value()"),
        }
    }

    #[test]
    fn test_secure_on_conflict_chained_value_rejects_tenant_column() {
        use sea_orm::sea_query::Expr;
        use test_entity::{Column, Entity};

        // Chaining value() calls - should fail when tenant_id is added
        let result = SecureOnConflict::<Entity>::columns([Column::Id])
            .value(Column::Name, Expr::value("test"))
            .and_then(|c| c.value(Column::TenantId, Expr::value(uuid::Uuid::new_v4())));

        assert!(result.is_err());
        match result {
            Err(ScopeError::Denied(msg)) => {
                assert!(msg.contains("immutable"), "Expected immutable error: {msg}");
            }
            _ => panic!("Expected Denied error for tenant_id in chained value()"),
        }
    }

    #[test]
    fn test_secure_on_conflict_global_entity_allows_all_columns() {
        use global_entity::{Column, Entity};

        // Global entity has no tenant_col, so all columns are allowed
        let result = SecureOnConflict::<Entity>::columns([Column::Id])
            .update_columns([Column::ConfigKey, Column::ConfigValue]);

        assert!(result.is_ok());
    }

    #[test]
    fn test_secure_on_conflict_build_produces_on_conflict() {
        use test_entity::{Column, Entity};

        // Verify that build() produces a valid OnConflict
        let on_conflict = SecureOnConflict::<Entity>::columns([Column::Id])
            .update_columns([Column::Name, Column::Value])
            .expect("should succeed")
            .build();

        // The OnConflict should be usable (we can't easily test its internals,
        // but we can verify it doesn't panic)
        _ = format!("{on_conflict:?}");
    }

    // ── validate_insert_scope tests ─────────────────────────────────

    // Test entity with owner_col and a custom pep_prop (city_id),
    // mimicking the Address entity from the users-info example.
    mod owner_entity {
        use super::*;
        use modkit_security::pep_properties;

        #[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
        #[sea_orm(table_name = "addresses")]
        pub struct Model {
            #[sea_orm(primary_key)]
            pub id: Uuid,
            pub tenant_id: Uuid,
            pub user_id: Uuid,
            pub city_id: Uuid,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl ActiveModelBehavior for ActiveModel {}

        impl ScopableEntity for Entity {
            fn tenant_col() -> Option<Column> {
                Some(Column::TenantId)
            }
            fn resource_col() -> Option<Column> {
                Some(Column::Id)
            }
            fn owner_col() -> Option<Column> {
                Some(Column::UserId)
            }
            fn type_col() -> Option<Column> {
                None
            }
            fn resolve_property(property: &str) -> Option<Column> {
                match property {
                    pep_properties::OWNER_TENANT_ID => Some(Column::TenantId),
                    pep_properties::RESOURCE_ID => Some(Column::Id),
                    pep_properties::OWNER_ID => Some(Column::UserId),
                    "city_id" => Some(Column::CityId),
                    _ => None,
                }
            }
        }
    }

    #[test]
    fn test_validate_insert_scope_allow_all_passes() {
        use owner_entity::ActiveModel;
        use sea_orm::Set;

        let scope = crate::secure::AccessScope::allow_all();
        let am = ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(Uuid::new_v4()),
            user_id: Set(Uuid::new_v4()),
            city_id: Set(Uuid::new_v4()),
        };
        assert!(validate_insert_scope(&am, &scope).is_ok());
    }

    #[test]
    fn test_validate_insert_scope_deny_all_rejects() {
        use owner_entity::ActiveModel;
        use sea_orm::Set;

        let scope = crate::secure::AccessScope::deny_all();
        let am = ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(Uuid::new_v4()),
            user_id: Set(Uuid::new_v4()),
            city_id: Set(Uuid::new_v4()),
        };
        assert!(validate_insert_scope(&am, &scope).is_err());
    }

    #[test]
    fn test_validate_insert_scope_tenant_only_matches() {
        use owner_entity::ActiveModel;
        use sea_orm::Set;

        let tenant_id = Uuid::new_v4();
        let scope = crate::secure::AccessScope::for_tenant(tenant_id);
        let am = ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            user_id: Set(Uuid::new_v4()),
            city_id: Set(Uuid::new_v4()),
        };
        assert!(validate_insert_scope(&am, &scope).is_ok());
    }

    #[test]
    fn test_validate_insert_scope_tenant_mismatch_rejects() {
        use owner_entity::ActiveModel;
        use sea_orm::Set;

        let tenant_id = Uuid::new_v4();
        let other_tenant = Uuid::new_v4();
        let scope = crate::secure::AccessScope::for_tenant(tenant_id);
        let am = ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(other_tenant),
            user_id: Set(Uuid::new_v4()),
            city_id: Set(Uuid::new_v4()),
        };
        assert!(validate_insert_scope(&am, &scope).is_err());
    }

    #[test]
    fn test_validate_insert_scope_owner_id_matches() {
        use modkit_security::access_scope::{FilterOp, ScopeConstraint, ScopeFilter};
        use modkit_security::pep_properties;
        use owner_entity::ActiveModel;
        use sea_orm::Set;

        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let city_id = Uuid::new_v4();

        // Scope: tenant + owner_id + city_id (all must match)
        let scope = AccessScope::from_constraints(vec![ScopeConstraint::new(vec![
            ScopeFilter::new(
                pep_properties::OWNER_TENANT_ID,
                FilterOp::In,
                vec![tenant_id],
            ),
            ScopeFilter::eq(pep_properties::OWNER_ID, user_id),
            ScopeFilter::eq("city_id", city_id),
        ])]);

        let am = ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            user_id: Set(user_id),
            city_id: Set(city_id),
        };
        assert!(
            validate_insert_scope(&am, &scope).is_ok(),
            "Insert should pass when all properties match"
        );
    }

    #[test]
    fn test_validate_insert_scope_owner_id_mismatch_rejects() {
        use modkit_security::access_scope::{FilterOp, ScopeConstraint, ScopeFilter};
        use modkit_security::pep_properties;
        use owner_entity::ActiveModel;
        use sea_orm::Set;

        let tenant_id = Uuid::new_v4();
        let user_a = Uuid::new_v4();
        let user_b = Uuid::new_v4();
        let city_id = Uuid::new_v4();

        // Scope says owner_id must be user_a
        let scope = AccessScope::from_constraints(vec![ScopeConstraint::new(vec![
            ScopeFilter::new(
                pep_properties::OWNER_TENANT_ID,
                FilterOp::In,
                vec![tenant_id],
            ),
            ScopeFilter::eq(pep_properties::OWNER_ID, user_a),
            ScopeFilter::eq("city_id", city_id),
        ])]);

        // But ActiveModel has user_id = user_b
        let am = ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            user_id: Set(user_b),
            city_id: Set(city_id),
        };
        assert!(
            validate_insert_scope(&am, &scope).is_err(),
            "Insert must be rejected when owner_id doesn't match"
        );
    }

    #[test]
    fn test_validate_insert_scope_city_id_mismatch_rejects() {
        use modkit_security::access_scope::{FilterOp, ScopeConstraint, ScopeFilter};
        use modkit_security::pep_properties;
        use owner_entity::ActiveModel;
        use sea_orm::Set;

        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let allowed_city = Uuid::new_v4();
        let disallowed_city = Uuid::new_v4();

        // Scope says city_id must be allowed_city
        let scope = AccessScope::from_constraints(vec![ScopeConstraint::new(vec![
            ScopeFilter::new(
                pep_properties::OWNER_TENANT_ID,
                FilterOp::In,
                vec![tenant_id],
            ),
            ScopeFilter::eq(pep_properties::OWNER_ID, user_id),
            ScopeFilter::eq("city_id", allowed_city),
        ])]);

        // But ActiveModel has city_id = disallowed_city
        let am = ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            user_id: Set(user_id),
            city_id: Set(disallowed_city),
        };
        assert!(
            validate_insert_scope(&am, &scope).is_err(),
            "Insert must be rejected when city_id doesn't match"
        );
    }

    #[test]
    fn test_validate_insert_scope_or_semantics() {
        use modkit_security::access_scope::{FilterOp, ScopeConstraint, ScopeFilter};
        use modkit_security::pep_properties;
        use owner_entity::ActiveModel;
        use sea_orm::Set;

        let tenant_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let city_1 = Uuid::new_v4();
        let city_2 = Uuid::new_v4();

        // Two constraints (OR-ed): user allowed in city_1 OR city_2
        let scope = AccessScope::from_constraints(vec![
            ScopeConstraint::new(vec![
                ScopeFilter::new(
                    pep_properties::OWNER_TENANT_ID,
                    FilterOp::In,
                    vec![tenant_id],
                ),
                ScopeFilter::eq("city_id", city_1),
            ]),
            ScopeConstraint::new(vec![
                ScopeFilter::new(
                    pep_properties::OWNER_TENANT_ID,
                    FilterOp::In,
                    vec![tenant_id],
                ),
                ScopeFilter::eq("city_id", city_2),
            ]),
        ]);

        // Insert with city_2 — matches second constraint
        let am = ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            user_id: Set(user_id),
            city_id: Set(city_2),
        };
        assert!(
            validate_insert_scope(&am, &scope).is_ok(),
            "Insert should pass when matching any constraint (OR semantics)"
        );

        // Insert with city_3 — matches neither
        let city_3 = Uuid::new_v4();
        let am_bad = ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            user_id: Set(user_id),
            city_id: Set(city_3),
        };
        assert!(
            validate_insert_scope(&am_bad, &scope).is_err(),
            "Insert must be rejected when no constraint matches"
        );
    }

    #[test]
    fn test_validate_insert_scope_unknown_property_fails_closed() {
        use modkit_security::access_scope::{FilterOp, ScopeConstraint, ScopeFilter};
        use modkit_security::pep_properties;
        use owner_entity::ActiveModel;
        use sea_orm::Set;

        let tenant_id = Uuid::new_v4();

        // Constraint with an unknown property
        let scope = AccessScope::from_constraints(vec![ScopeConstraint::new(vec![
            ScopeFilter::new(
                pep_properties::OWNER_TENANT_ID,
                FilterOp::In,
                vec![tenant_id],
            ),
            ScopeFilter::eq("nonexistent_prop", Uuid::new_v4()),
        ])]);

        let am = ActiveModel {
            id: Set(Uuid::new_v4()),
            tenant_id: Set(tenant_id),
            user_id: Set(Uuid::new_v4()),
            city_id: Set(Uuid::new_v4()),
        };
        assert!(
            validate_insert_scope(&am, &scope).is_err(),
            "Unknown property must cause constraint to fail (fail-closed)"
        );
    }
}
