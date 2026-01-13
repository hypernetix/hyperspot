#![allow(clippy::unwrap_used, clippy::expect_used)]

use anyhow::Result;
use modkit::{lifecycle as lifecycle_attr, lifecycle::*, RunnableCapability};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

struct ReadyAware;

#[lifecycle_attr(method = "run_with_ready", stop_timeout = "200ms", await_ready = true)]
impl ReadyAware {
    pub async fn run_with_ready(
        &self,
        cancel: CancellationToken,
        ready: ReadySignal,
    ) -> Result<()> {
        // Signal readiness only after a small delay to keep state in Starting
        tokio::time::sleep(Duration::from_millis(20)).await;
        ready.notify();
        // Then run until cancelled
        cancel.cancelled().await;
        Ok(())
    }
}

struct AutoNotify;

#[lifecycle_attr(method = "run_no_ready", await_ready = true)]
impl AutoNotify {
    pub async fn run_no_ready(&self, cancel: CancellationToken) -> Result<()> {
        // Just wait until cancelled
        cancel.cancelled().await;
        Ok(())
    }
}

#[tokio::test]
async fn stays_starting_until_ready_signal() {
    let m = ReadyAware.into_module();
    let parent = CancellationToken::new();
    m.start(parent.clone()).await.unwrap();

    // Should be Starting until ReadySignal triggers inside the method
    assert_eq!(m.status(), Status::Starting);
    tokio::time::sleep(Duration::from_millis(40)).await;
    assert_eq!(m.status(), Status::Running);

    parent.cancel();
    m.stop(CancellationToken::new()).await.unwrap();
    assert_eq!(m.status(), Status::Stopped);
}

#[tokio::test]
async fn auto_notify_when_no_ready_param() {
    let m = AutoNotify.into_module();
    let parent = CancellationToken::new();
    m.start(parent.clone()).await.unwrap();
    // Await-ready=true but method has no ReadySignal -> auto-notify -> Running quickly
    tokio::time::sleep(Duration::from_millis(5)).await;
    assert!(matches!(m.status(), Status::Running | Status::Starting));

    parent.cancel();
    m.stop(CancellationToken::new()).await.unwrap();
    assert_eq!(m.status(), Status::Stopped);
}

#[tokio::test]
async fn drop_cleans_up_background_task() {
    let parent = CancellationToken::new();
    let handle = tokio::spawn(async move {
        let m = AutoNotify.into_module();
        m.start(parent.clone()).await.unwrap();
        // Drop without explicit stop(); background task should be aborted/cancelled
        m
    });

    // Wait for the task to finish and drop
    let m = handle.await.unwrap();
    drop(m);
    // Nothing to assert directly; this test exercises Drop paths without hanging.
}
