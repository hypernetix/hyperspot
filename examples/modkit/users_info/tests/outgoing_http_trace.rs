//! Integration tests verifying that outbound HTTP calls include W3C traceparent headers

use std::sync::Arc;
use uuid::Uuid;

use httpmock::prelude::*;
use modkit::TracedClient;
use url::Url;
use users_info::domain::error::DomainError;
use users_info::domain::ports::AuditPort;
use users_info::infra::audit::HttpAuditClient;

#[tokio::test]
async fn audit_get_includes_traceparent_header() {
    // Start mock HTTP server
    let server = MockServer::start();

    // Configure mock to expect traceparent header
    let mock = server.mock(|when, then| {
        when.method(GET)
            .path_matches(r"/api/user-access/[\w-]+")
            .header_exists("traceparent");
        then.status(200);
    });

    // Create adapter
    let traced_client = TracedClient::default();
    let audit_base = Url::parse(&server.base_url()).unwrap();
    let notify_base = Url::parse("http://localhost:9999").unwrap();
    let adapter: Arc<dyn AuditPort> =
        Arc::new(HttpAuditClient::new(traced_client, audit_base, notify_base));

    // Call the adapter
    let user_id = Uuid::new_v4();
    let result = adapter.get_user_access(user_id).await;

    // Verify request was made with traceparent header
    mock.assert();
    assert!(result.is_ok());
}

#[tokio::test]
async fn notification_post_includes_traceparent_header() {
    // Start mock HTTP server
    let server = MockServer::start();

    // Configure mock to expect traceparent header
    let mock = server.mock(|when, then| {
        when.method(POST)
            .path("/api/user-created")
            .header_exists("traceparent");
        then.status(200);
    });

    // Create adapter
    let traced_client = TracedClient::default();
    let audit_base = Url::parse("http://localhost:9998").unwrap();
    let notify_base = Url::parse(&server.base_url()).unwrap();
    let adapter: Arc<dyn AuditPort> =
        Arc::new(HttpAuditClient::new(traced_client, audit_base, notify_base));

    // Call the adapter
    let result = adapter.notify_user_created().await;

    // Verify request was made with traceparent header
    mock.assert();
    assert!(result.is_ok());
}

#[tokio::test]
async fn audit_get_error_surfaces_as_domain_error() {
    // Start mock HTTP server
    let server = MockServer::start();

    // Configure mock to return error
    let _mock = server.mock(|when, then| {
        when.method(GET).path_matches(r"/api/user-access/[\w-]+");
        then.status(500);
    });

    // Create adapter
    let traced_client = TracedClient::default();
    let audit_base = Url::parse(&server.base_url()).unwrap();
    let notify_base = Url::parse("http://localhost:9999").unwrap();
    let adapter: Arc<dyn AuditPort> =
        Arc::new(HttpAuditClient::new(traced_client, audit_base, notify_base));

    // Call the adapter
    let user_id = Uuid::new_v4();
    let result = adapter.get_user_access(user_id).await;

    // Verify error is surfaced as DomainError
    assert!(result.is_err());
    match result.unwrap_err() {
        DomainError::Validation { field, .. } => {
            assert_eq!(field, "user_access");
        }
        _ => panic!("Expected Validation error"),
    }
}

#[tokio::test]
async fn notification_post_error_surfaces_as_domain_error() {
    // Start mock HTTP server
    let server = MockServer::start();

    // Configure mock to return error
    let _mock = server.mock(|when, then| {
        when.method(POST).path("/api/user-created");
        then.status(503);
    });

    // Create adapter
    let traced_client = TracedClient::default();
    let audit_base = Url::parse("http://localhost:9998").unwrap();
    let notify_base = Url::parse(&server.base_url()).unwrap();
    let adapter: Arc<dyn AuditPort> =
        Arc::new(HttpAuditClient::new(traced_client, audit_base, notify_base));

    // Call the adapter
    let result = adapter.notify_user_created().await;

    // Verify error is surfaced as DomainError
    assert!(result.is_err());
    match result.unwrap_err() {
        DomainError::Validation { field, .. } => {
            assert_eq!(field, "notifications");
        }
        _ => panic!("Expected Validation error"),
    }
}
