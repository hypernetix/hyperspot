use async_trait::async_trait;
use modkit_db::secure::DBRunner;
use modkit_odata::{ODataQuery, Page};
use modkit_security::AccessScope;
use users_info_sdk::City;
use uuid::Uuid;

use crate::domain::error::DomainError;

/// Repository trait for City persistence operations.
#[async_trait]
pub trait CitiesRepository: Send + Sync {
    /// Find a city by ID within the given security scope.
    async fn get<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        id: Uuid,
    ) -> Result<Option<City>, DomainError>;

    /// List cities with cursor-based pagination and `OData` filtering.
    async fn list_page<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        query: &ODataQuery,
    ) -> Result<Page<City>, DomainError>;

    /// Create a new city.
    async fn create<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        city: City,
    ) -> Result<City, DomainError>;

    /// Update an existing city.
    async fn update<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        city: City,
    ) -> Result<City, DomainError>;

    /// Delete a city by ID.
    async fn delete<C: DBRunner>(
        &self,
        runner: &C,
        scope: &AccessScope,
        id: Uuid,
    ) -> Result<bool, DomainError>;
}
