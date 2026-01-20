use std::future::Future;
use std::sync::Arc;

use parking_lot::RwLock;
use tokio::sync::Mutex;

/// A resettable, allocation-friendly selector for GTS plugin instance IDs.
///
/// Uses a single-flight pattern to ensure that the resolve function is called
/// at most once even under concurrent callers. The selected instance ID is
/// cached as `Arc<str>` to avoid allocations on the happy path.
pub struct GtsPluginSelector {
    /// Cached selected instance ID (sync lock for fast access and sync reset).
    cached: RwLock<Option<Arc<str>>>,
    /// Mutex to ensure single-flight resolution.
    resolve_lock: Mutex<()>,
}

impl Default for GtsPluginSelector {
    fn default() -> Self {
        Self::new()
    }
}

impl GtsPluginSelector {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cached: RwLock::new(None),
            resolve_lock: Mutex::new(()),
        }
    }

    /// Returns the cached instance ID, or resolves it using the provided function.
    ///
    /// Uses a single-flight pattern: even under concurrent callers, the resolve
    /// function is called at most once. Returns `Arc<str>` to avoid allocations
    /// on the happy path.
    /// # Errors
    ///
    /// Returns `Err(E)` if the provided `resolve` future fails.
    pub async fn get_or_init<F, Fut, E>(&self, resolve: F) -> Result<Arc<str>, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<String, E>>,
    {
        // Fast path: check if already cached (sync lock, no await)
        {
            let guard = self.cached.read();
            if let Some(ref id) = *guard {
                return Ok(Arc::clone(id));
            }
        }

        // Slow path: acquire resolve lock for single-flight
        let _resolve_guard = self.resolve_lock.lock().await;

        // Re-check after acquiring resolve lock (another caller may have resolved)
        {
            let guard = self.cached.read();
            if let Some(ref id) = *guard {
                return Ok(Arc::clone(id));
            }
        }

        // Resolve and cache
        let id_string = resolve().await?;
        let id: Arc<str> = id_string.into();

        {
            let mut guard = self.cached.write();
            *guard = Some(Arc::clone(&id));
        }

        Ok(id)
    }

    /// Clears the cached selected instance ID.
    ///
    /// Returns `true` if there was a cached value, `false` otherwise.
    pub async fn reset(&self) -> bool {
        let _resolve_guard = self.resolve_lock.lock().await;
        let mut guard = self.cached.write();
        guard.take().is_some()
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn resolve_called_once_returns_same_str() {
        let selector = GtsPluginSelector::new();
        let calls = Arc::new(AtomicUsize::new(0));

        let calls_a = calls.clone();
        let id_a = selector
            .get_or_init(|| async move {
                calls_a.fetch_add(1, Ordering::SeqCst);
                Ok::<_, std::convert::Infallible>(
                    "gts.x.core.modkit.plugin.v1~x.core.test.plugin.v1~a.test._.plugin.v1"
                        .to_owned(),
                )
            })
            .await
            .unwrap();

        let calls_b = calls.clone();
        let id_b = selector
            .get_or_init(|| async move {
                calls_b.fetch_add(1, Ordering::SeqCst);
                Ok::<_, std::convert::Infallible>(
                    "gts.x.core.modkit.plugin.v1~x.core.test.plugin.v1~b.test._.plugin.v1"
                        .to_owned(),
                )
            })
            .await
            .unwrap();

        assert_eq!(id_a, id_b);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn reset_triggers_reselection() {
        let selector = GtsPluginSelector::new();
        let calls = Arc::new(AtomicUsize::new(0));

        let calls_a = calls.clone();
        let id_a = selector
            .get_or_init(|| async move {
                calls_a.fetch_add(1, Ordering::SeqCst);
                Ok::<_, std::convert::Infallible>(
                    "gts.x.core.modkit.plugin.v1~x.core.test.plugin.v1~a.test._.plugin.v1"
                        .to_owned(),
                )
            })
            .await;
        assert_eq!(
            &*id_a.unwrap(),
            "gts.x.core.modkit.plugin.v1~x.core.test.plugin.v1~a.test._.plugin.v1"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(selector.reset().await);

        let calls_b = calls.clone();
        let id_b = selector
            .get_or_init(|| async move {
                calls_b.fetch_add(1, Ordering::SeqCst);
                Ok::<_, std::convert::Infallible>(
                    "gts.x.core.modkit.plugin.v1~x.core.test.plugin.v1~b.test._.plugin.v1"
                        .to_owned(),
                )
            })
            .await;
        assert_eq!(
            &*id_b.unwrap(),
            "gts.x.core.modkit.plugin.v1~x.core.test.plugin.v1~b.test._.plugin.v1"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn concurrent_get_or_init_resolves_once() {
        let selector = Arc::new(GtsPluginSelector::new());
        let calls = Arc::new(AtomicUsize::new(0));

        let mut handles = Vec::new();
        for _ in 0..10 {
            let selector = Arc::clone(&selector);
            let calls = Arc::clone(&calls);
            handles.push(tokio::spawn(async move {
                selector
                    .get_or_init(|| async {
                        // Small delay to increase chance of concurrent access
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                        calls.fetch_add(1, Ordering::SeqCst);
                        Ok::<_, std::convert::Infallible>(
                            "gts.x.core.modkit.plugin.v1~x.core.test.plugin.v1~concurrent.test._.plugin.v1"
                                .to_owned(),
                        )
                    })
                    .await
            }));
        }

        // Await each handle in a loop (no futures_util dependency)
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap().unwrap());
        }

        // All results should be the same
        for id in &results {
            assert_eq!(
                &**id,
                "gts.x.core.modkit.plugin.v1~x.core.test.plugin.v1~concurrent.test._.plugin.v1"
            );
        }

        // Resolve should have been called exactly once
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
