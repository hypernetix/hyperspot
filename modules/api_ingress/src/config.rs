use serde::{Deserialize, Serialize};

fn default_require_auth_by_default() -> bool {
    true
}

fn default_body_limit_bytes() -> usize {
    16 * 1024 * 1024
}

/// API ingress configuration - reused from api_ingress module
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ApiIngressConfig {
    pub bind_addr: String,
    #[serde(default)]
    pub enable_docs: bool,
    #[serde(default)]
    pub cors_enabled: bool,
    /// Optional detailed CORS configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cors: Option<CorsConfig>,

    /// Global defaults
    #[serde(default)]
    pub defaults: Defaults,

    /// Disable authentication and authorization completely.
    /// When true, middleware injects SecurityCtx::root_ctx() (full access).
    #[serde(default)]
    pub auth_disabled: bool,

    /// JWKS endpoint to validate JWT tokens (OIDC-compliant).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jwks_uri: Option<String>,

    /// Expected token issuer (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,

    /// Expected token audience (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,

    /// If true, routes without explicit role still require authentication (AuthN-only).
    #[serde(default = "default_require_auth_by_default")]
    pub require_auth_by_default: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Defaults {
    /// Fallback rate limit when operation does not specify one
    pub rate_limit: RateLimitDefaults,
    /// Global request body size limit in bytes
    pub body_limit_bytes: usize,
}

impl Default for Defaults {
    fn default() -> Self {
        Self {
            rate_limit: RateLimitDefaults::default(),
            body_limit_bytes: default_body_limit_bytes(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct RateLimitDefaults {
    pub rps: u32,
    pub burst: u32,
    pub in_flight: u32,
}

impl Default for RateLimitDefaults {
    fn default() -> Self {
        Self {
            rps: 50,
            burst: 100,
            in_flight: 64,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct CorsConfig {
    /// Allowed origins: ["*"] means any
    pub allowed_origins: Vec<String>,
    /// Allowed HTTP methods, e.g. ["GET","POST","OPTIONS","PUT","DELETE","PATCH"]
    pub allowed_methods: Vec<String>,
    /// Allowed request headers; ["*"] means any
    pub allowed_headers: Vec<String>,
    /// Whether to allow credentials
    pub allow_credentials: bool,
    /// Max age for preflight caching in seconds
    pub max_age_seconds: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "PATCH".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec!["*".to_string()],
            allow_credentials: false,
            max_age_seconds: 600,
        }
    }
}
