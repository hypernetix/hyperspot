use crate::SecurityContext;

pub type PolicyEngineRef = std::sync::Arc<dyn PolicyEngine>;

/// Policy Engine - Zero Trust Policy Engine, responsible for evaluating and enforcing policies or rules
pub trait PolicyEngine: Send + Sync {
    fn allows(&self, ctx: &SecurityContext, resource: &str, action: &str) -> bool;
}

pub struct DummyPolicyEngine;

impl Default for DummyPolicyEngine {
    fn default() -> Self {
        DummyPolicyEngine
    }
}

impl PolicyEngine for DummyPolicyEngine {
    fn allows(&self, _ctx: &SecurityContext, _resource: &str, _action: &str) -> bool {
        true
    }
}
