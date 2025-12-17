use crate::traits::PolicyEngineBuilder;
use axum::extract::State;
use modkit_security::SecurityContext;
use std::sync::Arc;

pub struct SimplePolicyEngineBuilder;

impl PolicyEngineBuilder for SimplePolicyEngineBuilder {
    fn build(&self, context: &SecurityContext) -> Arc<dyn modkit_security::PolicyEngine> {
        Arc::new(modkit_security::SimplePolicyEngine::new(context.clone()))
    }
}

pub async fn policy_engine_injector(
    State(policy_builder): State<Arc<dyn PolicyEngineBuilder>>,
    mut req: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    if let Some(sec_ctx) = req.extensions().get::<SecurityContext>() {
        let policy_engine = policy_builder.build(&sec_ctx);
        req.extensions_mut().insert(policy_engine);
    }

    next.run(req).await
}
