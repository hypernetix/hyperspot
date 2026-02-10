use serde::Deserialize;
use url::Url;

use super::error::TokenError;

/// Minimal subset of the `OpenID` Connect discovery document.
///
/// Only `token_endpoint` is required; all other fields are silently ignored.
#[derive(Deserialize)]
struct OidcDiscoveryDoc {
    token_endpoint: String,
}

/// Resolve the token endpoint from an OIDC issuer URL.
///
/// Fetches `{issuer_url}/.well-known/openid-configuration` and extracts
/// the `token_endpoint` field. This is a one-time operation at startup.
///
/// # Errors
///
/// Returns [`TokenError::Http`] if the discovery request fails or returns a
/// non-success status.
/// Returns [`TokenError::InvalidResponse`] if the response body cannot be
/// parsed, the `token_endpoint` field is missing, or it is not a valid URL.
pub async fn discover_token_endpoint(
    client: &modkit_http::HttpClient,
    issuer_url: &Url,
) -> Result<Url, TokenError> {
    let base = issuer_url.as_str().trim_end_matches('/');
    let discovery_url = format!("{base}/.well-known/openid-configuration");

    let doc: OidcDiscoveryDoc = client
        .get(&discovery_url)
        .send()
        .await
        .map_err(|e| TokenError::Http(crate::http_error::format_http_error(&e, "OIDC discovery")))?
        .error_for_status()
        .map_err(|e| TokenError::Http(crate::http_error::format_http_error(&e, "OIDC discovery")))?
        .json()
        .await
        .map_err(|e| {
            TokenError::InvalidResponse(crate::http_error::format_http_error(&e, "OIDC discovery"))
        })?;

    Url::parse(&doc.token_endpoint).map_err(|e| {
        TokenError::InvalidResponse(format!(
            "invalid token_endpoint URL in discovery document: {e}"
        ))
    })
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use httpmock::prelude::*;

    fn build_client(_server: &MockServer) -> modkit_http::HttpClient {
        modkit_http::HttpClientBuilder::with_config(modkit_http::HttpClientConfig::for_testing())
            .build()
            .unwrap()
    }

    fn issuer_url(server: &MockServer) -> Url {
        Url::parse(&format!("http://localhost:{}", server.port())).unwrap()
    }

    fn issuer_url_trailing_slash(server: &MockServer) -> Url {
        Url::parse(&format!("http://localhost:{}/", server.port())).unwrap()
    }

    #[tokio::test]
    async fn discover_valid_response() {
        let server = MockServer::start();
        let token_ep = format!("http://localhost:{}/oauth/token", server.port());

        let mock = server.mock(|when, then| {
            when.method(GET).path("/.well-known/openid-configuration");
            then.status(200)
                .header("content-type", "application/json")
                .body(format!(r#"{{"token_endpoint":"{token_ep}"}}"#));
        });

        let client = build_client(&server);
        let result = discover_token_endpoint(&client, &issuer_url(&server)).await;

        let url = result.unwrap();
        assert_eq!(url.as_str(), token_ep);
        mock.assert();
    }

    #[tokio::test]
    async fn discover_strips_trailing_slash() {
        let server = MockServer::start();
        let token_ep = format!("http://localhost:{}/oauth/token", server.port());

        let mock = server.mock(|when, then| {
            // Must NOT have a double slash — "/.well-known/..." not "//.well-known/..."
            when.method(GET).path("/.well-known/openid-configuration");
            then.status(200)
                .header("content-type", "application/json")
                .body(format!(r#"{{"token_endpoint":"{token_ep}"}}"#));
        });

        let client = build_client(&server);
        let result = discover_token_endpoint(&client, &issuer_url_trailing_slash(&server)).await;

        assert!(result.is_ok(), "expected Ok, got: {:?}", result.err());
        mock.assert();
    }

    #[tokio::test]
    async fn discover_no_trailing_slash() {
        let server = MockServer::start();
        let token_ep = format!("http://localhost:{}/oauth/token", server.port());

        let mock = server.mock(|when, then| {
            when.method(GET).path("/.well-known/openid-configuration");
            then.status(200)
                .header("content-type", "application/json")
                .body(format!(r#"{{"token_endpoint":"{token_ep}"}}"#));
        });

        let client = build_client(&server);
        let result = discover_token_endpoint(&client, &issuer_url(&server)).await;

        assert!(result.is_ok(), "expected Ok, got: {:?}", result.err());
        mock.assert();
    }

    #[tokio::test]
    async fn discover_missing_field() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET).path("/.well-known/openid-configuration");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"authorization_endpoint":"https://example.com/auth"}"#);
        });

        let client = build_client(&server);
        let err = discover_token_endpoint(&client, &issuer_url(&server))
            .await
            .unwrap_err();

        // serde deserialization error for missing required field → InvalidResponse
        assert!(
            matches!(err, TokenError::InvalidResponse(ref msg) if msg.contains("OIDC discovery")),
            "expected InvalidResponse with OIDC discovery prefix, got: {err}"
        );
        mock.assert();
    }

    #[tokio::test]
    async fn discover_invalid_url() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET).path("/.well-known/openid-configuration");
            then.status(200)
                .header("content-type", "application/json")
                .body(r#"{"token_endpoint":"not a valid url"}"#);
        });

        let client = build_client(&server);
        let err = discover_token_endpoint(&client, &issuer_url(&server))
            .await
            .unwrap_err();

        assert!(
            matches!(
                err,
                TokenError::InvalidResponse(ref msg)
                    if msg.contains("invalid token_endpoint")
            ),
            "expected InvalidResponse, got: {err}"
        );
        mock.assert();
    }

    #[tokio::test]
    async fn discover_http_error() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(GET).path("/.well-known/openid-configuration");
            then.status(500)
                .header("content-type", "application/json")
                .body(r#"{"error":"server_error"}"#);
        });

        let client = build_client(&server);
        let err = discover_token_endpoint(&client, &issuer_url(&server))
            .await
            .unwrap_err();

        assert!(
            matches!(
                err,
                TokenError::Http(ref msg)
                    if msg.contains("OIDC discovery")
                        && msg.contains("500")
            ),
            "expected Http error with 500 status, got: {err}"
        );
        mock.assert();
    }
}
