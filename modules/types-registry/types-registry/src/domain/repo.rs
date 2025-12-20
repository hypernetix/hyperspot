//! Repository trait for GTS entity storage.

use types_registry_sdk::{GtsEntity, ListQuery};

use super::error::DomainError;

/// Repository trait for GTS entity storage operations.
///
/// This trait defines the storage interface used by the domain service.
/// Implementations handle the actual storage mechanism (in-memory, database, etc.).
pub trait GtsRepository: Send + Sync {
    /// Registers a GTS entity in the repository.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to register
    /// * `validate` - Whether to perform full validation (production mode)
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The entity already exists
    /// - Validation fails (when `validate` is true)
    fn register(
        &self,
        entity: &serde_json::Value,
        validate: bool,
    ) -> Result<GtsEntity, DomainError>;

    /// Retrieves a GTS entity by its identifier.
    ///
    /// # Arguments
    ///
    /// * `gts_id` - The GTS identifier string
    ///
    /// # Errors
    ///
    /// Returns `NotFound` if the entity doesn't exist.
    fn get(&self, gts_id: &str) -> Result<GtsEntity, DomainError>;

    /// Lists GTS entities matching the given query.
    ///
    /// # Arguments
    ///
    /// * `query` - Query parameters for filtering
    fn list(&self, query: &ListQuery) -> Result<Vec<GtsEntity>, DomainError>;

    /// Checks if an entity with the given GTS ID exists.
    fn exists(&self, gts_id: &str) -> bool;

    /// Returns whether the repository is in production mode.
    fn is_production(&self) -> bool;

    /// Switches the repository from configuration mode to production mode.
    ///
    /// This validates all entities in temporary storage and moves them
    /// to persistent storage if validation succeeds.
    ///
    /// # Errors
    ///
    /// Returns a list of validation errors if any entity fails validation.
    fn switch_to_production(&self) -> Result<(), Vec<String>>;
}
