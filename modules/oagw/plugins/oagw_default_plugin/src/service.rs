//! Plugin service implementing OagwPluginApi.

use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use modkit_security::SecurityContext;
use oagw_sdk::{
    gts::{auth_types, protocols, strategies},
    Link, OagwError, OagwInvokeRequest, OagwInvokeResponse, OagwPluginApi, OagwResponseStream,
    Route, Secret,
};
use tracing::{info_span, instrument, Instrument};

use crate::config::PluginConfig;

/// Default HTTP plugin service.
pub struct HttpPluginService {
    client: reqwest::Client,
    config: PluginConfig,
    supported_protocols: Vec<String>,
    supported_stream_protocols: Vec<String>,
    supported_auth_types: Vec<String>,
    supported_strategies: Vec<String>,
}

impl HttpPluginService {
    /// Create a new HTTP plugin service.
    ///
    /// # Panics
    /// Panics if the HTTP client cannot be created (should never happen with valid config).
    pub fn new(config: PluginConfig) -> Self {
        #[allow(clippy::expect_used)]
        // Safe: reqwest client creation only fails with invalid TLS config
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.default_timeout_ms))
            .pool_max_idle_per_host(10)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            config,
            supported_protocols: vec![protocols::HTTP11.to_string(), protocols::HTTP2.to_string()],
            supported_stream_protocols: vec![protocols::SSE.to_string()],
            supported_auth_types: vec![
                auth_types::BEARER_TOKEN.to_string(),
                auth_types::API_KEY_HEADER.to_string(),
                // TODO(v2): Add API_KEY_QUERY support
                // TODO(v3): Add OAUTH2_CLIENT_CREDS support
                // TODO(v5): Add OAUTH2_TOKEN_EXCHANGE support
            ],
            supported_strategies: vec![
                strategies::PRIORITY.to_string(),
                // TODO(v4): Add STICKY_SESSION support
                // TODO(v4): Add ROUND_ROBIN support
            ],
        }
    }

    /// Build the target URL from route base URL and request path.
    fn build_url(&self, route: &Route, req: &OagwInvokeRequest) -> String {
        let base = route.base_url.trim_end_matches('/');
        let path = if req.path.starts_with('/') {
            req.path.clone()
        } else {
            format!("/{}", req.path)
        };

        let mut url = format!("{base}{path}");

        // Add query parameters
        if let Some(ref query) = req.query {
            if !query.is_empty() {
                let query_str: String = query
                    .iter()
                    .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
                    .collect::<Vec<_>>()
                    .join("&");
                url = format!("{url}?{query_str}");
            }
        }

        url
    }

    /// Apply authentication to the request builder.
    fn apply_auth(
        &self,
        builder: reqwest::RequestBuilder,
        route: &Route,
        secret: &Secret,
    ) -> Result<reqwest::RequestBuilder, OagwError> {
        let auth_type = &route.auth_type_gts_id;

        if auth_type == auth_types::BEARER_TOKEN {
            // Bearer token: Authorization: Bearer <token>
            Ok(builder.header("Authorization", format!("Bearer {}", secret.value)))
        } else if auth_type == auth_types::API_KEY_HEADER {
            // API key header: custom header with key
            let header_name = secret
                .metadata
                .as_ref()
                .and_then(|m| m.get("header_name"))
                .map_or("X-API-Key", String::as_str);

            Ok(builder.header(header_name, &secret.value))
        } else if auth_type == auth_types::API_KEY_QUERY {
            // TODO(v2): API key query parameter support
            // For now, return error
            Err(OagwError::authentication_failed(
                "API key query auth not yet supported",
            ))
        } else if auth_type == auth_types::OAUTH2_CLIENT_CREDS {
            // TODO(v3): OAuth2 client credentials support with token caching
            Err(OagwError::authentication_failed(
                "OAuth2 client credentials auth not yet supported (v3)",
            ))
        } else if auth_type == auth_types::OAUTH2_TOKEN_EXCHANGE {
            // TODO(v5): OAuth2 token exchange support
            Err(OagwError::authentication_failed(
                "OAuth2 token exchange auth not yet supported (v5)",
            ))
        } else {
            Err(OagwError::authentication_failed(format!(
                "Unknown auth type: {auth_type}"
            )))
        }
    }

    /// Convert reqwest method to HTTP method.
    fn to_reqwest_method(&self, method: oagw_sdk::HttpMethod) -> reqwest::Method {
        match method {
            oagw_sdk::HttpMethod::Get => reqwest::Method::GET,
            oagw_sdk::HttpMethod::Post => reqwest::Method::POST,
            oagw_sdk::HttpMethod::Put => reqwest::Method::PUT,
            oagw_sdk::HttpMethod::Patch => reqwest::Method::PATCH,
            oagw_sdk::HttpMethod::Delete => reqwest::Method::DELETE,
            oagw_sdk::HttpMethod::Head => reqwest::Method::HEAD,
            oagw_sdk::HttpMethod::Options => reqwest::Method::OPTIONS,
        }
    }

    /// Extract Retry-After header from response.
    fn extract_retry_after(headers: &reqwest::header::HeaderMap) -> Option<u64> {
        headers
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
    }
}

#[async_trait]
impl OagwPluginApi for HttpPluginService {
    fn supported_protocols(&self) -> &[String] {
        &self.supported_protocols
    }

