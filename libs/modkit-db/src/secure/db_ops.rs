use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, QueryFilter};
use std::marker::PhantomData;

use crate::secure::cond::build_scope_condition;
use crate::secure::error::ScopeError;
use crate::secure::{
    AccessScope, DBRunner, DBRunnerInternal, ScopableEntity, Scoped, SeaOrmRunner, SecureEntityExt,
    Unscoped,
};

fn extract_tenant_id<E>(am: &E::ActiveModel) -> Result<Option<uuid::Uuid>, ScopeError>
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
    E::ActiveModel: ActiveModelTrait<Entity = E>,
{
    let Some(tcol) = E::tenant_col() else {
        // Unrestricted/global table: no tenant dimension.
        return Ok(None);
    };

    match am.get(tcol) {
        sea_orm::ActiveValue::NotSet => Err(ScopeError::Invalid("tenant_id is required")),
        sea_orm::ActiveValue::Set(v) | sea_orm::ActiveValue::Unchanged(v) => match v {
            sea_orm::Value::Uuid(Some(u)) => Ok(Some(*u)),
            sea_orm::Value::Uuid(None) => Err(ScopeError::Invalid("tenant_id is required")),
            _ => Err(ScopeError::Invalid("tenant_id has unexpected type")),
        },
    }
}

fn extract_tenant_id_if_present<E>(am: &E::ActiveModel) -> Result<Option<uuid::Uuid>, ScopeError>
where
    E: ScopableEntity + EntityTrait,
    E::Column: ColumnTrait + Copy,
    E::ActiveModel: ActiveModelTrait<Entity = E>,
{
    let Some(tcol) = E::tenant_col() else {
        return Ok(None);
    };

    match am.get(tcol) {
        sea_orm::ActiveValue::NotSet => Ok(None),
        sea_orm::ActiveValue::Set(v) | sea_orm::ActiveValue::Unchanged(v) => match v {
            sea_orm::Value::Uuid(Some(u)) => Ok(Some(*u)),
            sea_orm::Value::Uuid(None) => Err(ScopeError::Invalid("tenant_id is required")),
            _ => Err(ScopeError::Invalid("tenant_id has unexpected type")),
        },
    }
}

/// Secure insert helper for Scopable entities.
///
/// This helper performs a standard `INSERT` through `SeaORM` but wraps database
/// errors into a unified `ScopeError` type for consistent error handling across
/// secure data-access code. For tenant-scoped entities it enforces tenant isolation
/// by validating the `ActiveModel`'s `tenant_id` against the provided `AccessScope`.
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
/// - Returns `ScopeError::Denied` / `ScopeError::TenantNotInScope` for tenant isolation violations.
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
    if let Some(tenant_id) = extract_tenant_id::<E>(&am)? {
        validate_tenant_in_scope(tenant_id, scope)?;
    }

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

        if let Some(incoming) = extract_tenant_id_if_present::<E>(&am)?
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
/// # Errors
/// Returns `ScopeError::Invalid` if the tenant ID is not in the scope.
pub fn validate_tenant_in_scope(
    tenant_id: uuid::Uuid,
    scope: &AccessScope,
) -> Result<(), ScopeError> {
    if !scope.has_tenants() {
        return Err(ScopeError::Denied(
            "tenant scope required for tenant-scoped insert",
        ));
    }
    if scope.tenant_ids().contains(&tenant_id) {
        return Ok(());
    }
    Err(ScopeError::TenantNotInScope { tenant_id })
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
/// let scope = AccessScope::tenants_only(vec![tenant_id]);
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
/// let scope = AccessScope::tenants_only(vec![tenant_id]);
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

    #[test]
    fn test_validate_tenant_in_scope() {
        let tenant_id = uuid::Uuid::new_v4();
        let scope = crate::secure::AccessScope::tenants_only(vec![tenant_id]);

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
}
