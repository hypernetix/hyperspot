//! Lock-free router caching using arc-swap for read-mostly data
//!
//! This module provides atomic swapping of read-mostly structures like
//! Axum routers to eliminate lock contention on hot paths.

use arc_swap::ArcSwap;
use std::sync::Arc;

/// Lock-free cache for read-mostly router data
///
/// Uses `arc-swap` to provide atomic, lock-free access to router instances.
/// This is ideal for scenarios where the router is rebuilt infrequently
/// but accessed very frequently (typical in HTTP servers).
///
/// # Benefits
///
/// - **Zero lock contention**: Readers never block each other or writers
/// - **Cache-friendly**: Readers get direct Arc references with minimal indirection
/// - **Atomic updates**: Router swaps are atomic and consistent
/// - **Memory efficient**: Old routers are freed when no longer referenced
///
/// # Usage
///
/// ```rust,ignore
/// let cache = RouterCache::new(initial_router);
///
/// // Hot path: load router (no locks)
/// let router = cache.load();
/// // Use router for request handling...
///
/// // Cold path: rebuild and swap router
/// let new_router = build_new_router().await;
/// cache.store(new_router);
/// ```
pub struct RouterCache<T> {
    inner: ArcSwap<T>,
}

impl<T> RouterCache<T> {
    /// Create a new router cache with an initial value
    pub fn new(initial: T) -> Self {
        Self {
            inner: ArcSwap::from_pointee(initial),
        }
    }

    /// Load the current router instance
    ///
    /// This operation is lock-free and very fast - it returns an Arc
    /// pointing to the current router instance. Multiple readers can
    /// call this concurrently without any contention.
    ///
    /// # Returns
    ///
    /// Arc<T> pointing to the current router instance
    pub fn load(&self) -> Arc<T> {
        self.inner.load_full()
    }

    /// Atomically replace the router with a new instance
    ///
    /// This operation is atomic - all subsequent calls to `load()` will
    /// return the new router instance. Existing Arc references to the old
    /// router remain valid until they are dropped.
    ///
    /// # Arguments
    ///
    /// * `new_router` - The new router instance to store
    pub fn store(&self, new_router: T) {
        self.inner.store(Arc::new(new_router));
    }

    /// Get a reference to the current router without cloning the Arc
    ///
    /// This provides temporary access to the router without incrementing
    /// the reference count. Use this for brief operations where you don't
    /// need to hold onto the router reference.
    ///
    /// # Safety
    ///
    /// The returned reference is only valid for the duration of the closure.
    /// Do not store this reference or use it after the closure returns.
    #[allow(dead_code)]
    pub fn with_current<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let current = self.inner.load();
        f(&*current)
    }

    /// Conditionally update the router
    ///
    /// Only updates the router if the provided predicate returns true
    /// when called with the current router instance.
    ///
    /// # Arguments
    ///
    /// * `predicate` - Function that determines whether to update
    /// * `new_router` - New router instance to store if predicate returns true
    ///
    /// # Returns
    ///
    /// `true` if the router was updated, `false` otherwise
    #[allow(dead_code)]
    pub fn update_if<P>(&self, predicate: P, new_router: T) -> bool
    where
        P: FnOnce(&T) -> bool,
    {
        let current = self.inner.load();
        if predicate(&*current) {
            self.store(new_router);
            true
        } else {
            false
        }
    }

    /// Compare and swap the router
    ///
    /// Only updates if the current router matches the expected value.
    /// This is useful for optimistic updates where you want to ensure
    /// the router hasn't changed since you last read it.
    ///
    /// # Arguments
    ///
    /// * `expected` - The expected current router instance
    /// * `new_router` - New router instance to store if current matches expected
    ///
    /// # Returns
    ///
    /// `Ok(())` if the swap succeeded, `Err(current)` if it failed
    #[allow(dead_code, clippy::needless_pass_by_value)] // Arc is used for pointer comparison
    pub fn compare_and_swap(&self, expected: Arc<T>, new_router: T) -> Result<(), Arc<T>>
    where
        T: PartialEq,
    {
        let new_arc = Arc::new(new_router);
        let result = self.inner.compare_and_swap(&expected, new_arc.clone());

        // The compare_and_swap returns the previous value, not a Result
        // If it matches our expected value, the swap succeeded
        if Arc::ptr_eq(&*result, &expected) {
            Ok(())
        } else {
            Err(result.clone())
        }
    }
}

impl<T> Clone for RouterCache<T> {
    fn clone(&self) -> Self {
        Self {
            inner: ArcSwap::new(self.inner.load_full()),
        }
    }
}

impl<T> Default for RouterCache<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

