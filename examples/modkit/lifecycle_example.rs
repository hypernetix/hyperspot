#![allow(clippy::use_debug)]

use anyhow::Result;
use async_trait::async_trait;
use modkit::contracts::StatefulModule;
use modkit::lifecycle::{Runnable, WithLifecycle};
use std::time::Duration;
use tokio_util::sync::CancellationToken;

/// Example server module that demonstrates the lifecycle framework
struct ExampleServer {
    port: u16,
    counter: std::sync::atomic::AtomicU32,
}

impl ExampleServer {
    fn new(port: u16) -> Self {
        Self {
            port,
            counter: std::sync::atomic::AtomicU32::new(0),
        }
    }

    fn get_counter(&self) -> u32 {
        self.counter.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// Implement Runnable for our server
/// This is the only async function you need to write!
#[async_trait]
impl Runnable for ExampleServer {
    async fn run(self: std::sync::Arc<Self>, cancel: CancellationToken) -> Result<()> {
        println!("Starting ExampleServer on port {}", self.port);

        // Simulate a server loop that processes requests
        let mut interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    // Simulate processing a request
                    let count = self.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if count.is_multiple_of(10) {
                        println!("Processed {} requests", count);
                    }
                }
                _ = cancel.cancelled() => {
                    println!("Shutdown signal received, stopping server");
                    break;
                }
            }
        }

        println!("Server stopped gracefully");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("Lifecycle Framework Example");
    println!("==============================\n");

    // Create our server
    let server = ExampleServer::new(8080);

    // Wrap it with WithLifecycle to get StatefulModule implementation
    let module = WithLifecycle::new(server).with_stop_timeout(Duration::from_secs(5)); // 5 second timeout

    println!("Module status: {:?}", module.status());

    // Create a cancellation token for external control
    let cancel_token = CancellationToken::new();

    // Start the module
    println!("Starting module...");
    module.start(cancel_token.clone()).await?;

    // Give it some time to run
    println!("Letting server run for 2 seconds...");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Check the counter
    println!("Requests processed: {}", module.inner().get_counter());

    // Stop the module
    println!("Stopping module...");
    module.stop(cancel_token.clone()).await?;

    println!("Final module status: {:?}", module.status());
    println!("Final request count: {}", module.inner().get_counter());

    println!("\nExample completed successfully!");
    Ok(())
}
