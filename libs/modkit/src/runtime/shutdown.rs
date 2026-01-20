use anyhow::Result;

/// Wait for a shutdown signal (SIGTERM, SIGINT, or Ctrl+C).
///
/// # Errors
/// Returns an error if signal handling fails.
pub async fn wait_for_shutdown() -> Result<()> {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let mut sigterm = signal(SignalKind::terminate())?;
        let mut sigint = signal(SignalKind::interrupt())?; // Ctrl+C
        tokio::select! {
            _ = sigterm.recv() => {},
            _ = sigint.recv()  => {},
            _ = tokio::signal::ctrl_c() => {}, // fallback
        }
        Ok(())
    }

    #[cfg(windows)]
    {
        use tokio::signal::windows::{ctrl_break, ctrl_c, ctrl_close, ctrl_logoff, ctrl_shutdown};
        use tokio::time::{Duration, timeout};

        async fn arm_once() -> std::io::Result<()> {
            // create signal listeners first
            let mut c = ctrl_c()?;
            let mut br = ctrl_break()?;
            let mut cl = ctrl_close()?;
            let mut lo = ctrl_logoff()?;
            let mut sh = ctrl_shutdown()?;

            tokio::select! {
                _ = c.recv()  => {},
                _ = br.recv() => {},
                _ = cl.recv() => {},
                _ = lo.recv() => {},
                _ = sh.recv() => {},
            }
            Ok(())
        }

        // Debounce: if a “signal” fires within 50ms after arming, ignore first and wait again.
        match timeout(Duration::from_millis(50), arm_once()).await {
            Ok(Ok(())) => {
                tracing::warn!("shutdown: early Windows console signal detected; debouncing");
                arm_once().await?; // wait for a real one
            }
            Ok(Err(e)) => return Err(e.into()),
            Err(_elapsed) => {
                // no early signal — just wait normally
                arm_once().await?;
            }
        }
        Ok(())
    }
}
