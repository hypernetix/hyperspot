use crate::{claims::Claims, claims_error::ClaimsError};
use async_trait::async_trait;
use jsonwebtoken::Header;
use serde_json::Value;

/// Plugin that knows how to normalize provider-specific claims into standard Claims format
pub trait ClaimsPlugin: Send + Sync {
    /// Returns the name of this plugin (for debugging/logging)
    fn name(&self) -> &str;

    /// Normalize provider-specific claims into our standard format
    ///
    /// Extract:
    /// - sub (must be UUID)
    /// - issuer
    /// - audiences
    /// - expiration/not-before times
    /// - tenants (must be UUIDs)
    /// - roles
    /// - any extra provider-specific fields
    fn normalize(&self, raw: &Value) -> Result<Claims, ClaimsError>;
}

/// Plugin that can validate JWT signatures and decode tokens
#[async_trait]
pub trait KeyProvider: Send + Sync {
    /// Returns the name of this provider (for debugging/logging)
    fn name(&self) -> &str;

    /// Attempt to validate the JWT signature and decode its header and claims
    ///
    /// Returns the JWT header and raw claims as JSON if validation succeeds.
    /// Returns an error if the signature is invalid or decoding fails.
    ///
    /// This method should:
    /// - Decode the JWT header
    /// - Find the appropriate key (e.g., by kid)
    /// - Validate the signature
    /// - Return raw claims for further processing
    async fn validate_and_decode(&self, token: &str) -> Result<(Header, Value), ClaimsError>;

    /// Optional: refresh keys if this provider supports it (e.g., JWKS)
    async fn refresh_keys(&self) -> Result<(), ClaimsError> {
        Ok(())
    }
}

/// Plugin that can introspect opaque tokens (RFC 7662)
#[async_trait]
pub trait IntrospectionProvider: Send + Sync {
    /// Returns the name of this provider (for debugging/logging)
    fn name(&self) -> &str;

    /// Introspect an opaque token and return the claims
    ///
    /// This should call the OAuth 2.0 Token Introspection endpoint
    /// and return the introspection response as JSON.
    async fn introspect(&self, token: &str) -> Result<Value, ClaimsError>;
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    struct TestPlugin;

    impl ClaimsPlugin for TestPlugin {
        fn name(&self) -> &'static str {
            "test"
        }

        fn normalize(&self, _raw: &Value) -> Result<Claims, ClaimsError> {
            Err(ClaimsError::Malformed("test plugin".into()))
        }
    }

    #[test]
    fn test_plugin_name() {
        let plugin = TestPlugin;
        assert_eq!(plugin.name(), "test");
    }
}
