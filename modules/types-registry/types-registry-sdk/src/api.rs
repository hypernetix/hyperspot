//! `TypesRegistryApi` trait definition.
//!
//! This trait defines the public API for the `types-registry` module.
//! All methods require a `SecurityCtx` for authorization and access control.

use async_trait::async_trait;
use modkit_security::SecurityCtx;

use crate::error::TypesRegistryError;
use crate::models::{GtsEntity, ListQuery};

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
    /// A vector of registered `GtsEntity` objects on success.
    ///
    /// # Errors
    ///
    /// * `InvalidGtsId` - If any entity has an invalid GTS ID format
    /// * `AlreadyExists` - If any entity with the same GTS ID already exists
    /// * `ValidationFailed` - If any entity fails schema validation
    async fn register(
        &self,
        ctx: &SecurityCtx,
        entities: Vec<serde_json::Value>,
    ) -> Result<Vec<GtsEntity>, TypesRegistryError>;

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
