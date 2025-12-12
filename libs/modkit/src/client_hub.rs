//! Minimalistic, type-safe ClientHub.
//!
//! Design goals:
//! - Providers register an implementation once (local or remote).
//! - Consumers fetch by *interface type* (trait object): `get::<dyn my::Api>()`.
//! - Optional scopes (e.g., multi-tenant): `register_scoped / get_scoped`.
//!
//! Implementation details:
//! - Key = (type name, scope). We use `type_name::<T>()`, which works for `T = dyn Trait`.
//! - Value = `Arc<T>` stored as `Box<dyn Any + Send + Sync>` (downcast on read).
//! - Sync hot path: `get()` is non-async; no hidden per-entry cells or lazy slots.
//!
//! Notes:
//! - Re-registering overwrites the previous value atomically; existing Arcs held by consumers remain valid.
//! - For testing, just register a mock under the same trait type.

use parking_lot::RwLock;
use std::{any::Any, collections::HashMap, fmt, sync::Arc};

/// Global scope constant.
pub const GLOBAL_SCOPE: &str = "global";

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

/// Optional scope (e.g., `global`, `tenant-42`, `user-17`).
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ScopeKey(Option<Arc<str>>);

impl ScopeKey {
    #[allow(dead_code)]
    #[inline]
    fn global() -> Self {
        ScopeKey(None)
    }
    #[inline]
    fn named(s: impl Into<Arc<str>>) -> Self {
        ScopeKey(Some(s.into()))
    }
}

impl fmt::Debug for ScopeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            None => f.write_str("global"),
            Some(s) => f.write_str(s),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientHubError {
    #[error("client not found: type={type_key:?}, scope={scope:?}")]
    NotFound { type_key: TypeKey, scope: ScopeKey },

    #[error("type mismatch in hub for type={type_key:?}, scope={scope:?}")]
    TypeMismatch { type_key: TypeKey, scope: ScopeKey },
}

type Boxed = Box<dyn Any + Send + Sync>;

/// Internal map type for the client hub.
type ClientMap = HashMap<(TypeKey, ScopeKey), Boxed>;

/// Type-safe registry of clients keyed by (interface type, scope).
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

impl Default for ClientHub {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientHub {
    /// Register a client in the *global* scope under the interface type `T`.
    /// `T` can be a trait object like `dyn my_module::contract::MyApi`.
    pub fn register<T>(&self, client: Arc<T>)
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.register_scoped::<T>(GLOBAL_SCOPE, client);
    }

    /// Register a client in a *named* scope under the interface type `T`.
    pub fn register_scoped<T>(&self, scope: impl Into<Arc<str>>, client: Arc<T>)
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let type_key = TypeKey::of::<T>();
        let scope_key = ScopeKey::named(scope);
        let mut w = self.map.write();
        w.insert((type_key, scope_key), Box::new(client));
    }

    /// Fetch a client from the *global* scope by interface type `T`.
    pub fn get<T>(&self) -> Result<Arc<T>, ClientHubError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        self.get_scoped::<T>(GLOBAL_SCOPE)
    }

    /// Fetch a client from a *named* scope by interface type `T`.
    pub fn get_scoped<T>(&self, scope: impl Into<Arc<str>>) -> Result<Arc<T>, ClientHubError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let type_key = TypeKey::of::<T>();
        let scope_key = ScopeKey::named(scope);
        let r = self.map.read();

        let boxed =
            r.get(&(type_key.clone(), scope_key.clone()))
                .ok_or(ClientHubError::NotFound {
                    type_key: type_key.clone(),
                    scope: scope_key.clone(),
                })?;

        // Stored value is exactly `Arc<T>`; downcast is safe and cheap.
        if let Some(arc_t) = boxed.downcast_ref::<Arc<T>>() {
            return Ok(arc_t.clone());
        }
        Err(ClientHubError::TypeMismatch {
            type_key,
            scope: scope_key,
        })
    }

    /// Remove a client; returns the removed client if it was present.
    pub fn remove<T>(&self, scope: impl Into<Arc<str>>) -> Option<Arc<T>>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let type_key = TypeKey::of::<T>();
        let scope_key = ScopeKey::named(scope);
        let mut w = self.map.write();
        let boxed = w.remove(&(type_key, scope_key))?;
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
#[cfg_attr(coverage_nightly, coverage(off))]
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
    async fn register_and_get_dyn_trait_global() {
        let hub = ClientHub::new();
        let api: Arc<dyn TestApi> = Arc::new(ImplA(7));
        hub.register::<dyn TestApi>(api.clone());

        let got = hub.get::<dyn TestApi>().unwrap();
        assert_eq!(got.id().await, 7);
        assert_eq!(Arc::as_ptr(&api), Arc::as_ptr(&got));
    }

    #[tokio::test]
    async fn scopes_are_independent() {
        let hub = ClientHub::new();
        hub.register_scoped::<dyn TestApi>("tenant-1", Arc::new(ImplA(1)));
        hub.register_scoped::<dyn TestApi>("tenant-2", Arc::new(ImplA(2)));

        assert_eq!(
            hub.get_scoped::<dyn TestApi>("tenant-1")
                .unwrap()
                .id()
                .await,
            1
        );
        assert_eq!(
            hub.get_scoped::<dyn TestApi>("tenant-2")
                .unwrap()
                .id()
                .await,
            2
        );
        assert!(hub.get::<dyn TestApi>().is_err()); // global not set
    }
}
