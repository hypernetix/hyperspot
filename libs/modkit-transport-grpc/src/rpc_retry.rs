//! RPC-level retry helper for unary gRPC calls.
//!
//! This module provides a generic retry helper [`call_with_retry`] that implements
//! safe retries with exponential backoff for unary gRPC calls.
//!
//! ## Retry Policy
//!
//! The helper retries only on transient, network-related errors:
//! - [`tonic::Code::Unavailable`] - Server is temporarily unavailable
//! - [`tonic::Code::DeadlineExceeded`] - Request timed out
//!
//! All other error codes are considered non-retryable and will be returned immediately.
//!
//! ## Idempotency Warning
//!
//! **This helper assumes the operation is idempotent.** Non-idempotent operations
//! (e.g., creating resources without deduplication) should **not** use this helper,
//! as retries may cause duplicate side effects.
//!
//! ## Example
//!
//! ```ignore
//! use modkit_transport_grpc::client::{connect_with_stack, GrpcClientConfig};
//! use modkit_transport_grpc::rpc_retry::{call_with_retry, RpcRetryConfig};
//! use std::sync::Arc;
//!
//! let cfg = GrpcClientConfig::new("my_service");
//! let retry_cfg = Arc::new(RpcRetryConfig::from(&cfg));
//! let mut client: MyServiceClient<Channel> = connect_with_stack(
//!     "http://127.0.0.1:50051",
//!     &cfg
//! ).await?;
//!
//! let req = MyRequest { /* ... */ };
//! let resp = call_with_retry(
//!     &mut client,
//!     retry_cfg.clone(),
//!     req,
//!     |c, r| async move { c.my_call(r).await.map(|resp| resp.into_inner()) },
//!     "my_service.my_call",
//! ).await?;
//! ```

use std::sync::Arc;
use std::time::Duration;

use tokio::time::sleep;
use tonic::{Code, Status};
use tracing::Instrument;

fn duration_to_i64_ms(duration: Duration) -> i64 {
    i64::try_from(duration.as_millis()).unwrap_or(i64::MAX)
}

/// Configuration for RPC-level retry policy.
///
/// This struct extracts retry-related settings from [`crate::client::GrpcClientConfig`]
/// for use with [`call_with_retry`].
#[derive(Debug, Clone)]
#[must_use]
pub struct RpcRetryConfig {
    /// Maximum number of retry attempts (not including the initial call).
    pub max_retries: u32,

    /// Base duration for exponential backoff.
    ///
    /// The actual backoff duration is `base_backoff * attempt_number`,
    /// capped at `max_backoff`.
    pub base_backoff: Duration,

    /// Maximum duration for exponential backoff.
    pub max_backoff: Duration,
}

impl Default for RpcRetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(5),
        }
    }
}

impl From<&crate::client::GrpcClientConfig> for RpcRetryConfig {
    fn from(cfg: &crate::client::GrpcClientConfig) -> Self {
        Self {
            max_retries: cfg.max_retries,
            base_backoff: cfg.base_backoff,
            max_backoff: cfg.max_backoff,
        }
    }
}

impl RpcRetryConfig {
    /// Create a new retry configuration with the given maximum retries.
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    /// Set the base backoff duration.
    pub fn with_base_backoff(mut self, duration: Duration) -> Self {
        self.base_backoff = duration;
        self
    }

    /// Set the maximum backoff duration.
    pub fn with_max_backoff(mut self, duration: Duration) -> Self {
        self.max_backoff = duration;
        self
    }
}

