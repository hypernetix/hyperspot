use async_trait::async_trait;
use modkit_db::secure::DBRunner;
use modkit_odata::{ODataQuery, Page};
use modkit_security::AccessScope;
use user_info_sdk::User;
use uuid::Uuid;

use crate::domain::error::DomainError;

/// Repository trait for User persistence operations.
///
/// This trait abstracts persistence operations for users, allowing the domain service
/// to remain independent of the underlying storage implementation.
///
/// All methods accept:
/// - `runner: &C` - secure DB runner (`&SecureConn` or `&SecureTx`)
/// - `scope: &AccessScope` - security scope prepared by the service layer
#[async_trait]
pub trait UsersRepository: Send + Sync {
    /// Find a user by ID within the given security scope.
    async fn get<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        id: Uuid,
    ) -> Result<Option<User>, DomainError>;

    /// List users with cursor-based pagination and `OData` filtering.
    async fn list_page<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        query: &ODataQuery,
    ) -> Result<Page<User>, DomainError>;

    /// Create a new user.
    async fn create<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        user: User,
    ) -> Result<User, DomainError>;

    /// Update an existing user.
    async fn update<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        user: User,
    ) -> Result<User, DomainError>;

    /// Delete a user by ID.
    async fn delete<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        id: Uuid,
    ) -> Result<bool, DomainError>;

    /// Check if a user with the given ID exists within the scope.
    async fn exists<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        id: Uuid,
    ) -> Result<bool, DomainError>;

    /// Count users matching the given email within the scope.
    async fn count_by_email<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        email: &str,
    ) -> Result<u64, DomainError>;
}
