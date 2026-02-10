use std::collections::HashMap;
use uuid::Uuid;

use modkit::runtime::InstanceState;

use crate::domain::model::{DeploymentMode, InstanceInfo, ModuleInfo};

/// Deployment mode of a module
#[derive(Debug, Clone)]
#[modkit_macros::api_dto(response)]
pub enum DeploymentModeDto {
    /// Module is compiled into the host binary
    CompiledIn,
    /// Module runs as a separate process
    OutOfProcess,
}

/// Response DTO for a single registered module
#[modkit_macros::api_dto(response)]
pub struct ModuleDto {
    /// Module name
    pub name: String,
    /// Module version (if reported by a running instance)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Declared capabilities (e.g., "rest", "grpc", "system", "db")
    pub capabilities: Vec<String>,
    /// Module dependencies (other module names)
    pub dependencies: Vec<String>,
    /// Whether the module is compiled-in or out-of-process
    pub deployment_mode: DeploymentModeDto,
    /// Running instances of this module
    pub instances: Vec<ModuleInstanceDto>,
    /// Plugins provided by this module (reserved for follow-up implementation)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub plugins: Vec<PluginDto>,
}

/// Response DTO for a running module instance
#[modkit_macros::api_dto(response)]
pub struct ModuleInstanceDto {
    /// Unique instance ID
    pub instance_id: Uuid,
    /// Module version (if reported during registration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Current instance state (e.g., "registered", "healthy", "quarantined")
    pub state: String,
    /// gRPC services provided by this instance (service name -> endpoint URI)
    pub grpc_services: HashMap<String, String>,
}

/// Response DTO for a plugin (reserved for follow-up implementation)
#[modkit_macros::api_dto(response)]
pub struct PluginDto {
    /// Plugin GTS identifier
    pub gts_id: String,
    /// Plugin version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl From<&ModuleInfo> for ModuleDto {
    fn from(module: &ModuleInfo) -> Self {
        // Derive module-level version from the first instance that reports one
        let version = module
            .instances
            .iter()
            .find_map(|inst| inst.version.clone());

        Self {
            name: module.name.clone(),
            version,
            capabilities: module.capabilities.clone(),
            dependencies: module.dependencies.clone(),
            deployment_mode: match module.deployment_mode {
                DeploymentMode::CompiledIn => DeploymentModeDto::CompiledIn,
                DeploymentMode::OutOfProcess => DeploymentModeDto::OutOfProcess,
            },
            instances: module.instances.iter().map(ModuleInstanceDto::from).collect(),
            plugins: vec![],
        }
    }
}

impl From<&InstanceInfo> for ModuleInstanceDto {
    fn from(instance: &InstanceInfo) -> Self {
        Self {
            instance_id: instance.instance_id,
            version: instance.version.clone(),
            state: match instance.state {
                InstanceState::Registered => "registered",
                InstanceState::Ready => "ready",
                InstanceState::Healthy => "healthy",
                InstanceState::Quarantined => "quarantined",
                InstanceState::Draining => "draining",
            }
            .to_owned(),
            grpc_services: instance.grpc_services.clone(),
        }
    }
}
