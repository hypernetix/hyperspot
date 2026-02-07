use async_trait::async_trait;
use modkit_db::secure::DBRunner;
use modkit_odata::{ODataQuery, Page};
use modkit_security::AccessScope;
use users_info_sdk::Address;
use uuid::Uuid;

use crate::domain::error::DomainError;

/// Repository trait for Address persistence operations.
#[async_trait]
pub trait AddressesRepository: Send + Sync {
    /// Find an address by ID within the given security scope.
    async fn get<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        id: Uuid,
    ) -> Result<Option<Address>, DomainError>;

    /// List addresses with cursor-based pagination and `OData` filtering.
    async fn list_page<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        query: &ODataQuery,
    ) -> Result<Page<Address>, DomainError>;

    /// Find an address by user ID.
    async fn get_by_user_id<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        user_id: Uuid,
    ) -> Result<Option<Address>, DomainError>;

    /// Create a new address.
    async fn create<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        address: Address,
    ) -> Result<Address, DomainError>;

    /// Update an existing address.
    async fn update<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        address: Address,
    ) -> Result<Address, DomainError>;

    /// Delete an address by ID.
    async fn delete<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        id: Uuid,
    ) -> Result<bool, DomainError>;

    /// Delete all addresses for a given user ID.
    async fn delete_by_user_id<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        user_id: Uuid,
    ) -> Result<u64, DomainError>;
}
