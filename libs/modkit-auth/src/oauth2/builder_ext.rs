use http::header::HeaderName;
use tower::ServiceExt;

use super::layer::BearerAuthLayer;
use super::token::Token;

/// Extension trait for adding bearer auth to [`modkit_http::HttpClientBuilder`].
///
/// # Example
///
/// ```ignore
/// use modkit_auth::HttpClientBuilderExt;
///
/// let token = Token::new(config).await?;
/// let client = HttpClientBuilder::new()
///     .with_bearer_auth(token)
///     .build()?;
/// ```
pub trait HttpClientBuilderExt {
    /// Add `Authorization: Bearer <token>` injection to the HTTP client.
    #[must_use]
    fn with_bearer_auth(self, token: Token) -> Self;

    /// Add `<header_name>: Bearer <token>` injection to the HTTP client.
    #[must_use]
    fn with_bearer_auth_header(self, token: Token, header_name: HeaderName) -> Self;
}

impl HttpClientBuilderExt for modkit_http::HttpClientBuilder {
    fn with_bearer_auth(self, token: Token) -> Self {
        let layer = BearerAuthLayer::new(token);
        self.with_auth_layer(move |svc| {
            tower::ServiceBuilder::new()
                .layer(layer)
                .service(svc)
                .boxed_clone()
        })
    }

    fn with_bearer_auth_header(self, token: Token, header_name: HeaderName) -> Self {
        let layer = BearerAuthLayer::with_header_name(token, header_name);
        self.with_auth_layer(move |svc| {
            tower::ServiceBuilder::new()
                .layer(layer)
                .service(svc)
                .boxed_clone()
        })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use httpmock::prelude::*;
    use modkit_utils::SecretString;
    use std::time::Duration;
    use url::Url;

    use crate::oauth2::config::OAuthClientConfig;

    /// Build a test config pointing at the given mock server for token acquisition.
    fn token_config(server: &MockServer) -> OAuthClientConfig {
        OAuthClientConfig {
            token_endpoint: Some(
                Url::parse(&format!("http://localhost:{}/token", server.port())).unwrap(),
            ),
            client_id: "test-client".into(),
            client_secret: SecretString::new("test-secret"),
            http_config: Some(modkit_http::HttpClientConfig::for_testing()),
            jitter_max: Duration::from_millis(0),
            min_refresh_period: Duration::from_millis(100),
            ..Default::default()
        }
    }

    fn token_json(token: &str, expires_in: u64) -> String {
        format!(r#"{{"access_token":"{token}","expires_in":{expires_in},"token_type":"Bearer"}}"#)
    }

    #[tokio::test]
    async fn with_bearer_auth_injects_header() {
        // OAuth token endpoint
        let oauth_server = MockServer::start();
        let _token_mock = oauth_server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(token_json("tok-builder-ext", 3600));
        });

        // Target API server
        let api_server = MockServer::start();
        let api_mock = api_server.mock(|when, then| {
            when.method(GET)
                .path("/api/data")
                .header("authorization", "Bearer tok-builder-ext");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"ok":true}"#);
        });

        let token = Token::new(token_config(&oauth_server)).await.unwrap();

        let client = modkit_http::HttpClientBuilder::new()
            .allow_insecure_http()
            .with_bearer_auth(token)
            .build()
            .unwrap();

        let _resp = client
            .get(&format!("http://localhost:{}/api/data", api_server.port()))
            .send()
            .await
            .unwrap();

        api_mock.assert();
    }

    #[tokio::test]
    async fn with_bearer_auth_header_injects_custom_header() {
        let oauth_server = MockServer::start();
        let _token_mock = oauth_server.mock(|when, then| {
            when.method(POST).path("/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(token_json("tok-custom-hdr-ext", 3600));
        });

        let api_server = MockServer::start();
        let api_mock = api_server.mock(|when, then| {
            when.method(GET)
                .path("/api/data")
                .header("x-api-key", "Bearer tok-custom-hdr-ext");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"ok":true}"#);
        });

        let token = Token::new(token_config(&oauth_server)).await.unwrap();
        let custom = HeaderName::from_static("x-api-key");

        let client = modkit_http::HttpClientBuilder::new()
            .allow_insecure_http()
            .with_bearer_auth_header(token, custom)
            .build()
            .unwrap();

        let _resp = client
            .get(&format!("http://localhost:{}/api/data", api_server.port()))
            .send()
            .await
            .unwrap();

        api_mock.assert();
    }

    #[tokio::test]
    async fn without_bearer_auth_no_header() {
        let api_server = MockServer::start();

        // Mock that REQUIRES Authorization header — should NOT be hit.
        let auth_mock = api_server.mock(|when, then| {
            when.method(GET)
                .path("/api/data")
                .header_exists("authorization");
            then.status(200).body("authed");
        });

        // Catch-all mock for the GET.
        let fallback_mock = api_server.mock(|when, then| {
            when.method(GET).path("/api/data");
            then.status(200).body("no-auth");
        });

        let client = modkit_http::HttpClientBuilder::new()
            .allow_insecure_http()
            .build()
            .unwrap();

        let _resp = client
            .get(&format!("http://localhost:{}/api/data", api_server.port()))
            .send()
            .await
            .unwrap();

        assert_eq!(
            auth_mock.calls(),
            0,
            "No Authorization header should be sent without bearer auth"
        );
        fallback_mock.assert();
    }
}
