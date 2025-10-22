use serde::{Deserialize, Serialize};

fn default_require_auth_by_default() -> bool {
    true
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
