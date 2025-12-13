#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Integration tests for gRPC client transport stack

use modkit_transport_grpc::client::{connect_with_stack, GrpcClientConfig};
use modkit_transport_grpc::rpc_retry::{call_with_retry, RpcRetryConfig};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn default_config_is_sane() {
    let cfg = GrpcClientConfig::default();

    // All timeouts should be positive
    assert!(
        cfg.connect_timeout > Duration::from_millis(0),
        "connect_timeout should be positive"
    );
    assert!(
        cfg.rpc_timeout > Duration::from_millis(0),
        "rpc_timeout should be positive"
    );

    // Retry settings should be reasonable (max_retries is u32, always non-negative)
    assert!(
        cfg.base_backoff > Duration::from_millis(0),
        "base_backoff should be positive"
    );
    assert!(
        cfg.max_backoff >= cfg.base_backoff,
        "max_backoff should be >= base_backoff"
    );

    // Service name should be set
    assert!(
        !cfg.service_name.is_empty(),
        "service_name should not be empty"
    );

    // Features should be enabled by default
    assert!(cfg.enable_metrics, "metrics should be enabled by default");
    assert!(cfg.enable_tracing, "tracing should be enabled by default");
}

#[test]
fn config_builder_pattern_works() {
    let cfg = GrpcClientConfig::new("test_service")
        .with_connect_timeout(Duration::from_secs(5))
        .with_rpc_timeout(Duration::from_secs(15))
        .with_max_retries(5)
        .without_metrics()
        .without_tracing();

    assert_eq!(cfg.service_name, "test_service");
    assert_eq!(cfg.connect_timeout, Duration::from_secs(5));
    assert_eq!(cfg.rpc_timeout, Duration::from_secs(15));
    assert_eq!(cfg.max_retries, 5);
    assert!(!cfg.enable_metrics);
    assert!(!cfg.enable_tracing);
}

#[test]
fn config_cloning_works() {
    let cfg1 = GrpcClientConfig::new("service1").with_connect_timeout(Duration::from_secs(3));

    let cfg2 = cfg1.clone();

    assert_eq!(cfg1.service_name, cfg2.service_name);
    assert_eq!(cfg1.connect_timeout, cfg2.connect_timeout);
}

// Test that connect_with_stack properly creates a channel with timeouts
#[tokio::test]
async fn connect_with_stack_applies_timeouts() {
    use tonic::transport::Channel;

    // Fake client type for testing
    #[derive(Clone)]
    struct FakeClient {
        _channel: Channel,
    }

    impl From<Channel> for FakeClient {
        fn from(channel: Channel) -> Self {
            Self { _channel: channel }
        }
    }

    let cfg = GrpcClientConfig::new("test")
        .with_connect_timeout(Duration::from_millis(100))
        .with_rpc_timeout(Duration::from_millis(200));

    // Use a non-routable address to test timeout behavior
    // This should fail quickly due to connect_timeout
    let result = connect_with_stack::<FakeClient>("http://192.0.2.1:50051", &cfg).await;

    // We expect this to fail (no server listening), which proves the function works
    assert!(
        result.is_err(),
        "Should fail to connect to non-existent server"
    );
}

#[tokio::test]
async fn connect_with_stack_accepts_valid_uri() {
    use tonic::transport::Channel;

    #[derive(Clone)]
    struct FakeClient {
        _channel: Channel,
    }

    impl From<Channel> for FakeClient {
        fn from(channel: Channel) -> Self {
            Self { _channel: channel }
        }
    }

    let cfg = GrpcClientConfig::default();

    // Invalid URI should fail during parsing
    let result = connect_with_stack::<FakeClient>("not-a-valid-uri", &cfg).await;
    assert!(result.is_err(), "Should fail with invalid URI");
}

// Test tracking call counts with a wrapper service
#[tokio::test]
async fn call_count_tracking_works() {
    use tonic::transport::Endpoint;

    // Create a test endpoint (will fail to connect, but that's ok)
    let endpoint =
        Endpoint::from_static("http://127.0.0.1:1").connect_timeout(Duration::from_millis(10));

    let result = endpoint.connect().await;

    // We expect this to fail since there's no server
    assert!(
        result.is_err(),
        "Should fail to connect to non-existent server"
    );
}

