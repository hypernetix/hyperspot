//! Client implementation for the static `AuthZ` resolver plugin.

use async_trait::async_trait;
use authz_resolver_sdk::{
    AuthZResolverError, AuthZResolverPluginClient, EvaluationRequest, EvaluationResponse,
};

use super::service::Service;

#[async_trait]
impl AuthZResolverPluginClient for Service {
    async fn evaluate(
        &self,
        request: EvaluationRequest,
    ) -> Result<EvaluationResponse, AuthZResolverError> {
        Ok(self.evaluate(&request))
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use authz_resolver_sdk::{Action, EvaluationRequestContext, Resource, Subject, TenantContext};
    use std::collections::HashMap;
    use uuid::Uuid;

    #[tokio::test]
    async fn plugin_trait_evaluates_successfully() {
        let service = Service::new();
        let plugin: &dyn AuthZResolverPluginClient = &service;

        let request = EvaluationRequest {
            subject: Subject {
                id: Uuid::nil(),
                subject_type: None,
                properties: HashMap::new(),
            },
            action: Action {
                name: "list".to_owned(),
            },
            resource: Resource {
                resource_type: "test".to_owned(),
                id: None,
                properties: HashMap::new(),
            },
            context: EvaluationRequestContext {
                tenant_context: Some(TenantContext {
                    root_id: Some(Uuid::nil()),
                    ..TenantContext::default()
                }),
                token_scopes: vec![],
                require_constraints: false,
                capabilities: vec![],
                supported_properties: vec![],
                bearer_token: None,
            },
        };

        let result = plugin.evaluate(request).await;
        assert!(result.is_ok());
        assert!(result.unwrap().decision);
    }
}
