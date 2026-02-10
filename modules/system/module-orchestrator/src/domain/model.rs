use std::collections::HashMap;
use uuid::Uuid;

use modkit::runtime::InstanceState;
use modkit_macros::domain_model;

/// Deployment mode of a module.
#[domain_model]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeploymentMode {
    CompiledIn,
    OutOfProcess,
}

/// Domain model for a registered module.
#[domain_model]
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub name: String,
    pub capabilities: Vec<String>,
    pub dependencies: Vec<String>,
    pub deployment_mode: DeploymentMode,
    pub instances: Vec<InstanceInfo>,
}

/// Domain model for a running module instance.
#[domain_model]
#[derive(Debug, Clone)]
pub struct InstanceInfo {
    pub instance_id: Uuid,
    pub version: Option<String>,
    pub state: InstanceState,
    pub grpc_services: HashMap<String, String>,
}
