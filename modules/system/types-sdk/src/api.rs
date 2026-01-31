//! `TypesClient` trait definition.
//!
//! This trait defines the public API for the `types` module.

use async_trait::async_trait;

use crate::error::TypesError;

/// Public API trait for the `types` module.
///
/// This trait can be consumed by other modules via `ClientHub`:
/// ```ignore
/// let client = hub.get::<dyn TypesClient>()?;
/// let ready = client.is_ready().await?;
/// ```
///
/// The types module is responsible for registering core GTS types
/// that other modules depend on (e.g., `BaseModkitPluginV1`).
#[async_trait]
pub trait TypesClient: Send + Sync {
    /// Check if core types have been registered.
    ///
    /// # Returns
    ///
    /// `true` if core types are registered and ready.
    ///
    /// # Errors
    ///
    /// Returns an error if the check fails.
    async fn is_ready(&self) -> Result<bool, TypesError>;
}
