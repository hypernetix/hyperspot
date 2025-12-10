use async_trait::async_trait;
use parking_lot::Mutex;
use std::sync::{
    atomic::{AtomicBool, AtomicU8, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::{oneshot, Notify};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

// ----- Results & aliases -----------------------------------------------------

/// Public result for lifecycle-level operations.
type LcResult<T = ()> = std::result::Result<T, LifecycleError>;

/// Result returned by user/background tasks.
type TaskResult<T = ()> = anyhow::Result<T>;

/// Type alias for ready function pointer to reduce complexity.
type ReadyFn<T> = fn(
    Arc<T>,
    CancellationToken,
    ReadySignal,
)
    -> std::pin::Pin<Box<dyn std::future::Future<Output = TaskResult<()>> + Send>>;

// ----- Status model ----------------------------------------------------------

/// Terminal/transition states for a background job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    Stopped,
    Starting,
    Running,
    Stopping,
}

impl Status {
    #[inline]
    pub const fn as_u8(self) -> u8 {
        match self {
            Status::Stopped => 0,
            Status::Starting => 1,
            Status::Running => 2,
            Status::Stopping => 3,
        }
    }
    #[inline]
    pub const fn from_u8(x: u8) -> Self {
        match x {
            1 => Status::Starting,
            2 => Status::Running,
            3 => Status::Stopping,
            _ => Status::Stopped,
        }
    }
}

/// Reason why a task stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    Finished,
    Cancelled,
    Timeout,
}

// ----- Ready signal ----------------------------------------------------------

/// Ready signal used by `start_with_ready*` to flip Starting -> Running.
pub struct ReadySignal(oneshot::Sender<()>);

impl ReadySignal {
    #[inline]
    pub fn notify(self) {
        let _ = self.0.send(());
    }
    /// Construct a ReadySignal from a oneshot sender (used by macro-generated shims).
    #[inline]
    pub fn from_sender(sender: tokio::sync::oneshot::Sender<()>) -> Self {
        ReadySignal(sender)
    }
}

// ----- Runnable --------------------------------------------------------------

/// Trait for modules that can run a long-running task.
/// Note: take `self` by `Arc` to make the spawned future `'static` and `Send`.
#[async_trait]
pub trait Runnable: Send + Sync + 'static {
    /// Long-running loop. Must return when `cancel` is cancelled.
    async fn run(self: Arc<Self>, cancel: CancellationToken) -> TaskResult<()>;
}

// ----- Errors ----------------------------------------------------------------

/// Library-level error for lifecycle operations.
#[derive(Debug, thiserror::Error)]
pub enum LifecycleError {
    #[error("already started")]
    AlreadyStarted,
}

// ----- Lifecycle -------------------------------------------------------------

/// Lifecycle controller for managing background tasks.
///
/// Concurrency notes:
/// - State is tracked with atomics and `Notify`.
/// - `handle` / `cancel` are protected by `Mutex`, and their locking scope is kept minimal.
/// - All public start methods are thin wrappers around `start_core`.
pub struct Lifecycle {
    name: &'static str,
    status: Arc<AtomicU8>,
    handle: Mutex<Option<JoinHandle<()>>>,
    cancel: Mutex<Option<CancellationToken>>,
    /// `true` once the background task has fully finished.
    finished: Arc<AtomicBool>,
    /// Set to `true` when `stop()` requested cancellation.
    was_cancelled: Arc<AtomicBool>,
    /// Notifies all waiters when the task finishes.
    finished_notify: Arc<Notify>,
}

impl Lifecycle {
    pub fn new_named(name: &'static str) -> Self {
        Self {
            name,
            status: Arc::new(AtomicU8::new(Status::Stopped.as_u8())),
            handle: Mutex::new(None),
            cancel: Mutex::new(None),
            finished: Arc::new(AtomicBool::new(false)),
            was_cancelled: Arc::new(AtomicBool::new(false)),
            finished_notify: Arc::new(Notify::new()),
        }
    }

