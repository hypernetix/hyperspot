//! Integration test for the full outbound `OAuth2` client-credentials flow.
//!
//! Wires up: mock token endpoint → `Token::new()` → `HttpClient` with bearer
//! auth → mock downstream API → verifies the Authorization header is injected.

use std::time::Duration;

use httpmock::prelude::*;
use modkit_auth::HttpClientBuilderExt;
use modkit_auth::oauth2::config::OAuthClientConfig;
use modkit_auth::oauth2::token::Token;
use modkit_utils::SecretString;
use url::Url;

fn token_json(token: &str, expires_in: u64) -> String {
    format!(r#"{{"access_token":"{token}","expires_in":{expires_in},"token_type":"Bearer"}}"#)
}

/// Full round-trip: token acquisition + authenticated downstream call.
#[tokio::test]
async fn full_oauth2_bearer_flow() {
    // OAuth token endpoint
    let oauth_server = MockServer::start();
    let token_mock = oauth_server.mock(|when, then| {
        when.method(POST).path("/token");
        then.status(200)
            .header("content-type", "application/json")
            .body(token_json("integration-tok", 3600));
    });

    // Downstream API
    let api_server = MockServer::start();
    let api_mock = api_server.mock(|when, then| {
        when.method(GET)
            .path("/api/resource")
            .header("authorization", "Bearer integration-tok");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"status":"ok"}"#);
    });

    // Build token + client
    let config = OAuthClientConfig {
        token_endpoint: Some(
            Url::parse(&format!("http://localhost:{}/token", oauth_server.port())).unwrap(),
        ),
        client_id: "int-test-client".into(),
        client_secret: SecretString::new("int-test-secret"),
        http_config: Some(modkit_http::HttpClientConfig::for_testing()),
        jitter_max: Duration::from_millis(0),
        min_refresh_period: Duration::from_millis(100),
        ..Default::default()
    };

    let token = Token::new(config).await.unwrap();

    let client = modkit_http::HttpClientBuilder::new()
        .allow_insecure_http()
        .with_bearer_auth(token)
        .build()
        .unwrap();

    // Make downstream call
    let resp = client
        .get(&format!(
            "http://localhost:{}/api/resource",
            api_server.port()
        ))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "ok");

    // Assertions
    token_mock.assert_calls(1);
    api_mock.assert_calls(1);
}

/// Full round-trip with OIDC discovery: issuer URL → discover token
/// endpoint → acquire token → authenticated downstream call.
#[tokio::test]
async fn full_oauth2_with_oidc_discovery() {
    let server = MockServer::start();

    // OIDC discovery
    let token_ep = format!("http://localhost:{}/oauth/token", server.port());
    let discovery_mock = server.mock(|when, then| {
        when.method(GET).path("/.well-known/openid-configuration");
        then.status(200)
            .header("content-type", "application/json")
            .body(format!(r#"{{"token_endpoint":"{token_ep}"}}"#));
    });

    // Token endpoint
    let token_mock = server.mock(|when, then| {
        when.method(POST).path("/oauth/token");
        then.status(200)
            .header("content-type", "application/json")
            .body(token_json("disc-int-tok", 3600));
    });

    // Downstream API
    let api_server = MockServer::start();
    let api_mock = api_server.mock(|when, then| {
        when.method(GET)
            .path("/api/data")
            .header("authorization", "Bearer disc-int-tok");
        then.status(200)
            .header("content-type", "application/json")
            .body(r#"{"discovered":true}"#);
    });

    // Build token + client
    let config = OAuthClientConfig {
        issuer_url: Some(Url::parse(&format!("http://localhost:{}", server.port())).unwrap()),
        client_id: "disc-client".into(),
        client_secret: SecretString::new("disc-secret"),
        http_config: Some(modkit_http::HttpClientConfig::for_testing()),
        jitter_max: Duration::from_millis(0),
        min_refresh_period: Duration::from_millis(100),
        ..Default::default()
    };

    let token = Token::new(config).await.unwrap();

    let client = modkit_http::HttpClientBuilder::new()
        .allow_insecure_http()
        .with_bearer_auth(token)
        .build()
        .unwrap();

    let resp = client
        .get(&format!("http://localhost:{}/api/data", api_server.port()))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["discovered"], true);

    discovery_mock.assert_calls(1);
    token_mock.assert_calls(1);
    api_mock.assert_calls(1);
}
