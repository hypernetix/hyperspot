use anyhow::Result;
use tokio::signal;

/// Wait for termination signals (Ctrl+C, SIGTERM)
pub async fn wait_for_shutdown() -> Result<()> {
    let ctrl_c = async {
        if let Err(e) = signal::ctrl_c().await {
            tracing::error!(%e, "Failed to install Ctrl+C handler");
            return Err(e);
        }
        Ok(())
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut signal_handler) => {
                signal_handler.recv().await;
                Ok(())
            }
            Err(e) => {
                tracing::error!(%e, "Failed to install SIGTERM handler");
                Err(e)
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = async { Ok::<(), std::io::Error>(()) };

    tokio::select! {
        result = ctrl_c => {
            match result {
                Ok(_) => tracing::info!("Received Ctrl+C signal"),
                Err(e) => {
                    tracing::error!(%e, "Error handling Ctrl+C signal");
                    return Err(e.into());
                }
            }
        },
        result = terminate => {
            match result {
                Ok(_) => tracing::info!("Received SIGTERM signal"),
                Err(e) => {
                    tracing::error!(%e, "Error handling SIGTERM signal");
                    return Err(e.into());
                }
            }
        },
    }

    tracing::info!("Shutdown signal received, initiating graceful shutdown");
    Ok(())
}
