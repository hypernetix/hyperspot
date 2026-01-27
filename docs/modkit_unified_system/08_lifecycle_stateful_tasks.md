# Lifecycle and Stateful Tasks

ModKit provides lifecycle management for modules with background tasks, graceful shutdown, and cancellation support.

## Core invariants

- **Rule**: Use `CancellationToken` for coordinated shutdown across the entire process tree.
- **Rule**: Use `WithLifecycle<T>` for stateful modules with background tasks.
- **Rule**: Pass child tokens to background tasks for cooperative shutdown.
- **Rule**: Implement `RunnableCapability` for custom lifecycle (rare).

## Declarative lifecycle with `#[modkit::module(...)]`

### Module with lifecycle

```rust
#[modkit::module(
    name = "api_gateway",
    capabilities = [rest_host, rest, stateful],
    lifecycle(entry = "serve", stop_timeout = "30s", await_ready)
)]
pub struct ApiGateway {
    /* ... */
}
```

### Lifecycle parameters

- `entry`: Method name to run as the background task
- `stop_timeout`: Graceful shutdown timeout (e.g., "30s", "1m")
- `await_ready`: Wait for ready signal before marking as Running

## WithLifecycle states and transitions

```
Stopped ── start() ── Starting ──(await_ready? then ready.notify())──▶ Running
   ▲                                  │
   │                                  └─ if await_ready = false → Running immediately
   └──────────── stop()/cancel ────────────────────────────────────────────────┘
```

## Lifecycle entry method

### Accepted signatures

```rust
// 1) Without ready signal
async fn serve(
    self: std::sync::Arc<Self>,
    cancel: tokio_util::sync::CancellationToken,
) -> anyhow::Result<()>

// 2) With ready signal
async fn serve(
    self: std::sync::Arc<Self>,
    cancel: tokio_util::sync::CancellationToken,
    ready: modkit::lifecycle::ReadySignal,
) -> anyhow::Result<()>
```

### Implementation example

```rust
impl ApiGateway {
    async fn serve(
        self: std::sync::Arc<Self>,
        cancel: tokio_util::sync::CancellationToken,
        ready: modkit::lifecycle::ReadySignal,
    ) -> anyhow::Result<()> {
        // Bind sockets/resources before flipping to Running
        let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
        let app = self.build_app();
        
        // Notify that we're ready
        ready.notify();
        
        // Run until cancelled
        axum::serve(listener, app)
            .with_graceful_shutdown(cancel.cancelled())
            .await?;
        
        Ok(())
    }
}
```

## CancellationToken usage

### Root token propagation

```rust
// In main()
let root_cancel = CancellationToken::new();
let mut runtime = ModuleRuntime::builder()
    .with_cancellation_token(root_cancel.clone())
    .build();

// Modules receive child tokens automatically
```

### Child tokens for background tasks

```rust
impl MyService {
    pub fn new(db: Arc<DbHandle>, cancel: CancellationToken) -> Self {
        let child = cancel.child_token();
        
        // Spawn background task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = child.cancelled() => break,
                    _ = tokio::time::sleep(Duration::from_secs(60)) => {
                        // Do periodic work
                    }
                }
            }
        });
        
        Self { db, cancel }
    }
}
```

### Cooperative shutdown

```rust
pub async fn process_with_shutdown(
    cancel: CancellationToken,
) -> Result<(), DomainError> {
    let mut tasks = Vec::new();
    
    for item in items {
        tokio::select! {
            result = process_item(item) => {
                tasks.push(result?);
            }
            _ = cancel.cancelled() => {
                // Graceful shutdown: return partial results
                return Ok(tasks);
            }
        }
    }
    
    Ok(tasks)
}
```

## Background task patterns

### Periodic task

```rust
pub struct PeriodicTask {
    cancel: CancellationToken,
}

impl PeriodicTask {
    pub fn spawn(cancel: CancellationToken) {
        let child = cancel.child_token();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                tokio::select! {
                    _ = child.cancelled() => break,
                    _ = interval.tick() => {
                        // Do periodic work
                        if let Err(e) = do_periodic_work().await {
                            tracing::error!("Periodic work failed: {}", e);
                        }
                    }
                }
            }
        });
    }
}
```

### Event listener

```rust
pub struct EventListener {
    receiver: broadcast::Receiver<Event>,
    cancel: CancellationToken,
}

impl EventListener {
    pub fn spawn(
        mut receiver: broadcast::Receiver<Event>,
        cancel: CancellationToken,
    ) {
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    event = receiver.recv() => {
                        match event {
                            Ok(event) => handle_event(event).await,
                            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                                tracing::warn!("Event listener lagged, skipped {} events", skipped);
                            }
                            Err(broadcast::error::RecvError::Closed) => break,
                        }
                    }
                }
            }
        });
    }
}
```

