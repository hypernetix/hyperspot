#![allow(clippy::unwrap_used, clippy::expect_used)]

use anyhow::Result;
use async_trait::async_trait;
use modkit::contracts::StatefulModule;
use modkit::lifecycle::*;
use std::sync::Arc;
use std::time::Duration;
use tokio_util::sync::CancellationToken;

struct Tick;
#[async_trait]
impl Runnable for Tick {
    async fn run(self: Arc<Self>, cancel: CancellationToken) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_millis(5));
        loop {
            tokio::select! {
                _ = interval.tick() => {},
                () = cancel.cancelled() => break,
            }
        }
        Ok(())
    }
}

#[tokio::test]
async fn status_transitions_plain_start() {
    let lc = Lifecycle::new();
    assert_eq!(lc.status(), Status::Stopped);

    lc.start(|cancel| async move {
        cancel.cancelled().await;
        Ok(())
    })
    .unwrap();

    // Starting -> Running happens immediately in `start`.
    assert!(matches!(lc.status(), Status::Running | Status::Starting));
    // Stop should cancel and end with Cancelled
    let reason = lc.stop(Duration::from_secs(1)).await.unwrap();
    assert!(matches!(
        reason,
        StopReason::Cancelled | StopReason::Finished
    ));
    assert_eq!(lc.status(), Status::Stopped);
}

#[tokio::test]
async fn status_transitions_ready() {
    let lc = Lifecycle::new();
    assert_eq!(lc.status(), Status::Stopped);

    lc.start_with_ready(|cancel, ready| async move {
        // simulate binding ok
        ready.notify();
        cancel.cancelled().await;
        Ok(())
    })
    .unwrap();

    // Wait for Running after ready-notify
    tokio::time::timeout(Duration::from_secs(1), async {
        while lc.status() != Status::Running {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();

    let reason = lc.stop(Duration::from_secs(1)).await.unwrap();
    assert!(matches!(
        reason,
        StopReason::Cancelled | StopReason::Finished
    ));
    assert_eq!(lc.status(), Status::Stopped);
}

#[tokio::test]
async fn stop_without_start_is_idempotent() {
    let lc = Lifecycle::new();
    assert_eq!(lc.status(), Status::Stopped);
    let reason = lc.stop(Duration::from_millis(50)).await.unwrap();
    assert_eq!(reason, StopReason::Finished);
    assert_eq!(lc.status(), Status::Stopped);
}

#[tokio::test]
async fn timeout_path_aborts_task() {
    let lc = Lifecycle::new();
    // Task ignores cancel — it never awaits cancel.cancelled()
    lc.start(|_cancel| async move {
        // Block longer than timeout
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok(())
    })
    .unwrap();

    let reason = lc.stop(Duration::from_millis(20)).await.unwrap();
    assert_eq!(reason, StopReason::Timeout);
    assert_eq!(lc.status(), Status::Stopped);
}

#[tokio::test]
async fn concurrent_stops_are_safe() {
    let lc = Arc::new(Lifecycle::new());

    lc.start(|cancel| async move {
        cancel.cancelled().await;
        Ok(())
    })
    .unwrap();

    let lc1 = lc.clone();
    let lc2 = lc.clone();

    let (r1, r2) = tokio::join!(
        lc1.stop(Duration::from_secs(1)),
        lc2.stop(Duration::from_secs(1))
    );
    assert!(r1.is_ok() && r2.is_ok());
    assert_eq!(lc.status(), Status::Stopped);
}

#[tokio::test]
async fn external_cancellation_is_linked_through_withlifecycle() {
    // WithLifecycle should link external token to internal cancel
    let runnable = Arc::new(Tick);
    let module = WithLifecycle::new(Arc::try_unwrap(runnable).ok().unwrap());
    let external = CancellationToken::new();
    module.start(external.clone()).await.unwrap();
    // Cancel externally and expect graceful stop
    external.cancel();
    module.stop(CancellationToken::new()).await.unwrap();
    assert_eq!(module.status(), Status::Stopped);
}