    pub fn new() -> Self {
        Self::new_named("lifecycle")
    }

    #[inline]
    pub fn name(&self) -> &'static str {
        self.name
    }

    // --- small helpers for atomics (keeps Ordering unified and code concise) ---

    #[inline]
    fn load_status(&self) -> Status {
        Status::from_u8(self.status.load(Ordering::Acquire))
    }

    #[inline]
    fn store_status(&self, s: Status) {
        self.status.store(s.as_u8(), Ordering::Release);
    }

    // --- public start APIs delegate to start_core --------------------------------

    /// Spawn the job using `make(cancel)`.
    ///
    /// The future is constructed inside the task to avoid leaving the lifecycle in `Starting`
    /// if `make` panics.
    #[tracing::instrument(skip(self, make), level = "debug")]
    pub fn start<F, Fut>(&self, make: F) -> LcResult
    where
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = TaskResult<()>> + Send + 'static,
    {
        self.start_core(CancellationToken::new(), move |tok, _| make(tok), false)
    }

    /// Spawn the job using a provided `CancellationToken` and `make(cancel)`.
    #[tracing::instrument(skip(self, make, token), level = "debug")]
    pub fn start_with_token<F, Fut>(&self, token: CancellationToken, make: F) -> LcResult
    where
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = TaskResult<()>> + Send + 'static,
    {
        self.start_core(token, move |tok, _| make(tok), false)
    }

    /// Spawn the job using `make(cancel, ready)`. Status becomes `Running` only after `ready.notify()`.
    #[tracing::instrument(skip(self, make), level = "debug")]
    pub fn start_with_ready<F, Fut>(&self, make: F) -> LcResult
    where
        F: FnOnce(CancellationToken, ReadySignal) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = TaskResult<()>> + Send + 'static,
    {
        self.start_core(
            CancellationToken::new(),
            move |tok, rdy| async move {
                let Some(rdy) = rdy else {
                    return Err(anyhow::anyhow!("ReadySignal must be present"));
                };
                make(tok, rdy).await
            },
            true,
        )
    }

    /// Ready-aware start variant that uses a provided `CancellationToken`.
    #[tracing::instrument(skip(self, make, token), level = "debug")]
    pub fn start_with_ready_and_token<F, Fut>(&self, token: CancellationToken, make: F) -> LcResult
    where
        F: FnOnce(CancellationToken, ReadySignal) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = TaskResult<()>> + Send + 'static,
    {
        self.start_core(
            token,
            move |tok, rdy| async move {
                let Some(rdy) = rdy else {
                    return Err(anyhow::anyhow!("ReadySignal must be present"));
                };
                make(tok, rdy).await
            },
            true,
        )
    }

    /// Unified start core
    ///
    /// `ready_mode = true`   => we expect a ReadySignal to flip `Starting -> Running` (upon notify).
    /// `ready_mode = false`  => we flip to `Running` immediately after spawn.
    fn start_core<F, Fut>(&self, token: CancellationToken, make: F, ready_mode: bool) -> LcResult
    where
        F: Send + 'static + FnOnce(CancellationToken, Option<ReadySignal>) -> Fut,
        Fut: std::future::Future<Output = TaskResult<()>> + Send + 'static,
    {
        // Stopped -> Starting (via CAS)
        let cas_ok = self
            .status
            .compare_exchange(
                Status::Stopped.as_u8(),
                Status::Starting.as_u8(),
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok();
        if !cas_ok {
            return Err(LifecycleError::AlreadyStarted);
        }

        self.finished.store(false, Ordering::Release);
        self.was_cancelled.store(false, Ordering::Release);

        // store cancellation token (bounded lock scope)
        {
            let mut c = self.cancel.lock();
            *c = Some(token.clone());
        }

        // In ready mode, we wait for `ready.notify()` to flip to Running.
        // Otherwise, we mark Running immediately.
        let (ready_tx, ready_rx) = oneshot::channel::<()>();
        if ready_mode {
            let status_on_ready = self.status.clone();
            tokio::spawn(async move {
                if ready_rx.await.is_ok() {
                    let _ = status_on_ready.compare_exchange(
                        Status::Starting.as_u8(),
                        Status::Running.as_u8(),
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    );
                    tracing::debug!("lifecycle status -> running (ready)");
                } else {
                    // Sender dropped: task didn't signal readiness; we will remain in Starting
                    // until finish. This is usually a bug or early-drop scenario.
                    tracing::debug!("ready signal dropped; staying in Starting until finish");
                }
            });
        } else {
            self.store_status(Status::Running);
            tracing::debug!("lifecycle status -> running");
        }

        let finished_flag = self.finished.clone();
        let finished_notify = self.finished_notify.clone();
        let status_on_finish = self.status.clone();

        // Spawn the actual task with descriptive logging
        let module_name = self.name;
        let task_id = format!("{module_name}-{:p}", self);
        let handle = tokio::spawn({
            let task_id = task_id.clone();
            async move {
                tracing::debug!(task_id = %task_id, module = %module_name, "lifecycle task starting");
                let res = make(token, ready_mode.then(|| ReadySignal(ready_tx))).await;
                if let Err(e) = res {
                    tracing::error!(error=%e, task_id=%task_id, module = %module_name, "lifecycle task error");
                }
                finished_flag.store(true, Ordering::Release);
                finished_notify.notify_waiters();
                status_on_finish.store(Status::Stopped.as_u8(), Ordering::Release);
                tracing::debug!(task_id=%task_id, module = %module_name, "lifecycle task finished");
            }
        });

        // store handle (bounded lock scope)
        {
            let mut h = self.handle.lock();
            *h = Some(handle);
        }

        Ok(())
    }

    /// Request graceful shutdown and wait up to `timeout`.
    #[tracing::instrument(skip(self, timeout), level = "debug")]
    pub async fn stop(&self, timeout: Duration) -> LcResult<StopReason> {
        let module_name = self.name;
        let task_id = format!("{module_name}-{:p}", self);
        let st = self.load_status();
        if !matches!(st, Status::Starting | Status::Running | Status::Stopping) {
            // Not running => already finished.
            return Ok(StopReason::Finished);
        }

        self.store_status(Status::Stopping);

        // Request cancellation only once (idempotent if multiple callers race here).
        if let Some(tok) = { self.cancel.lock().take() } {
            self.was_cancelled.store(true, Ordering::Release);
            tok.cancel();
        }

        // Waiter that works for all callers, even after the task already finished.
        let finished_flag = self.finished.clone();
        let notify = self.finished_notify.clone();
        let finished_wait = async move {
            if finished_flag.load(Ordering::Acquire) {
                return;
            }
            notify.notified().await;
        };

        let reason = tokio::select! {
            _ = finished_wait => {
                if self.was_cancelled.load(Ordering::Acquire) {
                    StopReason::Cancelled
                } else {
                    StopReason::Finished
                }
            }
            _ = tokio::time::sleep(timeout) => StopReason::Timeout,
        };

        // Join and ensure we notify waiters even if the task was aborted/panicked.
        let handle_opt = { self.handle.lock().take() };
        if let Some(handle) = handle_opt {
            if matches!(reason, StopReason::Timeout) && !handle.is_finished() {
                tracing::warn!("lifecycle stop timed out; aborting task");
                handle.abort();
            }

            match handle.await {
                Ok(_) => {
                    tracing::debug!(task_id = %task_id, module = %module_name, "lifecycle task completed successfully");
                }
                Err(e) if e.is_cancelled() => {
                    tracing::debug!(task_id = %task_id, module = %module_name, "lifecycle task was cancelled/aborted");
                }
                Err(e) if e.is_panic() => {
                    // Extract panic information if possible
                    if let Ok(panic_payload) = e.try_into_panic() {
                        let panic_msg = panic_payload
                            .downcast_ref::<&str>()
                            .map(|s| s.to_string())
                            .or_else(|| panic_payload.downcast_ref::<String>().cloned())
                            .unwrap_or_else(|| "unknown panic".to_string());

                        tracing::error!(
                            task_id = %task_id,
                            module = %module_name,
                            panic_message = %panic_msg,
                            "lifecycle task panicked - this indicates a serious bug"
                        );
                    } else {
                        tracing::error!(
                            task_id = %task_id,
                            module = %module_name,
                            "lifecycle task panicked (could not extract panic message)"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(task_id = %task_id, module = %module_name, error = %e, "lifecycle task join error");
                }
            }

            self.finished.store(true, Ordering::Release);
            self.finished_notify.notify_waiters();
        }

        self.store_status(Status::Stopped);
        tracing::info!(?reason, "lifecycle stopped");
        Ok(reason)
    }

    /// Current status.
    #[inline]
    #[must_use]
    pub fn status(&self) -> Status {
        self.load_status()
    }

    /// Whether it is in `Starting` or `Running`.
    #[inline]
    pub fn is_running(&self) -> bool {
        matches!(self.status(), Status::Starting | Status::Running)
    }

    /// Best-effort "try start" that swallows the error and returns bool.
    #[inline]
    #[must_use]
    pub fn try_start<F, Fut>(&self, make: F) -> bool
    where
        F: FnOnce(CancellationToken) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = TaskResult<()>> + Send + 'static,
    {
        self.start(make).is_ok()
    }

    /// Wait until the task is fully stopped.
    pub async fn wait_stopped(&self) {
        if self.finished.load(Ordering::Acquire) {
            return;
        }
        self.finished_notify.notified().await;
    }
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Lifecycle {
    /// Best-effort cleanup to avoid orphaned background tasks if caller forgets to call stop().
    fn drop(&mut self) {
        if let Some(tok) = self.cancel.get_mut().take() {
            tok.cancel();
        }
        if let Some(handle) = self.handle.get_mut().take() {
            handle.abort();
        }
    }
}

// ----- WithLifecycle wrapper -------------------------------------------------

/// Wrapper that implements `StatefulModule` for any `T: Runnable`.
pub struct WithLifecycle<T: Runnable> {
    inner: Arc<T>,
    lc: Arc<Lifecycle>,
    pub(crate) stop_timeout: Duration,
    // lifecycle start mode configuration
    await_ready: bool,
    has_ready_handler: bool,
    run_ready_fn: Option<ReadyFn<T>>,
}

impl<T: Runnable> WithLifecycle<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: Arc::new(inner),
            lc: Arc::new(Lifecycle::new_named(std::any::type_name::<T>())),
            stop_timeout: Duration::from_secs(30),
            await_ready: false,
            has_ready_handler: false,
            run_ready_fn: None,
        }
    }

    pub fn from_arc(inner: Arc<T>) -> Self {
        Self {
            inner,
            lc: Arc::new(Lifecycle::new_named(std::any::type_name::<T>())),
            stop_timeout: Duration::from_secs(30),
            await_ready: false,
            has_ready_handler: false,
            run_ready_fn: None,
        }
    }

    pub fn new_with_name(inner: T, name: &'static str) -> Self {
        Self {
            inner: Arc::new(inner),
            lc: Arc::new(Lifecycle::new_named(name)),
            stop_timeout: Duration::from_secs(30),
            await_ready: false,
            has_ready_handler: false,
            run_ready_fn: None,
        }
    }

    pub fn from_arc_with_name(inner: Arc<T>, name: &'static str) -> Self {
        Self {
            inner,
            lc: Arc::new(Lifecycle::new_named(name)),
            stop_timeout: Duration::from_secs(30),
            await_ready: false,
            has_ready_handler: false,
            run_ready_fn: None,
        }
    }

    pub fn with_stop_timeout(mut self, d: Duration) -> Self {
        self.stop_timeout = d;
        self
    }

    #[inline]
    pub fn status(&self) -> Status {
        self.lc.status()
    }

    #[inline]
    pub fn inner(&self) -> &T {
        self.inner.as_ref()
    }

    /// Sometimes callers need to hold an `Arc` to the inner runnable.
    #[inline]
    pub fn inner_arc(&self) -> Arc<T> {
        self.inner.clone()
    }

    /// Configure readiness behavior produced by proc-macros (`#[modkit::module(..., lifecycle(...))]`).
    pub fn with_ready_mode(
        mut self,
        await_ready: bool,
        has_ready_handler: bool,
        run_ready_fn: Option<ReadyFn<T>>,
    ) -> Self {
        self.await_ready = await_ready;
        self.has_ready_handler = has_ready_handler;
        self.run_ready_fn = run_ready_fn;
        self
    }
}

