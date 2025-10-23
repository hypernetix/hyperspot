use tower_http::cors::CorsLayer;

use crate::config::{ApiIngressConfig, CorsConfig};

/// Build a CORS layer from config. Returns None if config has no cors section.
pub fn build_cors_layer(cfg: &ApiIngressConfig) -> Option<CorsLayer> {
    let cors_cfg: CorsConfig = cfg.cors.clone().unwrap_or_default();

    let mut layer = CorsLayer::new();

    // Allowed origins
    if cors_cfg.allowed_origins.iter().any(|o| o == "*") {
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

    // Methods
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

    // Headers
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

    // Credentials
    if cors_cfg.allow_credentials {
        layer = layer.allow_credentials(true);
    }

    // Max Age
    if cors_cfg.max_age_seconds > 0 {
        layer = layer.max_age(std::time::Duration::from_secs(cors_cfg.max_age_seconds));
    }

    Some(layer)
}
