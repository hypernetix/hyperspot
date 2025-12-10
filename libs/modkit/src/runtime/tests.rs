#![cfg_attr(coverage_nightly, coverage(off))]
//! Tests for runtime lifecycle ready/timeout/cancel scenarios

use crate::lifecycle::{Lifecycle, Status, StopReason};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;

#[tokio::test]
async fn test_ready_signal_success() {
    let lc = Lifecycle::new();
    let ready_signaled = Arc::new(AtomicBool::new(false));
    let ready_signaled_clone = ready_signaled.clone();

    lc.start_with_ready(move |cancel, ready| {
        let ready_signaled = ready_signaled_clone.clone();
        async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            ready.notify();
            ready_signaled.store(true, Ordering::SeqCst);
            cancel.cancelled().await;
            Ok(())
        }
    })
    .unwrap();

    // Wait for ready signal
    tokio::time::timeout(Duration::from_millis(100), async {
        while lc.status() != Status::Running {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("Should become ready");

    assert!(ready_signaled.load(Ordering::SeqCst));
    assert_eq!(lc.status(), Status::Running);

    let reason = lc.stop(Duration::from_secs(1)).await.unwrap();
    assert!(matches!(
        reason,
        StopReason::Cancelled | StopReason::Finished
    ));
}

#[tokio::test]
async fn test_ready_timeout() {
    let lc = Lifecycle::new();

    // Start a task that never signals ready
    lc.start_with_ready(|cancel, _ready| async move {
        // Never call ready.notify()
        cancel.cancelled().await;
        Ok(())
    })
    .unwrap();

    // Should remain in Starting state since ready was never signaled
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(lc.status(), Status::Starting);

    // Stop should work even if never became ready
    let reason = lc.stop(Duration::from_millis(100)).await.unwrap();
    assert!(matches!(
        reason,
        StopReason::Cancelled | StopReason::Finished
    ));
}

#[tokio::test]
async fn test_cancel_before_ready() {
    let lc = Lifecycle::new();
    let cancel_received = Arc::new(AtomicBool::new(false));
    let cancel_received_clone = cancel_received.clone();

    lc.start_with_ready(move |cancel, ready| {
        let cancel_received = cancel_received_clone.clone();
        async move {
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    ready.notify();
                }
                _ = cancel.cancelled() => {
                    cancel_received.store(true, Ordering::SeqCst);
                }
            }
            Ok(())
        }
    })
    .unwrap();

    // Cancel immediately before ready signal
    tokio::time::sleep(Duration::from_millis(10)).await;
    let reason = lc.stop(Duration::from_millis(100)).await.unwrap();

    assert!(cancel_received.load(Ordering::SeqCst));
    assert!(matches!(
        reason,
        StopReason::Cancelled | StopReason::Finished
    ));
    assert_eq!(lc.status(), Status::Stopped);
}

#[tokio::test]
async fn test_timeout_during_stop() {
    let lc = Lifecycle::new();

    // Start a task that ignores cancellation
    lc.start(|_cancel| async move {
        // Ignore cancellation and block for a long time
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok(())
    })
    .unwrap();

    // Stop with a short timeout should result in timeout
    let reason = lc.stop(Duration::from_millis(50)).await.unwrap();
    assert_eq!(reason, StopReason::Timeout);
    assert_eq!(lc.status(), Status::Stopped);
}

#[tokio::test]
async fn test_graceful_cancel_with_cleanup() {
    let lc = Lifecycle::new();
    let cleanup_done = Arc::new(AtomicBool::new(false));
    let cleanup_done_clone = cleanup_done.clone();

    lc.start_with_ready(move |cancel, ready| {
        let cleanup_done = cleanup_done_clone.clone();
        async move {
            ready.notify();

            // Wait for cancellation
            cancel.cancelled().await;

            // Simulate cleanup work
            tokio::time::sleep(Duration::from_millis(10)).await;
            cleanup_done.store(true, Ordering::SeqCst);

            Ok(())
        }
    })
    .unwrap();

    // Wait for ready
    tokio::time::timeout(Duration::from_millis(100), async {
        while lc.status() != Status::Running {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("Should become ready");

    // Stop gracefully
    let reason = lc.stop(Duration::from_secs(1)).await.unwrap();

    assert!(cleanup_done.load(Ordering::SeqCst));
    assert!(matches!(
        reason,
        StopReason::Finished | StopReason::Cancelled
    ));
    assert_eq!(lc.status(), Status::Stopped);
}

#[tokio::test]
async fn test_ready_signal_single_use() {
    let lc = Lifecycle::new();
    let ready_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let ready_count_clone = ready_count.clone();

    lc.start_with_ready(move |cancel, ready| {
        let ready_count = ready_count_clone.clone();
        async move {
            // Signal ready once (ReadySignal can only be used once)
            ready.notify();
            ready_count.fetch_add(1, Ordering::SeqCst);

            tokio::time::sleep(Duration::from_millis(10)).await;
            ready_count.fetch_add(1, Ordering::SeqCst);

            cancel.cancelled().await;
            Ok(())
        }
    })
    .unwrap();

    // Wait for ready
    tokio::time::timeout(Duration::from_millis(100), async {
        while lc.status() != Status::Running {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("Should become ready");

    assert_eq!(lc.status(), Status::Running); // Ready signal worked

    let reason = lc.stop(Duration::from_secs(1)).await.unwrap();
    assert!(matches!(
        reason,
        StopReason::Cancelled | StopReason::Finished
    ));

    // After stop, both increments should have happened
    assert_eq!(ready_count.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn test_concurrent_stop_calls() {
    let lc = Arc::new(Lifecycle::new());

    lc.start(|cancel| async move {
        cancel.cancelled().await;
        Ok(())
    })
    .unwrap();

    let lc1 = lc.clone();
    let lc2 = lc.clone();
    let lc3 = lc.clone();

    // Multiple concurrent stop calls
    let (r1, r2, r3) = tokio::join!(
        lc1.stop(Duration::from_secs(1)),
        lc2.stop(Duration::from_secs(1)),
        lc3.stop(Duration::from_secs(1))
    );

    // All should succeed
    assert!(r1.is_ok());
    assert!(r2.is_ok());
    assert!(r3.is_ok());
    assert_eq!(lc.status(), Status::Stopped);
}

#[tokio::test]
async fn test_task_panic_handling() {
    let lc = Lifecycle::new();

    lc.start_with_ready(|_cancel, ready| async move {
        ready.notify();
        panic!("Test panic in lifecycle task");
    })
    .unwrap();

    // Wait for ready
    tokio::time::timeout(Duration::from_millis(100), async {
        while lc.status() != Status::Running {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("Should become ready despite upcoming panic");

    // Give the panic time to happen
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Stop should still work
    let reason = lc.stop(Duration::from_millis(100)).await.unwrap();
    assert!(matches!(reason, StopReason::Finished | StopReason::Timeout));
    assert_eq!(lc.status(), Status::Stopped);
}
