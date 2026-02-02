//! Local client implementation for the types module.

use async_trait::async_trait;
use modkit_macros::domain_model;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use types_sdk::{TypesClient, TypesError};

/// Local client implementation of [`TypesClient`].
///
/// This client is used for in-process communication when the types module
/// is running in the same process as the caller.
#[domain_model]
pub struct TypesLocalClient {
    ready: Arc<AtomicBool>,
}

impl TypesLocalClient {
    /// Creates a new local client.
    #[must_use]
    pub fn new(ready: Arc<AtomicBool>) -> Self {
        Self { ready }
    }

    /// Marks the types module as ready.
    pub fn set_ready(&self) {
        self.ready.store(true, Ordering::Release);
    }
}

#[async_trait]
impl TypesClient for TypesLocalClient {
    async fn is_ready(&self) -> Result<bool, TypesError> {
        Ok(self.ready.load(Ordering::Acquire))
    }
}