impl<T: Runnable + Default> Default for WithLifecycle<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

#[async_trait]
impl<T: Runnable> crate::contracts::StatefulModule for WithLifecycle<T> {
    #[tracing::instrument(skip(self, external_cancel), level = "debug")]
    async fn start(&self, external_cancel: CancellationToken) -> TaskResult<()> {
        let inner = self.inner.clone();
        let composed = external_cancel.child_token();

        if !self.await_ready {
            self.lc
                .start_with_token(composed, move |cancel| inner.run(cancel))
                .map_err(anyhow::Error::from)
        } else if self.has_ready_handler {
            let f = self.run_ready_fn.ok_or_else(|| {
                anyhow::anyhow!("run_ready_fn must be set when has_ready_handler")
            })?;
            self.lc
                .start_with_ready_and_token(composed, move |cancel, ready| f(inner, cancel, ready))
                .map_err(anyhow::Error::from)
        } else {
            self.lc
                .start_with_ready_and_token(composed, move |cancel, ready| async move {
                    // Auto-notify readiness and continue with normal run()
                    ready.notify();
                    inner.run(cancel).await
                })
                .map_err(anyhow::Error::from)
        }
    }

    #[tracing::instrument(skip(self, external_cancel), level = "debug")]
    async fn stop(&self, external_cancel: CancellationToken) -> TaskResult<()> {
        tokio::select! {
            res = self.lc.stop(self.stop_timeout) => {
                let _ = res.map_err(anyhow::Error::from)?;
                Ok(())
            }
            _ = external_cancel.cancelled() => {
                let _ = self.lc.stop(Duration::from_millis(0)).await?;
                Ok(())
            }
        }
    }
}