/// Generic helper for unary gRPC calls with retries.
///
/// Executes a gRPC call and retries on transient errors (`UNAVAILABLE`, `DEADLINE_EXCEEDED`)
/// with exponential backoff.
///
/// # Arguments
///
/// * `client` - The tonic gRPC client instance (e.g., `MyServiceClient<Channel>`)
/// * `cfg` - Shared retry configuration
/// * `req` - Request payload (must implement `Clone` for retry attempts)
/// * `call` - Closure that performs the actual RPC call
/// * `op_name` - Static name of the operation for logging/tracing (e.g., `"my_service.my_method"`)
///
/// # Type Parameters
///
/// * `TClient` - The gRPC client type
/// * `F` - The closure type that performs the RPC call
/// * `Fut` - The future returned by the closure
/// * `Req` - The request type (must be `Clone`)
/// * `Res` - The response type
///
/// # Returns
///
/// Returns `Ok(Res)` on success, or `Err(Status)` if all retry attempts fail
/// or a non-retryable error is encountered.
///
/// # Example
///
/// ```ignore
/// let resp = call_with_retry(
///     &mut client,
///     retry_cfg.clone(),
///     my_request,
///     |c, r| async move { c.get_user(r).await.map(|r| r.into_inner()) },
///     "users.get_user",
/// ).await?;
/// ```
///
/// # Errors
/// Returns `Status` error if the RPC fails after all retry attempts.
pub async fn call_with_retry<TClient, F, Fut, Req, Res>(
    client: &mut TClient,
    cfg: Arc<RpcRetryConfig>,
    req: Req,
    call: F,
    op_name: &'static str,
) -> Result<Res, Status>
where
    F: Fn(&mut TClient, Req) -> Fut,
    Fut: std::future::Future<Output = Result<Res, Status>>,
    Req: Clone,
{
    let mut attempt: u32 = 0;

    loop {
        attempt += 1;

        let span = tracing::debug_span!("grpc_call", op = op_name, attempt,);

        let result = async {
            let res = call(client, req.clone()).await;
            if let Err(ref status) = res {
                tracing::warn!(
                    code = ?status.code(),
                    message = %status.message(),
                    attempt,
                    op = op_name,
                    "gRPC call failed",
                );
            }
            res
        }
        .instrument(span)
        .await;

        match result {
            Ok(res) => {
                if attempt > 1 {
                    tracing::info!(op = op_name, attempt, "gRPC call succeeded after retries");
                }
                return Ok(res);
            }
            Err(status) => {
                let code = status.code();

                // Retry only on network-like errors
                let retryable = matches!(code, Code::Unavailable | Code::DeadlineExceeded);

                if !retryable || attempt > cfg.max_retries {
                    tracing::error!(
                        op = op_name,
                        attempt,
                        code = ?code,
                        "gRPC call giving up"
                    );
                    return Err(status);
                }

                // Exponential backoff with upper bound
                let mut backoff = cfg.base_backoff * attempt;
                if backoff > cfg.max_backoff {
                    backoff = cfg.max_backoff;
                }

                tracing::debug!(
                    op = op_name,
                    attempt,
                    backoff_ms = duration_to_i64_ms(backoff),
                    "Retrying gRPC call after backoff"
                );

                sleep(backoff).await;
            }
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn test_default_retry_config() {
        let cfg = RpcRetryConfig::default();
        assert_eq!(cfg.max_retries, 3);
        assert_eq!(cfg.base_backoff, Duration::from_millis(100));
        assert_eq!(cfg.max_backoff, Duration::from_secs(5));
    }

    #[test]
    fn test_retry_config_from_grpc_config() {
        let grpc_cfg = crate::client::GrpcClientConfig::new("test").with_max_retries(5);
        let retry_cfg = RpcRetryConfig::from(&grpc_cfg);

        assert_eq!(retry_cfg.max_retries, 5);
        assert_eq!(retry_cfg.base_backoff, grpc_cfg.base_backoff);
        assert_eq!(retry_cfg.max_backoff, grpc_cfg.max_backoff);
    }

    #[test]
    fn test_retry_config_builder() {
        let cfg = RpcRetryConfig::new(10)
            .with_base_backoff(Duration::from_millis(200))
            .with_max_backoff(Duration::from_secs(10));

        assert_eq!(cfg.max_retries, 10);
        assert_eq!(cfg.base_backoff, Duration::from_millis(200));
        assert_eq!(cfg.max_backoff, Duration::from_secs(10));
    }

    #[tokio::test]
    async fn test_call_with_retry_succeeds_first_attempt() {
        struct MockClient;

        let mut client = MockClient;
        let cfg = Arc::new(RpcRetryConfig::default());

        let result = call_with_retry(
            &mut client,
            cfg,
            "test_request".to_owned(),
            |_c, req| async move { Ok::<_, Status>(format!("response: {req}")) },
            "test.op",
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "response: test_request");
    }

    #[tokio::test]
    async fn test_call_with_retry_non_retryable_error() {
        struct MockClient;

        let mut client = MockClient;
        let cfg = Arc::new(RpcRetryConfig::new(3));

        let result = call_with_retry(
            &mut client,
            cfg,
            (),
            |_c, _req| async move { Err::<String, _>(Status::invalid_argument("bad request")) },
            "test.op",
        )
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), Code::InvalidArgument);
    }

    #[tokio::test]
    async fn test_call_with_retry_retries_on_unavailable() {
        use std::sync::atomic::{AtomicU32, Ordering};

        struct MockClient {
            call_count: Arc<AtomicU32>,
        }

        let call_count = Arc::new(AtomicU32::new(0));
        let mut client = MockClient {
            call_count: call_count.clone(),
        };

        let cfg = Arc::new(
            RpcRetryConfig::new(3)
                .with_base_backoff(Duration::from_millis(1))
                .with_max_backoff(Duration::from_millis(10)),
        );

        let result = call_with_retry(
            &mut client,
            cfg,
            (),
            |c, _req| {
                let count = c.call_count.fetch_add(1, Ordering::SeqCst) + 1;
                async move {
                    if count < 3 {
                        Err(Status::unavailable("temporarily unavailable"))
                    } else {
                        Ok("success".to_owned())
                    }
                }
            },
            "test.op",
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_call_with_retry_gives_up_after_max_retries() {
        use std::sync::atomic::{AtomicU32, Ordering};

        struct MockClient {
            call_count: Arc<AtomicU32>,
        }

        let call_count = Arc::new(AtomicU32::new(0));
        let mut client = MockClient {
            call_count: call_count.clone(),
        };

        let cfg = Arc::new(
            RpcRetryConfig::new(2)
                .with_base_backoff(Duration::from_millis(1))
                .with_max_backoff(Duration::from_millis(10)),
        );

        let result = call_with_retry(
            &mut client,
            cfg,
            (),
            |c, _req| {
                c.call_count.fetch_add(1, Ordering::SeqCst);
                async move { Err::<String, _>(Status::unavailable("always unavailable")) }
            },
            "test.op",
        )
        .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), Code::Unavailable);
        // Initial attempt + 2 retries = 3 total calls
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_call_with_retry_respects_max_backoff() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::time::Instant;

        struct MockClient {
            call_count: Arc<AtomicU32>,
        }

        let call_count = Arc::new(AtomicU32::new(0));
        let mut client = MockClient {
            call_count: call_count.clone(),
        };

        // Set base_backoff high enough that without max_backoff cap,
        // total time would be much longer
        let cfg = Arc::new(
            RpcRetryConfig::new(2)
                .with_base_backoff(Duration::from_millis(100))
                .with_max_backoff(Duration::from_millis(50)),
        );

        let start = Instant::now();
        let _ = call_with_retry(
            &mut client,
            cfg,
            (),
            |c, _req| {
                c.call_count.fetch_add(1, Ordering::SeqCst);
                async move { Err::<String, _>(Status::unavailable("unavailable")) }
            },
            "test.op",
        )
        .await;
        let elapsed = start.elapsed();

        // With max_backoff of 50ms and 2 retries, total backoff should be ~100ms max
        // (50ms + 50ms, since both attempts would hit the cap)
        // Without cap: 100ms + 200ms = 300ms
        assert!(
            elapsed < Duration::from_millis(200),
            "Backoff should be capped; elapsed: {elapsed:?}"
        );
    }
}
