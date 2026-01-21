use crate::SecurityContext;

/// Type alias for a reference-counted Policy Engine
pub type PolicyEngineRef = std::sync::Arc<dyn PolicyEngine>;

/// Policy Engine - Zero Trust Policy Engine, responsible for evaluating and enforcing policies or rules
pub trait PolicyEngine: Send + Sync {
    fn allows(&self, ctx: &SecurityContext, resource: &str, action: &str) -> bool;
}

pub struct NoopPolicyEngine;

impl Default for NoopPolicyEngine {
    fn default() -> Self {
        NoopPolicyEngine
    }
}

impl PolicyEngine for NoopPolicyEngine {
    fn allows(&self, _ctx: &SecurityContext, _resource: &str, _action: &str) -> bool {
        true
    }
}