    fn supported_stream_protocols(&self) -> &[String] {
        &self.supported_stream_protocols
    }

    fn supported_auth_types(&self) -> &[String] {
        &self.supported_auth_types
    }

    fn supported_strategies(&self) -> &[String] {
        &self.supported_strategies
    }

    fn priority(&self) -> i16 {
        self.config.priority
    }

    #[instrument(skip(self, _ctx, link, route, secret, req), fields(
        target_url,
        method = %req.method,
        link_id = %link.id
    ))]
    async fn invoke_unary(
        &self,
        _ctx: &SecurityContext,
        link: &Link,
        route: &Route,
        secret: &Secret,
        req: OagwInvokeRequest,
    ) -> Result<OagwInvokeResponse, OagwError> {
        let start = std::time::Instant::now();

        // Build URL
        let url = self.build_url(route, &req);
        tracing::Span::current().record("target_url", &url);

        // Create request builder
        let method = self.to_reqwest_method(req.method);
        let mut builder = self.client.request(method.clone(), &url);

        // Apply timeout
        if let Some(timeout_ms) = req.timeout_ms {
            builder = builder.timeout(Duration::from_millis(timeout_ms));
        }

        // Apply authentication
        builder = self.apply_auth(builder, route, secret)?;

        // Add custom headers
        if let Some(headers) = req.headers {
            for (key, value) in headers {
                builder = builder.header(&key, &value);
            }
        }

        // Add body
        if let Some(body) = req.body {
            builder = builder.body(body);
        }

        // Execute request
        let response = builder
            .send()
            .instrument(info_span!("http_request"))
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    if e.is_connect() {
                        OagwError::ConnectionTimeout
                    } else {
                        OagwError::RequestTimeout
                    }
                } else if e.is_connect() {
                    OagwError::protocol_error(format!("Connection error: {e}"))
                } else {
                    OagwError::protocol_error(format!("Request error: {e}"))
                }
            })?;

        let status_code = response.status().as_u16();
        let retry_after_sec = Self::extract_retry_after(response.headers());

        // Convert headers
        let mut headers = HashMap::new();
        for (name, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(name.to_string(), v.to_string());
            }
        }

        // Read body
        let body = response
            .bytes()
            .await
            .map_err(|e| OagwError::protocol_error(format!("Failed to read response body: {e}")))?;

        // Duration in ms is always small enough for u64 in practice
        #[allow(clippy::cast_possible_truncation)]
        let duration_ms = start.elapsed().as_millis() as u64;

        tracing::info!(
            status_code,
            duration_ms,
            body_size = body.len(),
            "HTTP request completed"
        );

        Ok(OagwInvokeResponse {
            status_code,
            headers,
            body,
            duration_ms,
            link_id: link.id,
            retry_after_sec,
            attempt_number: 1, // No retry in plugin - handled by gateway
        })
    }

    #[instrument(skip_all, fields(link_id = %link.id))]
    async fn invoke_stream(
        &self,
        _ctx: &SecurityContext,
        link: &Link,
        route: &Route,
        _secret: &Secret,
        _req: OagwInvokeRequest,
    ) -> Result<OagwResponseStream, OagwError> {
        // TODO(v2): Implement SSE streaming support
        // For v1, streaming is not supported - return error
        tracing::warn!(
            link_id = %link.id,
            route_id = %route.id,
            "Streaming invocation not yet implemented (v2)"
        );

        Err(OagwError::protocol_error(
            "Streaming not yet supported. Use invoke_unary for HTTP requests. Streaming support planned for v2.",
        ))
    }
}

// URL encoding helper
mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url_with_path() {
        let config = PluginConfig::default();
        let service = HttpPluginService::new(config);

        let route = Route {
            id: uuid::Uuid::nil(),
            tenant_id: uuid::Uuid::nil(),
            base_url: "https://api.example.com/v1".to_string(),
            rate_limit_req_per_min: 1000,
            auth_type_gts_id: auth_types::BEARER_TOKEN.to_string(),
            cache_ttl_sec: None,
            supported_protocols: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let req = OagwInvokeRequest {
            route_id: uuid::Uuid::nil(),
            method: oagw_sdk::HttpMethod::Get,
            path: "/users".to_string(),
            ..Default::default()
        };

        let url = service.build_url(&route, &req);
        assert_eq!(url, "https://api.example.com/v1/users");
    }

    #[test]
    fn test_build_url_with_query() {
        let config = PluginConfig::default();
        let service = HttpPluginService::new(config);

        let route = Route {
            id: uuid::Uuid::nil(),
            tenant_id: uuid::Uuid::nil(),
            base_url: "https://api.example.com".to_string(),
            rate_limit_req_per_min: 1000,
            auth_type_gts_id: auth_types::BEARER_TOKEN.to_string(),
            cache_ttl_sec: None,
            supported_protocols: vec![],
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let mut query = HashMap::new();
        query.insert("limit".to_string(), "10".to_string());
        query.insert("offset".to_string(), "0".to_string());

        let req = OagwInvokeRequest {
            route_id: uuid::Uuid::nil(),
            method: oagw_sdk::HttpMethod::Get,
            path: "/items".to_string(),
            query: Some(query),
            ..Default::default()
        };

        let url = service.build_url(&route, &req);
        assert!(url.starts_with("https://api.example.com/items?"));
        assert!(url.contains("limit=10"));
        assert!(url.contains("offset=0"));
    }
}
