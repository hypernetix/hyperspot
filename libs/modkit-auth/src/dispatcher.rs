use crate::{
    auth_mode::PluginRegistry,
    claims::Claims,
    claims_error::ClaimsError,
    config::AuthConfig,
    config_error::ConfigError,
    errors::AuthError,
    plugin_traits::{ClaimsPlugin, IntrospectionProvider, KeyProvider},
    traits::TokenValidator,
    validation::{validate_claims, ValidationConfig},
};
use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

/// Truncate UUID to first 8 characters for safe logging
fn truncate_uuid(uuid: &Uuid) -> String {
    let s = uuid.to_string();
    s.chars().take(8).collect()
}

/// Central dispatcher for JWT and opaque token validation
///
/// Orchestrates key providers and claims plugins to validate tokens
/// using a single configured plugin.
pub struct AuthDispatcher {
    /// Registered key providers (JWKS, etc.)
    key_providers: Vec<Arc<dyn KeyProvider>>,

    /// Registered introspection providers (for opaque tokens)
    introspection_providers: Vec<Arc<dyn IntrospectionProvider>>,

    /// The authentication plugin to use for claims normalization
    plugin: Arc<dyn ClaimsPlugin>,

    /// Common validation configuration
    validation_config: ValidationConfig,
}

impl AuthDispatcher {
    /// Create a new dispatcher with validation config and plugin
    pub fn new(
        validation_config: ValidationConfig,
        config: &AuthConfig,
        registry: &PluginRegistry,
    ) -> Result<Self, ConfigError> {
        // Get the configured plugin
        let plugin = registry.get(&config.mode.provider)?.clone();

        Ok(Self {
            key_providers: Vec::new(),
            introspection_providers: Vec::new(),
            plugin,
            validation_config,
        })
    }

    /// Add a key provider
    pub fn with_key_provider(mut self, provider: Arc<dyn KeyProvider>) -> Self {
        self.key_providers.push(provider);
        self
    }

    /// Add an introspection provider
    pub fn with_introspection_provider(mut self, provider: Arc<dyn IntrospectionProvider>) -> Self {
        self.introspection_providers.push(provider);
        self
    }

    /// Validate a JWT token
    ///
    /// Workflow:
    /// 1. Try each KeyProvider until one successfully validates the signature
    /// 2. Extract issuer from token
    /// 3. Use the configured plugin to normalize claims
    /// 4. Run common validation (issuer, audience, exp, nbf, UUIDs)
    /// 5. Return normalized claims
    pub async fn validate_jwt(&self, token: &str) -> Result<Claims, ClaimsError> {
        // Step 1: Try to validate signature with each key provider
        let (header, raw_claims) = {
            let mut last_error = None;
            let mut result = None;

            for provider in &self.key_providers {
                match provider.validate_and_decode(token).await {
                    Ok(r) => {
                        tracing::debug!(
                            provider = provider.name(),
                            kid = ?r.0.kid,
                            "Successfully validated token signature"
                        );
                        result = Some(r);
                        break;
                    }
                    Err(e) => {
                        tracing::debug!(
                            provider = provider.name(),
                            error = %e,
                            "Provider failed to validate token"
                        );
                        last_error = Some(e);
                    }
                }
            }

            result.ok_or_else(|| last_error.unwrap_or(ClaimsError::NoMatchingProvider))?
        };

        // Step 2: Extract issuer for logging
        let issuer = raw_claims
            .get("iss")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ClaimsError::Malformed("missing iss claim".into()))?;

        // Step 3: Use the configured plugin
        let plugin = &self.plugin;

        tracing::debug!(
            plugin = plugin.name(),
            issuer = issuer,
            "Using configured plugin"
        );

        // Step 4: Normalize claims
        let normalized = plugin.normalize(&raw_claims).map_err(|e| {
            tracing::error!(
                plugin = plugin.name(),
                error = %e,
                issuer = issuer,
                "Failed to normalize claims"
            );
            e
        })?;

        // Step 5: Run common validation
        validate_claims(&normalized, &self.validation_config).map_err(|e| {
            tracing::warn!(
                error = %e,
                sub_prefix = %truncate_uuid(&normalized.sub),
                issuer = %normalized.issuer,
                "Common validation failed"
            );
            e
        })?;

