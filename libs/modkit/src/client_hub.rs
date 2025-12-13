//! Minimalistic, type-safe ClientHub.
//!
//! Design goals:
//! - Providers register an implementation once (local or remote).
//! - Consumers fetch by *interface type* (trait object): `get::<dyn my::Api>()`.
//!
//! Implementation details:
//! - Key = type name. We use `type_name::<T>()`, which works for `T = dyn Trait`.
//! - Value = `Arc<T>` stored as `Box<dyn Any + Send + Sync>` (downcast on read).
//! - Sync hot path: `get()` is non-async; no hidden per-entry cells or lazy slots.
//!
//! Notes:
//! - Re-registering overwrites the previous value atomically; existing Arcs held by consumers remain valid.
//! - For testing, just register a mock under the same trait type.

use parking_lot::RwLock;
use std::{any::Any, collections::HashMap, fmt, sync::Arc};

/// Stable type key for trait objects — uses fully-qualified `type_name::<T>()`.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct TypeKey(&'static str);

impl TypeKey {
    #[inline]
    fn of<T: ?Sized + 'static>() -> Self {
        TypeKey(std::any::type_name::<T>())
    }
}

impl fmt::Debug for TypeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientHubError {
    #[error("client not found: type={type_key:?}")]
    NotFound { type_key: TypeKey },

    #[error("type mismatch in hub for type={type_key:?}")]
    TypeMismatch { type_key: TypeKey },
}

type Boxed = Box<dyn Any + Send + Sync>;

/// Internal map type for the client hub.
type ClientMap = HashMap<TypeKey, Boxed>;

/// Type-safe registry of clients keyed by interface type.
#[derive(Default)]
pub struct ClientHub {
    map: RwLock<ClientMap>,
}

impl ClientHub {
    #[inline]
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
        }
    }
}

impl ClientHub {
    /// Register a client under the interface type `T`.
    /// `T` can be a trait object like `dyn my_module::contract::MyApi`.
    pub fn register<T>(&self, client: Arc<T>)
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let type_key = TypeKey::of::<T>();
        let mut w = self.map.write();
        w.insert(type_key, Box::new(client));
    }

    /// Fetch a client by interface type `T`.
    pub fn get<T>(&self) -> Result<Arc<T>, ClientHubError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let type_key = TypeKey::of::<T>();
        let r = self.map.read();

        let boxed = r.get(&type_key).ok_or(ClientHubError::NotFound {
            type_key: type_key.clone(),
        })?;

        // Stored value is exactly `Arc<T>`; downcast is safe and cheap.
        if let Some(arc_t) = boxed.downcast_ref::<Arc<T>>() {
            return Ok(arc_t.clone());
        }
        Err(ClientHubError::TypeMismatch { type_key })
    }

    /// Remove a client by interface type; returns the removed client if it was present.
    pub fn remove<T>(&self) -> Option<Arc<T>>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let type_key = TypeKey::of::<T>();
        let mut w = self.map.write();
        let boxed = w.remove(&type_key)?;
        boxed.downcast::<Arc<T>>().ok().map(|b| *b)
    }

    /// Clear everything (useful in tests).
    pub fn clear(&self) {
        self.map.write().clear();
    }

    /// Introspection: (total entries).
    pub fn len(&self) -> usize {
        self.map.read().len()
    }

    /// Check if the hub is empty.
    pub fn is_empty(&self) -> bool {
        self.map.read().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[async_trait::async_trait]
    trait TestApi: Send + Sync {
        async fn id(&self) -> usize;
    }

    struct ImplA(usize);
    #[async_trait::async_trait]
    impl TestApi for ImplA {
        async fn id(&self) -> usize {
            self.0
        }
    }

    #[tokio::test]
    async fn register_and_get_dyn_trait() {
        let hub = ClientHub::new();
        let api: Arc<dyn TestApi> = Arc::new(ImplA(7));
        hub.register::<dyn TestApi>(api.clone());

        let got = hub.get::<dyn TestApi>().unwrap();
        assert_eq!(got.id().await, 7);
        assert_eq!(Arc::as_ptr(&api), Arc::as_ptr(&got));
    }

    #[tokio::test]
    async fn remove_works() {
        let hub = ClientHub::new();
        let api: Arc<dyn TestApi> = Arc::new(ImplA(42));
        hub.register::<dyn TestApi>(api);

        assert!(hub.get::<dyn TestApi>().is_ok());

        let removed = hub.remove::<dyn TestApi>();
        assert!(removed.is_some());
        assert!(hub.get::<dyn TestApi>().is_err());
    }

    #[tokio::test]
    async fn overwrite_replaces_atomically() {
        let hub = ClientHub::new();
        hub.register::<dyn TestApi>(Arc::new(ImplA(1)));

        let old = hub.get::<dyn TestApi>().unwrap();
        assert_eq!(old.id().await, 1);

        hub.register::<dyn TestApi>(Arc::new(ImplA(2)));

        let new = hub.get::<dyn TestApi>().unwrap();
        assert_eq!(new.id().await, 2);

        // Old Arc is still valid
        assert_eq!(old.id().await, 1);
    }
}
