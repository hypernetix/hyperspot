//! Client implementation for the static `AuthN` resolver plugin.
//!
//! Implements `AuthNResolverPluginClient` using the domain service.

use async_trait::async_trait;
use authn_resolver_sdk::{AuthNResolverError, AuthNResolverPluginClient, AuthenticationResult};

use super::service::Service;

#[async_trait]
impl AuthNResolverPluginClient for Service {
    async fn authenticate(
        &self,
        bearer_token: &str,
    ) -> Result<AuthenticationResult, AuthNResolverError> {
        self.authenticate(bearer_token)
            .ok_or_else(|| AuthNResolverError::Unauthorized("invalid token".to_owned()))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::config::StaticAuthNPluginConfig;

    #[tokio::test]
    async fn plugin_trait_accept_all_succeeds() {
        let service = Service::from_config(&StaticAuthNPluginConfig::default());
        let plugin: &dyn AuthNResolverPluginClient = &service;

        let result = plugin.authenticate("any-token").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn plugin_trait_empty_token_unauthorized() {
        let service = Service::from_config(&StaticAuthNPluginConfig::default());
        let plugin: &dyn AuthNResolverPluginClient = &service;

        let result = plugin.authenticate("").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AuthNResolverError::Unauthorized(_) => {}
            other => panic!("Expected Unauthorized, got: {other:?}"),
        }
    }
}