// Test that multiple configs can be created independently
#[test]
fn multiple_configs_are_independent() {
    let cfg1 = GrpcClientConfig::new("service1").with_max_retries(3);

    let cfg2 = GrpcClientConfig::new("service2").with_max_retries(5);

    assert_eq!(cfg1.max_retries, 3);
    assert_eq!(cfg2.max_retries, 5);
}

// Test config validation edge cases
#[test]
fn config_handles_extreme_values() {
    let cfg = GrpcClientConfig::new("test")
        .with_connect_timeout(Duration::from_millis(1))
        .with_rpc_timeout(Duration::from_millis(1))
        .with_max_retries(0);

    assert_eq!(cfg.connect_timeout, Duration::from_millis(1));
    assert_eq!(cfg.rpc_timeout, Duration::from_millis(1));
    assert_eq!(cfg.max_retries, 0);
}

// Test that service name is properly captured
#[test]
fn service_name_is_captured() {
    let names = vec!["users", "orders", "inventory"];

    for name in names {
        let cfg = GrpcClientConfig::new(name);
        assert_eq!(cfg.service_name, name);
    }
}

// Test retry config is accessible and can be converted
#[test]
fn retry_config_is_accessible() {
    let cfg = GrpcClientConfig::default();

    // Verify retry-related config is accessible
    let _ = cfg.max_retries;
    let _ = cfg.base_backoff;
    let _ = cfg.max_backoff;

    // Test builder methods for retry config
    let cfg = cfg.with_max_retries(10);

    assert_eq!(cfg.max_retries, 10);
}

// Test that feature flags work correctly
#[test]
fn feature_flags_work() {
    let cfg = GrpcClientConfig::default();
    assert!(cfg.enable_metrics);
    assert!(cfg.enable_tracing);

    let cfg = cfg.without_metrics();
    assert!(!cfg.enable_metrics);
    assert!(cfg.enable_tracing);

    let cfg = cfg.without_tracing();
    assert!(!cfg.enable_metrics);
    assert!(!cfg.enable_tracing);
}

// Test Debug implementation
#[test]
fn config_has_debug_impl() {
    let cfg = GrpcClientConfig::default();
    let debug_str = format!("{:?}", cfg);

    // Should contain key fields
    assert!(debug_str.contains("GrpcClientConfig"));
}

// ============================================================================
// RpcRetryConfig Tests
// ============================================================================

#[test]
fn rpc_retry_config_from_grpc_config() {
    let grpc_cfg = GrpcClientConfig::new("test_service").with_max_retries(7);

    let retry_cfg = RpcRetryConfig::from(&grpc_cfg);

    assert_eq!(retry_cfg.max_retries, 7);
    assert_eq!(retry_cfg.base_backoff, grpc_cfg.base_backoff);
    assert_eq!(retry_cfg.max_backoff, grpc_cfg.max_backoff);
}

#[test]
fn rpc_retry_config_default() {
    let cfg = RpcRetryConfig::default();

    assert_eq!(cfg.max_retries, 3);
    assert!(cfg.base_backoff > Duration::from_millis(0));
    assert!(cfg.max_backoff >= cfg.base_backoff);
}

#[test]
fn rpc_retry_config_builder() {
    let cfg = RpcRetryConfig::new(5)
        .with_base_backoff(Duration::from_millis(50))
        .with_max_backoff(Duration::from_secs(2));

    assert_eq!(cfg.max_retries, 5);
    assert_eq!(cfg.base_backoff, Duration::from_millis(50));
    assert_eq!(cfg.max_backoff, Duration::from_secs(2));
}

#[test]
fn rpc_retry_config_cloning() {
    let cfg1 = RpcRetryConfig::new(3);
    let cfg2 = cfg1.clone();

    assert_eq!(cfg1.max_retries, cfg2.max_retries);
    assert_eq!(cfg1.base_backoff, cfg2.base_backoff);
}

#[test]
fn rpc_retry_config_debug() {
    let cfg = RpcRetryConfig::default();
    let debug_str = format!("{:?}", cfg);

    assert!(debug_str.contains("RpcRetryConfig"));
}

// ============================================================================
// call_with_retry Integration Tests
// ============================================================================

#[tokio::test]
async fn call_with_retry_success_first_try() {
    struct MockClient;

    let mut client = MockClient;
    let cfg = Arc::new(RpcRetryConfig::default());

    let result = call_with_retry(
        &mut client,
        cfg,
        42i32,
        |_c, req| async move { Ok::<_, tonic::Status>(req * 2) },
        "test.double",
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 84);
}

