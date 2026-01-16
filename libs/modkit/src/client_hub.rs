//! Minimalistic, type-safe `ClientHub`.
//!
//! Design goals:
//! - Providers register an implementation once (local or remote).
//! - Consumers fetch by *interface type* (trait object): `get::<dyn my::Api>()`.
//! - For plugin-like scenarios, multiple implementations of the same interface can coexist
//!   under different scopes (e.g. selected by GTS instance ID).
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

/// A scope for resolving multiple implementations of the same interface type.
///
/// This is intentionally opaque: the scope semantics are defined by the caller.
/// One common scope is a full GTS entity/instance ID.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct ClientScope(Arc<str>);

impl ClientScope {
    /// Create a new scope from an arbitrary string.
    #[inline]
    #[must_use]
    pub fn new(scope: impl Into<Arc<str>>) -> Self {
        Self(scope.into())
    }

    /// Create a scope derived from a GTS identifier.
    ///
    /// Internally we prefix the scope to avoid accidental collisions with other scope kinds.
    #[must_use]
    pub fn gts_id(gts_id: &str) -> Self {
        let mut s = String::with_capacity("gts:".len() + gts_id.len());
        s.push_str("gts:");
        s.push_str(gts_id);
        Self(Arc::<str>::from(s))
    }

    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ClientScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct ScopedKey {
    type_key: TypeKey,
    scope: ClientScope,
}

impl fmt::Debug for ScopedKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScopedKey")
            .field("type_key", &self.type_key)
            .field("scope", &self.scope)
            .finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClientHubError {
    #[error("client not found: type={type_key:?}")]
    NotFound { type_key: TypeKey },

    #[error("type mismatch in hub for type={type_key:?}")]
    TypeMismatch { type_key: TypeKey },

    #[error("scoped client not found: type={type_key:?} scope={scope:?}")]
    ScopedNotFound {
        type_key: TypeKey,
        scope: ClientScope,
    },

    #[error("type mismatch in hub for type={type_key:?} scope={scope:?}")]
    ScopedTypeMismatch {
        type_key: TypeKey,
        scope: ClientScope,
    },
}

type Boxed = Box<dyn Any + Send + Sync>;

/// Internal map type for the client hub.
type ClientMap = HashMap<TypeKey, Boxed>;

/// Internal map type for the scoped client hub.
type ScopedClientMap = HashMap<ScopedKey, Boxed>;

/// Type-safe registry of clients keyed by interface type.
#[derive(Default)]
pub struct ClientHub {
    map: RwLock<ClientMap>,
    scoped_map: RwLock<ScopedClientMap>,
}

impl ClientHub {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            map: RwLock::new(HashMap::new()),
            scoped_map: RwLock::new(HashMap::new()),
        }
    }
}

impl ClientHub {
    /// Register a client under the interface type `T`.
    /// `T` can be a trait object like `dyn my_module::api::MyClient`.
    pub fn register<T>(&self, client: Arc<T>)
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let type_key = TypeKey::of::<T>();
        let mut w = self.map.write();
        w.insert(type_key, Box::new(client));
    }

    /// Register a scoped client under the interface type `T`.
    ///
    /// This enables multiple implementations of the same interface to coexist,
    /// distinguished by a caller-defined `ClientScope` (e.g., a GTS instance ID).
    pub fn register_scoped<T>(&self, scope: ClientScope, client: Arc<T>)
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let key = ScopedKey {
            type_key: TypeKey::of::<T>(),
            scope,
        };
        let mut w = self.scoped_map.write();
        w.insert(key, Box::new(client));
    }

    /// Fetch a client by interface type `T`.
    ///
    /// # Errors
    /// Returns `ClientHubError::NotFound` if no client is registered for the type.
    /// Returns `ClientHubError::TypeMismatch` if the stored type doesn't match.
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

    /// Fetch a scoped client by interface type `T` and scope.
    ///
    /// # Errors
    /// Returns `ClientHubError::ScopedNotFound` if no client is registered for the `(type, scope)` pair.
    /// Returns `ClientHubError::ScopedTypeMismatch` if the stored type doesn't match.
    pub fn get_scoped<T>(&self, scope: &ClientScope) -> Result<Arc<T>, ClientHubError>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let key = ScopedKey {
            type_key: TypeKey::of::<T>(),
            scope: scope.clone(),
        };
        let r = self.scoped_map.read();

        let boxed = r.get(&key).ok_or_else(|| ClientHubError::ScopedNotFound {
            type_key: key.type_key.clone(),
            scope: key.scope.clone(),
        })?;

        if let Some(arc_t) = boxed.downcast_ref::<Arc<T>>() {
            return Ok(arc_t.clone());
        }
        Err(ClientHubError::ScopedTypeMismatch {
            type_key: key.type_key,
            scope: key.scope,
        })
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

    /// Remove a scoped client by interface type + scope; returns the removed client if it was present.
    pub fn remove_scoped<T>(&self, scope: &ClientScope) -> Option<Arc<T>>
    where
        T: ?Sized + Send + Sync + 'static,
    {
        let key = ScopedKey {
            type_key: TypeKey::of::<T>(),
            scope: scope.clone(),
        };
        let mut w = self.scoped_map.write();
        let boxed = w.remove(&key)?;
        boxed.downcast::<Arc<T>>().ok().map(|b| *b)
    }

    /// Clear everything (useful in tests).
    pub fn clear(&self) {
        self.map.write().clear();
        self.scoped_map.write().clear();
    }

    /// Introspection: (total entries).
    pub fn len(&self) -> usize {
        self.map.read().len() + self.scoped_map.read().len()
    }

    /// Check if the hub is empty.
    pub fn is_empty(&self) -> bool {
        self.map.read().is_empty() && self.scoped_map.read().is_empty()
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

    #[tokio::test]
    async fn scoped_register_and_get_dyn_trait() {
        let hub = ClientHub::new();
        let scope_a = ClientScope::gts_id("gts.x.core.modkit.plugins.v1~x.core.tenant_resolver.plugin.v1~contoso.app._.plugin.v1.0");
        let scope_b = ClientScope::gts_id("gts.x.core.modkit.plugins.v1~x.core.tenant_resolver.plugin.v1~fabrikam.app._.plugin.v1.0");

        let api_a: Arc<dyn TestApi> = Arc::new(ImplA(1));
        let api_b: Arc<dyn TestApi> = Arc::new(ImplA(2));

        hub.register_scoped::<dyn TestApi>(scope_a.clone(), api_a.clone());
        hub.register_scoped::<dyn TestApi>(scope_b.clone(), api_b.clone());

        assert_eq!(
            hub.get_scoped::<dyn TestApi>(&scope_a).unwrap().id().await,
            1
        );
        assert_eq!(
            hub.get_scoped::<dyn TestApi>(&scope_b).unwrap().id().await,
            2
        );
    }

    #[test]
    fn scoped_get_is_independent_from_global_get() {
        let hub = ClientHub::new();
        let scope = ClientScope::gts_id("gts.x.core.modkit.plugins.v1~x.core.tenant_resolver.plugin.v1~fabrikam.app._.plugin.v1.0");
        hub.register::<str>(Arc::from("global"));
        hub.register_scoped::<str>(scope.clone(), Arc::from("scoped"));

        assert_eq!(&*hub.get::<str>().unwrap(), "global");
        assert_eq!(&*hub.get_scoped::<str>(&scope).unwrap(), "scoped");
    }
}