impl<T> std::fmt::Debug for RouterCache<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let current = self.inner.load();
        f.debug_struct("RouterCache")
            .field("current", &*current)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::thread;
    use std::time::Duration;

    #[derive(Debug, Clone, PartialEq)]
    struct TestRouter {
        id: usize,
        name: String,
    }

    impl TestRouter {
        fn new(id: usize, name: &str) -> Self {
            Self {
                id,
                name: name.to_string(),
            }
        }
    }

    #[test]
    fn test_basic_load_store() {
        let initial = TestRouter::new(1, "initial");
        let cache = RouterCache::new(initial.clone());

        // Load initial router
        let loaded = cache.load();
        assert_eq!(*loaded, initial);

        // Store new router
        let new_router = TestRouter::new(2, "updated");
        cache.store(new_router.clone());

        // Load updated router
        let loaded = cache.load();
        assert_eq!(*loaded, new_router);
    }

    #[test]
    fn test_concurrent_reads() {
        let initial = TestRouter::new(1, "concurrent_test");
        let cache = Arc::new(RouterCache::new(initial.clone()));

        let mut handles = vec![];

        // Spawn multiple readers
        for i in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    let router = cache_clone.load();
                    assert_eq!(router.name, "concurrent_test");
                    thread::sleep(Duration::from_micros(i * 10));
                }
            });
            handles.push(handle);
        }

        // Wait for all readers to complete
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_concurrent_read_write() {
        let initial = TestRouter::new(0, "router_0");
        let cache = Arc::new(RouterCache::new(initial));
        let update_count = Arc::new(AtomicUsize::new(0));

        let mut handles = vec![];

        // Spawn readers
        for _ in 0..5 {
            let cache_clone = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                for _ in 0..50 {
                    let router = cache_clone.load();
                    // Router ID should be monotonically increasing
                    assert!(router.name.starts_with("router_"));
                    thread::sleep(Duration::from_micros(10));
                }
            });
            handles.push(handle);
        }

        // Spawn writers
        for _ in 0..2 {
            let cache_clone = Arc::clone(&cache);
            let count_clone = Arc::clone(&update_count);
            let handle = thread::spawn(move || {
                for _ in 0..10 {
                    let id = count_clone.fetch_add(1, Ordering::SeqCst);
                    let new_router = TestRouter::new(id, &format!("router_{id}"));
                    cache_clone.store(new_router);
                    thread::sleep(Duration::from_millis(1));
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify final state
        let final_router = cache.load();
        assert!(final_router.name.starts_with("router_"));
    }

    #[test]
    fn test_with_current() {
        let initial = TestRouter::new(42, "test_with_current");
        let cache = RouterCache::new(initial);

        let result = cache.with_current(|router| format!("{}_{}", router.name, router.id));

        assert_eq!(result, "test_with_current_42");
    }

    #[test]
    fn test_update_if() {
        let initial = TestRouter::new(1, "conditional");
        let cache = RouterCache::new(initial);

        // Update should succeed (id is 1)
        let new_router1 = TestRouter::new(2, "updated");
        let updated = cache.update_if(|r| r.id == 1, new_router1.clone());
        assert!(updated);

        let current = cache.load();
        assert_eq!(*current, new_router1);

        // Update should fail (id is now 2, not 5)
        let new_router2 = TestRouter::new(3, "should_not_update");
        let updated = cache.update_if(|r| r.id == 5, new_router2);
        assert!(!updated);

        let current = cache.load();
        assert_eq!(*current, new_router1); // Should still be new_router1
    }

    #[test]
    fn test_compare_and_swap() {
        let initial = TestRouter::new(1, "cas_test");
        let cache = RouterCache::new(initial.clone());

        let current = cache.load();
        let new_router = TestRouter::new(2, "cas_updated");

        // CAS should succeed with correct expected value
        let result = cache.compare_and_swap(current, new_router.clone());
        assert!(result.is_ok());

        let updated = cache.load();
        assert_eq!(*updated, new_router);

        // CAS should fail with wrong expected value
        let wrong_expected = Arc::new(TestRouter::new(99, "wrong"));
        let another_router = TestRouter::new(3, "cas_failed");
        let result = cache.compare_and_swap(wrong_expected, another_router);
        assert!(result.is_err());

        // Router should remain unchanged
        let current = cache.load();
        assert_eq!(*current, new_router);
    }

    #[test]
    fn test_clone_and_debug() {
        let original = TestRouter::new(1, "clone_test");
        let cache1 = RouterCache::new(original.clone());
        let cache2 = cache1.clone();

        // Both caches should have the same initial value
        assert_eq!(*cache1.load(), *cache2.load());

        // Update one cache
        let new_router = TestRouter::new(2, "updated");
        cache1.store(new_router.clone());

        // Caches should now be independent
        assert_eq!(*cache1.load(), new_router);
        assert_eq!(*cache2.load(), original);

        // Debug should work
        let debug_str = format!("{cache1:?}");
        assert!(debug_str.contains("RouterCache"));
        assert!(debug_str.contains("updated"));
    }
}