impl<T: Runnable> Drop for WithLifecycle<T> {
    /// Best-effort, but only if we're the last owner of `lc` to avoid aborting someone else's task.
    fn drop(&mut self) {
        if Arc::strong_count(&self.lc) == 1 {
            if let Some(tok) = self.lc.cancel.lock().as_ref() {
                tok.cancel();
            }
            if let Some(handle) = self.lc.handle.lock().as_ref() {
                handle.abort();
            }
        }
    }
}

// ----- Tests -----------------------------------------------------------------

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering as AOrd};
    use tokio::time::{sleep, Duration};

    struct TestRunnable {
        counter: AtomicU32,
    }

    impl TestRunnable {
        fn new() -> Self {
            Self {
                counter: AtomicU32::new(0),
            }
        }
        fn count(&self) -> u32 {
            self.counter.load(AOrd::Relaxed)
        }
    }

    #[async_trait::async_trait]
    impl Runnable for TestRunnable {
        async fn run(self: Arc<Self>, cancel: CancellationToken) -> TaskResult<()> {
            let mut interval = tokio::time::interval(Duration::from_millis(10));
            loop {
                tokio::select! {
                    _ = interval.tick() => { self.counter.fetch_add(1, AOrd::Relaxed); }
                    _ = cancel.cancelled() => break,
                }
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn lifecycle_basic() {
        let lc = Arc::new(Lifecycle::new());
        assert_eq!(lc.status(), Status::Stopped);

        let result = lc.start(|cancel| async move {
            cancel.cancelled().await;
            Ok(())
        });
        assert!(result.is_ok());

        let stop_result = lc.stop(Duration::from_millis(100)).await;
        assert!(stop_result.is_ok());
        assert_eq!(lc.status(), Status::Stopped);
    }

    #[tokio::test]
    async fn with_lifecycle_wrapper_basics() {
        let runnable = TestRunnable::new();
        let wrapper = WithLifecycle::new(runnable);

        assert_eq!(wrapper.status(), Status::Stopped);
        assert_eq!(wrapper.inner().count(), 0);

        let wrapper = wrapper.with_stop_timeout(Duration::from_secs(60));
        assert_eq!(wrapper.stop_timeout.as_secs(), 60);
    }

    #[tokio::test]
    async fn start_sets_running_immediately() {
        let lc = Lifecycle::new();
        lc.start(|cancel| async move {
            cancel.cancelled().await;
            Ok(())
        })
        .unwrap();

        let s = lc.status();
        assert!(matches!(s, Status::Running | Status::Starting));

        let _ = lc.stop(Duration::from_millis(50)).await.unwrap();
        assert_eq!(lc.status(), Status::Stopped);
    }

    #[tokio::test]
    async fn start_with_ready_transitions_and_stop() {
        let lc = Lifecycle::new();

        let (ready_tx, ready_rx) = oneshot::channel::<()>();
        lc.start_with_ready(move |cancel, ready| async move {
            let _ = ready_rx.await;
            ready.notify();
            cancel.cancelled().await;
            Ok(())
        })
        .unwrap();

        assert_eq!(lc.status(), Status::Starting);

        let _ = ready_tx.send(());
        sleep(Duration::from_millis(10)).await;
        assert_eq!(lc.status(), Status::Running);

        let reason = lc.stop(Duration::from_millis(100)).await.unwrap();
        assert!(matches!(
            reason,
            StopReason::Cancelled | StopReason::Finished
        ));
        assert_eq!(lc.status(), Status::Stopped);
    }

    #[tokio::test]
    async fn stop_while_starting_before_ready() {
        let lc = Lifecycle::new();

        lc.start_with_ready(move |cancel, _ready| async move {
            cancel.cancelled().await;
            Ok(())
        })
        .unwrap();

        assert_eq!(lc.status(), Status::Starting);

        let reason = lc.stop(Duration::from_millis(100)).await.unwrap();
        assert!(matches!(
            reason,
            StopReason::Cancelled | StopReason::Finished
        ));
        assert_eq!(lc.status(), Status::Stopped);
    }

    #[tokio::test]
    async fn timeout_path_aborts_and_notifies() {
        let lc = Lifecycle::new();

        lc.start(|_cancel| async move {
            loop {
                sleep(Duration::from_secs(1000)).await;
            }
            #[allow(unreachable_code)]
            Ok::<_, anyhow::Error>(())
        })
        .unwrap();

        let reason = lc.stop(Duration::from_millis(30)).await.unwrap();
        assert_eq!(reason, StopReason::Timeout);
        assert_eq!(lc.status(), Status::Stopped);
    }

    #[tokio::test]
    async fn try_start_and_second_start_fails() {
        let lc = Lifecycle::new();

        assert!(lc.try_start(|cancel| async move {
            cancel.cancelled().await;
            Ok(())
        }));

        let err = lc.start(|_c| async { Ok(()) }).unwrap_err();
        match err {
            LifecycleError::AlreadyStarted => {}
        }

        let _ = lc.stop(Duration::from_millis(80)).await.unwrap();
        assert_eq!(lc.status(), Status::Stopped);
    }

    #[tokio::test]
    async fn stop_is_idempotent_and_safe_concurrent() {
        let lc = Arc::new(Lifecycle::new());

        lc.start(|cancel| async move {
            cancel.cancelled().await;
            Ok(())
        })
        .unwrap();

        let a = lc.clone();
        let b = lc.clone();
        let (r1, r2) = tokio::join!(
            async move { a.stop(Duration::from_millis(80)).await },
            async move { b.stop(Duration::from_millis(80)).await },
        );

        let r1 = r1.unwrap();
        let r2 = r2.unwrap();
        assert!(matches!(
            r1,
            StopReason::Finished | StopReason::Cancelled | StopReason::Timeout
        ));
        assert!(matches!(
            r2,
            StopReason::Finished | StopReason::Cancelled | StopReason::Timeout
        ));
        assert_eq!(lc.status(), Status::Stopped);
    }

    #[tokio::test]
    async fn stateful_wrapper_start_stop_roundtrip() {
        use crate::contracts::StatefulModule;

        let wrapper = WithLifecycle::new(TestRunnable::new());
        assert_eq!(wrapper.status(), Status::Stopped);

        wrapper.start(CancellationToken::new()).await.unwrap();
        assert!(wrapper.lc.is_running());

        wrapper.stop(CancellationToken::new()).await.unwrap();
        assert_eq!(wrapper.status(), Status::Stopped);
    }

    #[tokio::test]
    async fn with_lifecycle_double_start_fails() {
        use crate::contracts::StatefulModule;

        let wrapper = WithLifecycle::new(TestRunnable::new());
        let cancel = CancellationToken::new();
        wrapper.start(cancel.clone()).await.unwrap();
        let err = wrapper.start(cancel).await;
        assert!(err.is_err());
        wrapper.stop(CancellationToken::new()).await.unwrap();
    }

    #[tokio::test]
    async fn with_lifecycle_concurrent_stop_calls() {
        use crate::contracts::StatefulModule;
        let wrapper = Arc::new(WithLifecycle::new(TestRunnable::new()));
        wrapper.start(CancellationToken::new()).await.unwrap();
        let a = wrapper.clone();
        let b = wrapper.clone();
        let (r1, r2) = tokio::join!(
            async move { a.stop(CancellationToken::new()).await },
            async move { b.stop(CancellationToken::new()).await },
        );
        assert!(r1.is_ok());
        assert!(r2.is_ok());
        assert_eq!(wrapper.status(), Status::Stopped);
    }

    #[tokio::test]
    async fn lifecycle_handles_panics_properly() {
        let lc = Lifecycle::new();

        // Start a task that will panic
        lc.start(|_cancel| async {
            panic!("test panic message");
        })
        .unwrap();

        // Give the task a moment to start and panic
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Stop should handle the panic gracefully
        let reason = lc.stop(Duration::from_millis(1000)).await.unwrap();

        // The task panicked, but stop should complete successfully
        // The exact reason depends on timing, but it should not hang or fail
        assert!(matches!(
            reason,
            StopReason::Finished | StopReason::Cancelled | StopReason::Timeout
        ));
        assert_eq!(lc.status(), Status::Stopped);
    }

    #[tokio::test]
    async fn lifecycle_task_naming_and_logging() {
        let lc = Lifecycle::new();

        // Start a simple task
        lc.start(|cancel| async move {
            cancel.cancelled().await;
            Ok(())
        })
        .unwrap();

        // Verify task is running
        assert!(lc.is_running());

        // Stop and verify proper cleanup
        let reason = lc.stop(Duration::from_millis(100)).await.unwrap();
        assert!(matches!(
            reason,
            StopReason::Cancelled | StopReason::Finished
        ));
        assert_eq!(lc.status(), Status::Stopped);
    }

    #[tokio::test]
    async fn lifecycle_join_handles_all_tasks() {
        let lc = Arc::new(Lifecycle::new());

        // Start multiple tasks in sequence (lifecycle only supports one at a time)
        lc.start(|cancel| async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            cancel.cancelled().await;
            Ok(())
        })
        .unwrap();

        // Stop should wait for the task to complete
        let start = std::time::Instant::now();
        let reason = lc.stop(Duration::from_millis(200)).await.unwrap();
        let elapsed = start.elapsed();

        // Should have waited at least 10ms for the task
        assert!(elapsed >= Duration::from_millis(10));
        assert!(matches!(
            reason,
            StopReason::Cancelled | StopReason::Finished
        ));
        assert_eq!(lc.status(), Status::Stopped);
    }
}
