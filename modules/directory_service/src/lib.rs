//! Directory Service - gRPC service for module instance discovery

use anyhow::Result;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use modkit::context::ModuleCtx;
use modkit::DirectoryApi;

mod client;
mod config;
mod server;

use client::DirectoryLocalClient;
use config::DirectoryServiceConfig;
use server::make_directory_service;

/// Directory service module - hosts the gRPC DirectoryService
#[modkit::module(
    name = "directory_service",
    capabilities = [stateful],
    client = modkit::DirectoryApi
)]
pub struct DirectoryServiceModule {
    config: RwLock<DirectoryServiceConfig>,
    directory_api: RwLock<Option<Arc<dyn DirectoryApi>>>,
    server_handle: RwLock<Option<tokio::task::JoinHandle<Result<()>>>>,
}

impl Default for DirectoryServiceModule {
    fn default() -> Self {
        Self {
            config: RwLock::new(DirectoryServiceConfig::default()),
            directory_api: RwLock::new(None),
            server_handle: RwLock::new(None),
        }
    }
}

#[async_trait]
impl modkit::Module for DirectoryServiceModule {
    async fn init(&self, ctx: &ModuleCtx) -> Result<()> {
        let cfg = ctx.config::<DirectoryServiceConfig>()?;
        *self.config.write().await = cfg;

        // Build DirectoryApi over the global InstanceDirectory
        let api_impl: Arc<dyn DirectoryApi> = Arc::new(DirectoryLocalClient::new());

        // Register in ClientHub using the generated helper function
        expose_directory_service_client(ctx, &api_impl)?;

        *self.directory_api.write().await = Some(api_impl);

        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl modkit::contracts::StatefulModule for DirectoryServiceModule {
    async fn start(&self, cancel: CancellationToken) -> Result<()> {
        let cfg = self.config.read().await.clone();
        let addr: SocketAddr = cfg.bind_addr.parse()?;

        let api = self
            .directory_api
            .read()
            .await
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("DirectoryApi not initialized"))?;

        let svc = make_directory_service(api);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("directory_service gRPC bound on {}", addr);

        let cancel_for_server = cancel.clone();
        let handle = tokio::spawn(async move {
            let shutdown = async move {
                cancel_for_server.cancelled().await;
                tracing::info!("directory_service shutting down");
            };

            tonic::transport::Server::builder()
                .add_service(svc)
                .serve_with_incoming_shutdown(
                    tokio_stream::wrappers::TcpListenerStream::new(listener),
                    shutdown,
                )
                .await
                .map_err(|e| anyhow::anyhow!("gRPC server error: {}", e))
        });

        *self.server_handle.write().await = Some(handle);

        Ok(())
    }

    async fn stop(&self, _cancel: CancellationToken) -> Result<()> {
        // Take the handle and drop the write guard before awaiting
        let handle = self.server_handle.write().await.take();

        if let Some(handle) = handle {
            // Wait for the server to finish shutting down
            handle
                .await
                .map_err(|e| anyhow::anyhow!("failed to join server task: {}", e))??;
        }
        Ok(())
    }
}