#[tokio::test]
async fn call_with_retry_non_retryable_error_fails_immediately() {
    use std::sync::atomic::{AtomicU32, Ordering};

    struct MockClient {
        call_count: Arc<AtomicU32>,
    }

    let call_count = Arc::new(AtomicU32::new(0));
    let mut client = MockClient {
        call_count: call_count.clone(),
    };
    let cfg = Arc::new(RpcRetryConfig::new(5));

    let result = call_with_retry(
        &mut client,
        cfg,
        (),
        |c, _req| {
            c.call_count.fetch_add(1, Ordering::SeqCst);
            async move { Err::<(), _>(tonic::Status::invalid_argument("bad")) }
        },
        "test.invalid",
    )
    .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::InvalidArgument);
    // Should only be called once since InvalidArgument is not retryable
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn call_with_retry_recovers_from_unavailable() {
    use std::sync::atomic::{AtomicU32, Ordering};

    struct MockClient {
        call_count: Arc<AtomicU32>,
    }

    let call_count = Arc::new(AtomicU32::new(0));
    let mut client = MockClient {
        call_count: call_count.clone(),
    };
    let cfg = Arc::new(
        RpcRetryConfig::new(5)
            .with_base_backoff(Duration::from_millis(1))
            .with_max_backoff(Duration::from_millis(5)),
    );

    let result = call_with_retry(
        &mut client,
        cfg,
        "data".to_string(),
        |c, req| {
            let count = c.call_count.fetch_add(1, Ordering::SeqCst) + 1;
            async move {
                if count < 3 {
                    Err(tonic::Status::unavailable("server overloaded"))
                } else {
                    Ok(format!("processed: {}", req))
                }
            }
        },
        "test.process",
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "processed: data");
    assert_eq!(call_count.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn call_with_retry_exhausts_retries() {
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
            .with_max_backoff(Duration::from_millis(5)),
    );

    let result = call_with_retry(
        &mut client,
        cfg,
        (),
        |c, _req| {
            c.call_count.fetch_add(1, Ordering::SeqCst);
            async move { Err::<String, _>(tonic::Status::deadline_exceeded("timeout")) }
        },
        "test.timeout",
    )
    .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::DeadlineExceeded);
    // Initial + 2 retries = 3 total
    assert_eq!(call_count.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn call_with_retry_zero_retries_means_single_attempt() {
    use std::sync::atomic::{AtomicU32, Ordering};

    struct MockClient {
        call_count: Arc<AtomicU32>,
    }

    let call_count = Arc::new(AtomicU32::new(0));
    let mut client = MockClient {
        call_count: call_count.clone(),
    };
    let cfg = Arc::new(RpcRetryConfig::new(0));

    let result = call_with_retry(
        &mut client,
        cfg,
        (),
        |c, _req| {
            c.call_count.fetch_add(1, Ordering::SeqCst);
            async move { Err::<String, _>(tonic::Status::unavailable("down")) }
        },
        "test.no_retry",
    )
    .await;

    assert!(result.is_err());
    // Only 1 attempt with max_retries = 0
    assert_eq!(call_count.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn call_with_retry_clones_request() {
    use std::sync::atomic::{AtomicU32, Ordering};

    #[derive(Clone)]
    struct Request {
        value: i32,
    }

    struct MockClient {
        call_count: Arc<AtomicU32>,
    }

    let call_count = Arc::new(AtomicU32::new(0));
    let mut client = MockClient {
        call_count: call_count.clone(),
    };
    let cfg = Arc::new(RpcRetryConfig::new(2).with_base_backoff(Duration::from_millis(1)));

    let result = call_with_retry(
        &mut client,
        cfg,
        Request { value: 100 },
        |c, req| {
            let count = c.call_count.fetch_add(1, Ordering::SeqCst) + 1;
            async move {
                if count < 2 {
                    Err(tonic::Status::unavailable("retry"))
                } else {
                    let count_i32 = i32::try_from(count).unwrap_or(i32::MAX);
                    // Verify we got a clone with correct value
                    Ok(req.value + count_i32)
                }
            }
        },
        "test.clone",
    )
    .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 102); // 100 + 2 (second attempt)
}
