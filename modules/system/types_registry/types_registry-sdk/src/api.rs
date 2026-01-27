//! `TypesRegistryApi` trait definition.
//!
//! This trait defines the public API for the `types-registry` module.
//! GTS schemas and instances are global resources, so no security context is required.

use async_trait::async_trait;

use crate::error::TypesRegistryError;
use crate::models::{GtsEntity, ListQuery, RegisterResult};

/// Public API trait for the `types-registry` module.
///
/// This trait can be consumed by other modules via `ClientHub`:
/// ```ignore
/// let client = hub.get::<dyn TypesRegistryApi>()?;
/// let entity = client.get("gts.acme.core.events.user_created.v1~").await?;
/// ```
///
/// GTS schemas and instances are global resources (not tenant-scoped),
/// so no security context is required for these operations.
#[async_trait]
pub trait TypesRegistryClient: Send + Sync {
    /// Register GTS entities (types or instances) in batch.
    ///
    /// Each JSON value in the input should contain a valid GTS entity
    /// with a `$id` field containing the GTS identifier.
    ///
    /// # Arguments
    ///
    /// * `entities` - JSON values representing GTS entities to register
    ///
    /// # Returns
    ///
    /// A vector of `RegisterResult` for each input entity, preserving order.
    /// Each result indicates success (with the registered entity) or failure
    /// (with the error and attempted GTS ID if available).
    ///
    /// Use `RegisterSummary::from_results(&results)` for aggregate counts.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let results = registry.register(entities).await?;
    /// let summary = RegisterSummary::from_results(&results);
    /// println!("Registered {}/{} entities", summary.succeeded, summary.total());
    ///
    /// for result in results {
    ///     match result {
    ///         RegisterResult::Ok(entity) => println!("OK: {}", entity.gts_id),
    ///         RegisterResult::Err { gts_id, error } => {
    ///             eprintln!("FAIL {}: {}", gts_id.as_deref().unwrap_or("?"), error);
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err` only for catastrophic failures (e.g., database unavailable).
    /// Per-item errors are returned in the `RegisterResult::Err` variant.
    async fn register(
        &self,
        entities: Vec<serde_json::Value>,
    ) -> Result<Vec<RegisterResult>, TypesRegistryError>;

    /// List GTS entities with optional filtering.
    ///
    /// # Arguments
    ///
    /// * `query` - Query parameters for filtering results
    ///
    /// # Returns
    ///
    /// A vector of `GtsEntity` objects matching the query.
    async fn list(&self, query: ListQuery) -> Result<Vec<GtsEntity>, TypesRegistryError>;

    /// Retrieve a single GTS entity by its identifier.
    ///
    /// # Arguments
    ///
    /// * `gts_id` - The GTS identifier string
    ///
    /// # Returns
    ///
    /// The `GtsEntity` if found.
    ///
    /// # Errors
    ///
    /// * `NotFound` - If no entity with the given GTS ID exists
    /// * `InvalidGtsId` - If the GTS ID format is invalid
    async fn get(&self, gts_id: &str) -> Result<GtsEntity, TypesRegistryError>;
}
