//! Minimalistic, type-safe ClientHub.
//!
//! Design goals:
//! - Providers register an implementation once (local or remote).
//! - Consumers fetch by *interface type* (trait object) without knowing transport.
//! - Optional scopes (e.g., multi-tenant): register and resolve per tenant, user, or environment.
//!
//! Typical flows:
//! - During module initialization, a provider module exposes its client interface in the hub.
//! - Consumer modules resolve those interfaces from their `ModuleCtx` and keep an `Arc` for reuse.
//! - Scopes are used to keep separate client instances per tenant or deployment variant while
//!   still addressing them by the same interface type.
//! - In tests, you can clear the hub and register in-memory or mocked implementations under
//!   the same interface types to drive end-to-end module interactions.
//!
//! Implementation details:
//! - Key = (type name, scope). We use `type_name::<T>()`, which works for `T = dyn Trait`.
//! - Value = `Arc<T>` stored as `Box<dyn Any + Send + Sync>` (downcast on read).
//! - Sync hot path: `get()` is non-async; no hidden per-entry cells or lazy slots.
//!
//! Notes:
//! - Re-registering overwrites the previous value atomically; existing `Arc`s held by consumers remain valid.
//! - Explicit removal and `clear` are intended mainly for tests and one-off reconfiguration flows.

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

    #[tokio::test]
    async fn re_registering_overwrites_previous_client() {
        let hub = ClientHub::new();
        hub.register::<dyn TestApi>(Arc::new(ImplA(10)));
        hub.register::<dyn TestApi>(Arc::new(ImplA(20)));

        let client = hub.get::<dyn TestApi>().unwrap();
        assert_eq!(
            client.id().await,
            20,
            "Second registration should overwrite the first"
        );
    }

    #[tokio::test]
    async fn existing_arcs_remain_valid_after_re_registration() {
        let hub = ClientHub::new();
        hub.register::<dyn TestApi>(Arc::new(ImplA(100)));

        let client1 = hub.get::<dyn TestApi>().unwrap();

        // Re-register with a different implementation
        hub.register::<dyn TestApi>(Arc::new(ImplA(200)));

        let client2 = hub.get::<dyn TestApi>().unwrap();

        // First Arc should still work with original value
        assert_eq!(
            client1.id().await,
            100,
            "Original Arc should retain its value"
        );
        // New get should return new value
        assert_eq!(
            client2.id().await,
            200,
            "New registration should be retrievable"
        );
    }

    #[test]
    fn get_returns_not_found_for_unregistered_client() {
        let hub = ClientHub::new();

        let result = hub.get::<dyn TestApi>();

        assert!(result.is_err(), "Should fail when client not registered");
        match result {
            Err(ClientHubError::NotFound { type_key, scope }) => {
                assert!(
                    format!("{:?}", type_key).contains("TestApi"),
                    "Error should reference the trait type"
                );
                assert_eq!(
                    format!("{:?}", scope),
                    "global",
                    "Error should reference global scope"
                );
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[test]
    fn get_scoped_returns_not_found_for_wrong_scope() {
        let hub = ClientHub::new();
        hub.register_scoped::<dyn TestApi>("tenant-1", Arc::new(ImplA(5)));

        let result = hub.get_scoped::<dyn TestApi>("tenant-2");

        assert!(result.is_err(), "Should fail for unregistered scope");
        match result {
            Err(ClientHubError::NotFound { scope, .. }) => {
                assert_eq!(
                    format!("{:?}", scope),
                    "tenant-2",
                    "Error should reference the requested scope"
                );
            }
            _ => panic!("Expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn remove_returns_client_and_makes_it_unavailable() {
        let hub = ClientHub::new();
        hub.register_scoped::<dyn TestApi>("temp-scope", Arc::new(ImplA(42)));

        let removed = hub.remove::<dyn TestApi>("temp-scope");

        assert!(removed.is_some(), "Remove should return the client");
        assert_eq!(
            removed.unwrap().id().await,
            42,
            "Removed client should be usable"
        );

        let get_result = hub.get_scoped::<dyn TestApi>("temp-scope");
        assert!(
            get_result.is_err(),
            "Client should no longer be retrievable after removal"
        );
    }

    #[test]
    fn remove_returns_none_for_unregistered_client() {
        let hub = ClientHub::new();

        let removed = hub.remove::<dyn TestApi>("nonexistent");

        assert!(
            removed.is_none(),
            "Remove should return None for unregistered client"
        );
    }

    #[tokio::test]
    async fn clear_removes_all_clients() {
        let hub = ClientHub::new();
        hub.register::<dyn TestApi>(Arc::new(ImplA(1)));
        hub.register_scoped::<dyn TestApi>("tenant-1", Arc::new(ImplA(2)));
        hub.register_scoped::<dyn TestApi>("tenant-2", Arc::new(ImplA(3)));

        assert_eq!(hub.len(), 3, "Should have 3 registered clients");

        hub.clear();

        assert_eq!(hub.len(), 0, "All clients should be removed");
        assert!(hub.is_empty(), "Hub should be empty");
        assert!(
            hub.get::<dyn TestApi>().is_err(),
            "Global client should be unavailable"
        );
        assert!(
            hub.get_scoped::<dyn TestApi>("tenant-1").is_err(),
            "Scoped clients should be unavailable"
        );
    }

    #[tokio::test]
    async fn multiple_trait_types_coexist_independently() {
        #[async_trait::async_trait]
        trait AnotherApi: Send + Sync {
            async fn name(&self) -> &str;
        }

        struct ImplB(&'static str);
        #[async_trait::async_trait]
        impl AnotherApi for ImplB {
            async fn name(&self) -> &str {
                self.0
            }
        }

        let hub = ClientHub::new();
        hub.register::<dyn TestApi>(Arc::new(ImplA(99)));
        hub.register::<dyn AnotherApi>(Arc::new(ImplB("service-x")));

        let api1 = hub.get::<dyn TestApi>().unwrap();
        let api2 = hub.get::<dyn AnotherApi>().unwrap();

        assert_eq!(api1.id().await, 99, "First trait should be retrievable");
        assert_eq!(
            api2.name().await,
            "service-x",
            "Second trait should be retrievable independently"
        );
    }

    #[tokio::test]
    async fn same_trait_type_in_global_and_scoped_are_independent() {
        let hub = ClientHub::new();
        hub.register::<dyn TestApi>(Arc::new(ImplA(10)));
        hub.register_scoped::<dyn TestApi>("scope-a", Arc::new(ImplA(20)));

        let global = hub.get::<dyn TestApi>().unwrap();
        let scoped = hub.get_scoped::<dyn TestApi>("scope-a").unwrap();

        assert_eq!(global.id().await, 10, "Global should have its own value");
        assert_eq!(scoped.id().await, 20, "Scoped should have its own value");
    }

    #[test]
    fn len_and_is_empty_reflect_registration_state() {
        let hub = ClientHub::new();
        assert_eq!(hub.len(), 0);
        assert!(hub.is_empty());

        hub.register::<dyn TestApi>(Arc::new(ImplA(1)));
        assert_eq!(hub.len(), 1);
        assert!(!hub.is_empty());

        hub.register_scoped::<dyn TestApi>("tenant-1", Arc::new(ImplA(2)));
        assert_eq!(hub.len(), 2);

        hub.remove::<dyn TestApi>(GLOBAL_SCOPE);
        assert_eq!(hub.len(), 1);

        hub.clear();
        assert_eq!(hub.len(), 0);
        assert!(hub.is_empty());
    }

    #[tokio::test]
    async fn hub_is_thread_safe_under_concurrent_access() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        let hub = Arc::new(ClientHub::new());
        let success_count = Arc::new(AtomicUsize::new(0));

        // Register initial client
        hub.register::<dyn TestApi>(Arc::new(ImplA(0)));

        let mut handles = vec![];

        // Spawn multiple tasks doing concurrent reads and writes
        for i in 0..10 {
            let hub_clone = hub.clone();
            let success_clone = success_count.clone();
            handles.push(tokio::spawn(async move {
                // Register
                hub_clone.register::<dyn TestApi>(Arc::new(ImplA(i)));

                // Read
                if let Ok(client) = hub_clone.get::<dyn TestApi>() {
                    let _ = client.id().await;
                    success_clone.fetch_add(1, Ordering::SeqCst);
                }
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // All operations should have succeeded without panics
        assert_eq!(
            success_count.load(Ordering::SeqCst),
            10,
            "All concurrent reads should succeed"
        );

        // Final state should be consistent
        let final_client = hub.get::<dyn TestApi>().unwrap();
        let final_id = final_client.id().await;
        assert!(
            final_id < 10,
            "Final registered client should be one of the registered values"
        );
    }
}
