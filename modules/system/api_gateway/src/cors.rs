use tracing::warn;
use tower_http::cors::CorsLayer;

use crate::config::{ApiGatewayConfig, CorsConfig};

/// Build a CORS layer from config.
///
/// # Panics
///
/// Panics if `allow_credentials` is `true` while `allowed_origins` contains `"*"`.
/// This combination is forbidden by the CORS specification — browsers will reject the
/// response, and it signals a likely misconfiguration.
pub fn build_cors_layer(cfg: &ApiGatewayConfig) -> CorsLayer {
    let cors_cfg: CorsConfig = cfg.cors.clone().unwrap_or_default();

    let has_wildcard_origin = cors_cfg.allowed_origins.iter().any(|o| o == "*");

    // Reject invalid combination: wildcard origins + credentials
    assert!(
        !(has_wildcard_origin && cors_cfg.allow_credentials),
        "CORS misconfiguration: allowed_origins=['*'] cannot be combined with \
         allow_credentials=true. The CORS specification forbids this combination. \
         Please specify explicit origins when using credentials."
    );

    if has_wildcard_origin {
        warn!(
            "CORS is configured with allowed_origins=['*']. \
             This allows any website to make cross-origin requests to the API. \
             Consider specifying explicit origins for production deployments."
        );
    }

    let mut layer = CorsLayer::new();

    if has_wildcard_origin {
        layer = layer.allow_origin(tower_http::cors::Any);
    } else {
        let origins: Vec<axum::http::HeaderValue> = cors_cfg
            .allowed_origins
            .into_iter()
            .filter_map(|s| axum::http::HeaderValue::from_str(&s).ok())
            .collect();
        if !origins.is_empty() {
            layer = layer.allow_origin(origins);
        }
    }

    if cors_cfg.allowed_methods.iter().any(|m| m == "*") {
        layer = layer.allow_methods(tower_http::cors::Any);
    } else {
        let methods: Vec<axum::http::Method> = cors_cfg
            .allowed_methods
            .into_iter()
            .filter_map(|s| s.parse().ok())
            .collect();
        if !methods.is_empty() {
            layer = layer.allow_methods(methods);
        }
    }

    if cors_cfg.allowed_headers.iter().any(|h| h == "*") {
        layer = layer.allow_headers(tower_http::cors::Any);
    } else {
        let headers: Vec<axum::http::HeaderName> = cors_cfg
            .allowed_headers
            .into_iter()
            .filter_map(|s| s.parse().ok())
            .collect();
        if !headers.is_empty() {
            layer = layer.allow_headers(headers);
        }
    }

    if cors_cfg.allow_credentials {
        layer = layer.allow_credentials(true);
    }

    if cors_cfg.max_age_seconds > 0 {
        layer = layer.max_age(std::time::Duration::from_secs(cors_cfg.max_age_seconds));
    }

    layer
}
