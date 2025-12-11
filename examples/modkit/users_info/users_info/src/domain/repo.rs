use async_trait::async_trait;
use modkit_db::secure::SecurityCtx;
use modkit_odata::Error as ODataError;
use modkit_odata::{ODataQuery, Page};
use user_info_sdk::User;
use uuid::Uuid;

/// Port for the domain layer: persistence operations the domain needs.
///
/// All operations require a `SecurityCtx` to enforce access control at the database level.
/// This ensures that multi-tenant isolation and resource-level access control are
/// applied consistently across all data operations.
///
/// Object-safe and async-friendly via `async_trait`.
#[async_trait]
pub trait UsersRepository: Send + Sync {
    /// Load a user by id with security context.
    ///
    /// The security context ensures that only users within the allowed tenant/resource scope
    /// can be retrieved.
    async fn find_by_id(&self, ctx: &SecurityCtx, id: Uuid) -> anyhow::Result<Option<User>>;

    /// Check uniqueness by email within the security context.
    ///
    /// Email uniqueness is checked only within the accessible scope.
    async fn email_exists(&self, ctx: &SecurityCtx, email: &str) -> anyhow::Result<bool>;

    /// Insert a fully-formed domain user with security validation.
    ///
    /// Service computes id/timestamps/validation; repo persists with scope checks.
    /// The security context validates that the user is being created within an allowed tenant.
    async fn insert(&self, ctx: &SecurityCtx, u: User) -> anyhow::Result<()>;

    /// Update an existing user (by primary key in `u.id`) with security validation.
    ///
    /// Only users within the security scope can be updated.
    async fn update(&self, ctx: &SecurityCtx, u: User) -> anyhow::Result<()>;

    /// Delete by id with security validation. Returns true if a row was deleted.
    ///
    /// Only users within the security scope can be deleted.
    async fn delete(&self, ctx: &SecurityCtx, id: Uuid) -> anyhow::Result<bool>;

    /// List with cursor-based pagination and security filtering.
    ///
    /// Returns only users within the accessible security scope.
    /// Uses unified OData error type for all pagination/sorting errors.
    async fn list_users_page(
        &self,
        ctx: &SecurityCtx,
        query: &ODataQuery,
    ) -> Result<Page<User>, ODataError>;
}