### Work queue processor

```rust
pub struct WorkQueueProcessor {
    queue: Arc<Mutex<VecDeque<WorkItem>>>,
    cancel: CancellationToken,
}

impl WorkQueueProcessor {
    pub fn spawn(
        queue: Arc<Mutex<VecDeque<WorkItem>>>,
        cancel: CancellationToken,
    ) {
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {
                        let work = {
                            let mut q = queue.lock().await;
                            q.pop_front()
                        };
                        
                        if let Some(work) = work {
                            if let Err(e) = process_work(work).await {
                                tracing::error!("Work processing failed: {}", e);
                            }
                        }
                    }
                }
            }
        });
    }
}
```

## Custom lifecycle (advanced)

### Implement RunnableCapability

```rust
use modkit::lifecycle::{RunnableCapability, RunnableHandle};

pub struct CustomModule {
    // Fields
}

#[async_trait]
impl RunnableCapability for CustomModule {
    async fn run(
        self: Arc<Self>,
        cancel: CancellationToken,
    ) -> Result<RunnableHandle, anyhow::Error> {
        let handle = tokio::spawn(async move {
            // Custom lifecycle logic
            self.run_with_cancel(cancel).await
        });
        
        Ok(RunnableHandle::new(handle))
    }
}
```

## Graceful shutdown patterns

### Clean shutdown sequence

```rust
impl MyModule {
    async fn shutdown_gracefully(&self) -> Result<(), DomainError> {
        // 1. Stop accepting new work
        self.accepting_work.store(false, Ordering::SeqCst);
        
        // 2. Wait for in-flight work to complete
        self.wait_for_in_flight().await?;
        
        // 3. Close connections
        self.close_connections().await?;
        
        // 4. Flush buffers
        self.flush_buffers().await?;
        
        Ok(())
    }
}
```

### Shutdown timeout handling

```rust
impl MyModule {
    async fn serve(
        self: Arc<Self>,
        cancel: CancellationToken,
        ready: ReadySignal,
    ) -> anyhow::Result<()> {
        // Setup
        self.setup().await?;
        ready.notify();
        
        // Main loop
        tokio::select! {
            result = self.run_main_loop() => result,
            _ = cancel.cancelled() => {
                // Graceful shutdown with timeout
                tokio::select! {
                    result = self.shutdown_gracefully() => result,
                    _ = tokio::time::sleep(Duration::from_secs(30)) => {
                        tracing::warn!("Graceful shutdown timeout, forcing exit");
                        Ok(())
                    }
                }
            }
        }
    }
}
```

## Testing lifecycle

### Test with manual cancellation

```rust
#[tokio::test]
async fn test_lifecycle_shutdown() {
    let cancel = CancellationToken::new();
    let module = Arc::new(MyModule::new());
    
    // Start the module
    let handle = tokio::spawn({
        let module = module.clone();
        let cancel = cancel.clone();
        async move {
            module.serve(cancel, ReadySignal::new()).await
        }
    });
    
    // Let it run a bit
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Trigger shutdown
    cancel.cancel();
    
    // Wait for graceful shutdown
    let result = handle.await.unwrap();
    assert!(result.is_ok());
}
```

### Test background task cancellation

```rust
#[tokio::test]
async fn test_background_task_cancellation() {
    let cancel = CancellationToken::new();
    let counter = Arc::new(AtomicU32::new(0));
    
    let task_counter = counter.clone();
    let task_cancel = cancel.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(10));
        
        loop {
            tokio::select! {
                _ = task_cancel.cancelled() => break,
                _ = interval.tick() => {
                    task_counter.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    });
    
    // Let it run a bit
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Cancel and verify it stops
    cancel.cancel();
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    let count = counter.load(Ordering::SeqCst);
    assert!(count > 0, "Task should have run");
    assert!(count < 20, "Task should have stopped quickly");
}
```

## Quick checklist

- [ ] Add `lifecycle(entry = "...")` to `#[modkit::module(...)]` for background tasks.
- [ ] Use `CancellationToken` for shutdown coordination.
- [ ] Pass child tokens to background tasks.
- [ ] Call `ready.notify()` after setup when using `await_ready`.
- [ ] Use `tokio::select!` for cooperative shutdown.
- [ ] Implement graceful shutdown with timeout handling.
- [ ] Test lifecycle with manual cancellation.
