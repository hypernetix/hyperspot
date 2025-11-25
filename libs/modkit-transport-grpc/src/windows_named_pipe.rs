//! Windows Named Pipe transport for gRPC servers.
//!
//! This module provides named pipe support for Tonic gRPC servers on Windows platforms.

use tokio::net::windows::named_pipe::ServerOptions;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;

/// Wrapper for NamedPipeServer that implements the Connected trait for Tonic.
pub struct NamedPipeConnection(pub(crate) tokio::net::windows::named_pipe::NamedPipeServer);

impl tokio::io::AsyncRead for NamedPipeConnection {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_read(cx, buf)
    }
}

impl tokio::io::AsyncWrite for NamedPipeConnection {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut self.0).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.0).poll_shutdown(cx)
    }
}

impl tonic::transport::server::Connected for NamedPipeConnection {
    type ConnectInfo = ();

    fn connect_info(&self) -> Self::ConnectInfo {
        // No extra connection info for named pipes
    }
}

/// Type alias for the incoming stream of named pipe connections.
pub type NamedPipeIncoming = ReceiverStream<std::io::Result<NamedPipeConnection>>;

/// Creates an incoming stream of named pipe connections.
///
/// This function spawns a background task that continuously accepts connections
/// on the specified named pipe. The task will exit when either:
/// - The cancellation token is triggered
/// - An error occurs creating or connecting to the pipe
/// - The receiver is dropped
///
/// # Arguments
///
/// * `pipe_name` - The name or full path of the named pipe (e.g., `\\.\pipe\my_pipe`)
/// * `cancel` - A cancellation token to signal shutdown
///
/// # Returns
///
/// A stream of `Result<NamedPipeConnection, std::io::Error>` that can be used
/// with Tonic's `serve_with_incoming_shutdown`.
pub fn create_named_pipe_incoming(
    pipe_name: String,
    cancel: CancellationToken,
) -> NamedPipeIncoming {
    let (tx, rx) = mpsc::channel::<std::io::Result<NamedPipeConnection>>(16);

    // Spawn an accept loop for named pipe clients
    tokio::spawn(async move {
        loop {
            if cancel.is_cancelled() {
                tracing::debug!(pipe_name = %pipe_name, "Named pipe accept loop cancelled");
                break;
            }

            let server = match ServerOptions::new().create(&pipe_name) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!(
                        pipe_name = %pipe_name,
                        error = %e,
                        "Failed to create named pipe server"
                    );
                    let _ = tx.send(Err(e)).await;
                    break;
                }
            };

            // Wait for a client to connect
            match server.connect().await {
                Ok(()) => {
                    tracing::trace!(pipe_name = %pipe_name, "Named pipe client connected");
                    if tx.send(Ok(NamedPipeConnection(server))).await.is_err() {
                        tracing::debug!(
                            pipe_name = %pipe_name,
                            "Named pipe receiver dropped, exiting accept loop"
                        );
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!(
                        pipe_name = %pipe_name,
                        error = %e,
                        "Failed to connect to named pipe client"
                    );
                    let _ = tx.send(Err(e)).await;
                    break;
                }
            }
        }
    });

    ReceiverStream::new(rx)
}