        tracing::debug!(
            sub_prefix = %truncate_uuid(&normalized.sub),
            issuer = %normalized.issuer,
            plugin = plugin.name(),
            kid = ?header.kid,
            num_roles = normalized.roles.len(),
            num_tenants = normalized.tenants.len(),
            "Token validation successful"
        );

        Ok(normalized)
    }

    /// Try to introspect a token with each provider until one succeeds
    async fn try_introspect_with_providers(
        &self,
        token: &str,
    ) -> Result<serde_json::Value, ClaimsError> {
        let mut last_error = None;
        let mut result = None;

        for provider in &self.introspection_providers {
            match provider.introspect(token).await {
                Ok(r) => {
                    tracing::debug!(
                        provider = provider.name(),
                        "Successfully introspected token"
                    );
                    result = Some(r);
                    break;
                }
                Err(e) => {
                    tracing::debug!(
                        provider = provider.name(),
                        error = %e,
                        "Provider failed to introspect token"
                    );
                    last_error = Some(e);
                }
            }
        }

        result.ok_or_else(|| {
            last_error.unwrap_or_else(|| {
                ClaimsError::Provider("No introspection provider available".into())
            })
        })
    }

    /// Verify that an introspection response indicates an active token
    fn verify_token_active(introspection_result: &serde_json::Value) -> Result<(), ClaimsError> {
        if let Some(active) = introspection_result.get("active").and_then(|v| v.as_bool()) {
            if !active {
                return Err(ClaimsError::IntrospectionDenied);
            }
        }
        Ok(())
    }

    /// Extract issuer from introspection response
    fn extract_issuer(introspection_result: &serde_json::Value) -> Result<&str, ClaimsError> {
        introspection_result
            .get("iss")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ClaimsError::Malformed("missing iss claim".into()))
    }

    /// Normalize claims with error logging
    fn normalize_with_logging(
        &self,
        introspection_result: &serde_json::Value,
        issuer: &str,
    ) -> Result<Claims, ClaimsError> {
        self.plugin.normalize(introspection_result).map_err(|e| {
            tracing::error!(
                plugin = self.plugin.name(),
                error = %e,
                issuer = issuer,
                "Failed to normalize introspection response"
            );
            e
        })
    }

    /// Validate claims with error logging
    fn validate_and_log(claims: &Claims, config: &ValidationConfig) -> Result<(), ClaimsError> {
        validate_claims(claims, config).map_err(|e| {
            tracing::warn!(
                error = %e,
                sub_prefix = %truncate_uuid(&claims.sub),
                issuer = %claims.issuer,
                "Common validation failed"
            );
            e
        })
    }

    /// Log successful validation
    fn log_validation_success(claims: &Claims, plugin_name: &str) {
        tracing::debug!(
            sub_prefix = %truncate_uuid(&claims.sub),
            issuer = %claims.issuer,
            plugin = plugin_name,
            num_roles = claims.roles.len(),
            num_tenants = claims.tenants.len(),
            "Opaque token validation successful"
        );
    }

    /// Validate an opaque token via introspection
    ///
    /// Workflow:
    /// 1. Try each IntrospectionProvider until one succeeds
    /// 2. Extract issuer from introspection response
    /// 3. Use the configured plugin to normalize claims
    /// 4. Run common validation
    /// 5. Return normalized claims
    pub async fn validate_opaque(&self, token: &str) -> Result<Claims, ClaimsError> {
        // Step 1: Try to introspect with each provider
        let introspection_result = self.try_introspect_with_providers(token).await?;

        // Step 2: Check if token is active
        Self::verify_token_active(&introspection_result)?;

        // Step 3: Extract issuer for logging
        let issuer = Self::extract_issuer(&introspection_result)?;

        // Step 4: Log plugin usage
        tracing::debug!(
            plugin = self.plugin.name(),
            issuer = issuer,
            "Using configured plugin for introspection"
        );

        // Step 5: Normalize claims
        let normalized = self.normalize_with_logging(&introspection_result, issuer)?;

        // Step 6: Run common validation
        Self::validate_and_log(&normalized, &self.validation_config)?;

        // Step 7: Log success
        Self::log_validation_success(&normalized, self.plugin.name());

        Ok(normalized)
    }

    /// Get validation config (for inspection/testing)
    pub fn validation_config(&self) -> &ValidationConfig {
        &self.validation_config
    }

    /// Get the configured authentication plugin (for inspection/testing)
    pub fn plugin(&self) -> &Arc<dyn ClaimsPlugin> {
        &self.plugin
    }

    /// Trigger key refresh for all key providers
    pub async fn refresh_keys(&self) -> Result<(), Vec<ClaimsError>> {
        let mut errors = Vec::new();

        for provider in &self.key_providers {
            if let Err(e) = provider.refresh_keys().await {
                tracing::warn!(
                    provider = provider.name(),
                    error = %e,
                    "Key refresh failed"
                );
                errors.push(e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Implement TokenValidator trait for AuthDispatcher
#[async_trait]
impl TokenValidator for AuthDispatcher {
    async fn validate_and_parse(&self, token: &str) -> Result<Claims, AuthError> {
        self.validate_jwt(token).await.map_err(|e| match e {
            // All JWT validation errors should result in 401 Unauthenticated
            ClaimsError::InvalidSignature => AuthError::Unauthenticated,
            ClaimsError::Expired => AuthError::Unauthenticated,
            ClaimsError::NotYetValid => AuthError::Unauthenticated,
            ClaimsError::InvalidIssuer { .. } => AuthError::Unauthenticated,
            ClaimsError::InvalidAudience { .. } => AuthError::Unauthenticated,
            ClaimsError::Malformed(_) => AuthError::Unauthenticated,
            ClaimsError::Provider(_) => AuthError::Unauthenticated,
            ClaimsError::MissingClaim(_) => AuthError::Unauthenticated,
            ClaimsError::InvalidClaimFormat { .. } => AuthError::Unauthenticated,
            ClaimsError::NoMatchingPlugin => AuthError::Unauthenticated,
            ClaimsError::NoValidatingKey => AuthError::Unauthenticated,
            ClaimsError::NoMatchingProvider => AuthError::Unauthenticated,
            ClaimsError::UnknownKidAfterRefresh => AuthError::Unauthenticated,
            ClaimsError::IntrospectionDenied => AuthError::Unauthenticated,
            ClaimsError::ConfigError(_) => AuthError::Unauthenticated,
            ClaimsError::DecodeFailed(_) => AuthError::Unauthenticated,
            ClaimsError::JwksFetchFailed(_) => AuthError::Unauthenticated,
            ClaimsError::UnknownKeyId(_) => AuthError::Unauthenticated,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth_mode::AuthModeConfig;
    use crate::config::PluginConfig;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_dispatcher_creation() {
        let mut plugins = HashMap::new();
        plugins.insert(
            "oidc".to_string(),
            PluginConfig::Oidc {
                tenant_claim: "tenants".to_string(),
                roles_claim: "roles".to_string(),
            },
        );

        let config = AuthConfig {
            mode: AuthModeConfig {
                provider: "oidc".to_string(),
            },
            plugins,
            ..Default::default()
        };

        // This would need a real plugin in registry to work
        // Just testing that the structure compiles
        let _ = &config;
    }

    // ===== Test Mocks =====

    /// Mock IntrospectionProvider for testing
    struct MockIntrospectionProvider {
        response: Option<serde_json::Value>,
        error_msg: Option<String>,
        name: String,
    }

    impl MockIntrospectionProvider {
        fn success(response: serde_json::Value) -> Self {
            Self {
                response: Some(response),
                error_msg: None,
                name: "mock-introspection".to_string(),
            }
        }

        fn failure(error_msg: String) -> Self {
            Self {
                response: None,
                error_msg: Some(error_msg),
                name: "mock-introspection".to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl IntrospectionProvider for MockIntrospectionProvider {
        fn name(&self) -> &str {
            &self.name
        }

        async fn introspect(&self, _token: &str) -> Result<serde_json::Value, ClaimsError> {
            if let Some(msg) = &self.error_msg {
                Err(ClaimsError::Provider(msg.clone()))
            } else {
                Ok(self.response.clone().unwrap())
            }
        }
    }

    /// Mock ClaimsPlugin for testing
    struct MockClaimsPlugin {
        name: String,
        normalized: Option<Claims>,
        error_msg: Option<String>,
    }

    impl MockClaimsPlugin {
        fn success(normalized: Claims) -> Self {
            Self {
                name: "mock-plugin".to_string(),
                normalized: Some(normalized),
                error_msg: None,
            }
        }

        fn failure(error_msg: String) -> Self {
            Self {
                name: "mock-plugin".to_string(),
                normalized: None,
                error_msg: Some(error_msg),
            }
        }
    }

    impl ClaimsPlugin for MockClaimsPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn normalize(&self, _raw: &serde_json::Value) -> Result<Claims, ClaimsError> {
            if let Some(msg) = &self.error_msg {
                Err(ClaimsError::Malformed(msg.clone()))
            } else {
                Ok(self.normalized.clone().unwrap())
            }
        }
    }

    /// Helper to create test claims
    fn test_claims() -> Claims {
        Claims {
            sub: Uuid::new_v4(),
            issuer: "https://test.example.com".to_string(),
            audiences: vec!["test-api".to_string()],
            expires_at: Some(time::OffsetDateTime::now_utc() + time::Duration::hours(1)),
            not_before: None,
            tenants: vec![Uuid::new_v4()],
            roles: vec!["user".to_string()],
            extras: serde_json::Map::new(),
        }
    }

    // ===== Regression Tests for validate_opaque =====

    #[tokio::test]
    async fn test_validate_opaque_success() {
        // Given: A dispatcher with mock provider and plugin
        let introspection_response = json!({
            "active": true,
            "iss": "https://test.example.com",
            "sub": "user-123"
        });

        let claims = test_claims();
        let provider = Arc::new(MockIntrospectionProvider::success(introspection_response));
        let plugin = Arc::new(MockClaimsPlugin::success(claims.clone()));

        let validation_config = ValidationConfig {
            allowed_issuers: vec!["https://test.example.com".to_string()],
            allowed_audiences: vec!["test-api".to_string()],
            leeway_seconds: 60,
            require_uuid_subject: true,
            require_uuid_tenants: true,
        };

        let dispatcher = AuthDispatcher {
            key_providers: Vec::new(),
            introspection_providers: vec![provider],
            plugin,
            validation_config,
        };

        // When: We validate an opaque token
        let result = dispatcher.validate_opaque("test-token").await;

        // Then: Validation succeeds
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert_eq!(normalized.issuer, claims.issuer);
    }

    #[tokio::test]
    async fn test_validate_opaque_inactive_token() {
        // Given: A response with active=false
        let introspection_response = json!({
            "active": false,
            "iss": "https://test.example.com"
        });

        let provider = Arc::new(MockIntrospectionProvider::success(introspection_response));
        let plugin = Arc::new(MockClaimsPlugin::success(test_claims()));

        let validation_config = ValidationConfig {
            allowed_issuers: vec!["https://test.example.com".to_string()],
            allowed_audiences: vec!["test-api".to_string()],
            leeway_seconds: 60,
            require_uuid_subject: true,
            require_uuid_tenants: true,
        };

        let dispatcher = AuthDispatcher {
            key_providers: Vec::new(),
            introspection_providers: vec![provider],
            plugin,
            validation_config,
        };

        // When: We validate the token
        let result = dispatcher.validate_opaque("test-token").await;

        // Then: Validation fails with IntrospectionDenied
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ClaimsError::IntrospectionDenied
        ));
    }

    #[tokio::test]
    async fn test_validate_opaque_missing_issuer() {
        // Given: A response without issuer
        let introspection_response = json!({
            "active": true,
            "sub": "user-123"
        });

        let provider = Arc::new(MockIntrospectionProvider::success(introspection_response));
        let plugin = Arc::new(MockClaimsPlugin::success(test_claims()));

        let validation_config = ValidationConfig::default();

        let dispatcher = AuthDispatcher {
            key_providers: Vec::new(),
            introspection_providers: vec![provider],
            plugin,
            validation_config,
        };

        // When: We validate the token
        let result = dispatcher.validate_opaque("test-token").await;

        // Then: Validation fails with Malformed
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ClaimsError::Malformed(_)));
    }

    #[tokio::test]
    async fn test_validate_opaque_provider_failure() {
        // Given: A provider that fails
        let provider = Arc::new(MockIntrospectionProvider::failure(
            "Provider error".to_string(),
        ));
        let plugin = Arc::new(MockClaimsPlugin::success(test_claims()));

        let validation_config = ValidationConfig::default();

        let dispatcher = AuthDispatcher {
            key_providers: Vec::new(),
            introspection_providers: vec![provider],
            plugin,
            validation_config,
        };

        // When: We validate the token
        let result = dispatcher.validate_opaque("test-token").await;

        // Then: Validation fails with Provider error
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ClaimsError::Provider(_)));
    }

    #[tokio::test]
    async fn test_validate_opaque_no_providers() {
        // Given: A dispatcher with no providers
        let plugin = Arc::new(MockClaimsPlugin::success(test_claims()));
        let validation_config = ValidationConfig::default();

        let dispatcher = AuthDispatcher {
            key_providers: Vec::new(),
            introspection_providers: Vec::new(),
            plugin,
            validation_config,
        };

        // When: We validate the token
        let result = dispatcher.validate_opaque("test-token").await;

        // Then: Validation fails
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validate_opaque_normalization_failure() {
        // Given: A plugin that fails normalization
        let introspection_response = json!({
            "active": true,
            "iss": "https://test.example.com",
            "sub": "user-123"
        });

        let provider = Arc::new(MockIntrospectionProvider::success(introspection_response));
        let plugin = Arc::new(MockClaimsPlugin::failure(
            "Normalization failed".to_string(),
        ));

        let validation_config = ValidationConfig::default();

        let dispatcher = AuthDispatcher {
            key_providers: Vec::new(),
            introspection_providers: vec![provider],
            plugin,
            validation_config,
        };

        // When: We validate the token
        let result = dispatcher.validate_opaque("test-token").await;

        // Then: Validation fails with normalization error
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ClaimsError::Malformed(_)));
    }

    #[tokio::test]
    async fn test_validate_opaque_validation_failure() {
        // Given: Claims that fail common validation (wrong issuer)
        let introspection_response = json!({
            "active": true,
            "iss": "https://wrong.example.com",
            "sub": "user-123"
        });

        let mut claims = test_claims();
        claims.issuer = "https://wrong.example.com".to_string();

        let provider = Arc::new(MockIntrospectionProvider::success(introspection_response));
        let plugin = Arc::new(MockClaimsPlugin::success(claims));

        let validation_config = ValidationConfig {
            allowed_issuers: vec!["https://test.example.com".to_string()],
            allowed_audiences: vec!["test-api".to_string()],
            leeway_seconds: 60,
            require_uuid_subject: true,
            require_uuid_tenants: true,
        };

        let dispatcher = AuthDispatcher {
            key_providers: Vec::new(),
            introspection_providers: vec![provider],
            plugin,
            validation_config,
        };

        // When: We validate the token
        let result = dispatcher.validate_opaque("test-token").await;

        // Then: Validation fails with InvalidIssuer
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ClaimsError::InvalidIssuer { .. }
        ));
    }

    #[tokio::test]
    async fn test_validate_opaque_provider_fallback() {
        // Given: Two providers, first fails, second succeeds
        let failing_provider = Arc::new(MockIntrospectionProvider::failure(
            "First provider failed".to_string(),
        ));

        let introspection_response = json!({
            "active": true,
            "iss": "https://test.example.com",
            "sub": "user-123"
        });

        let success_provider = Arc::new(MockIntrospectionProvider::success(introspection_response));
        let claims = test_claims();
        let plugin = Arc::new(MockClaimsPlugin::success(claims.clone()));

        let validation_config = ValidationConfig {
            allowed_issuers: vec!["https://test.example.com".to_string()],
            allowed_audiences: vec!["test-api".to_string()],
            leeway_seconds: 60,
            require_uuid_subject: true,
            require_uuid_tenants: true,
        };

        let dispatcher = AuthDispatcher {
            key_providers: Vec::new(),
            introspection_providers: vec![failing_provider, success_provider],
            plugin,
            validation_config,
        };

        // When: We validate the token
        let result = dispatcher.validate_opaque("test-token").await;

        // Then: Validation succeeds with second provider
        assert!(result.is_ok());
        let normalized = result.unwrap();
        assert_eq!(normalized.issuer, claims.issuer);
    }
}
