use anyhow::Result;
use tokio::signal;

/// Signals that can trigger shutdown.
enum ShutdownSignal {
    CtrlC,
    #[cfg(unix)]
    Sigterm,
}

/// Wait for termination signals (Ctrl+C, SIGTERM).
///
/// # Errors
/// Returns an error if signal handling fails.
pub async fn wait_for_shutdown() -> Result<()> {
    let _signal = tokio::select! {
        result = wait_ctrl_c() => result?,
        result = wait_sigterm() => result?,
    };

    tracing::info!("Shutdown signal received, initiating graceful shutdown");
    Ok(())
}

async fn wait_ctrl_c() -> Result<ShutdownSignal> {
    signal::ctrl_c().await.map_err(|e| {
        tracing::error!(%e, "Error handling Ctrl+C signal");
        e
    })?;
    tracing::info!("Received Ctrl+C signal");
    Ok(ShutdownSignal::CtrlC)
}

#[cfg(unix)]
async fn wait_sigterm() -> Result<ShutdownSignal> {
    let mut signal_handler =
        signal::unix::signal(signal::unix::SignalKind::terminate()).map_err(|e| {
            tracing::error!(%e, "Failed to install SIGTERM handler");
            e
        })?;
    signal_handler.recv().await;
    tracing::info!("Received SIGTERM signal");
    Ok(ShutdownSignal::Sigterm)
}

#[cfg(not(unix))]
async fn wait_sigterm() -> Result<ShutdownSignal> {
    std::future::pending::<Result<ShutdownSignal>>().await
}
