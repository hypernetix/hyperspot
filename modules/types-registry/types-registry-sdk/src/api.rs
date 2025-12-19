//! `TypesRegistryApi` trait definition.
//!
//! This trait defines the public API for the `types-registry` module.
//! All methods require a `SecurityCtx` for authorization and access control.

use async_trait::async_trait;
use modkit_security::SecurityCtx;

use crate::error::TypesRegistryError;
use crate::models::{GtsEntity, ListQuery, RegisterResult};

/// Public API trait for the `types-registry` module.
///
/// This trait can be consumed by other modules via `ClientHub`:
/// ```ignore
/// let client = hub.get::<dyn TypesRegistryApi>()?;
/// let entity = client.get(&ctx, "gts.acme.core.events.user_created.v1~").await?;
/// ```
///
/// All methods require a `SecurityCtx` for proper authorization and access control.
#[async_trait]
pub trait TypesRegistryApi: Send + Sync {
    /// Register GTS entities (types or instances) in batch.
    ///
    /// Each JSON value in the input should contain a valid GTS entity
    /// with a `$id` field containing the GTS identifier.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context for authorization
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
    /// let results = registry.register(&ctx, entities).await?;
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
        ctx: &SecurityCtx,
        entities: Vec<serde_json::Value>,
    ) -> Result<Vec<RegisterResult>, TypesRegistryError>;

    /// List GTS entities with optional filtering.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context for authorization
    /// * `query` - Query parameters for filtering results
    ///
    /// # Returns
    ///
    /// A vector of `GtsEntity` objects matching the query.
    async fn list(
        &self,
        ctx: &SecurityCtx,
        query: ListQuery,
    ) -> Result<Vec<GtsEntity>, TypesRegistryError>;

    /// Retrieve a single GTS entity by its identifier.
    ///
    /// # Arguments
    ///
    /// * `ctx` - Security context for authorization
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
    async fn get(&self, ctx: &SecurityCtx, gts_id: &str) -> Result<GtsEntity, TypesRegistryError>;
}
