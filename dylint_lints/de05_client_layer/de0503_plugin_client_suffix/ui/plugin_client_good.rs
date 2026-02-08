#![allow(dead_code)]

use async_trait::async_trait;

// Should not trigger DE0503 - PluginApi suffix
#[async_trait]
pub trait ThrPluginClient: Send + Sync {
    async fn get_root_tenant(&self) -> Result<(), ()>;
}

// Should not trigger DE0503 - PluginApi suffix
#[async_trait]
pub trait OagwPluginClient: Send + Sync {
    async fn execute(&self) -> Result<(), ()>;
}

// Should not trigger DE0503 - PluginApi suffix
#[async_trait]
pub trait TenantResolverClient: Send + Sync {
    async fn list_tenants(&self) -> Result<(), ()>;
}

fn main() {}
